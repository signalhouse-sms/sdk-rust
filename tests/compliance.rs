//! Request-contract coverage for the compliance namespace (customer case methods).
//! Run from sdks/rust: `cargo test --test compliance`.
use serde_json::{json, Value};
use signalhouse_sms::domains::compliance::GetComplianceCasesParams;
use signalhouse_sms::{RequestOptions, SignalHouseClient, SignalHouseConfig, SignalHouseError};
use std::io::{Read, Write};
use std::net::TcpListener;

/// A one-shot capture server that reads the whole request (multipart included) before answering.
fn capture_server() -> (SignalHouseClient, std::thread::JoinHandle<String>) {
    response_server(200, "application/json", b"{}".to_vec())
}

/// Capture a request and respond with an exact binary body and status.
fn response_server(
    status: u16,
    content_type: &str,
    body: Vec<u8>,
) -> (SignalHouseClient, std::thread::JoinHandle<String>) {
    let response_header = format!("HTTP/1.1 {status} Response\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
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
        stream.write_all(response_header.as_bytes()).unwrap();
        stream.write_all(&body).unwrap();
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

/// The request line, e.g. `GET /compliance/cases`.
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
fn rejected<T>(result: Result<T, SignalHouseError>) -> bool {
    matches!(result, Err(SignalHouseError::Validation(_)))
}

#[tokio::test]
async fn get_cases_sends_only_the_given_filters() {
    let wire = capture!(client => client.compliance.get_cases(
        GetComplianceCasesParams {
            group_id: Some("G0000001"),
            status: Some("OPEN"),
            stage: Some("customer"),
            object_type: Some("CAMPAIGN"),
            object_id: Some("C1"),
            subgroup_id: Some("S1"),
            suspended: Some(true),
            from: Some("2026-11-01T00:00:00Z"),
            to: Some("2026-11-13T00:00:00Z"),
            sort: Some("fixBy"),
            page: Some(2),
            limit: Some(50),
        },
        None
    ));
    assert_eq!(
        request_line(&wire),
        "GET /compliance/cases?groupId=G0000001&status=OPEN&stage=customer&objectType=CAMPAIGN&objectId=C1&subgroupId=S1&suspended=true&from=2026-11-01T00%3A00%3A00Z&to=2026-11-13T00%3A00%3A00Z&sort=fixBy&page=2&limit=50"
    );
    assert_eq!(raw_body(&wire), "");

    let wire = capture!(client => client.compliance.get_cases(
        GetComplianceCasesParams {
            suspended: Some(false),
            limit: Some(200),
            ..Default::default()
        },
        None
    ));
    assert_eq!(
        request_line(&wire),
        "GET /compliance/cases?suspended=false&limit=200"
    );

    let wire = capture!(client => client.compliance.get_cases(Default::default(), None));
    assert_eq!(request_line(&wire), "GET /compliance/cases");
}

#[tokio::test]
async fn get_case_reads_the_encoded_case() {
    let wire = capture!(client => client.compliance.get_case("CASE-1/2", None));
    assert_eq!(request_line(&wire), "GET /compliance/cases/CASE-1%2F2");
    assert_eq!(raw_body(&wire), "");
    assert!(rejected(offline().compliance.get_case("", None).await));
}

#[tokio::test]
async fn answer_requirement_puts_only_the_given_keys() {
    let options = RequestOptions {
        idempotency_key: Some("k1".into()),
        ..Default::default()
    };
    let file_ids = vec!["F1".to_string(), "F2".to_string()];
    let wire = capture!(client => client.compliance.answer_requirement(
        "CASE-1",
        "CASE-1-1",
        Some("Removed the number"),
        Some(&file_ids),
        Some(&options)
    ));
    assert_eq!(
        request_line(&wire),
        "PUT /compliance/cases/CASE-1/requirements/CASE-1-1"
    );
    assert!(wire.to_lowercase().contains("\r\nidempotency-key: k1\r\n"));
    assert_eq!(
        body(&wire),
        json!({ "answer": "Removed the number", "fileIds": ["F1", "F2"] })
    );

    let wire = capture!(client => client.compliance.answer_requirement(
        "CASE-1", "CASE-1-1", Some("Done"), None, None
    ));
    assert_eq!(body(&wire), json!({ "answer": "Done" }));

    let wire = capture!(client => client.compliance.answer_requirement(
        "CASE-1", "CASE-1-1", None, Some(&file_ids), None
    ));
    assert_eq!(body(&wire), json!({ "fileIds": ["F1", "F2"] }));

    let wire = capture!(client => client.compliance.answer_requirement(
        "CASE 1/x", "CASE 1/x-1", None, None, None
    ));
    assert_eq!(
        request_line(&wire),
        "PUT /compliance/cases/CASE%201%2Fx/requirements/CASE%201%2Fx-1"
    );
    assert_eq!(body(&wire), json!({}));

    let compliance = &offline().compliance;
    assert!(rejected(
        compliance
            .answer_requirement("", "CASE-1-1", Some("Done"), None, None)
            .await
    ));
    assert!(rejected(
        compliance
            .answer_requirement("CASE-1", "", Some("Done"), None, None)
            .await
    ));
}

#[tokio::test]
async fn upload_file_posts_case_requirement_and_file_as_multipart() {
    let wire = capture!(client => client.compliance.upload_file(
        "CASE-1",
        Some("CASE-1-1"),
        b"PDF".to_vec(),
        "records.pdf".to_string(),
        None
    ));
    assert_eq!(request_line(&wire), "POST /compliance/files");
    assert!(wire
        .to_lowercase()
        .contains("content-type: multipart/form-data; boundary="));
    assert!(wire.contains("name=\"caseId\"\r\n\r\nCASE-1\r\n"));
    assert!(wire.contains("name=\"requirementId\"\r\n\r\nCASE-1-1\r\n"));
    assert!(wire.contains("name=\"file\"; filename=\"records.pdf\""));
    assert!(wire.contains("\r\n\r\nPDF\r\n"));

    let wire = capture!(client => client.compliance.upload_file(
        "CASE-1",
        None,
        b"CSV".to_vec(),
        "optins.csv".to_string(),
        None
    ));
    assert!(wire.contains("name=\"caseId\"\r\n\r\nCASE-1\r\n"));
    assert!(!wire.contains("name=\"requirementId\""));
    assert!(wire.contains("name=\"file\"; filename=\"optins.csv\""));

    let compliance = &offline().compliance;
    assert!(rejected(
        compliance
            .upload_file("", None, b"PDF".to_vec(), "records.pdf".to_string(), None)
            .await
    ));
    assert!(rejected(
        compliance
            .upload_file("CASE-1", None, Vec::new(), "records.pdf".to_string(), None)
            .await
    ));
}

#[tokio::test]
async fn get_file_reads_the_encoded_file() {
    let wire = capture!(client => client.compliance.get_file("CF1/2", None));
    assert_eq!(request_line(&wire), "GET /compliance/files/CF1%2F2");
    assert_eq!(raw_body(&wire), "");
    assert!(rejected(offline().compliance.get_file("", None).await));
}

#[tokio::test]
async fn get_file_preserves_binary_and_empty_bodies_and_request_options() {
    for (content_type, bytes) in [
        ("application/pdf", b"%PDF\0\xff\x80".to_vec()),
        ("image/png", b"\x89PNG\r\n\x1a\n\0\xff".to_vec()),
        ("text/csv", b"a,b\n".to_vec()),
        ("text/plain", Vec::new()),
    ] {
        let (client, request) = response_server(200, content_type, bytes.clone());
        let options = RequestOptions {
            token: Some("customer-token".into()),
            ..Default::default()
        };
        let response = client
            .compliance
            .get_file("CF1", Some(&options))
            .await
            .unwrap();
        assert!(response.success);
        assert_eq!(response.status, 200);
        assert_eq!(response.data, Some(bytes));
        assert!(response.error.is_none());
        assert!(request.join().unwrap().contains("Bearer customer-token"));
    }
}

#[tokio::test]
async fn get_file_keeps_normal_api_errors() {
    for (body, expected) in [
        (r#"{"message":"Forbidden"}"#, "Forbidden"),
        ("Unavailable", "Unavailable"),
    ] {
        let (client, request) = response_server(403, "application/json", body.as_bytes().to_vec());
        let response = client.compliance.get_file("CF1", None).await.unwrap();
        assert!(!response.success);
        assert_eq!(response.status, 403);
        assert_eq!(response.data, None);
        assert_eq!(response.error.as_deref(), Some(expected));
        request.join().unwrap();
    }
}
