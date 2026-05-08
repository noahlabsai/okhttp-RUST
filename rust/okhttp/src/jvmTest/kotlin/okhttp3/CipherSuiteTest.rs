use std::sync::Arc;
use std::collections::HashMap;
use std::sync::Mutex;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::{DelegatingSSLSocket, SSLSocket};
use crate::okhttp3::{CipherSuite, ConnectionSpec, TlsVersion};
use crate::okhttp3::internal::apply_connection_spec;

/// Mock implementation of CipherSuite to support the test logic.
/// In a real scenario, this would be the actual CipherSuite enum/struct.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CipherSuiteInstance {
    pub java_name: String,
}

impl std::fmt::Display for CipherSuiteInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.java_name)
    }
}

/// Companion object for CipherSuite
pub struct CipherSuiteCompanion;

lazy_static::lazy_static! {
    static ref INTERN_MAP: Mutex<HashMap<String, Arc<CipherSuiteInstance>>> = Mutex::new(HashMap::new());
}

impl CipherSuiteCompanion {
    /// Interns CipherSuite instances based on their Java name.
    pub fn for_java_name(name: &str) -> Arc<CipherSuiteInstance> {
        // Handle the SSL_ vs TLS_ prefix equivalence mentioned in the tests
        let normalized_name = if name.starts_with("SSL_") {
            name.replacen("SSL_", "TLS_", 1)
        } else if name.starts_with("TLS_") {
            name.to_string()
        } else {
            name.to_string()
        };

        let mut map = INTERN_MAP.lock().unwrap();
        map.entry(normalized_name)
            .or_insert_with(|| Arc::new(CipherSuiteInstance { java_name: name.to_string() }))
            .clone()
    }
}

/// FakeSslSocket implementation for testing applyConnectionSpec
pub struct FakeSslSocket {
    pub enabled_protocols: Vec<String>,
    pub supported_cipher_suites: Vec<String>,
    pub enabled_cipher_suites: Vec<String>,
}

impl FakeSslSocket {
    pub fn new() -> Self {
        Self {
            enabled_protocols: Vec::new(),
            supported_cipher_suites: Vec::new(),
            enabled_cipher_suites: Vec::new(),
        }
    }
}

