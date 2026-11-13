//! Request-contract coverage for RCS agents and the registration draft methods.
//! Run from sdks/rust: `cargo test` and `cargo test --features admin`.
use serde_json::{json, Value};
use signalhouse_sms::domains::messages::GetMessagesParams;
use signalhouse_sms::domains::rcs::{
    CreateRcsAgentParams, EstimateRcsMessageParams, ListRcsAgentsParams, ListRcsMessagesParams,
    SendRcsMessageParams, SimulateRcsInboundParams, UpdateRcsAgentParams,
};
use signalhouse_sms::domains::registrations::{GetRegistrationsParams, LaunchAcknowledgements};
use signalhouse_sms::{
    RequestOptions, SignalHouseClient, SignalHouseConfig, SignalHouseError, SignalHouseResponse,
};
use std::io::{Read, Write};
use std::net::TcpListener;

const AGENT: &str = "RCSA7K2M1QZ8RP";

/// A one-shot capture server that reads the whole request (multipart included) before answering.
fn capture_server() -> (SignalHouseClient, std::thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        let mut wire: Vec<u8> = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            let size = match stream.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(size) => size,
            };
            wire.extend_from_slice(&chunk[..size]);
            if let Some(end) = wire.windows(4).position(|w| w == b"\r\n\r\n") {
                let head = String::from_utf8_lossy(&wire[..end]).to_lowercase();
                if head.contains("transfer-encoding: chunked") {
                    if wire.ends_with(b"0\r\n\r\n") {
                        break;
                    }
                    continue;
                }
                let length = head
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length:"))
                    .map(|v| v.trim().parse::<usize>().unwrap())
                    .unwrap_or(0);
                if wire.len() >= end + 4 + length {
                    break;
                }
            }
        }
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}")
            .unwrap();
        String::from_utf8_lossy(&wire).into_owned()
    });
    let client = SignalHouseClient::new(SignalHouseConfig {
        api_key: "test".into(),
        base_url: url,
    })
    .unwrap();
    (client, handle)
}

/// Run one call against a fresh capture server and return the raw request.
macro_rules! capture {
    ($client:ident => $call:expr) => {{
        let ($client, request) = capture_server();
        $call.await.unwrap();
        request.join().unwrap()
    }};
}

/// The request line, e.g. `GET /rcs/agent?status=DRAFT`.
fn request_line(wire: &str) -> String {
    let mut parts = wire.lines().next().unwrap().split(' ');
    format!("{} {}", parts.next().unwrap(), parts.next().unwrap())
}

/// The raw body after the blank line.
fn raw_body(wire: &str) -> &str {
    wire.split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("")
}

/// The JSON body after the blank line.
fn body(wire: &str) -> Value {
    serde_json::from_str(raw_body(wire)).unwrap()
}

/// A client whose transport always fails, for checks that must fail before sending.
fn offline() -> SignalHouseClient {
    SignalHouseClient::new(SignalHouseConfig {
        api_key: "test".into(),
        base_url: "http://127.0.0.1:1".into(),
    })
    .unwrap()
}

/// True when the call was rejected client-side, before any transport.
fn rejected(result: Result<SignalHouseResponse, SignalHouseError>) -> bool {
    matches!(result, Err(SignalHouseError::Validation(_)))
}

#[tokio::test]
async fn create_posts_the_agent_body_with_the_brand_link() {
    let agent = json!({ "name": "Acme" });
    let options = RequestOptions {
        idempotency_key: Some("k1".into()),
        ..Default::default()
    };
    let wire = capture!(client => client.rcs.agents.create(
        CreateRcsAgentParams {
            brand_id: "BXYZ123",
            campaign_id: Some("CABC123"),
            agent: Some(&agent),
            use_case: Some("TRANSACTIONAL"),
            ..Default::default()
        },
        Some(&options)
    ));
    assert_eq!(request_line(&wire), "POST /rcs/agent");
    assert!(wire.to_lowercase().contains("\r\nidempotency-key: k1\r\n"));
    assert_eq!(
        body(&wire),
        json!({ "brandId": "BXYZ123", "campaignId": "CABC123", "agent": { "name": "Acme" }, "useCase": "TRANSACTIONAL" })
    );
}

