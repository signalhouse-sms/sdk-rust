//! Request-contract coverage for RCS agents and the registration draft methods.
//! Run from sdks/rust: `cargo test` and `cargo test --features admin`.
use serde_json::{json, Value};
use signalhouse_sms::domains::rcs::{
    CreateRcsAgentParams, ListRcsAgentsParams, UpdateRcsAgentParams,
};
use signalhouse_sms::domains::registrations::GetRegistrationsParams;
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

    let wire = capture!(client => client.rcs.agents.get("RCSA/1", None));
    assert_eq!(request_line(&wire), "GET /rcs/agent/RCSA%2F1");
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
