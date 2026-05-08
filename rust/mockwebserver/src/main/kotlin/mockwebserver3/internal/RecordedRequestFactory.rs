use std::error::Error;
use std::fmt;
use std::net::IpAddr;
use std::sync::Arc;

use crate::mockwebserver::src::main::kotlin::mockwebserver3::RecordedRequest;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrl;
use crate::okio::ByteString;

// MockWebServerSocket is a dependency not provided in the snippet, 
// but required for the logic. We define it based on the usage in the Kotlin source.
pub struct MockWebServerSocket {
    pub scheme: String,
    pub handshake: Option<crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Handshake>,
    pub handshake_server_names: Vec<String>,
    pub local_address: IpAddr,
    pub local_port: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RequestLine {
    pub method: String,
    pub target: String,
    pub version: String,
}

impl fmt::Display for RequestLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.method, self.target, self.version)
    }
}

impl RequestLine {
    fn new(method: &str, target: &str, version: &str) -> Self {
        Self {
            method: method.to_string(),
            target: target.to_string(),
            version: version.to_string(),
        }
    }
}

pub fn default_request_line_http_1() -> RequestLine {
    RequestLine::new("GET", "/", "HTTP/1.1")
}

pub fn default_request_line_http_2() -> RequestLine {
    RequestLine::new("GET", "/", "HTTP/2")
}

pub fn recorded_request(
    request_line: RequestLine,
    headers: Headers,
    chunk_sizes: Option<Vec<i32>>,
    body_size: i64,
    body: Option<ByteString>,
    connection_index: i32,
    exchange_index: i32,
    socket: &MockWebServerSocket,
    failure: Option<Arc<dyn Error + Send + Sync>>,
) -> RecordedRequest {
    let request_url = if request_line.method == "CONNECT" {
        format!("{}://{}/", socket.scheme, request_line.target)
            .to_http_url_or_null()
    } else {
        None
    }
    .or_else(|| request_line.target.to_http_url_or_null())
    .unwrap_or_else(|| request_url_internal(socket, &request_line, &headers));

    RecordedRequest {
        connection_index,
        exchange_index,
        handshake: socket.handshake.clone(),
        handshake_server_names: socket.handshake_server_names.clone(),
        method: request_line.method,
        target: request_line.target,
        version: request_line.version,
        url: request_url,
        headers,
        body,
        body_size,
        chunk_sizes,
        failure,
    }
}

pub fn decode_request_line(request_line: Option<String>) -> Result<RequestLine, Box<dyn Error>> {
    let line = match request_line {
        Some(l) => l,
        None => return Ok(default_request_line_http_1()),
    };

    let parts: Vec<&str> = line.splitn(3, ' ').collect();

    if parts.len() != 3 {
        return Err(Box::from(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("unexpected request line: {}", line),
        )));
    }

    Ok(RequestLine {
        method: parts[0].to_string(),
        target: parts[1].to_string(),
        version: parts[2].to_string(),
    })
}

fn request_url_internal(
    socket: &MockWebServerSocket,
    request_line: &RequestLine,
    headers: &Headers,
) -> HttpUrl {
    let host_and_port = headers.get(":authority")
        .or_else(|| headers.get("Host"))
        .unwrap_or_else(|| {
            match socket.local_address {
                IpAddr::V6(addr) => format!("[{}]:{}", addr, socket.local_port),
                IpAddr::V4(addr) => format!("{}:{}", addr, socket.local_port),
            }
        });

    let path = if request_line.method == "OPTIONS" && request_line.target == "*" {
        "/"
    } else {
        &request_line.target
    };

    format!("{}://{}{}", socket.scheme, host_and_port, path).to_http_url()
}

// Extension trait to mimic Kotlin's String.toHttpUrl() and String.toHttpUrlOrNull()
pub trait HttpUrlExt {
    fn to_http_url(&self) -> HttpUrl;
    fn to_http_url_or_null(&self) -> Option<HttpUrl>;
}

impl HttpUrlExt for String {
    fn to_http_url(&self) -> HttpUrl {
        self.to_http_url_or_null().expect("Invalid URL")
    }

    fn to_http_url_or_null(&self) -> Option<HttpUrl> {
        HttpUrl::parse(self)
    }
}

impl HttpUrlExt for &str {
    fn to_http_url(&self) -> HttpUrl {
        self.to_http_url_or_null().expect("Invalid URL")
    }

    fn to_http_url_or_null(&self) -> Option<HttpUrl> {
        HttpUrl::parse(self)
    }
}