#[tokio::test]
async fn list_forwards_the_link_and_status_filters() {
    let wire = capture!(client => client.rcs.agents.list(
        ListRcsAgentsParams {
            brand_id: Some("BXYZ123"),
            campaign_id: Some("CABC123"),
            status: Some("DRAFT"),
            page: Some(2),
            limit: Some(50),
            ..Default::default()
        },
        None
    ));
    assert_eq!(
        request_line(&wire),
        "GET /rcs/agent?status=DRAFT&brandId=BXYZ123&campaignId=CABC123&page=2&limit=50"
    );
}

#[tokio::test]
async fn get_update_validate_submit_and_archive_address_the_agent() {
    let wire = capture!(client => client.rcs.agents.get(AGENT, None));
    assert_eq!(request_line(&wire), format!("GET /rcs/agent/{AGENT}"));

    let agent = json!({ "description": "Alerts" });
    let contact = json!({ "firstName": "Ada" });
    let wire = capture!(client => client.rcs.agents.update(
        UpdateRcsAgentParams {
            agent_id: AGENT,
            campaign_id: Some(None),
            agent: Some(&agent),
            contact: Some(&contact),
            use_case: Some(None),
            default_fallback: Some(None),
            ..Default::default()
        },
        None
    ));
    assert_eq!(request_line(&wire), format!("PUT /rcs/agent/{AGENT}"));
    assert_eq!(
        body(&wire),
        json!({ "campaignId": null, "agent": { "description": "Alerts" }, "contact": { "firstName": "Ada" }, "useCase": null, "defaultFallback": null })
    );

    let wire = capture!(client => client.rcs.agents.validate(AGENT, None));
    assert_eq!(
        request_line(&wire),
        format!("POST /rcs/agent/{AGENT}/validate")
    );

    let wire = capture!(client => client.rcs.agents.submit(AGENT, None));
    assert_eq!(
        request_line(&wire),
        format!("POST /rcs/agent/{AGENT}/submit")
    );

    let wire = capture!(client => client.rcs.agents.launch(AGENT, None));
    assert_eq!(
        request_line(&wire),
        format!("POST /registration/{AGENT}/launch")
    );
    assert!(rejected(offline().rcs.agents.launch("", None).await));

    let wire = capture!(client => client.rcs.agents.request_carrier_update(AGENT, None));
    assert_eq!(
        request_line(&wire),
        format!("POST /registration/{AGENT}/carrier-update")
    );
    assert!(rejected(
        offline().rcs.agents.request_carrier_update("", None).await
    ));

    let wire = capture!(client => client.rcs.agents.launch_acknowledging_fees(AGENT, None));
    assert_eq!(
        request_line(&wire),
        format!("POST /registration/{AGENT}/launch")
    );
    assert_eq!(body(&wire), json!({ "acknowledgeFees": true }));
    assert!(rejected(
        offline()
            .rcs
            .agents
            .launch_acknowledging_fees("", None)
            .await
    ));

    let wire = capture!(client => client.registrations.launch_registration_acknowledging_fees(AGENT, None));
    assert_eq!(body(&wire), json!({ "acknowledgeFees": true }));

    let both = LaunchAcknowledgements {
        fees: true,
        phi: true,
    };
    let wire = capture!(client => client.registrations.launch_registration_acknowledging(AGENT, both, None));
    assert_eq!(
        request_line(&wire),
        format!("POST /registration/{AGENT}/launch")
    );
    assert_eq!(
        body(&wire),
        json!({ "acknowledgeFees": true, "acknowledgePhi": true })
    );

    let phi_only = LaunchAcknowledgements {
        phi: true,
        ..Default::default()
    };
    let wire = capture!(client => client.rcs.agents.launch_acknowledging(AGENT, phi_only, None));
    assert_eq!(body(&wire), json!({ "acknowledgePhi": true }));

    let wire = capture!(client => client.rcs.agents.launch_acknowledging(AGENT, LaunchAcknowledgements::default(), None));
    assert_eq!(raw_body(&wire), "");

    let wire = capture!(client => client.billing.get_transaction_history_by_category("G1", "RCS_INBOUND,RCS_CARRIER_PASSTHROUGH", None, None, Some(2), Some(1000), None));
    assert_eq!(
        request_line(&wire),
        "GET /billing/wallet/transactionHistory?groupId=G1&category=RCS_INBOUND%2CRCS_CARRIER_PASSTHROUGH&page=2&limit=1000"
    );

    let wire = capture!(client => client.rcs.agents.archive(AGENT, None));
    assert_eq!(
        request_line(&wire),
        format!("POST /rcs/agent/{AGENT}/archive")
    );
    assert_eq!(raw_body(&wire), "");

    let wire =
        capture!(client => client.rcs.agents.resend_tester_invite(AGENT, "+15125550199", None));
    assert_eq!(
        request_line(&wire),
        format!("POST /rcs/agent/{AGENT}/testers/%2B15125550199/resend")
    );
    assert_eq!(raw_body(&wire), "");

    let wire = capture!(client => client.rcs.agents.remove_tester(AGENT, "+15125550199", None));
    assert_eq!(
        request_line(&wire),
        format!("DELETE /rcs/agent/{AGENT}/testers/%2B15125550199")
    );
    assert_eq!(raw_body(&wire), "");

    let wire = capture!(client => client.rcs.agents.get("RCSA/1", None));
    assert_eq!(request_line(&wire), "GET /rcs/agent/RCSA%2F1");

    let wire = capture!(client => client.rcs.agents.create_entry_points("RCSA/1", None, None));
    assert_eq!(request_line(&wire), "POST /rcs/agent/RCSA%2F1/entry-points");
    assert_eq!(body(&wire), json!({}));
}

