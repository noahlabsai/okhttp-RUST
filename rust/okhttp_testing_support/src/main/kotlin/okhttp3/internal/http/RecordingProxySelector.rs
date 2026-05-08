use std::fmt;
use std::net::SocketAddr;
use std::sync::Mutex;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;

// Mocking the Java Proxy class as it is a dependency of RecordingProxySelector

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProxyType {
    Direct,
    Http,
    Socks,
}

impl Default for ProxyType {
    fn default() -> Self {
        ProxyType::Direct
    }
}

pub const Direct: ProxyType = ProxyType::Direct;
pub const Http: ProxyType = ProxyType::Http;
pub const Socks: ProxyType = ProxyType::Socks;

// Mocking the Java URI class
#[derive(Debug, Clone, PartialEq)]
pub struct Uri(pub String);

impl fmt::Display for Uri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Mocking the Java ProxySelector class
pub trait ProxySelectorTrait {
    fn select(&self, uri: &Uri) -> Vec<Proxy>;
    fn connect_failed(&self, uri: &Uri, sa: SocketAddr, ioe: &dyn std::error::Error);
}

#[derive(Debug, Default)]
pub struct RecordingProxySelector {
    pub proxies: Mutex<Vec<Proxy>>,
    pub requested_uris: Mutex<Vec<Uri>>,
    pub failures: Mutex<Vec<String>>,
}

impl RecordingProxySelector {
    pub fn new() -> Self {
        Self::default()
    }

    // Equivalent to assertRequests(vararg expectedUris: URI?)
    pub fn assert_requests(&self, expected_uris: Vec<Option<Uri>>) {
        let mut requested = self.requested_uris.lock().unwrap();
        
        // Convert requested_uris to Vec<Option<Uri>> for comparison
        let actual: Vec<Option<Uri>> = requested.iter().map(|u| Some(u.clone())).collect();
        
        assert_eq!(actual, expected_uris, "Requested URIs did not match expected URIs");
        
        requested.clear();
    }
}

impl ProxySelectorTrait for RecordingProxySelector {
    fn select(&self, uri: &Uri) -> Vec<Proxy> {
        let mut requested = self.requested_uris.lock().unwrap();
        requested.push(uri.clone());
        
        let proxies = self.proxies.lock().unwrap();
        proxies.clone()
    }

    fn connect_failed(&self, uri: &Uri, sa: SocketAddr, ioe: &dyn std::error::Error) {
        // In Rust, SocketAddr is the equivalent of SocketAddress.
        let socket_address = sa; 
        let port = socket_address.port();
        
        // Kotlin: ioe.message!! -> .to_string()
        let message = ioe.to_string();
        
        let failure_msg = format!("{} {}:{} {}", uri, socket_address, port, message);
        let mut failures = self.failures.lock().unwrap();
        failures.push(failure_msg);
    }
}

impl fmt::Display for RecordingProxySelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RecordingProxySelector")
    }
}
