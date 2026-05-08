use std::error::Error;
use std::fmt;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::HeldCertificate;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::internal::der::Certificate;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::DelegatingSSLSession;

// Mocking the Handshake and related types as they are part of the okhttp3 package 
// and required for the test logic.
#[derive(Debug, Clone, PartialEq)]
pub enum TlsVersion {
    TLS_1_2,
    TLS_1_3,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CipherSuite {
    TLS_AES_128_GCM_SHA256,
    #[allow(dead_code)]
    SSL_NULL_WITH_NULL_NULL,
    #[allow(dead_code)]
    TLS_NULL_WITH_NULL_NULL,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Handshake {
    pub tls_version: TlsVersion,
    pub cipher_suite: CipherSuite,
    pub peer_certificates: Vec<Certificate>,
    pub local_certificates: Vec<Certificate>,
    pub peer_principal: Option<String>,
    pub local_principal: Option<String>,
}

impl Handshake {
    pub fn get(
        tls_version: TlsVersion,
        cipher_suite: CipherSuite,
        peer_certificates: Vec<Certificate>,
        local_certificates: Vec<Certificate>,
    ) -> Self {
        let peer_principal = peer_certificates
            .first()
            .map(|cert| cert.subject_x500_principal.clone());

        Handshake {
            tls_version,
            cipher_suite,
            peer_certificates,
            local_certificates,
            peer_principal,
            local_principal: None,
        }
    }
}

// Extension trait to mimic the Kotlin extension function sslSession.handshake()
pub trait SslSessionExt {
    fn handshake(&self) -> Result<Handshake, Box<dyn Error>>;
}

impl SslSessionExt for DelegatingSSLSession {
    fn handshake(&self) -> Result<Handshake, Box<dyn Error>> {
        let protocol = self.get_protocol();
        let cipher_suite_str = self.get_cipher_suite();

        if cipher_suite_str == "SSL_NULL_WITH_NULL_NULL" || cipher_suite_str == "TLS_NULL_WITH_NULL_NULL" {
            return Err(Box::new(HandshakeError::cipher_suite_null(cipher_suite_str)));
        }

        let tls_version = match protocol.as_str() {
            "TLSv1.3" => TlsVersion::TLS_1_3,
            _ => TlsVersion::TLS_1_2,
        };

        let cipher_suite = match cipher_suite_str.as_str() {
            "TLS_AES_128_GCM_SHA256" => CipherSuite::TLS_AES_128_GCM_SHA256,
            _ => return Err(Box::new(HandshakeError::unknown_cipher_suite(cipher_suite_str))),
        };

        let peer_certs = self.get_peer_certificates()
            .map(|certs| certs.into_iter().cloned().collect())
            .unwrap_or_default();

        let local_certs = self.get_local_certificates()
            .map(|certs| certs.into_iter().cloned().collect())
            .unwrap_or_default();

        Ok(Handshake::get(
            tls_version,
            cipher_suite,
            peer_certs,
            local_certs,
        ))
    }
}

#[derive(Debug)]
struct HandshakeError(String);
impl fmt::Display for HandshakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for HandshakeError {}

impl HandshakeError {
    fn cipher_suite_null(suite: String) -> Self {
        HandshakeError(format!("cipherSuite == {}", suite))
    }
    fn unknown_cipher_suite(suite: String) -> Self {
        HandshakeError(format!("Unknown cipher suite: {}", suite))
    }
}

pub struct FakeSSLSession {
    protocol: String,
    cipher_suite: String,
    peer_certificates: Option<Vec<Certificate>>,
    local_certificates: Option<Vec<Certificate>>,
}

impl FakeSSLSession {
    pub fn new(
        protocol: &str,
        cipher_suite: &str,
        peer_certificates: Option<Vec<Certificate>>,
        local_certificates: Option<Vec<Certificate>>,
    ) -> Self {
        FakeSSLSession {
            protocol: protocol.to_string(),
            cipher_suite: cipher_suite.to_string(),
            peer_certificates,
            local_certificates,
        }
    }
}

impl DelegatingSSLSession for FakeSSLSession {
    fn get_protocol(&self) -> String {
        self.protocol.clone()
    }

    fn get_cipher_suite(&self) -> String {
        self.cipher_suite.clone()
    }

    fn get_peer_certificates(&self) -> Option<Vec<Certificate>> {
        self.peer_certificates.clone()
    }

    fn get_local_certificates(&self) -> Option<Vec<Certificate>> {
        self.local_certificates.clone()
    }
}

pub struct HandshakeTest {
    server_root: HeldCertificate,
    server_intermediate: HeldCertificate,
    server_certificate: HeldCertificate,
}

impl HandshakeTest {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let server_root = HeldCertificate::builder()
            .certificate_authority(1)
            .build()?;
        
        let server_intermediate = HeldCertificate::builder()
            .certificate_authority(0)
            .signed_by(Some(server_root.clone()))
            .build()?;
            
        let server_certificate = HeldCertificate::builder()
            .signed_by(Some(server_intermediate.clone()))
            .build()?;

        Ok(HandshakeTest {
            server_root,
            server_intermediate,
            server_certificate,
        })
    }

    pub fn create_from_parts(&self) {
        let handshake = Handshake::get(
            TlsVersion::TLS_1_3,
            CipherSuite::TLS_AES_128_GCM_SHA256,
            vec![self.server_certificate.certificate.clone(), self.server_intermediate.certificate.clone()],
            vec![],
        );

        assert_eq!(handshake.tls_version, TlsVersion::TLS_1_3);
        assert_eq!(handshake.cipher_suite, CipherSuite::TLS_AES_128_GCM_SHA256);
        assert_eq!(handshake.peer_certificates, vec![
            self.server_certificate.certificate.clone(),
            self.server_intermediate.certificate.clone(),
        ]);
        assert!(handshake.local_principal.is_none());
        assert_eq!(handshake.peer_principal, Some(self.server_certificate.certificate.subject_x500_principal.clone()));
        assert!(handshake.local_certificates.is_empty());
    }

    pub fn create_from_ssl_session(&self) {
        let ssl_session = FakeSSLSession::new(
            "TLSv1.3",
            "TLS_AES_128_GCM_SHA256",
            Some(vec![self.server_certificate.certificate.clone(), self.server_intermediate.certificate.clone()]),
            None,
        );

        let handshake = ssl_session.handshake().expect("Handshake should succeed");

        assert_eq!(handshake.tls_version, TlsVersion::TLS_1_3);
        assert_eq!(handshake.cipher_suite, CipherSuite::TLS_AES_128_GCM_SHA256);
        assert_eq!(handshake.peer_certificates, vec![
            self.server_certificate.certificate.clone(),
            self.server_intermediate.certificate.clone(),
        ]);
        assert!(handshake.local_principal.is_none());
        assert_eq!(handshake.peer_principal, Some(self.server_certificate.certificate.subject_x500_principal.clone()));
        assert!(handshake.local_certificates.is_empty());
    }

    pub fn ssl_with_null_null_null(&self) {
        let ssl_session = FakeSSLSession::new(
            "TLSv1.3",
            "SSL_NULL_WITH_NULL_NULL",
            Some(vec![self.server_certificate.certificate.clone(), self.server_intermediate.certificate.clone()]),
            None,
        );

        let result = ssl_session.handshake();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("cipherSuite == SSL_NULL_WITH_NULL_NULL"));
    }

    pub fn tls_with_null_null_null(&self) {
        let ssl_session = FakeSSLSession::new(
            "TLSv1.3",
            "TLS_NULL_WITH_NULL_NULL",
            Some(vec![self.server_certificate.certificate.clone(), self.server_intermediate.certificate.clone()]),
            None,
        );

        let result = ssl_session.handshake();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("cipherSuite == TLS_NULL_WITH_NULL_NULL"));
    }
}
