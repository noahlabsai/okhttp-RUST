use std::fmt;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Handshake;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrl;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;

// ByteString is typically a wrapper around Vec<u8> in Rust for Okio-like behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteString(pub Vec<u8>);

// An HTTP request that came into the mock web server.
#[derive(Clone)]
pub struct RecordedRequest {
    // The index of the socket connection that carried this request. If two recorded requests share a
    // connection index, they also shared a socket connection.
    pub connection_index: i32,
    // The index of this exchange on its HTTP connection. A request is uniquely identified by the
    // (connection index, exchange index) pair.
    pub exchange_index: i32,
    // The TLS handshake of the connection that carried this request, or null if the request was
    // received without TLS.
    pub handshake: Option<Handshake>,
    // Returns the name of the server the client requested via the SNI (Server Name Indication)
    // attribute in the TLS handshake. Unlike the rest of the HTTP exchange, this name is sent in
    // cleartext and may be monitored or blocked by a proxy or other middlebox.
    pub handshake_server_names: Vec<String>,
    // A string like `GET` or `POST`.
    pub method: String,
    // The request target from the original HTTP request.
    //
    // For origin-form requests this is a path like `/index.html`, that is combined with the `Host`
    // header to create the request URL.
    //
    // For HTTP proxy requests this will be either an absolute-form string like
    // `http://example.com/index.html` (HTTP proxy) or an authority-form string like
    // `example.com:443` (HTTPS proxy).
    //
    // For OPTIONS requests, this may be an asterisk, `*`.
    pub target: String,
    // A string like `HTTP/1.1` or `HTTP/2`.
    pub version: String,
    // The request URL built using the request line, headers, and local host name.
    pub url: HttpUrl,
    // All headers.
    pub headers: Headers,
    // The body of this request, or null if it has none. This may be truncated.
    pub body: Option<ByteString>,
    // The total size of the body of this request (before truncation).
    pub body_size: i64,
    // The sizes of the chunks of this request's body, or null if the request's body was not encoded
    // with chunked encoding.
    pub chunk_sizes: Option<Vec<i32>>,
    // The failure MockWebServer recorded when attempting to decode this request. If, for example,
    // the inbound request was truncated, this exception will be non-null.
    // Note: Box<dyn Error> is not Clone, so we use a custom approach or omit Clone for the whole struct.
    // Since the Kotlin source has a default null, we use Option.
    pub failure: Option<std::sync::Arc<dyn std::error::Error + Send + Sync>>,
}

impl RecordedRequest {
    // The request line: "$method $target $version"
    pub fn request_line(&self) -> String {
        format!("{} {} {}", self.method, self.target, self.version)
    }
}

impl fmt::Display for RecordedRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.request_line())
    }
}

impl fmt::Debug for RecordedRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RecordedRequest")
            .field("connection_index", &self.connection_index)
            .field("exchange_index", &self.exchange_index)
            .field("handshake", &self.handshake)
            .field("handshake_server_names", &self.handshake_server_names)
            .field("method", &self.method)
            .field("target", &self.target)
            .field("version", &self.version)
            .field("url", &self.url)
            .field("headers", &self.headers)
            .field("body", &self.body)
            .field("body_size", &self.body_size)
            .field("chunk_sizes", &self.chunk_sizes)
            .field("failure", &self.failure)
            .finish()
    }
}

impl PartialEq for RecordedRequest {
    fn eq(&self, other: &Self) -> bool {
        self.connection_index == other.connection_index
            && self.exchange_index == other.exchange_index
            && self.handshake == other.handshake
            && self.handshake_server_names == other.handshake_server_names
            && self.method == other.method
            && self.target == other.target
            && self.version == other.version
            && self.url == other.url
            && self.headers == other.headers
            && self.body == other.body
            && self.body_size == other.body_size
            && self.chunk_sizes == other.chunk_sizes
            // failure is a trait object and cannot be compared for equality easily
    }
}