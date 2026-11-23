//! The PEP over the TLS transport (feature `tls`). The test starts a small PDP over TLS on
//! the loopback address. The PDP answers from a script.
//!
//! The files of `tests/data` are for these tests only. The CA, the server certificate and the
//! key were made for the tests. They protect nothing.
#![cfg(feature = "tls")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use common::*;
use procurauth_pep::clock::ManualClock;
use procurauth_pep::config::PepConfig;
use procurauth_pep::operation::{Operation, node};
use procurauth_pep::refusal::UnavailableCause;
use procurauth_pep::tls::TlsTransport;
use procurauth_pep::transport::{PdpTransport, TransportError};
use procurauth_pep::{Pep, Refusal};
use rustls::server::ServerConnection;
use rustls::{ServerConfig, StreamOwned, SupportedProtocolVersion};
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use serde_json::{Value, json};

const WAIT: Duration = Duration::from_secs(3);

fn data(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data")
        .join(name)
}

fn server_config(versions: &[&'static SupportedProtocolVersion]) -> Arc<ServerConfig> {
    let cert = CertificateDer::from_pem_file(data("tls-test-server.pem")).unwrap();
    let key = PrivateKeyDer::from_pem_file(data("tls-test-server-key.pem")).unwrap();
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    Arc::new(
        ServerConfig::builder_with_provider(provider)
            .with_protocol_versions(versions)
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .unwrap(),
    )
}

/// One request that the PDP read: the request line, the headers and the body.
#[derive(Clone, Debug)]
struct Request {
    line: String,
    head: String,
    body: Vec<u8>,
}

/// What the PDP does with a request.
enum Reply {
    /// Send this answer and keep the connection.
    Answer(u16, Value),
    /// Send this answer, then end the TLS session and close the connection.
    AnswerAndClose(u16, Value),
    /// Close the connection with no answer.
    Close,
}

struct TlsPdp {
    url: String,
    accepted: Arc<AtomicUsize>,
    requests: Arc<Mutex<Vec<Request>>>,
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn read_request(stream: &mut impl Read) -> Option<Request> {
    let mut data = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        if let Some(end) = find(&data, b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&data[..end]).into_owned();
            let length = head
                .to_lowercase()
                .lines()
                .find_map(|l| l.strip_prefix("content-length:").map(str::to_owned))
                .and_then(|v| v.trim().parse::<usize>().ok())
                .unwrap_or(0);
            if data.len() >= end + 4 + length {
                return Some(Request {
                    line: head.lines().next().unwrap_or("").to_owned(),
                    head,
                    body: data[end + 4..end + 4 + length].to_vec(),
                });
            }
        }
        match stream.read(&mut chunk) {
            Ok(0) | Err(_) => return None,
            Ok(n) => data.extend_from_slice(&chunk[..n]),
        }
    }
}

