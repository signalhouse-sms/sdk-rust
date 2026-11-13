//! Request-contract coverage for the verify namespace (SIM Swap and Number Verify).
//! Run from sdks/rust: `cargo test --test verify`.
use serde_json::{json, Value};
use signalhouse_sms::{
    RequestOptions, SignalHouseClient, SignalHouseConfig, SignalHouseError, SignalHouseResponse,
};
use std::io::{Read, Write};
use std::net::TcpListener;

const PHONE: &str = "+12025550123";

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

/// The request line, e.g. `POST /verify/sim-swap`.
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
async fn check_sim_swap_posts_only_the_given_fields() {
    let options = RequestOptions {
        idempotency_key: Some("k1".into()),
        ..Default::default()
    };
    let wire = capture!(client => client.verify.check_sim_swap(PHONE, Some(240), Some(true), Some(&options)));
    assert_eq!(request_line(&wire), "POST /verify/sim-swap");
    assert!(wire.to_lowercase().contains("\r\nidempotency-key: k1\r\n"));
    assert_eq!(
        body(&wire),
        json!({ "phoneNumber": PHONE, "maxAge": 240, "includeSwapDate": true })
    );

    let wire = capture!(client => client.verify.check_sim_swap(PHONE, None, Some(false), None));
    assert_eq!(
        body(&wire),
        json!({ "phoneNumber": PHONE, "includeSwapDate": false })
    );

    let wire = capture!(client => client.verify.check_sim_swap(PHONE, None, None, None));
    assert_eq!(body(&wire), json!({ "phoneNumber": PHONE }));
    assert!(rejected(
        offline().verify.check_sim_swap("", None, None, None).await
    ));
}

#[tokio::test]
async fn start_number_verification_posts_the_hint_and_redirect() {
    let wire = capture!(client => client.verify.start_number_verification(
        PHONE,
        Some(PHONE),
        Some("https://example.com/done"),
        None
    ));
    assert_eq!(request_line(&wire), "POST /verify/number");
    assert_eq!(
        body(&wire),
        json!({ "phoneNumber": PHONE, "devicePhoneNumber": PHONE, "redirectUrl": "https://example.com/done" })
    );

    let wire = capture!(client => client.verify.start_number_verification(PHONE, None, None, None));
    assert_eq!(body(&wire), json!({ "phoneNumber": PHONE }));
    assert!(rejected(
        offline()
            .verify
            .start_number_verification("", None, None, None)
            .await
    ));
}

#[tokio::test]
async fn get_number_verification_reads_the_encoded_check() {
    let wire = capture!(client => client.verify.get_number_verification("NVC1/2", None));
    assert_eq!(request_line(&wire), "GET /verify/number/NVC1%2F2");
    assert_eq!(raw_body(&wire), "");
    assert!(rejected(
        offline().verify.get_number_verification("", None).await
    ));
}
