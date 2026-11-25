//! Request-contract coverage for the `search` terms on the subgroup, brand, campaign and number lists.
//! Run from sdks/rust: `cargo test`.
use signalhouse_sms::{SignalHouseClient, SignalHouseConfig};
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

/// The request line's target, e.g. `/subgroup?search=apex`.
fn target(wire: &str) -> String {
    wire.lines()
        .next()
        .unwrap()
        .split(' ')
        .nth(1)
        .unwrap()
        .to_string()
}

const SEARCH: &[&str] = &["apex", "displayName:dental"];
const ENCODED_SEARCH: &str = "search=apex&search=displayName%3Adental";

#[tokio::test]
async fn list_reads_send_each_search_term_as_its_own_key() {
    let (client, request) = capture_server();
    client
        .subgroups
        .get_subgroups_with_search(None, Some("G1"), None, None, SEARCH, None)
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        format!("/subgroup?groupId=G1&{ENCODED_SEARCH}")
    );

    let (client, request) = capture_server();
    client
        .brands
        .get_brands_with_search(
            None,
            None,
            Some("G1"),
            None,
            None,
            None,
            None,
            Some(true),
            SEARCH,
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        format!("/brand?groupId=G1&currentTollFreeCarrier=true&{ENCODED_SEARCH}")
    );

    let (client, request) = capture_server();
    client
        .campaigns
        .get_campaigns_with_search(
            None,
            None,
            None,
            Some("G1"),
            None,
            None,
            None,
            None,
            SEARCH,
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        format!("/campaign?groupId=G1&{ENCODED_SEARCH}")
    );

    let (client, request) = capture_server();
    client
        .numbers
        .get_phone_numbers_with_search(
            None,
            None,
            None,
            None,
            None,
            Some("G1"),
            None,
            None,
            None,
            Some(true),
            SEARCH,
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        format!("/number?groupId=G1&released=true&{ENCODED_SEARCH}")
    );
}

#[tokio::test]
async fn list_reads_without_search_are_unchanged() {
    let (client, request) = capture_server();
    client
        .subgroups
        .get_subgroups(None, Some("G1"), Some(2), None, None)
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/subgroup?groupId=G1&page=2"
    );

    let (client, request) = capture_server();
    client
        .subgroups
        .get_subgroups_with_search(None, Some("G1"), None, None, &[], None)
        .await
        .unwrap();
    assert_eq!(target(&request.join().unwrap()), "/subgroup?groupId=G1");

    let (client, request) = capture_server();
    client
        .brands
        .get_brands(None, None, Some("G1"), None, None, None, None, None)
        .await
        .unwrap();
    assert_eq!(target(&request.join().unwrap()), "/brand?groupId=G1");

    let (client, request) = capture_server();
    client
        .campaigns
        .get_campaigns(
            None,
            None,
            None,
            Some("G1"),
            None,
            None,
            None,
            Some("TEN_DLC"),
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        target(&request.join().unwrap()),
        "/campaign?groupId=G1&registrationType=TEN_DLC"
    );

    let (client, request) = capture_server();
    client
        .numbers
        .get_phone_numbers(None, None, None, None, None, Some("G1"), None, None, None)
        .await
        .unwrap();
    assert_eq!(target(&request.join().unwrap()), "/number?groupId=G1");
}
