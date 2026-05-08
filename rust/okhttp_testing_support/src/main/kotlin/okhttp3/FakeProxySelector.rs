use std::net::{Proxy, SocketAddr};
use std::io;
use url::Url;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::proxy::NullProxySelector::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

// A fake implementation of `ProxySelector` for testing purposes.
// 
// In Rust, since there is no direct equivalent to the JVM's `java.net.ProxySelector` 
// base class, we define this as a struct that provides the same logic.
#[derive(Debug, Clone, Default)]
pub struct FakeProxySelector {
    pub proxies: Vec<Proxy>,
}

impl FakeProxySelector {
    pub fn new() -> Self {
        Self {
            proxies: Vec::new(),
        }
    }

    // Adds a proxy to the list of available proxies.
    // Returns a mutable reference to self to allow chaining, mimicking the Kotlin return this.
    pub fn add_proxy(&mut self, proxy: Proxy) -> &mut Self {
        self.proxies.push(proxy);
        self
    }

    // Selects the proxies to use for the given URI.
    // 
    // Behavioral correctness:
    // - If the scheme is "http" or "https", return the configured proxies.
    // - Otherwise (e.g., 'socket' schemes), return a list containing only NO_PROXY.
    pub fn select(&self, uri: &Url) -> Vec<Proxy> {
        match uri.scheme() {
            "http" | "https" => self.proxies.clone(),
            _ => vec![Proxy::NO_PROXY],
        }
    }

    // Called when a connection to the given socket address fails.
    // This is a no-op in the original Kotlin implementation.
    pub fn connect_failed(
        &self,
        _uri: &Url,
        _sa: SocketAddr,
        _ioe: io::Error,
    ) {
        // No-op to preserve business behavior
    }
}

// To maintain the "interface" feel of the original ProxySelector, 
// we can define a trait if other selectors are needed.
pub trait ProxySelectorTrait {
    fn select(&self, uri: &Url) -> Vec<Proxy>;
    fn connect_failed(&self, uri: &Url, sa: SocketAddr, ioe: io::Error);
}

impl ProxySelectorTrait for FakeProxySelector {
    fn select(&self, uri: &Url) -> Vec<Proxy> {
        self.select(uri)
    }

    fn connect_failed(&self, uri: &Url, sa: SocketAddr, ioe: io::Error) {
        self.connect_failed(uri, sa, ioe);
    }
}