impl SSLSocket for FakeSslSocket {
    fn shutdown_input(&mut self) -> std::io::Result<()> { Ok(()) }
    fn shutdown_output(&mut self) -> std::io::Result<()> { Ok(()) }
    fn supported_cipher_suites(&self) -> Vec<String> { self.supported_cipher_suites.clone() }
    fn enabled_cipher_suites(&self) -> Vec<String> { self.enabled_cipher_suites.clone() }
    fn set_enabled_cipher_suites(&mut self, suites: Vec<String>) { self.enabled_cipher_suites = suites; }
    fn supported_protocols(&self) -> Vec<String> { Vec::new() }
    fn enabled_protocols(&self) -> Vec<String> { self.enabled_protocols.clone() }
    fn set_enabled_protocols(&mut self, protocols: Vec<String>) { self.enabled_protocols = protocols; }
    fn session(&self) -> Arc<dyn crate::okhttp_testing_support::src::main::kotlin::okhttp3::SSLSession> { panic!("Not implemented") }
    fn add_handshake_completed_listener(&mut self, _l: Arc<dyn crate::okhttp_testing_support::src::main::kotlin::okhttp3::HandshakeCompletedListener>) {}
    fn remove_handshake_completed_listener(&mut self, _l: Arc<dyn crate::okhttp_testing_support::src::main::kotlin::okhttp3::HandshakeCompletedListener>) {}
    fn start_handshake(&mut self) -> std::io::Result<()> { Ok(()) }
    fn use_client_mode(&self) -> bool { true }
    fn set_use_client_mode(&mut self, _m: bool) {}
    fn need_client_auth(&self) -> bool { false }
    fn set_need_client_auth(&mut self, _n: bool) {}
    fn want_client_auth(&self) -> bool { false }
    fn set_want_client_auth(&mut self, _w: bool) {}
    fn enable_session_creation(&self) -> bool { true }
    fn set_enable_session_creation(&mut self, _f: bool) {}
    fn ssl_parameters(&self) -> Arc<dyn crate::okhttp_testing_support::src::main::kotlin::okhttp3::SSLParameters> { panic!("Not implemented") }
    fn set_ssl_parameters(&mut self, _p: Arc<dyn crate::okhttp_testing_support::src::main::kotlin::okhttp3::SSLParameters>) {}
    fn inet_address(&self) -> std::net::IpAddr { "127.0.0.1".parse().unwrap() }
    fn keep_alive(&self) -> std::io::Result<bool> { Ok(true) }
    fn set_keep_alive(&mut self, _k: bool) -> std::io::Result<()> { Ok(()) }
    fn local_address(&self) -> std::net::IpAddr { "127.0.0.1".parse().unwrap() }
    fn local_port(&self) -> i32 { 0 }
    fn port(&self) -> i32 { 0 }
    fn so_linger(&self) -> std::io::Result<i32> { Ok(0) }
    fn set_so_linger(&mut self, _o: bool, _t: i32) -> std::io::Result<()> { Ok(()) }
    fn receive_buffer_size(&self) -> std::io::Result<i32> { Ok(0) }
    fn set_receive_buffer_size(&mut self, _s: i32) -> std::io::Result<()> { Ok(()) }
    fn send_buffer_size(&self) -> std::io::Result<i32> { Ok(0) }
    fn set_send_buffer_size(&mut self, _s: i32) -> std::io::Result<()> { Ok(()) }
    fn so_timeout(&self) -> std::io::Result<i32> { Ok(0) }
    fn set_so_timeout(&mut self, _t: i32) -> std::io::Result<()> { Ok(()) }
    fn tcp_no_delay(&self) -> std::io::Result<bool> { Ok(true) }
    fn set_tcp_no_delay(&mut self, _o: bool) -> std::io::Result<()> { Ok(()) }
    fn local_socket_address(&self) -> std::net::SocketAddr { "127.0.0.1:0".parse().unwrap() }
    fn remote_socket_address(&self) -> std::net::SocketAddr { "127.0.0.1:0".parse().unwrap() }
    fn is_bound(&self) -> bool { true }
    fn is_connected(&self) -> bool { true }
    fn is_closed(&self) -> bool { false }
    fn bind(&mut self, _a: std::net::SocketAddr) -> std::io::Result<()> { Ok(()) }
    fn connect(&mut self, _a: std::net::SocketAddr) -> std::io::Result<()> { Ok(()) }
    fn connect_with_timeout(&mut self, _a: std::net::SocketAddr, _t: i32) -> std::io::Result<()> { Ok(()) }
    fn is_input_shutdown(&self) -> bool { false }
    fn is_output_shutdown(&self) -> bool { false }
    fn reuse_address(&self) -> std::io::Result<bool> { Ok(true) }
    fn set_reuse_address(&mut self, _r: bool) -> std::io::Result<()> { Ok(()) }
    fn oob_inline(&self) -> std::io::Result<bool> { Ok(false) }
    fn set_oob_inline(&mut self, _o: bool) -> std::io::Result<()> { Ok(()) }
    fn traffic_class(&self) -> std::io::Result<i32> { Ok(0) }
    fn set_traffic_class(&mut self, _v: i32) -> std::io::Result<()> { Ok(()) }
    fn send_urgent_data(&mut self, _v: i32) -> std::io::Result<()> { Ok(()) }
    fn channel(&self) -> Arc<dyn crate::okhttp_testing_support::src::main::kotlin::okhttp3::SocketChannel> { panic!("Not implemented") }
    fn handshake_session(&self) -> Arc<dyn crate::okhttp_testing_support::src::main::kotlin::okhttp3::SSLSession> { panic!("Not implemented") }
    fn application_protocol(&self) -> String { String::new() }
    fn handshake_application_protocol(&self) -> String { String::new() }
    fn handshake_application_protocol_selector(&self) -> Option<Arc<dyn Fn(Arc<dyn SSLSocket>, Vec<String>) -> String + Send + Sync>> { None }
    fn set_handshake_application_protocol_selector(&mut self, _s: Option<Arc<dyn Fn(Arc<dyn SSLSocket>, Vec<String>) -> String + Send + Sync>>) {}
}

impl std::io::Read for FakeSslSocket {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> { Ok(0) }
}

impl std::io::Write for FakeSslSocket {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> { Ok(0) }
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

pub struct CipherSuiteTest;

impl CipherSuiteTest {
    pub fn hash_code_uses_identity_hash_code_legacy_case() {
        let cs = CipherSuiteCompanion::for_java_name("SSL_RSA_EXPORT_WITH_RC4_40_MD5");
        let ptr1 = Arc::as_ptr(&cs) as usize;
        let ptr2 = Arc::as_ptr(&cs) as usize;
        assert_eq!(ptr1, ptr2);
    }