#[tokio::test]
async fn upload_asset_posts_kind_and_file_as_multipart() {
    let wire = capture!(client => client.rcs.agents.upload_asset(
        AGENT, "logo", b"PNG".to_vec(), Some("logo.png"), None
    ));
    assert_eq!(
        request_line(&wire),
        format!("POST /rcs/agent/{AGENT}/asset")
    );
    assert!(wire
        .to_lowercase()
        .contains("content-type: multipart/form-data; boundary="));
    assert!(wire.contains("name=\"kind\"\r\n\r\nlogo\r\n"));
    assert!(wire.contains("name=\"file\"; filename=\"logo.png\""));
    assert!(wire.contains("\r\n\r\nPNG\r\n"));
}

#[tokio::test]
async fn registrations_gain_draft_update_validate_submit_and_link_filters() {
    let data = json!({ "useCase": "OTP" });
    let wire = capture!(client => client.registrations.update_registration(AGENT, &data, None));
    assert_eq!(request_line(&wire), format!("PUT /registration/{AGENT}"));
    assert_eq!(body(&wire), data);

    let wire = capture!(client => client.registrations.validate_registration(AGENT, None));
    assert_eq!(
        request_line(&wire),
        format!("POST /registration/{AGENT}/validate")
    );

    let wire = capture!(client => client.registrations.submit_registration(AGENT, None));
    assert_eq!(
        request_line(&wire),
        format!("POST /registration/{AGENT}/submit")
    );

    let wire = capture!(client => client.registrations.launch_registration(AGENT, None));
    assert_eq!(
        request_line(&wire),
        format!("POST /registration/{AGENT}/launch")
    );

    let wire = capture!(client => client.registrations.get_registrations(
        GetRegistrationsParams {
            type_: Some("RCS_AGENT"),
            brand_id: Some("BXYZ123"),
            campaign_id: Some("CABC123"),
            ..Default::default()
        },
        None
    ));
    assert_eq!(
        request_line(&wire),
        "GET /registration?type=RCS_AGENT&brandId=BXYZ123&campaignId=CABC123"
    );
}

