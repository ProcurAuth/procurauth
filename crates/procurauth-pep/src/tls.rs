//! A transport over TLS for the link to the PDP. It needs the feature `tls`.
//!
//! [`TlsTransport`] is the client of [`crate::http_client`] over a TLS session.
//! It uses rustls with the crypto provider aws-lc-rs. It accepts TLS 1.2 and
//! TLS 1.3 only. The name of the server comes from the host of the URL of the PDP.
//!
//! # Trusted roots
//!
//! The transport trusts the CA bundle of the system and an optional extra CA
//! file. Both are PEM files that the transport reads one time, in
//! [`TlsTransport::new`]. It reads no environment variable. Thus `SSL_CERT_FILE`
//! and `SSL_CERT_DIR` have no effect.
//!
//! - `pdp.tls_system_roots` (default `true`): read the first file of
//!   [`SYSTEM_CA_BUNDLES`] that holds a certificate.
//! - `pdp.tls_ca_file` (default none): also trust each certificate of this file.
//!
//! A transport with no trusted root is a configuration error.
//!
//! # Failures (10.6)
//!
//! The transport makes the handshake when it opens a connection, before it
//! writes a byte of the request. Thus a failed handshake is
//! [`TransportError::NotSent`]. A
//! failure after the handshake is `Lost` or `Timeout`, as for the plain client.

use std::fs::File;
use std::io::{self, ErrorKind, Read};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rustls::client::ClientConnection;
use rustls::{ClientConfig, RootCertStore, StreamOwned};
use rustls_pki_types::pem::{PemObject, SectionKind};
use rustls_pki_types::{CertificateDer, ServerName};

use crate::config::{ConfigError, PdpConfig};
use crate::http_client::{Client, Connector, HttpTransportConfig, Wire};
use crate::transport::{HttpResponse, PdpTransport, TransportError};
use crate::url::{Scheme, Url};

/// The CA bundles of the system, in the order that the transport tries them.
/// The transport reads the first file that exists and holds a certificate.
///
/// | Path | System |
/// | --- | --- |
/// | `/etc/ssl/certs/ca-certificates.crt` | Debian, Ubuntu |
/// | `/etc/pki/tls/certs/ca-bundle.crt` | Fedora, RHEL |
/// | `/etc/pki/ca-trust/extracted/pem/tls-ca-bundle.pem` | Fedora, RHEL 7 and later |
/// | `/etc/ssl/cert.pem` | Alpine |
/// | `/etc/ssl/ca-bundle.pem` | openSUSE |
pub const SYSTEM_CA_BUNDLES: [&str; 5] = [
    "/etc/ssl/certs/ca-certificates.crt",
    "/etc/pki/tls/certs/ca-bundle.crt",
    "/etc/pki/ca-trust/extracted/pem/tls-ca-bundle.pem",
    "/etc/ssl/cert.pem",
    "/etc/ssl/ca-bundle.pem",
];

/// The largest CA file that the transport reads, in bytes.
const MAX_CA_FILE_BYTES: u64 = 8 * 1024 * 1024;

/// The most reads that the check of an idle connection makes.
const MAX_IDLE_READS: usize = 8;

/// The roots that the transport trusts, and the system bundle that it read.
#[derive(Debug)]
pub(crate) struct Roots {
    store: RootCertStore,
    /// The CA bundle of the system that the transport read.
    system_bundle: Option<PathBuf>,
}

/// An HTTP/1.1 client over TLS 1.2 or 1.3, with a small pool of connections, for `https` URLs.
pub struct TlsTransport {
    client: Client<TlsConnector>,
    roots: usize,
    system_bundle: Option<PathBuf>,
}

impl std::fmt::Debug for TlsTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TlsTransport")
            .field("config", self.client.config())
            .field("roots", &self.roots)
            .field("system_bundle", &self.system_bundle)
            .finish_non_exhaustive()
    }
}

impl TlsTransport {
    /// A client with the bounds and the trusted roots of the configuration of the link.
    /// The function reads the CA files one time.
    ///
    /// # Errors
    /// Returns an error for `pdp.tls_ca_file` when the transport cannot read the
    /// file. Returns the same error when the file holds no certificate, or a
    /// section that is not a valid certificate.
    /// Returns an error for `pdp.tls_system_roots` when it is true and no file of
    /// [`SYSTEM_CA_BUNDLES`] holds a certificate. Returns an error when the
    /// transport has no trusted root.
    pub fn new(pdp: &PdpConfig) -> Result<Self, Vec<ConfigError>> {
        Self::with_bounds(pdp, HttpTransportConfig::from_pdp(pdp))
    }

