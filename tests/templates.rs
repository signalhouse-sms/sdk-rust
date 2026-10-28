//! Request-contract coverage for the template door.
//! Run from sdks/rust: `cargo test` and `cargo test --features admin`.
use serde_json::{json, Value};
use signalhouse_sms::domains::templates::{GetTemplatesParams, UpdateTemplateParams};
use signalhouse_sms::{
    RequestOptions, SignalHouseClient, SignalHouseConfig, SignalHouseError, SignalHouseResponse,
};
use std::io::{Read, Write};
use std::net::TcpListener;

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

/// The request line, e.g. `GET /template?groupId=G1`.
fn request_line(wire: &str) -> String {
    let mut parts = wire.lines().next().unwrap().split(' ');
    format!("{} {}", parts.next().unwrap(), parts.next().unwrap())
}

/// The JSON body after the blank line.
fn body(wire: &str) -> Value {
    serde_json::from_str(wire.split("\r\n\r\n").nth(1).unwrap_or("{}")).unwrap()
}

/// A client whose transport always fails, for checks that must fail before sending.
fn offline() -> SignalHouseClient {
    SignalHouseClient::new(SignalHouseConfig {
        api_key: "test".into(),
        base_url: "http://127.0.0.1:1".into(),
    })
    .unwrap()
}

/// A minimal publishable RcsTemplateBody.
fn rcs_body() -> Value {
    json!({
        "channel": "rcs",
        "contentFormat": "rcs.blocks.v1",
        "content": { "blocks": [{ "id": "b1", "kind": "text", "text": "Hi {{first_name}}" }] },
        "variables": [{ "key": "first_name", "type": "text", "required": true }],
        "defaults": {},
        "fallback": {}
    })
}

#[tokio::test]
async fn create_sends_the_raw_body_and_tenant() {
    let template = rcs_body();
    let options = RequestOptions {
        idempotency_key: Some("key-1".into()),
        ..Default::default()
    };
    let wire = capture!(client => client.templates.create(
        "G1", "Welcome", &template, None, Some(&options)
    ));
    assert_eq!(request_line(&wire), "POST /template");
    assert!(wire
        .to_lowercase()
        .contains("\r\nidempotency-key: key-1\r\n"));
    let sent = body(&wire);
    assert_eq!(sent["groupId"], "G1");
    assert_eq!(sent["name"], "Welcome");
    assert_eq!(sent["body"], template);
    assert!(sent.get("tag").is_none());
}

#[tokio::test]
async fn reads_put_tenant_and_selectors_in_the_query() {
    let wire = capture!(client => client.templates.list(
        GetTemplatesParams {
            group_id: "G1",
            state: Some("DRAFT"),
            channel: Some("rcs"),
            tag: Some("Promo"),
            search: Some("wel"),
            sort_by: Some("name"),
            sort_order: Some("asc"),
            page: Some(2),
            limit: Some(10),
        },
        None
    ));
    assert_eq!(
        request_line(&wire),
        "GET /template?groupId=G1&state=DRAFT&channel=rcs&tag=Promo&search=wel&sortBy=name&sortOrder=asc&page=2&limit=10"
    );

    let wire = capture!(client => client.templates.get("TPL/1", "G1", Some("draft"), None));
    assert_eq!(
        request_line(&wire),
        "GET /template/TPL%2F1?groupId=G1&version=draft"
    );

    let wire = capture!(client => client.templates.versions("TPL1", "G1", Some(1), Some(5), None));
    assert_eq!(
        request_line(&wire),
        "GET /template/TPL1/versions?groupId=G1&page=1&limit=5"
    );

    let wire =
        capture!(client => client.templates.capabilities("G1", Some("rcs"), Some("sinch"), None));
    assert_eq!(
        request_line(&wire),
        "GET /template/capabilities?groupId=G1&channel=rcs&provider=sinch"
    );
}