#[tokio::test]
async fn missing_inputs_fail_before_transport() {
    let client = offline();
    let agents = &client.rcs.agents;
    assert!(rejected(
        agents.create(CreateRcsAgentParams::default(), None).await
    ));
    assert!(rejected(agents.get("", None).await));
    assert!(rejected(
        agents.update(UpdateRcsAgentParams::default(), None).await
    ));
    assert!(rejected(agents.validate("", None).await));
    assert!(rejected(agents.submit("", None).await));
    assert!(rejected(agents.archive("", None).await));
    assert!(rejected(agents.resend_tester_invite(AGENT, "", None).await));
    assert!(rejected(agents.remove_tester(AGENT, "", None).await));
    assert!(rejected(
        agents.create_entry_points("", Some("Hi there"), None).await
    ));
    assert!(rejected(
        agents
            .upload_asset(AGENT, "", b"PNG".to_vec(), None, None)
            .await
    ));
    assert!(rejected(
        agents
            .upload_asset(AGENT, "logo", Vec::new(), None, None)
            .await
    ));
    let registrations = &client.registrations;
    assert!(rejected(
        registrations
            .update_registration(AGENT, &Value::Null, None)
            .await
    ));
    assert!(rejected(
        registrations
            .update_registration("", &json!({}), None)
            .await
    ));
    assert!(rejected(
        registrations.validate_registration("", None).await
    ));
    assert!(rejected(registrations.submit_registration("", None).await));
}

#[tokio::test]
async fn send_posts_the_given_fields() {
    let recipients = [json!({ "phoneNumber": "+15125550199", "variables": { "name": "Ada" } })];
    let content = json!({ "blocks": [] });
    let fallback = json!({ "mode": "auto", "content": { "text": "Hi" } });
    let options = RequestOptions {
        idempotency_key: Some("k1".into()),
        ..Default::default()
    };
    let wire = capture!(client => client.rcs.messages.send(
        SendRcsMessageParams {
            agent_id: AGENT,
            recipients: &recipients,
            content: Some(&content),
            content_format: Some("content.v1"),
            traffic_type: Some("TRANSACTION"),
            ttl_seconds: Some(900),
            fallback: Some(&fallback),
            status_callback_url: Some("https://example.com/cb"),
            enable_shortlink: Some(false),
            is_test: Some(true),
            simulate: Some("delivered"),
            ..Default::default()
        },
        Some(&options),
    ));
    assert_eq!(request_line(&wire), "POST /message/rcs");
    assert!(wire.to_lowercase().contains("idempotency-key: k1"));
    assert_eq!(
        body(&wire),
        json!({
            "agentId": AGENT, "recipients": recipients, "content": content, "contentFormat": "content.v1",
            "trafficType": "TRANSACTION", "ttlSeconds": 900, "fallback": fallback,
            "statusCallbackUrl": "https://example.com/cb", "enableShortlink": false, "isTest": true,
            "simulate": "delivered",
        })
    );
}

#[tokio::test]
async fn send_batch_posts_the_send_body() {
    let recipients = [json!({ "phoneNumber": "+15125550199" })];
    let options = RequestOptions {
        idempotency_key: Some("k2".into()),
        ..Default::default()
    };
    let wire = capture!(client => client.rcs.messages.send_batch(
        SendRcsMessageParams {
            agent_id: AGENT,
            recipients: &recipients,
            template_id: Some("T1"),
            ..Default::default()
        },
        Some(&options),
    ));
    assert_eq!(request_line(&wire), "POST /message/rcs/batch");
    assert!(wire.to_lowercase().contains("idempotency-key: k2"));
    assert_eq!(
        body(&wire),
        json!({ "agentId": AGENT, "recipients": recipients, "templateId": "T1" })
    );
}

