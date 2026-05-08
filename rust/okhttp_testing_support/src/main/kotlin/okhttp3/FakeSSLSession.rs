use std::error::Error;
use std::fmt;
use std::sync::Arc;

/// Equivalent to java.security.Principal
pub trait Principal: Send + Sync {
    fn get_name(&self) -> String;
}

/// Equivalent to java.security.cert.Certificate
pub trait Certificate: Send + Sync {}

/// Equivalent to javax.security.cert.X509Certificate
pub trait X509Certificate: Certificate {}

/// Equivalent to javax.net.ssl.SSLSessionContext
pub trait SSLSessionContext: Send + Sync {}

/// Equivalent to javax.net.ssl.SSLPeerUnverifiedException
#[derive(Debug)]
pub struct SSLPeerUnverifiedException(pub String);

impl fmt::Display for SSLPeerUnverifiedException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SSLPeerUnverifiedException: {}", self.0)
    }
}

impl Error for SSLPeerUnverifiedException {}

/// Equivalent to javax.net.ssl.SSLSession
pub trait SSLSession: Send + Sync {
    fn get_application_buffer_size(&self) -> i32;
    fn get_cipher_suite(&self) -> String;
    fn get_creation_time(&self) -> i64;
    fn get_id(&self) -> Vec<u8>;
    fn get_last_accessed_time(&self) -> i64;
    fn get_local_certificates(&self) -> Vec<Arc<dyn Certificate>>;
    fn get_local_principal(&self) -> Arc<dyn Principal>;
    fn get_packet_buffer_size(&self) -> i32;
    fn get_peer_certificates(&self) -> Result<Vec<Arc<dyn Certificate>>, Box<dyn Error>>;
    fn get_peer_certificate_chain(&self) -> Result<Vec<Arc<dyn X509Certificate>>, Box<dyn Error>>;
    fn get_peer_host(&self) -> String;
    fn get_peer_port(&self) -> i32;
    fn get_peer_principal(&self) -> Result<Arc<dyn Principal>, Box<dyn Error>>;
    fn get_protocol(&self) -> String;
    fn get_session_context(&self) -> Arc<dyn SSLSessionContext>;
    fn put_value(&self, s: String, obj: Arc<dyn std::any::Any + Send + Sync>);
    fn remove_value(&self, s: String);
    fn get_value(&self, s: String) -> Arc<dyn std::any::Any + Send + Sync>;
    fn get_value_names(&self) -> Vec<String>;
    fn invalidate(&self);
    fn is_valid(&self) -> bool;
}

/// FakeSSLSession implementation for testing purposes.
pub struct FakeSSLSession {
    pub certificates: Vec<Arc<dyn Certificate>>,
}

impl FakeSSLSession {
    pub fn new(certificates: Vec<Arc<dyn Certificate>>) -> Self {
        Self { certificates }
    }
}

impl SSLSession for FakeSSLSession {
    fn get_application_buffer_size(&self) -> i32 {
        panic!("UnsupportedOperationException")
    }

    fn get_cipher_suite(&self) -> String {
        panic!("UnsupportedOperationException")
    }

    fn get_creation_time(&self) -> i64 {
        panic!("UnsupportedOperationException")
    }

    fn get_id(&self) -> Vec<u8> {
        panic!("UnsupportedOperationException")
    }

    fn get_last_accessed_time(&self) -> i64 {
        panic!("UnsupportedOperationException")
    }

    fn get_local_certificates(&self) -> Vec<Arc<dyn Certificate>> {
        panic!("UnsupportedOperationException")
    }

    fn get_local_principal(&self) -> Arc<dyn Principal> {
        panic!("UnsupportedOperationException")
    }

    fn get_packet_buffer_size(&self) -> i32 {
        panic!("UnsupportedOperationException")
    }

    fn get_peer_certificates(&self) -> Result<Vec<Arc<dyn Certificate>>, Box<dyn Error>> {
        if self.certificates.is_empty() {
            Err(Box::new(SSLPeerUnverifiedException("peer not authenticated".to_string())))
        } else {
            Ok(self.certificates.clone())
        }
    }

    fn get_peer_certificate_chain(&self) -> Result<Vec<Arc<dyn X509Certificate>>, Box<dyn Error>> {
        panic!("UnsupportedOperationException")
    }

    fn get_peer_host(&self) -> String {
        panic!("UnsupportedOperationException")
    }

    fn get_peer_port(&self) -> i32 {
        panic!("UnsupportedOperationException")
    }

    fn get_peer_principal(&self) -> Result<Arc<dyn Principal>, Box<dyn Error>> {
        panic!("UnsupportedOperationException")
    }

    fn get_protocol(&self) -> String {
        panic!("UnsupportedOperationException")
    }

    fn get_session_context(&self) -> Arc<dyn SSLSessionContext> {
        panic!("UnsupportedOperationException")
    }

    fn put_value(&self, _s: String, _obj: Arc<dyn std::any::Any + Send + Sync>) {
        panic!("UnsupportedOperationException")
    }

    fn remove_value(&self, _s: String) {
        panic!("UnsupportedOperationException")
    }

    fn get_value(&self, _s: String) -> Arc<dyn std::any::Any + Send + Sync> {
        panic!("UnsupportedOperationException")
    }

    fn get_value_names(&self) -> Vec<String> {
        panic!("UnsupportedOperationException")
    }

    fn invalidate(&self) {
        panic!("UnsupportedOperationException")
    }

    fn is_valid(&self) -> bool {
        panic!("UnsupportedOperationException")
    }
}