#[tokio::test]
async fn update_sends_only_tenant_and_changed_fields_and_distinguishes_clear_from_omit() {
    let wire = capture!(client => client.templates.update(
        UpdateTemplateParams { template_id: "TPL1", group_id: "G1", ..Default::default() },
        None
    ));
    assert_eq!(request_line(&wire), "PUT /template/TPL1");
    assert_eq!(body(&wire), json!({ "groupId": "G1" }));

    let wire = capture!(client => client.templates.update(
        UpdateTemplateParams {
            template_id: "TPL1",
            group_id: "G1",
            tag: Some(None),
            ..Default::default()
        },
        None
    ));
    assert_eq!(body(&wire), json!({ "groupId": "G1", "tag": null }));

    let template = rcs_body();
    let wire = capture!(client => client.templates.update(
        UpdateTemplateParams {
            template_id: "TPL1",
            group_id: "G1",
            name: Some("Renamed"),
            tag: Some(Some("Promo")),
            body: Some(&template),
        },
        None
    ));
    assert_eq!(
        body(&wire),
        json!({ "groupId": "G1", "name": "Renamed", "tag": "Promo", "body": template })
    );
}

#[tokio::test]
async fn lifecycle_writes_carry_only_the_tenant() {
    let wire = capture!(client => client.templates.publish("TPL1", "G1", None));
    assert_eq!(request_line(&wire), "POST /template/TPL1/publish");
    assert_eq!(body(&wire), json!({ "groupId": "G1" }));

    let wire = capture!(client => client.templates.archive("TPL1", "G1", None));
    assert_eq!(request_line(&wire), "POST /template/TPL1/archive");
    assert_eq!(body(&wire), json!({ "groupId": "G1" }));

    let wire = capture!(client => client.templates.validate("TPL1", "G1", None, Some("3"), None));
    assert_eq!(request_line(&wire), "POST /template/TPL1/validate");
    assert_eq!(body(&wire), json!({ "groupId": "G1", "version": 3 }));

    let wire =
        capture!(client => client.templates.validate("TPL1", "G1", None, Some("published"), None));
    assert_eq!(
        body(&wire),
        json!({ "groupId": "G1", "version": "published" })
    );

    let template = rcs_body();
    let wire =
        capture!(client => client.templates.validate("TPL1", "G1", Some(&template), None, None));
    assert_eq!(body(&wire), json!({ "groupId": "G1", "body": template }));
}

#[tokio::test]
async fn upload_media_posts_file_and_group_id_as_multipart() {
    let wire =
        capture!(client => client.templates.upload_media("G1", b"PNG".to_vec(), "hero.png", None));
    assert_eq!(request_line(&wire), "POST /template/media");
    assert!(wire
        .to_lowercase()
        .contains("content-type: multipart/form-data; boundary="));
    assert!(wire.contains("name=\"file\"; filename=\"hero.png\""));
    assert!(wire.contains("\r\n\r\nPNG\r\n"));
    assert!(wire.contains("name=\"groupId\"\r\n\r\nG1\r\n"));
}

/// True when the call was rejected client-side, before any transport.
fn rejected(result: Result<SignalHouseResponse, SignalHouseError>) -> bool {
    matches!(result, Err(SignalHouseError::Validation(_)))
}

#[tokio::test]
async fn missing_inputs_fail_before_transport() {
    let client = offline();
    let mt = &client.templates;
    let template = rcs_body();
    assert!(rejected(
        mt.create("", "Welcome", &template, None, None).await
    ));
    assert!(rejected(mt.create("G1", "", &template, None, None).await));
    assert!(rejected(
        mt.create("G1", "Welcome", &Value::Null, None, None).await
    ));
    assert!(rejected(mt.list(GetTemplatesParams::default(), None).await));
    assert!(rejected(mt.get("", "G1", None, None).await));
    assert!(rejected(mt.get("TPL1", "", None, None).await));
    assert!(rejected(
        mt.update(
            UpdateTemplateParams {
                template_id: "",
                group_id: "G1",
                ..Default::default()
            },
            None
        )
        .await
    ));
    assert!(rejected(mt.publish("TPL1", "", None).await));
    assert!(rejected(mt.archive("", "G1", None).await));
    assert!(rejected(mt.versions("", "G1", None, None, None).await));
    assert!(rejected(mt.validate("TPL1", "G1", None, None, None).await));
    assert!(rejected(
        mt.validate("TPL1", "G1", Some(&template), Some("draft"), None)
            .await
    ));
    assert!(rejected(mt.capabilities("", None, None, None).await));
    assert!(rejected(
        mt.upload_media("G1", Vec::new(), "hero.png", None).await
    ));
}