#[tokio::test]
async fn validate_posts_the_send_body() {
    let recipients = [json!({ "phoneNumber": "+15125550199" })];
    let wire = capture!(client => client.rcs.messages.validate(
        SendRcsMessageParams {
            agent_id: AGENT,
            recipients: &recipients,
            template_id: Some("T1"),
            template_version: Some(2),
            expire_time: Some("2026-11-05T00:00:00Z"),
            ..Default::default()
        },
        None,
    ));
    assert_eq!(request_line(&wire), "POST /message/rcs/validate");
    assert_eq!(
        body(&wire),
        json!({
            "agentId": AGENT, "recipients": recipients, "templateId": "T1", "templateVersion": 2,
            "expireTime": "2026-11-05T00:00:00Z",
        })
    );
}

#[tokio::test]
async fn estimate_adds_the_channel() {
    let fallback = json!({ "mode": "rcs_only" });
    let wire = capture!(client => client.rcs.messages.estimate(
        EstimateRcsMessageParams {
            agent_id: AGENT,
            recipient_count: Some(500),
            template_id: Some("T1"),
            template_version: Some(2),
            traffic_type: Some("PROMOTION"),
            fallback: Some(&fallback),
            ..Default::default()
        },
        None,
    ));
    assert_eq!(request_line(&wire), "POST /message/estimate");
    assert_eq!(
        body(&wire),
        json!({
            "channel": "rcs", "agentId": AGENT, "recipientCount": 500, "templateId": "T1",
            "templateVersion": 2, "trafficType": "PROMOTION", "fallback": fallback,
        })
    );
}

#[tokio::test]
async fn simulate_inbound_posts_the_event() {
    let wire = capture!(client => client.rcs.messages.simulate_inbound(
        SimulateRcsInboundParams { agent_id: AGENT, from: "+15125550199", event_type: "SUGGESTION", postback_data: Some("track_pkg"), ..Default::default() },
        None
    ));
    assert_eq!(request_line(&wire), "POST /message/rcs/simulate-inbound");
    assert_eq!(
        body(&wire),
        json!({ "agentId": AGENT, "from": "+15125550199", "type": "SUGGESTION", "postbackData": "track_pkg" })
    );
    assert!(rejected(
        offline()
            .rcs
            .messages
            .simulate_inbound(
                SimulateRcsInboundParams {
                    agent_id: AGENT,
                    event_type: "TEXT",
                    ..Default::default()
                },
                None
            )
            .await
    ));
}

#[tokio::test]
async fn capabilities_posts_the_numbers() {
    let wire = capture!(client => client.rcs.messages.capabilities(AGENT, &["+15125550199"], None));
    assert_eq!(request_line(&wire), "POST /rcs/capabilities");
    assert_eq!(
        body(&wire),
        json!({ "agentId": AGENT, "phoneNumbers": ["+15125550199"] })
    );
}

#[tokio::test]
async fn get_and_get_message_address_the_message() {
    let wire = capture!(client => client.rcs.messages.get("a/1", None));
    assert_eq!(request_line(&wire), "GET /message/a%2F1");
    assert_eq!(raw_body(&wire), "");

    let wire = capture!(client => client.messages.get_message("a/1", None));
    assert_eq!(request_line(&wire), "GET /message/a%2F1");
    assert_eq!(raw_body(&wire), "");
}

