//! Request-contract coverage for registrations and regional reporting filters.
//! Run from sdks/rust: `cargo test` and `cargo test --features admin`.
use signalhouse_sms::domains::messages::{
    GetAnalyticsByErrorCodeParams, GetAnalyticsBySubgroupParams, GetAnalyticsDetailParams,
    GetAnalyticsFilterOptionsParams, GetAnalyticsParams, GetAnalyticsThroughputParams,
    GetDncAnalyticsParams, GetDncRecordsParams, GetMessagesParams,
};
use signalhouse_sms::domains::registrations::GetRegistrationsParams;
use signalhouse_sms::{RequestOptions, SignalHouseClient, SignalHouseConfig};
use std::io::{Read, Write};
use std::net::TcpListener;

/// A one-shot capture server: returns the client pointed at it and the raw request it receives.
fn capture_server() -> (SignalHouseClient, std::thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        let mut buffer = vec![0; 16384];
        let size = stream.read(&mut buffer).unwrap();
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}")
            .unwrap();
        String::from_utf8(buffer[..size].to_vec()).unwrap()
    });
    let client = SignalHouseClient::new(SignalHouseConfig {
        api_key: "test".into(),
        base_url: url,
    })
    .unwrap();
    (client, handle)
}

/// The request line's target, e.g. `/message?region=GB`.
fn target(wire: &str) -> String {
    wire.lines()
        .next()
        .unwrap()
        .split(' ')
        .nth(1)
        .unwrap()
        .to_string()
}

/// The JSON body after the blank line.
fn body(wire: &str) -> serde_json::Value {
    serde_json::from_str(wire.split("\r\n\r\n").nth(1).unwrap_or("{}")).unwrap()
}

const SCOPES: &str =
    r#"[{"region":"GB","channels":["virtualLongCode"],"phoneNumber":["447400123456"]}]"#;
const ENCODED_SCOPES: &str = "%5B%7B%22region%22%3A%22GB%22%2C%22channels%22%3A%5B%22virtualLongCode%22%5D%2C%22phoneNumber%22%3A%5B%22447400123456%22%5D%7D%5D";

