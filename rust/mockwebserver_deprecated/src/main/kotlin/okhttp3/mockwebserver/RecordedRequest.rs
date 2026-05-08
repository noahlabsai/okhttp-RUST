use std::io::IOException;
use std::net::{IpAddr};
use std::fmt;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Handshake;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrl;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::TlsVersion;
use okio::Buffer;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::TlsVersion::*;
use crate::mockwebserver_deprecated::src::main::kotlin::okhttp3::mockwebserver::Dispatcher::*;


impl RecordedRequest {
    // Internal constructor equivalent to the Kotlin internal constructor
    pub fn new_internal(
        request_line: String,
        headers: Headers,
        chunk_sizes: Vec<i32>,
        body_size: i64,
        body: Buffer,
        sequence_number: i32,
        failure: Option<IOException>,
        method: Option<String>,
        path: Option<String>,
        handshake: Option<Handshake>,
        request_url: Option<HttpUrl>,
    ) -> Self {
        Self {
            request_line,
            headers,
            chunk_sizes,
            body_size,
            body,
            sequence_number,
            failure,
            method,
            path,
            handshake,
            request_url,
        }
    }

    // Primary constructor equivalent to the Kotlin @JvmOverloads constructor
    pub fn new(
        request_line: String,
        headers: Headers,
        chunk_sizes: Vec<i32>,
        body_size: i64,
        body: Buffer,
        sequence_number: i32,
        socket: &std::net::TcpStream,
        failure: Option<IOException>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // In the original Kotlin code, SSLSocket is checked to extract the handshake.
        // In this Rust translation, we simulate the logic.
        let handshake = None; 

        let mut method = None;
        let mut path = None;
        let mut request_url = None;

        if !request_line.is_empty() {
            if let Some(method_end) = request_line.find(' ') {
                let method_str = &request_line[0..method_end];
                
                if let Some(path_end_relative) = request_line[method_end + 1..].find(' ') {
                    let path_start = method_end + 1;
                    let path_end_abs = path_start + path_end_relative;
                    let mut path_str = request_line[path_start..path_end_abs].to_string();
                    
                    if !path_str.starts_with('/') {
                        path_str = "/".to_string();
                    }
                    
                    method = Some(method_str.to_string());
                    path = Some(path_str.clone());

                    // Logic for scheme and hostname
                    let scheme = "http"; // Simplified: would be "https" if SSLSocket
                    let local_addr = socket.local_addr()?;
                    
                    let hostname = match local_addr.ip() {
                        IpAddr::V6(ipv6) => format!("[{}]", ipv6),
                        IpAddr::V4(ipv4) => ipv4.to_string(),
                    };

                    let local_port = local_addr.port();
                    let url_string = format!("{}://{}:{}{}", scheme, hostname, local_port, path_str);
                    
                    request_url = HttpUrl::parse_or_null(&url_string);
                }
            }
        }

        Ok(Self {
            request_line,
            headers,
            chunk_sizes,
            body_size,
            body,
            sequence_number,
            failure,
            method,
            path,
            handshake,
            request_url,
        })
    }

    // @Deprecated: Use body.read_utf8()
    pub fn get_utf8_body(&self) -> String {
        let mut buf_clone = self.body.clone();
        buf_clone.read_utf8().unwrap_or_default()
    }

    pub fn get_header(&self, name: &str) -> Option<String> {
        self.headers.values(name).first().cloned()
    }

    pub fn tls_version(&self) -> Option<TlsVersion> {
        self.handshake.as_ref().and_then(|h| h.tls_version)
    }
}

impl fmt::Display for RecordedRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.request_line)
    }
}

impl fmt::Debug for RecordedRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RecordedRequest")
            .field("request_line", &self.request_line)
            .field("headers", &self.headers)
            .field("body_size", &self.body_size)
            .field("method", &self.method)
            .field("path", &self.path)
            .field("request_url", &self.request_url)
            .finish()
    }
}