#[tokio::test]
async fn list_maps_the_filters_and_adds_the_type() {
    let wire = capture!(client => client.rcs.messages.list(
        ListRcsMessagesParams {
            agent_id: Some(AGENT),
            from: "2026-11-01",
            to: "2026-11-02",
            group_id: "G1",
            status: Some("READ"),
            page: Some(2),
            limit: Some(50),
            template_id: Some("T1"),
            template_version: Some(3),
            bulk_id: Some("B1"),
            fallback_reason: Some("NOT_CAPABLE"),
            billing_class: Some("RICH_MEDIA"),
            delivered_channel: Some("sms"),
            is_test: Some(false),
        },
        None,
    ));
    assert_eq!(
        request_line(&wire),
        format!(
            "GET /message?registrationId={AGENT}&startDate=2026-11-01&endDate=2026-11-02&groupId=G1\
             &status=READ&page=2&limit=50&templateId=T1&templateVersion=3&bulkId=B1\
             &fallbackReason=NOT_CAPABLE&billingClass=RICH_MEDIA&deliveredChannel=sms&isTest=false\
             &messageType=RCS&dateBounds=exact"
        )
    );

    let client = offline();
    for (field, params) in [
        (
            "groupId",
            ListRcsMessagesParams {
                agent_id: Some(AGENT),
                from: "2026-11-01",
                to: "2026-11-02",
                ..Default::default()
            },
        ),
        (
            "from",
            ListRcsMessagesParams {
                group_id: "G1",
                to: "2026-11-02",
                ..Default::default()
            },
        ),
        (
            "to",
            ListRcsMessagesParams {
                group_id: "G1",
                from: "2026-11-01",
                ..Default::default()
            },
        ),
    ] {
        let result = client.rcs.messages.list(params, None).await;
        assert!(matches!(result, Err(SignalHouseError::Validation(name)) if name == field));
    }
}

#[tokio::test]
async fn get_messages_forwards_the_rcs_filters() {
    let wire = capture!(client => client.messages.get_messages(
        GetMessagesParams {
            group_id: Some("G1"),
            start_date: Some("2026-11-01"),
            end_date: Some("2026-11-02"),
            template_id: Some("T1"),
            template_version: Some(3),
            bulk_id: Some("B1"),
            fallback_reason: Some("TTL_EXPIRED"),
            billing_class: Some("RICH"),
            delivered_channel: Some("sms"),
            is_test: Some(false),
            ..Default::default()
        },
        None,
    ));
    assert_eq!(
        request_line(&wire),
        "GET /message?groupId=G1&startDate=2026-11-01&endDate=2026-11-02&templateId=T1\
         &templateVersion=3&bulkId=B1&fallbackReason=TTL_EXPIRED&billingClass=RICH\
         &deliveredChannel=sms&isTest=false"
    );
}

#[tokio::test]
async fn missing_message_inputs_fail_before_transport() {
    let client = offline();
    let messages = &client.rcs.messages;
    let recipients = [json!({ "phoneNumber": "+15125550199" })];
    assert!(rejected(
        messages
            .send(
                SendRcsMessageParams {
                    recipients: &recipients,
                    ..Default::default()
                },
                None
            )
            .await
    ));
    assert!(rejected(
        messages
            .send(
                SendRcsMessageParams {
                    agent_id: AGENT,
                    ..Default::default()
                },
                None
            )
            .await
    ));
    assert!(rejected(
        messages
            .validate(
                SendRcsMessageParams {
                    agent_id: AGENT,
                    ..Default::default()
                },
                None
            )
            .await
    ));
    assert!(rejected(
        messages
            .send_batch(
                SendRcsMessageParams {
                    agent_id: AGENT,
                    ..Default::default()
                },
                None
            )
            .await
    ));
    assert!(rejected(messages.estimate(Default::default(), None).await));
    assert!(rejected(
        messages.capabilities("", &["+15125550199"], None).await
    ));
    assert!(rejected(messages.capabilities(AGENT, &[], None).await));
    assert!(rejected(messages.get("", None).await));
    assert!(rejected(client.messages.get_message("", None).await));
}

