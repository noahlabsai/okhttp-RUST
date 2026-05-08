use std::net::SocketAddr;
use std::sync::OnceLock;

/// Equivalent to java.net.Proxy
#[derive(Debug, Clone, PartialEq)]
pub enum Proxy {
    Direct,
    Http(SocketAddr),
    Socks(SocketAddr),
    NoProxy,
}

impl Proxy {
    pub const NO_PROXY: Proxy = Proxy::NoProxy;
}

/// Equivalent to java.net.URI
#[derive(Debug, Clone, PartialEq)]
pub struct Uri(pub String);

/// Equivalent to java.net.ProxySelector
pub trait ProxySelector: Send + Sync {
    fn select(&self, uri: Option<Uri>) -> Vec<Proxy>;
    fn connect_failed(&self, uri: Option<Uri>, sa: Option<SocketAddr>, ioe: Option<Box<dyn std::error::Error + Send + Sync>>);
}

/// A proxy selector that always returns the [Proxy::NO_PROXY].
pub struct NullProxySelector;

impl ProxySelector for NullProxySelector {
    fn select(&self, uri: Option<Uri>) -> Vec<Proxy> {
        // requireNotNull(uri) { "uri must not be null" }
        let _ = uri.expect("uri must not be null");
        vec![Proxy::NO_PROXY]
    }

    fn connect_failed(
        &self,
        _uri: Option<Uri>,
        _sa: Option<SocketAddr>,
        _ioe: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) {
        // No-op implementation as per Kotlin source
    }
}

static INSTANCE: OnceLock<NullProxySelector> = OnceLock::new();

/// Singleton instance of NullProxySelector
pub fn get_null_proxy_selector() -> &'static NullProxySelector {
    INSTANCE.get_or_init(|| NullProxySelector)
}