    /// A client with these bounds, and the trusted roots of the configuration of the link.
    ///
    /// # Errors
    /// Returns the same errors as [`TlsTransport::new`].
    pub fn with_bounds(
        pdp: &PdpConfig,
        bounds: HttpTransportConfig,
    ) -> Result<Self, Vec<ConfigError>> {
        let roots = load_roots(
            pdp.tls_ca_file.as_deref(),
            pdp.tls_system_roots,
            &SYSTEM_CA_BUNDLES,
        )?;
        let count = roots.store.len();
        let connector = TlsConnector::new(roots.store).map_err(|e| {
            vec![ConfigError::new(
                "pdp.url",
                "an `https` URL that rustls can serve with TLS 1.3 and TLS 1.2",
                format!("a TLS setup that rustls refuses ({e})"),
            )]
        })?;
        Ok(Self {
            client: Client::new(connector, bounds),
            roots: count,
            system_bundle: roots.system_bundle,
        })
    }

    /// The number of trusted roots. Write it to the log at start.
    pub fn trusted_roots(&self) -> usize {
        self.roots
    }

    /// The CA bundle of the system that the transport read, if any. Write it to the log at start.
    pub fn system_bundle(&self) -> Option<&Path> {
        self.system_bundle.as_deref()
    }

    /// The number of connections that exist now, idle or in use.
    pub fn open_connections(&self) -> usize {
        self.client.open_connections()
    }
}

impl PdpTransport for TlsTransport {
    fn supports_scheme(&self, scheme: &str) -> bool {
        scheme == "https"
    }

    fn post_json(
        &self,
        url: &str,
        headers: &[(String, String)],
        body: &[u8],
        timeout: Duration,
    ) -> Result<HttpResponse, TransportError> {
        self.client
            .exchange("POST", url, headers, Some(body), timeout)
    }

    fn get_json(
        &self,
        url: &str,
        headers: &[(String, String)],
        timeout: Duration,
    ) -> Result<HttpResponse, TransportError> {
        self.client.exchange("GET", url, headers, None, timeout)
    }

    fn set_idle_close(&self, idle_close_after: Duration) {
        self.client.set_idle_close(idle_close_after);
    }
}

/// The connector of TLS: a handshake on a new TCP connection.
pub(crate) struct TlsConnector {
    config: Arc<ClientConfig>,
}

impl TlsConnector {
    /// The client setup of rustls: the provider aws-lc-rs, TLS 1.3 and TLS 1.2, and these roots.
    fn new(roots: RootCertStore) -> Result<Self, rustls::Error> {
        let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
        // rustls has no version older than TLS 1.2.
        let mut config = ClientConfig::builder_with_provider(provider)
            .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])?
            .with_root_certificates(roots)
            .with_no_client_auth();
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        Ok(Self {
            config: Arc::new(config),
        })
    }
}

impl Connector for TlsConnector {
    type Stream = StreamOwned<ClientConnection, TcpStream>;

    fn scheme(&self) -> Scheme {
        Scheme::Https
    }

    fn other_scheme(&self) -> &'static str {
        "this client uses TLS: give it an `https` URL"
    }

    fn open(
        &self,
        url: &Url,
        mut socket: TcpStream,
        deadline: Instant,
    ) -> Result<Self::Stream, String> {
        let name = ServerName::try_from(url.host.clone())
            .map_err(|e| format!("the host is not a valid TLS server name: {e}"))?;
        let mut conn = ClientConnection::new(Arc::clone(&self.config), name)
            .map_err(|e| format!("the TLS session cannot start: {e}"))?;
        handshake(&mut conn, &mut socket, deadline)
            .map_err(|e| format!("the TLS handshake failed: {e}"))?;
        Ok(StreamOwned::new(conn, socket))
    }
}

fn time_ran_out() -> io::Error {
    io::Error::new(ErrorKind::TimedOut, "the time ran out")
}

/// Make the whole handshake before `deadline`. No byte of a request goes out before the end.
fn handshake(
    conn: &mut ClientConnection,
    socket: &mut TcpStream,
    deadline: Instant,
) -> io::Result<()> {
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(time_ran_out());
        }
        socket.set_read_timeout(Some(remaining))?;
        socket.set_write_timeout(Some(remaining))?;
        while conn.wants_write() {
            match conn.write_tls(socket) {
                Ok(0) => return Err(io::Error::from(ErrorKind::WriteZero)),
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                    return Err(time_ran_out());
                }
                Err(e) => return Err(e),
            }
        }
        if !conn.is_handshaking() {
            return Ok(());
        }
        match conn.read_tls(socket) {
            Ok(0) => {
                return Err(io::Error::new(
                    ErrorKind::UnexpectedEof,
                    "the PDP closed the connection",
                ));
            }
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                return Err(time_ran_out());
            }
            Err(e) => return Err(e),
        }
        if let Err(error) = conn.process_new_packets() {
            // Send the alert to the PDP, then stop.
            let _ = conn.write_tls(socket);
            return Err(io::Error::new(ErrorKind::InvalidData, error));
        }
    }
}