#[tokio::test]
async fn webhooks_filter_by_agent_and_send_the_agent_fields() {
    let wire = capture!(client => client.webhooks.get_webhooks_by_agent(Some("G1"), AGENT, None, None, None));
    assert_eq!(
        request_line(&wire),
        format!("GET /webhook?groupId=G1&agentId={AGENT}")
    );
    let wire = capture!(client => client.webhooks.get_webhooks(None, Some("G1"), Some("Agent"), None, Some(1), Some(25), None));
    assert_eq!(
        request_line(&wire),
        "GET /webhook?groupId=G1&endpointType=Agent&page=1&limit=25"
    );
    let data = json!({ "groupId": "G1", "endpointType": "Agent", "agentId": AGENT, "url": "https://a.example/rcs", "secondaryUrl": "https://b.example/rcs" });
    let wire = capture!(client => client.webhooks.create_webhook(&data, None));
    assert_eq!(body(&wire), data);
}

#[tokio::test]
async fn get_message_interactions_addresses_the_message() {
    let wire = capture!(client => client.messages.get_message_interactions("m1", None));
    assert_eq!(request_line(&wire), "GET /message/m1/interactions");
    assert!(rejected(
        offline().messages.get_message_interactions("", None).await
    ));
}

/// One corpus case's string field.
fn text<'a>(input: &'a Value, key: &str) -> Option<&'a str> {
    input[key].as_str()
}

/// One corpus case's number field.
fn number(input: &Value, key: &str) -> Option<u32> {
    input[key].as_u64().map(|value| value as u32)
}

/// One corpus case's object field.
fn object<'a>(input: &'a Value, key: &str) -> Option<&'a Value> {
    (!input[key].is_null()).then(|| &input[key])
}

/// One corpus case's list of strings.
fn strings<'a>(input: &'a Value, key: &str) -> Option<Vec<&'a str>> {
    input[key]
        .as_array()
        .map(|values| values.iter().map(|value| value.as_str().unwrap()).collect())
}

/// The parameters of a send or a validate.
fn send_params(input: &Value) -> SendRcsMessageParams<'_> {
    SendRcsMessageParams {
        agent_id: text(input, "agentId").unwrap(),
        recipients: input["recipients"].as_array().unwrap(),
        template_id: text(input, "templateId"),
        template_version: number(input, "templateVersion"),
        content: object(input, "content"),
        content_format: text(input, "contentFormat"),
        traffic_type: text(input, "trafficType"),
        ttl_seconds: number(input, "ttlSeconds"),
        expire_time: text(input, "expireTime"),
        fallback: object(input, "fallback"),
        status_callback_url: text(input, "statusCallbackUrl"),
        enable_shortlink: input["enableShortlink"].as_bool(),
        is_test: input["isTest"].as_bool(),
        simulate: text(input, "simulate"),
    }
}

