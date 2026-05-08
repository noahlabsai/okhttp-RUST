use std::error::Error;
use std::fmt;
use std::net::TcpStream;

/// Mocking the Java security types as they are not part of the Rust standard library.
/// In a real production environment, these would be provided by a TLS crate like `rustls` or `openssl`.

#[derive(Debug, Clone, PartialEq)]
pub struct X509Certificate {
    pub subject: String,
}

pub trait X509TrustManager {
    fn get_accepted_issuers(&self) -> Vec<X509Certificate>;
    fn check_server_trusted(
        &self,
        chain: &[X509Certificate],
        auth_type: &str,
    ) -> Result<(), Box<dyn Error>>;
    fn check_client_trusted(
        &self,
        chain: &[X509Certificate],
        auth_type: Option<&str>,
    ) -> Result<(), Box<dyn Error>>;
}

pub trait X509ExtendedTrustManager: X509TrustManager {
    fn check_server_trusted_socket(
        &self,
        chain: &[X509Certificate],
        auth_type: &str,
        socket: &TcpStream,
    ) -> Result<(), Box<dyn Error>>;

    fn check_server_trusted_engine(
        &self,
        chain: &[X509Certificate],
        auth_type: &str,
        engine: &SSLEngine,
    ) -> Result<(), Box<dyn Error>>;

    fn check_client_trusted_engine(
        &self,
        chain: &[X509Certificate],
        auth_type: &str,
        engine: Option<&SSLEngine>,
    ) -> Result<(), Box<dyn Error>>;

    fn check_client_trusted_socket(
        &self,
        chain: &[X509Certificate],
        auth_type: &str,
        socket: Option<&TcpStream>,
    ) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SSLEngine {
    pub peer_host: String,
}

#[derive(Debug)]
pub struct CertificateException(pub String);

impl fmt::Display for CertificateException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CertificateException: {}", self.0)
    }
}

impl Error for CertificateException {}

/// This extends X509ExtendedTrustManager to disable verification for a set of hosts.
pub struct InsecureExtendedTrustManager<T: X509ExtendedTrustManager> {
    delegate: T,
    insecure_hosts: Vec<String>,
}

impl<T: X509ExtendedTrustManager> InsecureExtendedTrustManager<T> {
    pub fn new(delegate: T, insecure_hosts: Vec<String>) -> Self {
        Self {
            delegate,
            insecure_hosts,
        }
    }

    fn peer_name(socket: &TcpStream) -> String {
        // In Rust, TcpStream::peer_addr() returns a SocketAddr.
        // To get the hostname, one would typically need a reverse DNS lookup.
        // To preserve the Kotlin logic: if it's an InetSocketAddress (which SocketAddr is), 
        // we try to get the host. Since SocketAddr doesn't store the original hostname,
        // we use the debug representation as a fallback similar to .toString().
        match socket.peer_addr() {
            Ok(addr) => addr.to_string(),
            Err(_) => "unknown".to_string(),
        }
    }
}

impl<T: X509ExtendedTrustManager> X509TrustManager for InsecureExtendedTrustManager<T> {
    fn get_accepted_issuers(&self) -> Vec<X509Certificate> {
        self.delegate.get_accepted_issuers()
    }

    fn check_server_trusted(
        &self,
        _chain: &[X509Certificate],
        _auth_type: &str,
    ) -> Result<(), Box<dyn Error>> {
        // Kotlin: throw CertificateException("Unsupported operation")
        Err(Box::new(CertificateException("Unsupported operation".to_string())))
    }

    fn check_client_trusted(
        &self,
        _chain: &[X509Certificate],
        _auth_type: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        // Kotlin: throw CertificateException("Unsupported operation")
        Err(Box::new(CertificateException("Unsupported operation".to_string())))
    }
}

impl<T: X509ExtendedTrustManager> X509ExtendedTrustManager for InsecureExtendedTrustManager<T> {
    fn check_server_trusted_socket(
        &self,
        chain: &[X509Certificate],
        auth_type: &str,
        socket: &TcpStream,
    ) -> Result<(), Box<dyn Error>> {
        let name = Self::peer_name(socket);
        if !self.insecure_hosts.contains(&name) {
            self.delegate.check_server_trusted_socket(chain, auth_type, socket)?;
        }
        Ok(())
    }

    fn check_server_trusted_engine(
        &self,
        chain: &[X509Certificate],
        auth_type: &str,
        engine: &SSLEngine,
    ) -> Result<(), Box<dyn Error>> {
        if !self.insecure_hosts.contains(&engine.peer_host) {
            self.delegate.check_server_trusted_engine(chain, auth_type, engine)?;
        }
        Ok(())
    }

    fn check_client_trusted_engine(
        &self,
        _chain: &[X509Certificate],
        _auth_type: &str,
        _engine: Option<&SSLEngine>,
    ) -> Result<(), Box<dyn Error>> {
        // Kotlin: throw CertificateException("Unsupported operation")
        Err(Box::new(CertificateException("Unsupported operation".to_string())))
    }

    fn check_client_trusted_socket(
        &self,
        _chain: &[X509Certificate],
        _auth_type: &str,
        _socket: Option<&TcpStream>,
    ) -> Result<(), Box<dyn Error>> {
        // Kotlin: throw CertificateException("Unsupported operation")
        Err(Box::new(CertificateException("Unsupported operation".to_string())))
    }
}