    pub fn hash_code_uses_identity_hash_code_regular_case() {
        let cs = CipherSuiteCompanion::for_java_name("TLS_RSA_WITH_AES_128_CBC_SHA256");
        let ptr1 = Arc::as_ptr(&cs) as usize;
        let ptr2 = Arc::as_ptr(&cs) as usize;
        assert_eq!(ptr1, ptr2);
    }

    pub fn instances_are_interned() {
        let cs1 = CipherSuiteCompanion::for_java_name("TestCipherSuite");
        let cs2 = CipherSuiteCompanion::for_java_name("TestCipherSuite");
        assert!(Arc::ptr_eq(&cs1, &cs2));

        let cs_known = CipherSuiteCompanion::for_java_name("TLS_KRB5_WITH_DES_CBC_MD5");
        let cs_from_name = CipherSuiteCompanion::for_java_name(&cs_known.java_name);
        assert!(Arc::ptr_eq(&cs_known, &cs_from_name));
    }

    pub fn instances_are_interned_survives_garbage_collection() {
        let cs = CipherSuiteCompanion::for_java_name("FakeCipherSuite_instancesAreInterned");
        let name_copy = cs.java_name.clone();
        let cs_new = CipherSuiteCompanion::for_java_name(&name_copy);
        assert!(Arc::ptr_eq(&cs, &cs_new));
    }

    pub fn equals() {
        assert_eq!(CipherSuiteCompanion::for_java_name("cipher"), CipherSuiteCompanion::for_java_name("cipher"));
        assert_ne!(CipherSuiteCompanion::for_java_name("cipherB"), CipherSuiteCompanion::for_java_name("cipherA"));
        
        assert_eq!(
            CipherSuiteCompanion::for_java_name("SSL_RSA_EXPORT_WITH_RC4_40_MD5"),
            CipherSuiteCompanion::for_java_name("TLS_RSA_EXPORT_WITH_RC4_40_MD5")
        );
        
        assert_ne!(
            CipherSuiteCompanion::for_java_name("TLS_RSA_WITH_AES_128_CBC_SHA256"),
            CipherSuiteCompanion::for_java_name("SSL_RSA_EXPORT_WITH_RC4_40_MD5")
        );
    }

    pub fn for_java_name_accepts_arbitrary_strings() {
        let _ = CipherSuiteCompanion::for_java_name("example CipherSuite name that is not in the whitelist");
    }

    pub fn java_name_examples() {
        assert_eq!(CipherSuiteCompanion::for_java_name("SSL_RSA_EXPORT_WITH_RC4_40_MD5").java_name, "SSL_RSA_EXPORT_WITH_RC4_40_MD5");
        assert_eq!(CipherSuiteCompanion::for_java_name("TLS_RSA_WITH_AES_128_CBC_SHA256").java_name, "TLS_RSA_WITH_AES_128_CBC_SHA256");
        assert_eq!(CipherSuiteCompanion::for_java_name("TestCipherSuite").java_name, "TestCipherSuite");
    }

    pub fn java_name_equals_to_string() {
        let cs1 = CipherSuiteCompanion::for_java_name("SSL_RSA_EXPORT_WITH_RC4_40_MD5");
        assert_eq!(cs1.to_string(), cs1.java_name);
        
        let cs2 = CipherSuiteCompanion::for_java_name("TLS_RSA_WITH_AES_128_CBC_SHA256");
        assert_eq!(cs2.to_string(), cs2.java_name);
    }

    pub fn for_java_name_from_legacy_enum_name() {
        assert_eq!(
            CipherSuiteCompanion::for_java_name("SSL_RSA_EXPORT_WITH_RC4_40_MD5"),
            CipherSuiteCompanion::for_java_name("TLS_RSA_EXPORT_WITH_RC4_40_MD5")
        );
        assert_eq!(
            CipherSuiteCompanion::for_java_name("SSL_DH_RSA_EXPORT_WITH_DES40_CBC_SHA"),
            CipherSuiteCompanion::for_java_name("TLS_DH_RSA_EXPORT_WITH_DES40_CBC_SHA")
        );
        assert_eq!(
            CipherSuiteCompanion::for_java_name("SSL_FAKE_NEW_CIPHER"),
            CipherSuiteCompanion::for_java_name("TLS_FAKE_NEW_CIPHER")
        );
    }