/// Run the operation a corpus case names against a fresh capture server, and return the raw request.
async fn run_fixture(operation: &str, input: &Value) -> String {
    let (client, request) = capture_server();
    let agents = &client.rcs.agents;
    let messages = &client.rcs.messages;
    let agent_id = text(input, "agentId").unwrap_or("");
    let phone = text(input, "phoneNumber").unwrap_or("");
    let excluded = strings(input, "excludedCarriers");
    let testers = strings(input, "testers");
    let result = match operation {
        "messages.send" => messages.send(send_params(input), None).await,
        "messages.sendBatch" => messages.send_batch(send_params(input), None).await,
        "messages.validate" => messages.validate(send_params(input), None).await,
        "messages.estimate" => {
            let params = EstimateRcsMessageParams {
                agent_id,
                recipient_count: number(input, "recipientCount"),
                content: object(input, "content"),
                ..Default::default()
            };
            messages.estimate(params, None).await
        }
        "messages.capabilities" => {
            let numbers = strings(input, "phoneNumbers").unwrap();
            messages.capabilities(agent_id, &numbers, None).await
        }
        "messages.get" => messages.get(text(input, "messageId").unwrap(), None).await,
        "messages.list" => {
            let params = ListRcsMessagesParams {
                agent_id: text(input, "agentId"),
                from: text(input, "from").unwrap(),
                to: text(input, "to").unwrap(),
                group_id: text(input, "groupId").unwrap(),
                status: text(input, "status"),
                page: number(input, "page"),
                limit: number(input, "limit"),
                ..Default::default()
            };
            messages.list(params, None).await
        }
        "agents.create" => {
            let params = CreateRcsAgentParams {
                brand_id: text(input, "brandId").unwrap(),
                campaign_id: text(input, "campaignId"),
                agent: object(input, "agent"),
                use_case: text(input, "useCase"),
                default_fallback: object(input, "defaultFallback"),
                excluded_carriers: excluded.as_deref(),
                testers: testers.as_deref(),
                ..Default::default()
            };
            agents.create(params, None).await
        }
        "agents.list" => {
            let params = ListRcsAgentsParams {
                status: text(input, "status"),
                brand_id: text(input, "brandId"),
                page: number(input, "page"),
                limit: number(input, "limit"),
                ..Default::default()
            };
            agents.list(params, None).await
        }
        "agents.get" => agents.get(agent_id, None).await,
        "agents.update" => {
            let params = UpdateRcsAgentParams {
                agent_id,
                agent: object(input, "agent"),
                consent: object(input, "consent"),
                testers: testers.as_deref(),
                autofill: object(input, "autofill"),
                ..Default::default()
            };
            agents.update(params, None).await
        }
        "agents.validate" => agents.validate(agent_id, None).await,
        "agents.submit" => agents.submit(agent_id, None).await,
        "agents.launch" => {
            let acknowledgements = LaunchAcknowledgements {
                fees: input["acknowledgeFees"].as_bool().unwrap_or(false),
                phi: input["acknowledgePhi"].as_bool().unwrap_or(false),
            };
            agents
                .launch_acknowledging(agent_id, acknowledgements, None)
                .await
        }
        "agents.requestCarrierUpdate" => agents.request_carrier_update(agent_id, None).await,
        "agents.archive" => agents.archive(agent_id, None).await,
        "agents.createEntryPoints" => {
            agents
                .create_entry_points(agent_id, text(input, "body"), None)
                .await
        }
        "agents.resendTesterInvite" => agents.resend_tester_invite(agent_id, phone, None).await,
        "agents.removeTester" => agents.remove_tester(agent_id, phone, None).await,
        other => panic!("no adapter for {other}"),
    };
    result.unwrap();
    request.join().unwrap()
}

#[tokio::test]
async fn every_case_in_the_shared_corpus_produces_its_request() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../fixtures/rcs/requests.json");
    let Ok(raw) = std::fs::read_to_string(path) else {
        return; // The corpus lives in the monorepo.
    };
    let corpus: Value = serde_json::from_str(&raw).unwrap();
    for case in corpus["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let wire = run_fixture(case["operation"].as_str().unwrap(), &case["input"]).await;
        let request = &case["request"];
        let line = request_line(&wire);
        let (method, target) = line.split_once(' ').unwrap();
        let (path, query) = target.split_once('?').unwrap_or((target, ""));
        assert_eq!(method, request["method"], "{name}: method");
        assert_eq!(path, request["path"], "{name}: path");
        let pairs: serde_json::Map<String, Value> = query
            .split('&')
            .filter(|pair| !pair.is_empty())
            .map(|pair| pair.split_once('=').unwrap())
            .map(|(key, value)| (key.to_string(), Value::from(value)))
            .collect();
        let wanted = request["query"].as_object().cloned().unwrap_or_default();
        assert_eq!(pairs, wanted, "{name}: query");
        let sent = raw_body(&wire);
        let sent: Value = if sent.is_empty() {
            json!({})
        } else {
            serde_json::from_str(sent).unwrap()
        };
        let wanted = if request["body"].is_null() {
            json!({})
        } else {
            request["body"].clone()
        };
        assert_eq!(sent, wanted, "{name}: body");
    }
}