#[tokio::test]
async fn regional_reads_send_region_and_scopes() {
    let (client, request) = capture_server();
    client
        .messages
        .get_messages(
            GetMessagesParams {
                group_id: Some("G1"),
                region: Some("US,GB"),
                region_scopes: Some(SCOPES),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        format!("/message?groupId=G1&region=US%2CGB&regionScopes={ENCODED_SCOPES}")
    );

    let (client, request) = capture_server();
    client
        .messages
        .get_analytics(
            GetAnalyticsParams {
                group_id: Some("G1"),
                region: Some("GB"),
                region_scopes: Some("[]"),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/message/analytics?groupId=G1&region=GB&regionScopes=%5B%5D"
    );

    let (client, request) = capture_server();
    client
        .messages
        .get_analytics_detail(
            GetAnalyticsDetailParams {
                group_id: Some("G1"),
                region: Some("GB"),
                region_scopes: Some("[]"),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/message/analytics/detail?groupId=G1&region=GB&regionScopes=%5B%5D"
    );

    let (client, request) = capture_server();
    client
        .messages
        .get_analytics_by_subgroup(
            GetAnalyticsBySubgroupParams {
                group_id: Some("G1"),
                region: Some("GB"),
                region_scopes: Some("[]"),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/message/analytics/by-subgroup?groupId=G1&region=GB&regionScopes=%5B%5D"
    );

    let (client, request) = capture_server();
    client
        .messages
        .get_analytics_by_error_code(
            GetAnalyticsByErrorCodeParams {
                group_id: Some("G1"),
                region: Some("GB"),
                region_scopes: Some("[]"),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/message/analytics/by-error-code?groupId=G1&region=GB&regionScopes=%5B%5D"
    );

    let (client, request) = capture_server();
    client
        .messages
        .get_dnc_analytics(
            GetDncAnalyticsParams {
                group_id: Some("G1"),
                region: Some("GB"),
                region_scopes: Some("[]"),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/message/dnc/analytics?groupId=G1&region=GB&regionScopes=%5B%5D"
    );

    let (client, request) = capture_server();
    client
        .messages
        .get_dnc_records(
            GetDncRecordsParams {
                group_id: Some("G1"),
                region: Some("GB"),
                region_scopes: Some("[]"),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/message/dnc?groupId=G1&region=GB&regionScopes=%5B%5D"
    );
}

#[tokio::test]
async fn reads_without_region_keep_the_legacy_query() {
    let (client, request) = capture_server();
    client
        .messages
        .get_messages(
            GetMessagesParams {
                page: Some(1),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(target(&request.join().unwrap()), "/message?page=1");
}

#[tokio::test]
async fn filter_options_accept_region_only() {
    let (client, request) = capture_server();
    client
        .messages
        .get_analytics_filter_options(
            GetAnalyticsFilterOptionsParams {
                group_id: "G1",
                region: Some("GB"),
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/message/analytics/filter-options?groupId=G1&region=GB"
    );
}

#[tokio::test]
async fn catalog_quotes_list_read_and_submission_contracts() {
    let (client, request) = capture_server();
    client.registrations.get_catalog("GB", None).await.unwrap();
    assert!(request
        .join()
        .unwrap()
        .starts_with("GET /registration/catalog?region=GB "));

    let (client, request) = capture_server();
    client
        .registrations
        .get_quotes("GB", "VIRTUAL_LONG_CODE", 2, None, None)
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/registration/quotes?region=GB&type=VIRTUAL_LONG_CODE&quantity=2"
    );

    let (client, request) = capture_server();
    client
        .registrations
        .get_quotes("UK", "VIRTUAL_LONG_CODE", 1, Some("G1"), None)
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/registration/quotes?region=UK&type=VIRTUAL_LONG_CODE&quantity=1&groupId=G1"
    );

    let (client, request) = capture_server();
    client
        .registrations
        .get_registrations(
            GetRegistrationsParams {
                group_id: Some("G1"),
                subgroup_id: Some("S1"),
                region: Some("GB,CA"),
                type_: Some("VIRTUAL_LONG_CODE"),
                kind: Some("sender"),
                status: Some("PENDING_PROVIDER,APPROVED"),
                parent_registration_id: Some("REG0"),
                phone_number: Some("15551234567"),
                sort_by: Some("status"),
                sort_order: Some("asc"),
                page: Some(2),
                limit: Some(10),
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/registration?groupId=G1&subgroupId=S1&region=GB%2CCA&type=VIRTUAL_LONG_CODE&kind=sender&status=PENDING_PROVIDER%2CAPPROVED&parentRegistrationId=REG0&phoneNumber=15551234567&sortBy=status&sortOrder=asc&page=2&limit=10"
    );

    let (client, request) = capture_server();
    client
        .registrations
        .get_registration("REG 1", None)
        .await
        .unwrap();
    assert!(request
        .join()
        .unwrap()
        .starts_with("GET /registration/REG%201 "));

    let (client, request) = capture_server();
    client
        .registrations
        .cancel_registration("GB8T9GTD5AZY", None)
        .await
        .unwrap();
    assert!(request
        .join()
        .unwrap()
        .starts_with("DELETE /registration/GB8T9GTD5AZY "));

    let (client, request) = capture_server();
    let data = serde_json::json!({
        "region": "GB", "type": "VIRTUAL_LONG_CODE", "subgroupId": "S1",
        "capabilities": ["SMS"],
        "data": { "quantity": 1, "useCases": ["Support"] }
    });
    client
        .registrations
        .create_registration(&data, None)
        .await
        .unwrap();
    let wire = request.join().unwrap();
    assert!(wire.starts_with("POST /registration "));
    assert_eq!(body(&wire), data);
}

#[tokio::test]
async fn missing_identifiers_fail_before_transport() {
    let client = SignalHouseClient::new(SignalHouseConfig {
        api_key: "test".into(),
        base_url: "http://127.0.0.1:1".into(),
    })
    .unwrap();
    assert!(client.registrations.get_catalog("", None).await.is_err());
    assert!(client
        .registrations
        .get_registration("", None)
        .await
        .is_err());
    assert!(client
        .registrations
        .cancel_registration("", None)
        .await
        .is_err());
    assert!(client
        .registrations
        .get_quotes("GB", "", 1, None, None)
        .await
        .is_err());
    assert!(client
        .registrations
        .get_quotes("GB", "VIRTUAL_LONG_CODE", 0, None, None)
        .await
        .is_err());
}

#[cfg(feature = "admin")]
#[tokio::test]
async fn staff_review_corrections_numbers_reconcile_and_application() {
    let (client, request) = capture_server();
    client
        .registrations
        .review_registration("REG1", "APPROVE", None, None)
        .await
        .unwrap();
    let wire = request.join().unwrap();
    assert!(wire.starts_with("POST /registration/REG1/review "));
    assert_eq!(body(&wire), serde_json::json!({"action": "APPROVE"}));

    let (client, request) = capture_server();
    client
        .registrations
        .review_registration("REG1", "REJECT", Some("Not a business use case"), None)
        .await
        .unwrap();
    assert_eq!(
        body(&request.join().unwrap()),
        serde_json::json!({"action": "REJECT", "reason": "Not a business use case"})
    );

    let (client, request) = capture_server();
    let corrections = serde_json::json!({"useCases": ["Support"]});
    client
        .registrations
        .correct_registration("REG1", "Provider asked", &corrections, None)
        .await
        .unwrap();
    let wire = request.join().unwrap();
    assert!(wire.starts_with("POST /registration/REG1/corrections "));
    assert_eq!(
        body(&wire),
        serde_json::json!({"reason": "Provider asked", "data": {"useCases": ["Support"]}})
    );

    let (client, request) = capture_server();
    client
        .registrations
        .add_number("REG1", "447400123456", None)
        .await
        .unwrap();
    let wire = request.join().unwrap();
    assert!(wire.starts_with("POST /registration/REG1/numbers "));
    assert_eq!(
        body(&wire),
        serde_json::json!({"phoneNumber": "447400123456"})
    );

    let (client, request) = capture_server();
    client
        .registrations
        .reconcile("REG1", "ORDER-123", None)
        .await
        .unwrap();
    let wire = request.join().unwrap();
    assert!(wire.starts_with("POST /registration/REG1/reconcile "));
    assert_eq!(body(&wire), serde_json::json!({"externalId": "ORDER-123"}));

    let (client, request) = capture_server();
    client
        .registrations
        .get_application("REG1", None)
        .await
        .unwrap();
    assert!(request
        .join()
        .unwrap()
        .starts_with("GET /registration/REG1/application?format=json "));
}

#[tokio::test]
async fn registration_reads_forward_registration_id() {
    let (client, request) = capture_server();
    client
        .messages
        .get_analytics(
            GetAnalyticsParams {
                group_id: Some("G1"),
                campaign_id: Some("C1"),
                registration_id: Some("REG1,REG2"),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/message/analytics?groupId=G1&campaignId=C1&registrationId=REG1%2CREG2"
    );

    let (client, request) = capture_server();
    client
        .messages
        .get_analytics_detail(
            GetAnalyticsDetailParams {
                group_id: Some("G1"),
                campaign_id: Some("C1"),
                registration_id: Some("REG1,REG2"),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/message/analytics/detail?groupId=G1&campaignId=C1&registrationId=REG1%2CREG2"
    );

    let (client, request) = capture_server();
    client
        .messages
        .get_analytics_throughput(
            GetAnalyticsThroughputParams {
                group_id: Some("G1"),
                campaign_id: Some("C1"),
                registration_id: Some("REG1,REG2"),
                start_date: Some("2026-09-01"),
                end_date: Some("2026-09-02"),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/message/analytics/throughput?groupId=G1&campaignId=C1&registrationId=REG1%2CREG2&startDate=2026-09-01&endDate=2026-09-02"
    );

    let (client, request) = capture_server();
    client
        .messages
        .get_dnc_analytics(
            GetDncAnalyticsParams {
                group_id: Some("G1"),
                campaign_id: Some("C1"),
                registration_id: Some("REG1,REG2"),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/message/dnc/analytics?groupId=G1&campaignId=C1&registrationId=REG1%2CREG2"
    );

    let (client, request) = capture_server();
    client
        .messages
        .get_analytics_by_subgroup(
            GetAnalyticsBySubgroupParams {
                group_id: Some("G1"),
                campaign_id: Some("C1"),
                registration_id: Some("REG1,REG2"),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    let wire = target(&request.join().unwrap());
    assert!(
        wire.starts_with("/message/analytics/by-subgroup?"),
        "get_analytics_by_subgroup path: {wire}"
    );
    assert!(
        wire.contains("campaignId=C1&registrationId=REG1%2CREG2"),
        "get_analytics_by_subgroup registrationId: {wire}"
    );

    let (client, request) = capture_server();
    client
        .messages
        .get_analytics_by_error_code(
            GetAnalyticsByErrorCodeParams {
                group_id: Some("G1"),
                campaign_id: Some("C1"),
                registration_id: Some("REG1,REG2"),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    let wire = target(&request.join().unwrap());
    assert!(
        wire.starts_with("/message/analytics/by-error-code?"),
        "get_analytics_by_error_code path: {wire}"
    );
    assert!(
        wire.contains("campaignId=C1&registrationId=REG1%2CREG2"),
        "get_analytics_by_error_code registrationId: {wire}"
    );

    let (client, request) = capture_server();
    client
        .messages
        .get_dnc_records(
            GetDncRecordsParams {
                group_id: Some("G1"),
                campaign_id: Some("C1"),
                registration_id: Some("REG1,REG2"),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    let wire = target(&request.join().unwrap());
    assert!(
        wire.starts_with("/message/dnc?"),
        "get_dnc_records path: {wire}"
    );
    assert!(
        wire.contains("campaignId=C1&registrationId=REG1%2CREG2"),
        "get_dnc_records registrationId: {wire}"
    );

    let (client, request) = capture_server();
    client
        .messages
        .get_messages(
            GetMessagesParams {
                group_id: Some("G1"),
                campaign_id: Some("C1"),
                registration_id: Some("REG1,REG2"),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    let wire = target(&request.join().unwrap());
    assert!(wire.starts_with("/message?"), "get_messages path: {wire}");
    assert!(
        wire.contains("campaignId=C1&registrationId=REG1%2CREG2"),
        "get_messages registrationId: {wire}"
    );

    let (client, request) = capture_server();
    client
        .numbers
        .get_phone_numbers(
            None,
            Some("C1"),
            Some("REG1"),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/number?campaignId=C1&registrationId=REG1"
    );
}

#[tokio::test]
async fn alphanumeric_sender_submission_posts_data_unchanged() {
    let (client, request) = capture_server();
    let data = serde_json::json!({
        "region": "GB", "type": "ALPHANUMERIC_SENDER_ID", "subgroupId": "S1",
        "capabilities": ["SMS"],
        "data": {
            "requestedSenderId": "ACME", "trafficOrigin": "LOCAL", "trafficType": "TRANSACTIONAL",
            "companyName": "Acme Ltd", "companyCountry": "United Kingdom",
            "companyWebsite": "https://acme.example", "industry": "Retail",
            "senderRelationship": "Trading name of Acme Ltd",
            "messageExample": "Your Acme order has shipped."
        }
    });
    client
        .registrations
        .create_registration(&data, None)
        .await
        .unwrap();
    let wire = request.join().unwrap();
    assert!(wire.starts_with("POST /registration "));
    assert_eq!(body(&wire), data);
}

#[tokio::test]
async fn shortlink_opt_out_contracts() {
    let (client, request) = capture_server();
    client.shortlinks.get_opt_out("ABC/1", None).await.unwrap();
    let wire = request.join().unwrap();
    assert!(wire.starts_with("GET /shortlink/optout/ABC%2F1 "));
    assert_eq!(wire.split("\r\n\r\n").nth(1).unwrap_or(""), "");

    let (client, request) = capture_server();
    client
        .shortlinks
        .confirm_opt_out("ABC/1", None)
        .await
        .unwrap();
    let wire = request.join().unwrap();
    assert!(wire.starts_with("POST /shortlink/optout/ABC%2F1 "));
    assert_eq!(wire.split("\r\n\r\n").nth(1).unwrap_or(""), "");

    let client = SignalHouseClient::new(SignalHouseConfig {
        api_key: "test".into(),
        base_url: "http://127.0.0.1:1".into(),
    })
    .unwrap();
    assert!(client.shortlinks.get_opt_out("", None).await.is_err());
    assert!(client.shortlinks.confirm_opt_out("", None).await.is_err());
}

#[tokio::test]
async fn idempotency_key_is_sent_as_a_header() {
    let (client, request) = capture_server();
    let data =
        serde_json::json!({ "region": "GB", "type": "VIRTUAL_LONG_CODE", "subgroupId": "S1" });
    let options = RequestOptions {
        idempotency_key: Some("key-1".into()),
        ..Default::default()
    };
    client
        .registrations
        .create_registration(&data, Some(&options))
        .await
        .unwrap();
    let wire = request.join().unwrap().to_lowercase();
    assert!(wire.contains("\r\nidempotency-key: key-1\r\n"));
}

#[tokio::test]
async fn brand_and_number_lists_send_current_toll_free_carrier() {
    let (client, request) = capture_server();
    client
        .brands
        .get_brands_on_current_toll_free_carrier(
            None,
            None,
            Some("G1"),
            None,
            None,
            None,
            Some("TOLL_FREE"),
            Some(true),
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/brand?groupId=G1&registrationType=TOLL_FREE&currentTollFreeCarrier=true"
    );

    let (client, request) = capture_server();
    client
        .brands
        .get_brands_on_current_toll_free_carrier(
            None,
            Some("S1"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
    assert_eq!(target(&request.join().unwrap()), "/brand?subgroupId=S1");

    let (client, request) = capture_server();
    client
        .brands
        .get_brands(None, Some("S1"), None, None, None, None, None, None)
        .await
        .unwrap();
    assert_eq!(target(&request.join().unwrap()), "/brand?subgroupId=S1");

    let (client, request) = capture_server();
    client
        .numbers
        .get_phone_numbers_on_current_toll_free_carrier(
            None,
            None,
            None,
            None,
            Some("S1"),
            None,
            Some(1),
            Some(25),
            Some(true),
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/number?subgroupId=S1&page=1&limit=25&currentTollFreeCarrier=true"
    );

    let (client, request) = capture_server();
    client
        .numbers
        .get_phone_numbers_on_current_toll_free_carrier(
            None,
            None,
            None,
            None,
            None,
            Some("G1"),
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
    assert_eq!(target(&request.join().unwrap()), "/number?groupId=G1");

    let (client, request) = capture_server();
    client
        .numbers
        .get_phone_numbers(None, None, None, None, None, Some("G1"), None, None, None)
        .await
        .unwrap();
    assert_eq!(target(&request.join().unwrap()), "/number?groupId=G1");
}

#[cfg(feature = "admin")]
#[tokio::test]
async fn staff_set_toll_free_provider_contract() {
    let (client, request) = capture_server();
    client
        .onboarding
        .set_toll_free_provider("G 1", "twilio", None)
        .await
        .unwrap();
    let wire = request.join().unwrap();
    assert!(wire.starts_with("PUT /group/onboarding/G%201/toll-free-provider "));
    assert_eq!(
        body(&wire),
        serde_json::json!({"tollFreeProvider": "twilio"})
    );

    let client = SignalHouseClient::new(SignalHouseConfig {
        api_key: "test".into(),
        base_url: "http://127.0.0.1:1".into(),
    })
    .unwrap();
    assert!(client
        .onboarding
        .set_toll_free_provider("", "twilio", None)
        .await
        .is_err());
    assert!(client
        .onboarding
        .set_toll_free_provider("G0000001", "", None)
        .await
        .is_err());
}

#[tokio::test]
async fn business_verification_list_read_create_and_update_contracts() {
    let (client, request) = capture_server();
    client
        .brands
        .get_business_verifications("G 1", None)
        .await
        .unwrap();
    let wire = request.join().unwrap();
    assert!(wire.starts_with("GET /brand/business-verification/G%201 "));
    assert_eq!(wire.split("\r\n\r\n").nth(1).unwrap_or(""), "");

    let (client, request) = capture_server();
    let details = serde_json::json!({
        "businessName": "Acme Inc", "businessType": "CORPORATION", "industry": "RETAIL",
        "registrationType": "EIN", "registrationNumber": "12-3456789",
        "website": "https://acme.example", "street": "1 Main St", "city": "Austin",
        "state": "TX", "postalCode": "78701", "country": "US",
        "representative": {
            "firstName": "Ada", "lastName": "Lee", "email": "ada@acme.example",
            "phone": "+15125550100", "jobTitle": "Owner", "jobLevel": "CEO"
        }
    });
    client
        .brands
        .create_business_verification("G 1", &details, None)
        .await
        .unwrap();
    let wire = request.join().unwrap();
    assert!(wire.starts_with("POST /brand/business-verification/G%201 "));
    assert_eq!(body(&wire), details);

    let (client, request) = capture_server();
    client
        .brands
        .get_business_verification("G 1", "BV 1", None)
        .await
        .unwrap();
    let wire = request.join().unwrap();
    assert!(wire.starts_with("GET /brand/business-verification/G%201/BV%201 "));
    assert_eq!(wire.split("\r\n\r\n").nth(1).unwrap_or(""), "");

    let (client, request) = capture_server();
    client
        .brands
        .update_business_verification("G 1", "BV 1", &details, None)
        .await
        .unwrap();
    let wire = request.join().unwrap();
    assert!(wire.starts_with("PUT /brand/business-verification/G%201/BV%201 "));
    assert_eq!(body(&wire), details);

    let client = SignalHouseClient::new(SignalHouseConfig {
        api_key: "test".into(),
        base_url: "http://127.0.0.1:1".into(),
    })
    .unwrap();
    assert!(matches!(
        client.brands.get_business_verifications("", None).await,
        Err(signalhouse_sms::SignalHouseError::Validation(field)) if field == "groupId"
    ));
    assert!(matches!(
        client
            .brands
            .create_business_verification("", &details, None)
            .await,
        Err(signalhouse_sms::SignalHouseError::Validation(field)) if field == "groupId"
    ));
    for (group_id, id, missing) in [
        ("", "BV1", "groupId"),
        ("G0000001", "", "businessVerificationId"),
    ] {
        assert!(
            matches!(
                client.brands.get_business_verification(group_id, id, None).await,
                Err(signalhouse_sms::SignalHouseError::Validation(field)) if field == missing
            ),
            "get_business_verification without {missing}"
        );
        assert!(
            matches!(
                client
                    .brands
                    .update_business_verification(group_id, id, &details, None)
                    .await,
                Err(signalhouse_sms::SignalHouseError::Validation(field)) if field == missing
            ),
            "update_business_verification without {missing}"
        );
    }
}

#[tokio::test]
async fn business_verification_for_subgroup_sends_subgroup_id_only_when_given() {
    let (client, request) = capture_server();
    client
        .brands
        .get_business_verifications_for_subgroup("G 1", Some("S 1"), None)
        .await
        .unwrap();
    let wire = request.join().unwrap();
    assert!(wire.starts_with("GET /brand/business-verification/G%201?subgroupId=S%201 "));
    assert_eq!(wire.split("\r\n\r\n").nth(1).unwrap_or(""), "");

    let (client, request) = capture_server();
    client
        .brands
        .get_business_verifications_for_subgroup("G 1", None, None)
        .await
        .unwrap();
    assert!(request
        .join()
        .unwrap()
        .starts_with("GET /brand/business-verification/G%201 "));

    let details = serde_json::json!({ "businessName": "Acme Inc", "country": "US" });
    let (client, request) = capture_server();
    client
        .brands
        .create_business_verification_for_subgroup("G 1", &details, Some("S 1"), None)
        .await
        .unwrap();
    let wire = request.join().unwrap();
    assert!(wire.starts_with("POST /brand/business-verification/G%201?subgroupId=S%201 "));
    assert_eq!(body(&wire), details);

    let (client, request) = capture_server();
    client
        .brands
        .create_business_verification_for_subgroup("G 1", &details, None, None)
        .await
        .unwrap();
    let wire = request.join().unwrap();
    assert!(wire.starts_with("POST /brand/business-verification/G%201 "));
    assert_eq!(body(&wire), details);

    let client = SignalHouseClient::new(SignalHouseConfig {
        api_key: "test".into(),
        base_url: "http://127.0.0.1:1".into(),
    })
    .unwrap();
    assert!(matches!(
        client
            .brands
            .get_business_verifications_for_subgroup("", Some("S0000001"), None)
            .await,
        Err(signalhouse_sms::SignalHouseError::Validation(field)) if field == "groupId"
    ));
    assert!(matches!(
        client
            .brands
            .create_business_verification_for_subgroup("", &details, Some("S0000001"), None)
            .await,
        Err(signalhouse_sms::SignalHouseError::Validation(field)) if field == "groupId"
    ));
}

#[cfg(feature = "admin")]
#[tokio::test]
async fn staff_toll_free_isv_account_contracts() {
    let (client, request) = capture_server();
    client
        .brands
        .get_toll_free_isv_account("G 1", None)
        .await
        .unwrap();
    let wire = request.join().unwrap();
    assert!(wire.starts_with("GET /brand/toll-free-isv-account/G%201 "));
    assert_eq!(wire.split("\r\n\r\n").nth(1).unwrap_or(""), "");

    let account_sid = "AC00000000000000000000000000000000";
    let api_key_sid = "SK11111111111111111111111111111111";
    let (client, request) = capture_server();
    client
        .brands
        .set_toll_free_isv_account("G 1", account_sid, api_key_sid, "key-secret", None)
        .await
        .unwrap();
    let wire = request.join().unwrap();
    assert!(wire.starts_with("PUT /brand/toll-free-isv-account/G%201 "));
    assert_eq!(
        body(&wire),
        serde_json::json!({"accountSid": account_sid, "apiKeySid": api_key_sid, "apiKeySecret": "key-secret"})
    );

    let client = SignalHouseClient::new(SignalHouseConfig {
        api_key: "test".into(),
        base_url: "http://127.0.0.1:1".into(),
    })
    .unwrap();
    assert!(matches!(
        client.brands.get_toll_free_isv_account("", None).await,
        Err(signalhouse_sms::SignalHouseError::Validation(field)) if field == "groupId"
    ));
    for (group_id, account, key, secret, missing) in [
        ("", account_sid, api_key_sid, "key-secret", "groupId"),
        ("G0000001", "", api_key_sid, "key-secret", "accountSid"),
        ("G0000001", account_sid, "", "key-secret", "apiKeySid"),
        ("G0000001", account_sid, api_key_sid, "", "apiKeySecret"),
    ] {
        assert!(
            matches!(
                client
                    .brands
                    .set_toll_free_isv_account(group_id, account, key, secret, None)
                    .await,
                Err(signalhouse_sms::SignalHouseError::Validation(field)) if field == missing
            ),
            "set_toll_free_isv_account without {missing}"
        );
    }
}
