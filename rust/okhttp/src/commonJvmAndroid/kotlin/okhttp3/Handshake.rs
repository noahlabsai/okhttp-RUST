use std::sync::OnceLock;
use std::error::Error;
use std::io::{Error as IoError, ErrorKind};
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::internal::der::Certificate as DerCertificate;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okhttp::src::commonTest::kotlin::okhttp3::internal::IsProbablyUtf8Test::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::letsencrypt::LetsEncryptClientTest::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;

// Mocking types that would be defined in other parts of the okhttp crate
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TlsVersion {
    SSL_3_0,
    TLS_1_0,
    TLS_1_1,
    TLS_1_2,
    TLS_1_3,
}

impl Default for TlsVersion {
    fn default() -> Self {
        TlsVersion::SSL_3_0
    }
}

pub const SSL_3_0: TlsVersion = TlsVersion::SSL_3_0;
pub const TLS_1_0: TlsVersion = TlsVersion::TLS_1_0;
pub const TLS_1_1: TlsVersion = TlsVersion::TLS_1_1;
pub const TLS_1_2: TlsVersion = TlsVersion::TLS_1_2;
pub const TLS_1_3: TlsVersion = TlsVersion::TLS_1_3;

impl TlsVersion {
    pub fn for_java_name(name: &str) -> Self {
        match name {
            "SSLv3" => TlsVersion::SSL_3_0,
            "TLSv1" => TlsVersion::TLS_1_0,
            "TLSv1.1" => TlsVersion::TLS_1_1,
            "TLSv1.2" => TlsVersion::TLS_1_2,
            "TLSv1.3" => TlsVersion::TLS_1_3,
            _ => TlsVersion::SSL_3_0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CipherSuite {
    TlsNullWithNullNull,
    SslNullWithNullNull,
    // Other variants would be here
    Unknown,
}

impl Default for CipherSuite {
    fn default() -> Self {
        CipherSuite::TlsNullWithNullNull
    }
}

pub const TlsNullWithNullNull: CipherSuite = CipherSuite::TlsNullWithNullNull;
pub const SslNullWithNullNull: CipherSuite = CipherSuite::SslNullWithNullNull;
pub const Unknown: CipherSuite = CipherSuite::Unknown;

impl CipherSuite {
    pub fn for_java_name(name: &str) -> Self {
        match name {
            "TLS_NULL_WITH_NULL_NULL" => CipherSuite::TlsNullWithNullNull,
            "SSL_NULL_WITH_NULL_NULL" => CipherSuite::SslNullWithNullNull,
            _ => CipherSuite::Unknown,
        }
    }
}

// Representing java.security.cert.Certificate and X509Certificate
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Certificate {
    X509(X509Certificate),
    Other { cert_type: String },
}

impl Default for Certificate {
    fn default() -> Self {
        Certificate::X509
    }
}

pub const X509: Certificate = Certificate::X509;
pub const Other: Certificate = Certificate::Other;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Principal(pub String);

// Mocking SSLSession for the extension function

// A record of a TLS handshake. For HTTPS clients, the client is *local* and the remote server is
// its *peer*.
#[derive(Debug, Clone)]
pub struct Handshake {
    pub tls_version: TlsVersion,
    pub cipher_suite: CipherSuite,
    pub local_certificates: Vec<Certificate>,
    // Using OnceLock to simulate Kotlin's 'by lazy'
    peer_certificates_cache: OnceLock<Vec<Certificate>>,
    peer_certificates_fn: Box<dyn Fn() -> Vec<Certificate> + Send + Sync>,
}

impl Handshake {
    pub fn new(
        tls_version: TlsVersion,
        cipher_suite: CipherSuite,
        local_certificates: Vec<Certificate>,
        peer_certificates_fn: impl Fn() -> Vec<Certificate> + Send + Sync + 'static,
    ) -> Self {
        Self {
            tls_version,
            cipher_suite,
            local_certificates,
            peer_certificates_cache: OnceLock::new(),
            peer_certificates_fn: Box::new(peer_certificates_fn),
        }
    }

    // Returns a possibly-empty list of certificates that identify the remote peer.
    pub fn peer_certificates(&self) -> Vec<Certificate> {
        self.peer_certificates_cache.get_or_init(|| {
            // In Kotlin, this catches SSLPeerUnverifiedException. 
            // Since we are in Rust, the closure provided to Handshake should handle the error 
            // or we wrap the call.
            (self.peer_certificates_fn)()
        }).clone()
    }

    // Returns the remote peer's principle, or null if that peer is anonymous.
    pub fn peer_principal(&self) -> Option<Principal> {
        let certs = self.peer_certificates();
        if let Some(Certificate::X509(x509)) = certs.first() {
            Some(Principal(x509.subject_x500_principal.clone()))
        } else {
            None
        }
    }

    // Returns the local principle, or null if this peer is anonymous.
    pub fn local_principal(&self) -> Option<Principal> {
        if let Some(Certificate::X509(x509)) = self.local_certificates.first() {
            Some(Principal(x509.subject_x500_principal.clone()))
        } else {
            None
        }
    }

    // Deprecated methods preserved for API compatibility
    pub fn tls_version_deprecated(&self) -> TlsVersion {
        self.tls_version.clone()
    }

    pub fn cipher_suite_deprecated(&self) -> CipherSuite {
        self.cipher_suite.clone()
    }

    pub fn peer_certificates_deprecated(&self) -> Vec<Certificate> {
        self.peer_certificates()
    }

    pub fn peer_principal_deprecated(&self) -> Option<Principal> {
        self.peer_principal()
    }

    pub fn local_certificates_deprecated(&self) -> Vec<Certificate> {
        self.local_certificates.clone()
    }

    pub fn local_principal_deprecated(&self) -> Option<Principal> {
        self.local_principal()
    }
}

impl PartialEq for Handshake {
    fn eq(&self, other: &Self) -> bool {
        self.tls_version == other.tls_version
            && self.cipher_suite == other.cipher_suite
            && self.peer_certificates() == other.peer_certificates()
            && self.local_certificates == other.local_certificates
    }
}

impl Eq for Handshake {}

impl std::hash::Hash for Handshake {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tls_version.hash(state);
        self.cipher_suite.hash(state);
        // Note: peer_certificates() is a method, we hash the resulting Vec
        self.peer_certificates().hash(state);
        self.local_certificates.hash(state);
    }
}

impl std::fmt::Display for Handshake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let peer_certs_names: Vec<String> = self.peer_certificates()
            .iter()
            .map(|c| match c {
                Certificate::X509(x) => x.subject_dn.clone(),
                Certificate::Other { cert_type } => cert_type.clone(),
            })
            .collect();

        let local_certs_names: Vec<String> = self.local_certificates
            .iter()
            .map(|c| match c {
                Certificate::X509(x) => x.subject_dn.clone(),
                Certificate::Other { cert_type } => cert_type.clone(),
            })
            .collect();

        write!(
            f,
            "Handshake{{tlsVersion={:?} cipherSuite={:?} peerCertificates={:?} localCertificates={:?}}}",
            self.tls_version, self.cipher_suite, peer_certs_names, local_certs_names
        )
    }
}