fn write_answer(stream: &mut impl Write, status: u16, body: &Value) -> bool {
    let body = body.to_string();
    let text = format!(
        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(text.as_bytes()).is_ok() && stream.flush().is_ok()
}

/// Start a PDP over TLS on this address. `script` answers each request.
fn serve_on<F>(address: &str, config: Arc<ServerConfig>, script: F) -> TlsPdp
where
    F: Fn(&Request) -> Reply + Send + Sync + 'static,
{
    let listener = TcpListener::bind(address).unwrap();
    let url = format!("https://{}", listener.local_addr().unwrap());
    let accepted = Arc::new(AtomicUsize::new(0));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let (count, seen, script) = (
        Arc::clone(&accepted),
        Arc::clone(&requests),
        Arc::new(script),
    );
    thread::spawn(move || {
        for tcp in listener.incoming() {
            let Ok(tcp) = tcp else { return };
            count.fetch_add(1, Ordering::SeqCst);
            let (config, seen, script) =
                (Arc::clone(&config), Arc::clone(&seen), Arc::clone(&script));
            thread::spawn(move || {
                let conn = ServerConnection::new(config).unwrap();
                let mut tls = StreamOwned::new(conn, tcp);
                while let Some(request) = read_request(&mut tls) {
                    seen.lock().unwrap().push(request.clone());
                    match script(&request) {
                        Reply::Answer(status, body) => {
                            if !write_answer(&mut tls, status, &body) {
                                return;
                            }
                        }
                        Reply::AnswerAndClose(status, body) => {
                            write_answer(&mut tls, status, &body);
                            tls.conn.send_close_notify();
                            let _ = tls.flush();
                            return;
                        }
                        Reply::Close => return,
                    }
                }
            });
        }
    });
    TlsPdp {
        url,
        accepted,
        requests,
    }
}

fn serve<F>(script: F) -> TlsPdp
where
    F: Fn(&Request) -> Reply + Send + Sync + 'static,
{
    serve_on(
        "127.0.0.1:0",
        server_config(&[&rustls::version::TLS13, &rustls::version::TLS12]),
        script,
    )
}

/// A PDP with no metadata document: it answers `true` to each question.
fn yes_pdp(request: &Request) -> Reply {
    if request.line.starts_with("GET ") {
        Reply::Answer(404, json!({}))
    } else {
        Reply::Answer(200, json!({"decision": true}))
    }
}

/// The configuration of a PEP for a PDP over TLS. It trusts the test CA only.
fn tls_config(url: &str, edge: &Edge, ca: &str) -> PepConfig {
    let mut cfg = config(url, edge);
    cfg.pdp.allow_plain_http = false;
    cfg.pdp.tls_system_roots = false;
    cfg.pdp.tls_ca_file = Some(data(ca));
    cfg
}

fn tls_pep(url: &str, ca: &str) -> (Pep, Edge, Arc<TlsTransport>) {
    let edge = Edge::new();
    let cfg = tls_config(url, &edge, ca);
    let transport = Arc::new(TlsTransport::new(&cfg.pdp).unwrap());
    let pep = Pep::with_clock(
        cfg,
        datastore_document(),
        transport.clone(),
        Arc::new(ManualClock::at_seconds(NOW)),
    )
    .unwrap();
    (pep, edge, transport)
}

fn orders() -> Operation {
    Operation::new(
        "data.read",
        node(
            SERVICE,
            &[("host", "prod"), ("database", "test2"), ("table", "orders")],
        )
        .unwrap(),
    )
}

#[test]
fn p_10_0_the_pep_asks_the_pdp_over_tls_and_reuses_the_connection() {
    let pdp = serve(yes_pdp);
    let (pep, edge, transport) = tls_pep(&pdp.url, "tls-test-ca.pem");
    let who = pep
        .authenticate([(HEADER, edge.token("alice").as_str())])
        .unwrap();
    for _ in 0..2 {
        // `check` asks without `context.usage`: the PDP answers `true` with no units.
        let outcome = pep.check(&who, &orders());
        assert!(outcome.is_allowed(), "{outcome:?}");
    }
    let requests = pdp.requests.lock().unwrap().clone();
    let lines: Vec<&str> = requests.iter().map(|r| r.line.as_str()).collect();
    assert_eq!(
        lines,
        [
            "GET /.well-known/authzen-configuration HTTP/1.1",
            "POST /access/v1/evaluation HTTP/1.1",
            "POST /access/v1/evaluation HTTP/1.1",
        ]
    );
    assert!(
        requests[1]
            .head
            .contains("\r\nAuthorization: Bearer test-credential-of-the-service"),
        "{}",
        requests[1].head
    );
    let question: Value = serde_json::from_slice(&requests[1].body).unwrap();
    assert_eq!(question["subject"]["id"], "alice");
    // The host header has no port 443, and one connection served the three requests (10.7).
    assert!(requests[0].head.contains("\r\nHost: 127.0.0.1:"));
    assert_eq!(pdp.accepted.load(Ordering::SeqCst), 1);
    assert_eq!(transport.open_connections(), 1);
    assert_eq!(transport.trusted_roots(), 1);
}

#[test]
fn p_10_0_a_pdp_with_tls_1_2_only_is_accepted() {
    let pdp = serve_on(
        "127.0.0.1:0",
        server_config(&[&rustls::version::TLS12]),
        yes_pdp,
    );
    let (pep, edge, _) = tls_pep(&pdp.url, "tls-test-ca.pem");
    let who = pep
        .authenticate([(HEADER, edge.token("alice").as_str())])
        .unwrap();
    assert!(pep.check(&who, &orders()).is_allowed());
}

#[test]
fn p_10_6_a_certificate_of_an_unknown_ca_fails_before_the_request() {
    let pdp = serve(yes_pdp);
    let (pep, edge, transport) = tls_pep(&pdp.url, "tls-test-other-ca.pem");
    let error = transport
        .post_json(
            &format!("{}/access/v1/evaluation", pdp.url),
            &[],
            b"{}",
            WAIT,
        )
        .unwrap_err();
    assert!(error.request_was_not_sent(), "{error}");
    assert!(error.to_string().contains("handshake"), "{error}");
    let who = pep
        .authenticate([(HEADER, edge.token("alice").as_str())])
        .unwrap();
    let refusal = pep.check(&who, &orders()).into_result().unwrap_err();
    assert!(
        matches!(
            refusal,
            Refusal::Unavailable(UnavailableCause::Transport(_))
        ),
        "{refusal}"
    );
    assert_eq!(refusal.status(), 503);
    assert!(pdp.requests.lock().unwrap().is_empty());
    assert_eq!(transport.open_connections(), 0);
}

#[test]
fn p_10_6_a_server_name_that_the_certificate_does_not_hold_fails_before_the_request() {
    // The certificate holds 127.0.0.1 and localhost. The URL names 127.0.0.2.
    let pdp = serve_on(
        "127.0.0.2:0",
        server_config(&[&rustls::version::TLS13]),
        yes_pdp,
    );
    let (_, _, transport) = tls_pep(&pdp.url, "tls-test-ca.pem");
    let error = transport
        .get_json(&format!("{}/a", pdp.url), &[], WAIT)
        .unwrap_err();
    assert!(error.request_was_not_sent(), "{error}");
    assert!(pdp.requests.lock().unwrap().is_empty());
}

#[test]
fn p_10_7_a_session_that_the_pdp_ended_is_not_reused() {
    let pdp = serve(|_| Reply::AnswerAndClose(200, json!({"ok": true})));
    let (_, _, transport) = tls_pep(&pdp.url, "tls-test-ca.pem");
    let url = format!("{}/a", pdp.url);
    assert_eq!(transport.get_json(&url, &[], WAIT).unwrap().status, 200);
    thread::sleep(Duration::from_millis(100));
    assert_eq!(transport.get_json(&url, &[], WAIT).unwrap().status, 200);
    assert_eq!(pdp.accepted.load(Ordering::SeqCst), 2);
}

#[test]
fn p_10_6_a_connection_that_closes_after_the_request_is_a_lost_answer() {
    let pdp = serve(|_| Reply::Close);
    let (_, _, transport) = tls_pep(&pdp.url, "tls-test-ca.pem");
    let error = transport
        .post_json(&format!("{}/a", pdp.url), &[], b"{}", WAIT)
        .unwrap_err();
    assert!(matches!(error, TransportError::Lost(_)), "{error}");
    assert!(!error.request_was_not_sent());
}

#[test]
fn p_10_6_a_silent_pdp_gives_a_timeout_after_the_request() {
    let pdp = serve(|_| {
        thread::sleep(Duration::from_secs(2));
        Reply::Close
    });
    let (_, _, transport) = tls_pep(&pdp.url, "tls-test-ca.pem");
    let error = transport
        .post_json(
            &format!("{}/a", pdp.url),
            &[],
            b"{}",
            Duration::from_millis(300),
        )
        .unwrap_err();
    assert_eq!(error, TransportError::Timeout);
}

#[test]
fn p_10_6_a_server_that_never_ends_the_handshake_gives_a_failure_before_the_request() {
    // A TCP server that accepts and sends nothing: the handshake waits until the time runs out.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("https://{}/a", listener.local_addr().unwrap());
    let keep: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));
    let copy = Arc::clone(&keep);
    thread::spawn(move || {
        for tcp in listener.incoming().flatten() {
            copy.lock().unwrap().push(tcp);
        }
    });
    let edge = Edge::new();
    let mut cfg = tls_config("https://127.0.0.1:1", &edge, "tls-test-ca.pem");
    cfg.pdp.request_timeout = Duration::from_millis(300);
    let transport = TlsTransport::new(&cfg.pdp).unwrap();
    let error = transport.post_json(&url, &[], b"{}", WAIT).unwrap_err();
    assert!(error.request_was_not_sent(), "{error}");
}

#[test]
fn the_transport_for_the_url_is_chosen_by_the_scheme() {
    let edge = Edge::new();
    let tls = procurauth_pep::transport::for_pdp(
        &tls_config("https://127.0.0.1:1", &edge, "tls-test-ca.pem").pdp,
    )
    .unwrap();
    assert!(tls.supports_scheme("https"));
    let plain =
        procurauth_pep::transport::for_pdp(&config("http://127.0.0.1:1", &edge).pdp).unwrap();
    assert!(plain.supports_scheme("http"));
    let mut bad = tls_config("https://127.0.0.1:1", &edge, "no-such-ca.pem");
    bad.pdp.tls_system_roots = false;
    let errors = procurauth_pep::transport::for_pdp(&bad.pdp).err().unwrap();
    assert_eq!(errors[0].key, "pdp.tls_ca_file");
}