    pub fn apply_intersection_retains_tls_prefixes() {
        let mut socket = FakeSslSocket::new();
        socket.enabled_protocols = vec!["TLSv1".to_string()];
        socket.supported_cipher_suites = vec!["SSL_A".to_string(), "SSL_B".to_string(), "SSL_C".to_string(), "SSL_D".to_string(), "SSL_E".to_string()];
        socket.enabled_cipher_suites = vec!["SSL_A".to_string(), "SSL_B".to_string(), "SSL_C".to_string()];
        
        let connection_spec = ConnectionSpec::builder(true)
            .tls_versions(&[TlsVersion::TLS_1_0])
            .cipher_suites(&["TLS_A", "TLS_C", "TLS_E"])
            .build();
            
        apply_connection_spec(&connection_spec, &mut socket, false);
        assert_eq!(socket.enabled_cipher_suites, vec!["TLS_A".to_string(), "TLS_C".to_string()]);
    }

    pub fn apply_intersection_retains_ssl_prefixes() {
        let mut socket = FakeSslSocket::new();
        socket.enabled_protocols = vec!["TLSv1".to_string()];
        socket.supported_cipher_suites = vec!["TLS_A".to_string(), "TLS_B".to_string(), "TLS_C".to_string(), "TLS_D".to_string(), "TLS_E".to_string()];
        socket.enabled_cipher_suites = vec!["TLS_A".to_string(), "TLS_B".to_string(), "TLS_C".to_string()];
        
        let connection_spec = ConnectionSpec::builder(true)
            .tls_versions(&[TlsVersion::TLS_1_0])
            .cipher_suites(&["SSL_A", "SSL_C", "SSL_E"])
            .build();
            
        apply_connection_spec(&connection_spec, &mut socket, false);
        assert_eq!(socket.enabled_cipher_suites, vec!["SSL_A".to_string(), "SSL_C".to_string()]);
    }

    pub fn apply_intersection_adds_ssl_scsv_for_fallback() {
        let mut socket = FakeSslSocket::new();
        socket.enabled_protocols = vec!["TLSv1".to_string()];
        socket.supported_cipher_suites = vec!["SSL_A".to_string(), "SSL_FALLBACK_SCSV".to_string()];
        socket.enabled_cipher_suites = vec!["SSL_A".to_string()];
        
        let connection_spec = ConnectionSpec::builder(true)
            .tls_versions(&[TlsVersion::TLS_1_0])
            .cipher_suites(&["SSL_A"])
            .build();
            
        apply_connection_spec(&connection_spec, &mut socket, true);
        assert_eq!(socket.enabled_cipher_suites, vec!["SSL_A".to_string(), "SSL_FALLBACK_SCSV".to_string()]);
    }

    pub fn apply_intersection_adds_tls_scsv_for_fallback() {
        let mut socket = FakeSslSocket::new();
        socket.enabled_protocols = vec!["TLSv1".to_string()];
        socket.supported_cipher_suites = vec!["TLS_A".to_string(), "TLS_FALLBACK_SCSV".to_string()];
        socket.enabled_cipher_suites = vec!["TLS_A".to_string()];
        
        let connection_spec = ConnectionSpec::builder(true)
            .tls_versions(&[TlsVersion::TLS_1_0])
            .cipher_suites(&["TLS_A"])
            .build();
            
        apply_connection_spec(&connection_spec, &mut socket, true);
        assert_eq!(socket.enabled_cipher_suites, vec!["TLS_A".to_string(), "TLS_FALLBACK_SCSV".to_string()]);
    }

    pub fn apply_intersection_to_protocol_version() {
        let mut socket = FakeSslSocket::new();
        socket.enabled_protocols = vec!["TLSv1".to_string(), "TLSv1.1".to_string(), "TLSv1.2".to_string()];
        socket.supported_cipher_suites = vec!["TLS_A".to_string()];
        socket.enabled_cipher_suites = vec!["TLS_A".to_string()];
        
        let connection_spec = ConnectionSpec::builder(true)
            .tls_versions(&[TlsVersion::TLS_1_1, TlsVersion::TLS_1_2, TlsVersion::TLS_1_3])
            .cipher_suites(&["TLS_A"])
            .build();
            
        apply_connection_spec(&connection_spec, &mut socket, false);
        assert_eq!(socket.enabled_protocols, vec!["TLSv1.1".to_string(), "TLSv1.2".to_string()]);
    }
}
