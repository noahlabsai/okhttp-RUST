use std::collections::HashSet;
use std::sync::LazyLock;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::FallbackTestClientSocketFactory::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;

// Mocking the SSLSocket as it is a JVM-specific class. 
// In a real Rust implementation, this would be a wrapper around a TLS stream.

impl SSLSocket {
    pub fn set_enabled_protocols(&mut self, protocols: Vec<String>) {
        self.enabled_protocols = protocols;
    }
    pub fn set_enabled_cipher_suites(&mut self, suites: Vec<String>) {
        self.enabled_cipher_suites = suites;
    }
}

// These types are assumed to be defined in the project as per the provided context
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TlsVersion {
    TLS_1_3,
    TLS_1_2,
    TLS_1_1,
    TLS_1_0,
}

impl Default for TlsVersion {
    fn default() -> Self {
        TlsVersion::TLS_1_3
    }
}

pub const TLS_1_3: TlsVersion = TlsVersion::TLS_1_3;
pub const TLS_1_2: TlsVersion = TlsVersion::TLS_1_2;
pub const TLS_1_1: TlsVersion = TlsVersion::TLS_1_1;
pub const TLS_1_0: TlsVersion = TlsVersion::TLS_1_0;

impl TlsVersion {
    pub fn java_name(&self) -> &'static str {
        match self {
            TlsVersion::TLS_1_3 => "TLSv1.3",
            TlsVersion::TLS_1_2 => "TLSv1.2",
            TlsVersion::TLS_1_1 => "TLSv1.1",
            TlsVersion::TLS_1_0 => "TLSv1.0",
        }
    }

    pub fn for_java_name(name: &str) -> Self {
        match name {
            "TLSv1.3" => TlsVersion::TLS_1_3,
            "TLSv1.2" => TlsVersion::TLS_1_2,
            "TLSv1.1" => TlsVersion::TLS_1_1,
            "TLSv1.0" => TlsVersion::TLS_1_0,
            _ => TlsVersion::TLS_1_2, // Default fallback
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CipherSuite {
    TLS_AES_128_GCM_SHA256,
    TLS_AES_256_GCM_SHA384,
    TLS_CHACHA20_POLY1305_SHA256,
    TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
    TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
    TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
    TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
    TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
    TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
    TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
    TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
    TLS_RSA_WITH_AES_128_GCM_SHA256,
    TLS_RSA_WITH_AES_256_GCM_SHA384,
    TLS_RSA_WITH_AES_128_CBC_SHA,
    TLS_RSA_WITH_AES_256_CBC_SHA,
    TLS_RSA_WITH_3DES_EDE_CBC_SHA,
}

impl Default for CipherSuite {
    fn default() -> Self {
        CipherSuite::TLS_AES_128_GCM_SHA256
    }
}

pub const TLS_AES_128_GCM_SHA256: CipherSuite = CipherSuite::TLS_AES_128_GCM_SHA256;
pub const TLS_AES_256_GCM_SHA384: CipherSuite = CipherSuite::TLS_AES_256_GCM_SHA384;
pub const TLS_CHACHA20_POLY1305_SHA256: CipherSuite = CipherSuite::TLS_CHACHA20_POLY1305_SHA256;
pub const TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256: CipherSuite = CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256;
pub const TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256: CipherSuite = CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256;
pub const TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384: CipherSuite = CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384;
pub const TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384: CipherSuite = CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384;
pub const TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256: CipherSuite = CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256;
pub const TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256: CipherSuite = CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256;
pub const TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA: CipherSuite = CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA;
pub const TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA: CipherSuite = CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA;
pub const TLS_RSA_WITH_AES_128_GCM_SHA256: CipherSuite = CipherSuite::TLS_RSA_WITH_AES_128_GCM_SHA256;
pub const TLS_RSA_WITH_AES_256_GCM_SHA384: CipherSuite = CipherSuite::TLS_RSA_WITH_AES_256_GCM_SHA384;
pub const TLS_RSA_WITH_AES_128_CBC_SHA: CipherSuite = CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA;
pub const TLS_RSA_WITH_AES_256_CBC_SHA: CipherSuite = CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA;
pub const TLS_RSA_WITH_3DES_EDE_CBC_SHA: CipherSuite = CipherSuite::TLS_RSA_WITH_3DES_EDE_CBC_SHA;

impl CipherSuite {
    pub fn java_name(&self) -> &'static str {
        match self {
            CipherSuite::TLS_AES_128_GCM_SHA256 => "TLS_AES_128_GCM_SHA256",
            CipherSuite::TLS_AES_256_GCM_SHA384 => "TLS_AES_256_GCM_SHA384",
            CipherSuite::TLS_CHACHA20_POLY1305_SHA256 => "TLS_CHACHA20_POLY1305_SHA256",
            CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256 => "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256",
            CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256 => "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
            CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384 => "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384",
            CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384 => "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
            CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256 => "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256",
            CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256 => "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256",
            CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA => "TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA",
            CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA => "TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA",
            CipherSuite::TLS_RSA_WITH_AES_128_GCM_SHA256 => "TLS_RSA_WITH_AES_128_GCM_SHA256",
            CipherSuite::TLS_RSA_WITH_AES_256_GCM_SHA384 => "TLS_RSA_WITH_AES_256_GCM_SHA384",
            CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA => "TLS_RSA_WITH_AES_128_CBC_SHA",
            CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA => "TLS_RSA_WITH_AES_256_CBC_SHA",
            CipherSuite::TLS_RSA_WITH_3DES_EDE_CBC_SHA => "TLS_RSA_WITH_3DES_EDE_CBC_SHA",
        }
    }

    pub fn for_java_name(name: &str) -> Self {
        match name {
            "TLS_AES_128_GCM_SHA256" => CipherSuite::TLS_AES_128_GCM_SHA256,
            "TLS_AES_256_GCM_SHA384" => CipherSuite::TLS_AES_256_GCM_SHA384,
            "TLS_CHACHA20_POLY1305_SHA256" => CipherSuite::TLS_CHACHA20_POLY1305_SHA256,
            "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256" => CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
            "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256" => CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
            "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384" => CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
            "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384" => CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
            "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256" => CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
            "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256" => CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
            "TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA" => CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
            "TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA" => CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
            "TLS_RSA_WITH_AES_128_GCM_SHA256" => CipherSuite::TLS_RSA_WITH_AES_128_GCM_SHA256,
            "TLS_RSA_WITH_AES_256_GCM_SHA384" => CipherSuite::TLS_RSA_WITH_AES_256_GCM_SHA384,
            "TLS_RSA_WITH_AES_128_CBC_SHA" => CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA,
            "TLS_RSA_WITH_AES_256_CBC_SHA" => CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA,
            "TLS_RSA_WITH_3DES_EDE_CBC_SHA" => CipherSuite::TLS_RSA_WITH_3DES_EDE_CBC_SHA,
            _ => CipherSuite::TLS_AES_128_GCM_SHA256,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionSpec {
    pub is_tls: bool,
    pub supports_tls_extensions: bool,
    pub cipher_suites_as_string: Option<Vec<String>>,
    pub tls_versions_as_string: Option<Vec<String>>,
}

impl ConnectionSpec {
    pub fn cipher_suites(&self) -> Option<Vec<CipherSuite>> {
        self.cipher_suites_as_string
            .as_ref()
            .map(|list| list.iter().map(|s| CipherSuite::for_java_name(s)).collect())
    }

    pub fn tls_versions(&self) -> Option<Vec<TlsVersion>> {
        self.tls_versions_as_string
            .as_ref()
            .map(|list| list.iter().map(|s| TlsVersion::for_java_name(s)).collect())
    }

    pub fn apply(&self, ssl_socket: &mut SSLSocket, is_fallback: bool) {
        let spec_to_apply = self.supported_spec(ssl_socket, is_fallback);

        if let Some(ref versions) = spec_to_apply.tls_versions_as_string {
            ssl_socket.set_enabled_protocols(versions.clone());
        }

        if let Some(ref suites) = spec_to_apply.cipher_suites_as_string {
            ssl_socket.set_enabled_cipher_suites(suites.clone());
        }
    }

    fn supported_spec(&self, ssl_socket: &SSLSocket, is_fallback: bool) -> ConnectionSpec {
        let socket_enabled_cipher_suites = &ssl_socket.enabled_cipher_suites;
        
        // effective_cipher_suites logic: intersection of socket enabled and spec enabled
        let mut cipher_suites_intersection = match &self.cipher_suites_as_string {
            Some(spec_suites) => {
                let set: HashSet<_> = spec_suites.iter().collect();
                socket_enabled_cipher_suites.iter()
                    .filter(|s| set.contains(s))
                    .cloned()
                    .collect::<Vec<_>>()
            }
            None => socket_enabled_cipher_suites.clone(),
        };

        let tls_versions_intersection = match &self.tls_versions_as_string {
            Some(spec_versions) => {
                let set: HashSet<_> = spec_versions.iter().collect();
                ssl_socket.enabled_protocols.iter()
                    .filter(|s| set.contains(s))
                    .cloned()
                    .collect::<Vec<_>>()
            }
            None => ssl_socket.enabled_protocols.clone(),
        };

        if is_fallback && ssl_socket.supported_cipher_suites.contains(&"TLS_FALLBACK_SCSV".to_string()) {
            cipher_suites_intersection.push("TLS_FALLBACK_SCSV".to_string());
        }

        ConnectionSpec::Builder::new(self)
            .cipher_suites_strings(cipher_suites_intersection)
            .tls_versions_strings(tls_versions_intersection)
            .build()
    }

    pub fn is_compatible(&self, socket: &SSLSocket) -> bool {
        if !self.is_tls {
            return false;
        }

        if let Some(ref versions) = self.tls_versions_as_string {
            let set: HashSet<_> = versions.iter().collect();
            if !socket.enabled_protocols.iter().any(|s| set.contains(s)) {
                return false;
            }
        }

        if let Some(ref suites) = self.cipher_suites_as_string {
            let set: HashSet<_> = suites.iter().collect();
            if !socket.enabled_cipher_suites.iter().any(|s| set.contains(s)) {
                return false;
            }
        }

        true
    }
}

impl std::fmt::Debug for ConnectionSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.is_tls {
            return write!(f, "ConnectionSpec()");
        }
        let suites = self.cipher_suites()
            .map(|s| format!("{:?}", s))
            .unwrap_or_else(|| "[all enabled]".to_string());
        let versions = self.tls_versions()
            .map(|v| format!("{:?}", v))
            .unwrap_or_else(|| "[all enabled]".to_string());
        
        write!(f, "ConnectionSpec(cipherSuites={}, tlsVersions={}, supportsTlsExtensions={})", 
               suites, versions, self.supports_tls_extensions)
    }
}

impl ConnectionSpec {

    impl ConnectionSpec::Builder {
        pub fn new(tls: bool) -> Self {
            Self {
                tls,
                cipher_suites: None,
                tls_versions: None,
                supports_tls_extensions: false,
            }
        }

        pub fn from_spec(spec: &ConnectionSpec) -> Self {
            Self {
                tls: spec.is_tls,
                cipher_suites: spec.cipher_suites_as_string.clone(),
                tls_versions: spec.tls_versions_as_string.clone(),
                supports_tls_extensions: spec.supports_tls_extensions,
            }
        }

        pub fn all_enabled_cipher_suites(mut self) -> Self {
            if !self.tls { panic!("no cipher suites for cleartext connections"); }
            self.cipher_suites = None;
            self
        }

        pub fn cipher_suites(mut self, suites: &[CipherSuite]) -> Self {
            if !self.tls { panic!("no cipher suites for cleartext connections"); }
            let strings = suites.iter().map(|s| s.java_name().to_string()).collect();
            self.cipher_suites_strings(strings)
        }

        pub fn cipher_suites_strings(mut self, suites: Vec<String>) -> Self {
            if !self.tls { panic!("no cipher suites for cleartext connections"); }
            if suites.is_empty() { panic!("At least one cipher suite is required"); }
            self.cipher_suites = Some(suites);
            self
        }

        pub fn all_enabled_tls_versions(mut self) -> Self {
            if !self.tls { panic!("no TLS versions for cleartext connections"); }
            self.tls_versions = None;
            self
        }

        pub fn tls_versions(mut self, versions: &[TlsVersion]) -> Self {
            if !self.tls { panic!("no TLS versions for cleartext connections"); }
            let strings = versions.iter().map(|v| v.java_name().to_string()).collect();
            self.tls_versions_strings(strings)
        }

        pub fn tls_versions_strings(mut self, versions: Vec<String>) -> Self {
            if !self.tls { panic!("no TLS versions for cleartext connections"); }
            if versions.is_empty() { panic!("At least one TLS version is required"); }
            self.tls_versions = Some(versions);
            self
        }

        pub fn supports_tls_extensions(mut self, supports: bool) -> Self {
            if !self.tls { panic!("no TLS extensions for cleartext connections"); }
            self.supports_tls_extensions = supports;
            self
        }

        pub fn build(self) -> ConnectionSpec {
            ConnectionSpec {
                is_tls: self.tls,
                supports_tls_extensions: self.supports_tls_extensions,
                cipher_suites_as_string: self.cipher_suites,
                tls_versions_as_string: self.tls_versions,
            }
        }
    }
}

// Companion Object Constants
pub static RESTRICTED_TLS: LazyLock<ConnectionSpec> = LazyLock::new(|| {
    let suites = vec![
        CipherSuite::TLS_AES_128_GCM_SHA256,
        CipherSuite::TLS_AES_256_GCM_SHA384,
        CipherSuite::TLS_CHACHA20_POLY1305_SHA256,
        CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
        CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
        CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
        CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
        CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
        CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
    ];
    ConnectionSpec::Builder::new(true)
        .cipher_suites(&suites)
        .tls_versions(&[TlsVersion::TLS_1_3, TlsVersion::TLS_1_2])
        .supports_tls_extensions(true)
        .build()
});

pub static MODERN_TLS: LazyLock<ConnectionSpec> = LazyLock::new(|| {
    let suites = vec![
        CipherSuite::TLS_AES_128_GCM_SHA256,
        CipherSuite::TLS_AES_256_GCM_SHA384,
        CipherSuite::TLS_CHACHA20_POLY1305_SHA256,
        CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
        CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
        CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
        CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
        CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
        CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
        CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
        CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
        CipherSuite::TLS_RSA_WITH_AES_128_GCM_SHA256,
        CipherSuite::TLS_RSA_WITH_AES_256_GCM_SHA384,
        CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA,
        CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA,
        CipherSuite::TLS_RSA_WITH_3DES_EDE_CBC_SHA,
    ];
    ConnectionSpec::Builder::new(true)
        .cipher_suites(&suites)
        .tls_versions(&[TlsVersion::TLS_1_3, TlsVersion::TLS_1_2])
        .supports_tls_extensions(true)
        .build()
});

pub static COMPATIBLE_TLS: LazyLock<ConnectionSpec> = LazyLock::new(|| {
    let suites = vec![
        CipherSuite::TLS_AES_128_GCM_SHA256,
        CipherSuite::TLS_AES_256_GCM_SHA384,
        CipherSuite::TLS_CHACHA20_POLY1305_SHA256,
        CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
        CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
        CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
        CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
        CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
        CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
        CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
        CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
        CipherSuite::TLS_RSA_WITH_AES_128_GCM_SHA256,
        CipherSuite::TLS_RSA_WITH_AES_256_GCM_SHA384,
        CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA,
        CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA,
        CipherSuite::TLS_RSA_WITH_3DES_EDE_CBC_SHA,
    ];
    ConnectionSpec::Builder::new(true)
        .cipher_suites(&suites)
        .tls_versions(&[TlsVersion::TLS_1_3, TlsVersion::TLS_1_2, TlsVersion::TLS_1_1, TlsVersion::TLS_1_0])
        .supports_tls_extensions(true)
        .build()
});

pub static CLEARTEXT: LazyLock<ConnectionSpec> = LazyLock::new(|| {
    ConnectionSpec::Builder::new(false).build()
});
)}