impl Wire for StreamOwned<ClientConnection, TcpStream> {
    fn socket(&self) -> &TcpStream {
        &self.sock
    }

    /// True when the session has no unread data, the PDP did not end it, and
    /// the PDP did not close the TCP connection. The check reads, with no block,
    /// the TLS records that wait. A session ticket of TLS 1.3 keeps the connection usable.
    fn is_alive(&mut self) -> bool {
        let clean = |conn: &mut ClientConnection| {
            conn.process_new_packets()
                .is_ok_and(|state| state.plaintext_bytes_to_read() == 0 && !state.peer_has_closed())
        };
        if !clean(&mut self.conn) || self.sock.set_nonblocking(true).is_err() {
            return false;
        }
        let mut alive = false;
        for _ in 0..MAX_IDLE_READS {
            match self.conn.read_tls(&mut self.sock) {
                Ok(0) => break,
                Ok(_) => {
                    if !clean(&mut self.conn) {
                        break;
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    alive = true;
                    break;
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(_) => break,
            }
        }
        alive && self.sock.set_nonblocking(false).is_ok()
    }
}

/// Read a whole file, up to the bound.
fn read_bounded(path: &Path) -> io::Result<Vec<u8>> {
    let mut data = Vec::new();
    File::open(path)?
        .take(MAX_CA_FILE_BYTES + 1)
        .read_to_end(&mut data)?;
    if u64::try_from(data.len()).unwrap_or(u64::MAX) > MAX_CA_FILE_BYTES {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("the file has more than {MAX_CA_FILE_BYTES} bytes"),
        ));
    }
    Ok(data)
}

/// Add each certificate of the extra CA file. Every section must be a certificate that rustls accepts.
fn add_ca_file(store: &mut RootCertStore, path: &Path) -> Result<(), String> {
    let data = read_bounded(path).map_err(|e| e.to_string())?;
    let mut count = 0_usize;
    for (index, section) in <(SectionKind, Vec<u8>)>::pem_slice_iter(&data).enumerate() {
        let number = index + 1;
        let (kind, der) = section.map_err(|e| format!("PEM section {number}: {e}"))?;
        if kind != SectionKind::Certificate {
            return Err(format!(
                "PEM section {number} is a {kind:?}, not a certificate"
            ));
        }
        store
            .add(CertificateDer::from(der))
            .map_err(|e| format!("PEM section {number} is not a valid CA certificate: {e}"))?;
        count += 1;
    }
    if count == 0 {
        return Err("the file holds no PEM certificate".to_owned());
    }
    Ok(())
}

/// Add the certificates of the first bundle of the list that holds one. A
/// certificate that rustls refuses is ignored, as a large bundle can hold old ones.
fn add_system_bundle(store: &mut RootCertStore, paths: &[&str]) -> Option<PathBuf> {
    for path in paths {
        let Ok(data) = read_bounded(Path::new(path)) else {
            continue;
        };
        let certificates = CertificateDer::pem_slice_iter(&data).filter_map(Result::ok);
        let (added, _ignored) = store.add_parsable_certificates(certificates);
        if added > 0 {
            return Some(PathBuf::from(path));
        }
    }
    None
}

