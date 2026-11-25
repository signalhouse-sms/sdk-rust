//! Request-contract coverage for the voice mobile push namespaces.
//! Run from sdks/rust: `cargo test --test push`.
use serde_json::json;
use signalhouse_sms::{
    RequestOptions, SignalHouseClient, SignalHouseConfig, SignalHouseError, SignalHouseResponse,
};
use std::io::{Read, Write};
use std::net::TcpListener;

const CREDENTIALS: &str = "/voice/api/v1/push-credentials";
const DEVICES: &str = "/voice/api/v1/push-devices";
const DELIVERIES: &str = "/voice/api/v1/push-devices/deliveries";
const ID: &str = "5a3c2f8e-1b7d-4c9a-9e2f-0d4b6a8c1e33";

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

/// The request line, e.g. `GET /voice/api/v1/push-credentials`.
fn request_line(wire: &str) -> String {
    let mut parts = wire.lines().next().unwrap().split(' ');
    format!("{} {}", parts.next().unwrap(), parts.next().unwrap())
}

/// The JSON body after the blank line.
fn body(wire: &str) -> serde_json::Value {
    serde_json::from_str(wire.split("\r\n\r\n").nth(1).unwrap()).unwrap()
}

/// Per-request options carrying the app's voice token, as the push device calls require.
fn voice_token() -> RequestOptions {
    RequestOptions {
        token: Some("voice-jwt".into()),
        ..Default::default()
    }
}

/// True when the request sent the voice token, not the client's API key, as its bearer token.
fn sends_voice_token(wire: &str) -> bool {
    let wire = wire.to_lowercase();
    wire.contains("\r\nauthorization: bearer voice-jwt\r\n")
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
async fn credential_reads_and_delete_use_the_expected_paths() {
    let wire = capture!(client => client.voice.push_credentials.list(None));
    assert_eq!(request_line(&wire), format!("GET {CREDENTIALS}"));

    let wire = capture!(client => client.voice.push_credentials.get(ID, None));
    assert_eq!(request_line(&wire), format!("GET {CREDENTIALS}/{ID}"));

    let wire = capture!(client => client.voice.push_credentials.get("a/b", None));
    assert_eq!(request_line(&wire), format!("GET {CREDENTIALS}/a%2Fb"));

    let wire = capture!(client => client.voice.push_credentials.delete(ID, None));
    assert_eq!(request_line(&wire), format!("DELETE {CREDENTIALS}/{ID}"));
}

#[tokio::test]
async fn credential_writes_send_the_credential() {
    let apns = json!({
        "provider": "apns", "keyId": "ABC123DEFG", "teamId": "TEAM123456",
        "bundleId": "com.example.app", "privateKey": "p8-key-contents",
    });
    let wire = capture!(client => client.voice.push_credentials.create(&apns, None));
    assert_eq!(request_line(&wire), format!("POST {CREDENTIALS}"));
    assert_eq!(body(&wire), apns);

    let fcm = json!({ "provider": "fcm", "serviceAccount": "{}" });
    let wire = capture!(client => client.voice.push_credentials.update(ID, &fcm, None));
    assert_eq!(request_line(&wire), format!("PUT {CREDENTIALS}/{ID}"));
    assert_eq!(body(&wire), fcm);
}

#[tokio::test]
async fn devices_send_the_voice_token_and_encode_the_device_token() {
    let options = voice_token();
    let token = Some(&options);

    let device = json!({ "provider": "fcm", "deviceToken": "fcm:tok/1" });
    let wire = capture!(client => client.voice.push_devices.register(&device, token));
    assert_eq!(request_line(&wire), format!("POST {DEVICES}"));
    assert_eq!(body(&wire), device);
    assert!(sends_voice_token(&wire));

    let wire = capture!(client => client.voice.push_devices.unregister("fcm:tok/1", token));
    let expected = format!("DELETE {DEVICES}/fcm%3Atok%2F1");
    assert_eq!(request_line(&wire), expected);
    assert!(sends_voice_token(&wire));

    let wire = capture!(client => client.voice.push_devices.list_deliveries(Some(5), token));
    assert_eq!(request_line(&wire), format!("GET {DELIVERIES}?limit=5"));
    assert!(sends_voice_token(&wire));

    let wire = capture!(client => client.voice.push_devices.list_deliveries(None, token));
    assert_eq!(request_line(&wire), format!("GET {DELIVERIES}"));
    assert!(sends_voice_token(&wire));
}

#[tokio::test]
async fn missing_ids_fail_before_transport() {
    let client = offline();
    let credentials = &client.voice.push_credentials;
    let devices = &client.voice.push_devices;
    let fcm = json!({ "provider": "fcm", "serviceAccount": "{}" });
    assert!(rejected(credentials.get("", None).await));
    assert!(rejected(credentials.update("", &fcm, None).await));
    assert!(rejected(credentials.delete("", None).await));
    assert!(rejected(devices.unregister("", None).await));
}