impl Handshake {
    // Static factory method equivalent to companion object get()
    pub fn get(
        tls_version: TlsVersion,
        cipher_suite: CipherSuite,
        peer_certificates: Vec<Certificate>,
        local_certificates: Vec<Certificate>,
    ) -> Self {
        let peer_certs_copy = peer_certificates.clone();
        Self::new(
            tls_version,
            cipher_suite,
            local_certificates,
            move || peer_certs_copy.clone(),
        )
    }
}

// Extension trait for SSLSession to provide the .handshake() method
pub trait SSLSessionExt {
    fn handshake(&self) -> Result<Handshake, Box<dyn Error>>;
}

impl SSLSessionExt for SSLSession {
    fn handshake(&self) -> Result<Handshake, Box<dyn Error>> {
        let cipher_suite_string = self.cipher_suite.as_ref()
            .ok_or_else(|| IoError::new(ErrorKind::Other, "cipherSuite == null"))?;

        let cipher_suite = match cipher_suite_string.as_str() {
            "TLS_NULL_WITH_NULL_NULL" | "SSL_NULL_WITH_NULL_NULL" => {
                return Err(Box::new(IoError::new(ErrorKind::Other, format!("cipherSuite == {}", cipher_suite_string))));
            }
            _ => CipherSuite::for_java_name(cipher_suite_string),
        };

        let tls_version_string = self.protocol.as_ref()
            .ok_or_else(|| IoError::new(ErrorKind::Other, "tlsVersion == null"))?;

        if tls_version_string == "NONE" {
            return Err(Box::new(IoError::new(ErrorKind::Other, "tlsVersion == NONE")));
        }
        let tls_version = TlsVersion::for_java_name(tls_version_string);

        let peer_certificates_copy = self.peer_certificates.clone();
        let local_certificates_copy = self.local_certificates.clone();

        Ok(Handshake::new(
            tls_version,
            cipher_suite,
            local_certificates_copy,
            move || peer_certificates_copy.clone(),
        ))
    }
}