/// Read the trusted roots. `paths` is the list of the system bundles.
pub(crate) fn load_roots(
    ca_file: Option<&Path>,
    system_roots: bool,
    paths: &[&str],
) -> Result<Roots, Vec<ConfigError>> {
    let mut errors = Vec::new();
    let mut store = RootCertStore::empty();
    let mut system_bundle = None;
    if system_roots {
        system_bundle = add_system_bundle(&mut store, paths);
        if system_bundle.is_none() {
            errors.push(ConfigError::new(
                "pdp.tls_system_roots",
                &format!(
                    "a CA bundle with 1 or more certificates at one of the paths {}, or the value false",
                    paths.join(", ")
                ),
                "true, and no file of the list holds a certificate".to_owned(),
            ));
        }
    }
    if let Some(path) = ca_file
        && let Err(why) = add_ca_file(&mut store, path)
    {
        errors.push(ConfigError::new(
            "pdp.tls_ca_file",
            "a readable PEM file with 1 or more CA certificates and no other section",
            format!("{path:?} ({why})"),
        ));
    }
    if !system_roots && ca_file.is_none() {
        errors.push(ConfigError::new(
            "pdp.tls_ca_file",
            "a PEM file with CA certificates, because `pdp.tls_system_roots` is false and the transport needs 1 or more trusted roots",
            "no value".to_owned(),
        ));
    }
    if errors.is_empty() {
        Ok(Roots {
            store,
            system_bundle,
        })
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Secret;

    fn data(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data")
            .join(name)
    }

    fn keys_of(errors: &[ConfigError]) -> Vec<&str> {
        errors.iter().map(|e| e.key.as_str()).collect()
    }

    fn text(name: &str) -> String {
        data(name).to_string_lossy().into_owned()
    }

    #[test]
    fn the_extra_ca_file_alone_gives_its_roots() {
        let roots = load_roots(Some(&data("tls-test-ca.pem")), false, &[]).unwrap();
        assert_eq!(roots.store.len(), 1);
        assert!(roots.system_bundle.is_none());
    }

    #[test]
    fn the_first_system_bundle_with_a_certificate_is_read_and_the_extra_file_adds_to_it() {
        let missing = text("no-such-bundle.pem");
        let empty = text("tls-test-server-key.pem");
        let bundle = text("tls-test-other-ca.pem");
        let second = text("tls-test-ca.pem");
        let paths = [
            missing.as_str(),
            empty.as_str(),
            bundle.as_str(),
            second.as_str(),
        ];
        let roots = load_roots(Some(&data("tls-test-ca.pem")), true, &paths).unwrap();
        assert_eq!(roots.store.len(), 2);
        assert_eq!(roots.system_bundle, Some(data("tls-test-other-ca.pem")));
    }

    #[test]
    fn no_system_bundle_is_an_error_that_names_the_key() {
        let missing = text("no-such-bundle.pem");
        let errors = load_roots(Some(&data("tls-test-ca.pem")), true, &[&missing]).unwrap_err();
        assert_eq!(keys_of(&errors), ["pdp.tls_system_roots"]);
        assert!(errors[0].expected.contains(&missing), "{}", errors[0]);
        assert_eq!(
            errors[0].found,
            "true, and no file of the list holds a certificate"
        );
    }

    #[test]
    fn no_root_at_all_is_an_error_that_names_the_key() {
        let errors = load_roots(None, false, &SYSTEM_CA_BUNDLES).unwrap_err();
        assert_eq!(keys_of(&errors), ["pdp.tls_ca_file"]);
        assert_eq!(errors[0].found, "no value");
    }

    #[test]
    fn a_bad_extra_ca_file_is_an_error_with_the_path_and_the_cause() {
        for (name, cause) in [
            ("no-such-file.pem", "No such file"),
            ("tls-test-server-key.pem", "not a certificate"),
        ] {
            let errors = load_roots(Some(&data(name)), false, &[]).unwrap_err();
            assert_eq!(keys_of(&errors), ["pdp.tls_ca_file"], "{name}");
            assert!(errors[0].found.contains(name), "{}", errors[0]);
            assert!(errors[0].found.contains(cause), "{}", errors[0]);
        }
    }

    #[test]
    fn a_file_with_no_pem_section_is_not_a_ca_file() {
        let mut store = RootCertStore::empty();
        let why = add_ca_file(
            &mut store,
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("Cargo.toml")
                .as_path(),
        )
        .unwrap_err();
        assert_eq!(why, "the file holds no PEM certificate");
    }

    #[test]
    fn the_transport_takes_https_only_and_reports_its_roots() {
        let mut pdp = PdpConfig::new("https://127.0.0.1:1", Secret::new("credential"));
        pdp.tls_system_roots = false;
        pdp.tls_ca_file = Some(data("tls-test-ca.pem"));
        let transport = TlsTransport::new(&pdp).unwrap();
        assert!(transport.supports_scheme("https"));
        assert!(!transport.supports_scheme("http"));
        assert_eq!(transport.trusted_roots(), 1);
        assert_eq!(transport.system_bundle(), None);
        let error = transport
            .get_json("http://127.0.0.1:1/a", &[], Duration::from_secs(1))
            .unwrap_err();
        assert!(matches!(error, TransportError::Unsupported(_)), "{error}");
    }

    #[test]
    fn a_refused_connection_is_a_failure_before_the_request() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("https://{}/a", listener.local_addr().unwrap());
        drop(listener);
        let mut pdp = PdpConfig::new("https://127.0.0.1:1", Secret::new("credential"));
        pdp.tls_system_roots = false;
        pdp.tls_ca_file = Some(data("tls-test-ca.pem"));
        let error = TlsTransport::new(&pdp)
            .unwrap()
            .post_json(&url, &[], b"{}", Duration::from_secs(2))
            .unwrap_err();
        assert!(error.request_was_not_sent(), "{error}");
    }
}
