//! Request-contract coverage for the voice call flows namespace.
//! Run from sdks/rust: `cargo test --test call_flows`.
use serde_json::json;
use signalhouse_sms::domains::voice::ListCallFlowsParams;
use signalhouse_sms::{
    RequestOptions, SignalHouseClient, SignalHouseConfig, SignalHouseError, SignalHouseResponse,
};
use std::io::{Read, Write};
use std::net::TcpListener;

const BASE: &str = "/voice/api/v1/call-flows";
const ID: &str = "0088947b-7821-4853-bfe6-744fce926de2";

/// A one-shot capture server that reads the whole request before answering.
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

/// The request line, e.g. `GET /voice/api/v1/call-flows/catalog`.
fn request_line(wire: &str) -> String {
    let mut parts = wire.lines().next().unwrap().split(' ');
    format!("{} {}", parts.next().unwrap(), parts.next().unwrap())
}

/// The JSON body after the blank line.
fn body(wire: &str) -> serde_json::Value {
    serde_json::from_str(wire.split("\r\n\r\n").nth(1).unwrap()).unwrap()
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
async fn reads_use_the_expected_paths() {
    let wire = capture!(client => client.voice.call_flows.list(Default::default(), None));
    assert_eq!(request_line(&wire), format!("GET {BASE}"));

    let wire = capture!(client => client.voice.call_flows.catalog(None));
    assert_eq!(request_line(&wire), format!("GET {BASE}/catalog"));

    let wire = capture!(client => client.voice.call_flows.get(ID, None));
    assert_eq!(request_line(&wire), format!("GET {BASE}/{ID}"));

    let wire = capture!(client => client.voice.call_flows.get("a/b", None));
    assert_eq!(request_line(&wire), format!("GET {BASE}/a%2Fb"));
}

#[tokio::test]
async fn list_sends_only_the_filters_that_are_set() {
    let wire = capture!(client => client.voice.call_flows.list(
        ListCallFlowsParams {
            page: Some(2),
            subgroup_id: Some("S1"),
            include_deleted: Some(true),
            ..Default::default()
        },
        None
    ));
    let expected = format!("GET {BASE}?page=2&subgroupId=S1&includeDeleted=true");
    assert_eq!(request_line(&wire), expected);
}

#[tokio::test]
async fn writes_use_the_expected_methods_and_bodies() {
    let flow = json!({ "name": "Main line", "subgroupId": "S1" });
    let wire = capture!(client => client.voice.call_flows.create(&flow, None));
    assert_eq!(request_line(&wire), format!("POST {BASE}"));
    assert_eq!(body(&wire), flow);

    let update = json!({ "status": "DISABLED" });
    let wire = capture!(client => client.voice.call_flows.update(ID, &update, None));
    assert_eq!(request_line(&wire), format!("PATCH {BASE}/{ID}"));
    assert_eq!(body(&wire), update);

    let draft = json!({ "nodes": [], "edges": [] });
    let wire = capture!(client => client.voice.call_flows.save_draft(ID, &draft, None));
    assert_eq!(request_line(&wire), format!("PUT {BASE}/{ID}/draft"));
    assert_eq!(body(&wire), draft);

    let wire = capture!(client => client.voice.call_flows.delete(ID, None));
    assert_eq!(request_line(&wire), format!("DELETE {BASE}/{ID}"));
}

#[tokio::test]
async fn input_less_posts_send_an_empty_object() {
    let wire = capture!(client => client.voice.call_flows.publish(ID, None));
    assert_eq!(request_line(&wire), format!("POST {BASE}/{ID}/publish"));
    assert_eq!(body(&wire), json!({}));

    let wire = capture!(client => client.voice.call_flows.restore(ID, None));
    assert_eq!(request_line(&wire), format!("POST {BASE}/{ID}/restore"));
    assert_eq!(body(&wire), json!({}));

    let wire = capture!(client => client.voice.call_flows.duplicate(ID, None, None));
    assert_eq!(request_line(&wire), format!("POST {BASE}/{ID}/duplicate"));
    assert_eq!(body(&wire), json!({}));

    let wire = capture!(client => client.voice.call_flows.duplicate(ID, Some("Weekend"), None));
    assert_eq!(body(&wire), json!({ "name": "Weekend" }));
}

#[tokio::test]
async fn request_options_pass_through() {
    let options = RequestOptions {
        token: Some("tok-1".into()),
        idempotency_key: Some("k1".into()),
        ..Default::default()
    };
    let wire = capture!(client => client.voice.call_flows.publish(ID, Some(&options)));
    let wire = wire.to_lowercase();
    assert!(wire.contains("\r\nauthorization: bearer tok-1\r\n"));
    assert!(wire.contains("\r\nidempotency-key: k1\r\n"));
}

#[tokio::test]
async fn missing_ids_fail_before_transport() {
    let client = offline();
    let flows = &client.voice.call_flows;
    let draft = json!({ "nodes": [], "edges": [] });
    assert!(rejected(flows.get("", None).await));
    assert!(rejected(flows.publish("", None).await));
    assert!(rejected(flows.save_draft("", &draft, None).await));
}
