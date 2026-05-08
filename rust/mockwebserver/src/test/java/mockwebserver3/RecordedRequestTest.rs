use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;

use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::{
    decode_request_line, DEFAULT_REQUEST_LINE_HTTP_1, MockWebServerSocket, RecordedRequest,
};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::RecordedRequest::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::SocketHandler::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::RecordedRequestFactory::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::UrlComponentEncodingTester::*;

// Mock implementation of a Socket to simulate network behavior in tests.
// In Kotlin, this extends java.net.Socket.
struct FakeSocket {
    local_address: IpAddr,
    local_port: u16,
    remote_address: IpAddr,
    remote_port: u16,
}

impl FakeSocket {
    fn new(local_address: IpAddr, local_port: u16) -> Self {
        Self {
            local_address,
            local_port,
            remote_address: local_address,
            remote_port: 1234,
        }
    }

    fn get_input_stream(&self) -> Box<dyn Read + Send + Sync> {
        // Simulating okio.Buffer().inputStream()
        Box::new(std::io::Cursor::new(vec![]))
    }

    fn get_output_stream(&self) -> Box<dyn Write + Send + Sync> {
        // Simulating okio.Buffer().outputStream()
        Box::new(std::io::Cursor::new(vec![]))
    }

    fn get_inet_address(&self) -> IpAddr {
        self.remote_address
    }

    fn get_local_address(&self) -> IpAddr {
        self.local_address
    }

    fn get_local_port(&self) -> u16 {
        self.local_port
    }

    fn get_port(&self) -> u16 {
        self.remote_port
    }
}

// Implementation of the MockWebServerSocket wrapper for the FakeSocket.
// This allows the RecordedRequest to query socket properties.
impl MockWebServerSocket {
    pub fn create_fake(local_addr: IpAddr, local_port: u16) -> Self {
        let fake = FakeSocket::new(local_addr, local_port);
        // In the actual implementation, MockWebServerSocket wraps a java.net.Socket.
        // Here we simulate the construction.
        MockWebServerSocket::new(fake)
    }
}

pub struct RecordedRequestTest {
    headers: Headers,
}

impl RecordedRequestTest {
    pub fn new() -> Self {
        Self {
            headers: Headers::EMPTY,
        }
    }

    #[test]
    pub fn test_ipv4(&self) {
        let socket = MockWebServerSocket::create_fake(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            80,
        );
        let request = RecordedRequest::new(
            DEFAULT_REQUEST_LINE_HTTP_1.clone(),
            self.headers.clone(),
            vec![],
            0,
            vec![], // ByteString.EMPTY
            0,
            0,
            socket,
        );
        assert_eq!(request.url().to_string(), "http://127.0.0.1/");
    }

    #[test]
    pub fn test_authority_form(&self) {
        let socket = MockWebServerSocket::create_fake(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            80,
        );
        let request_line = decode_request_line("CONNECT example.com:8080 HTTP/1.1");
        let request = RecordedRequest::new(
            request_line,
            self.headers.clone(),
            vec![],
            0,
            vec![],
            0,
            0,
            socket,
        );
        assert_eq!(request.target(), "example.com:8080");
        assert_eq!(request.url().to_string(), "http://example.com:8080/");
    }

    #[test]
    pub fn test_absolute_form(&self) {
        let socket = MockWebServerSocket::create_fake(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            80,
        );
        let request_line = decode_request_line("GET http://example.com:8080/index.html HTTP/1.1");
        let request = RecordedRequest::new(
            request_line,
            self.headers.clone(),
            vec![],
            0,
            vec![],
            0,
            0,
            socket,
        );
        assert_eq!(request.target(), "http://example.com:8080/index.html");
        assert_eq!(request.url().to_string(), "http://example.com:8080/index.html");
    }

    #[test]
    pub fn test_asterisk_form(&self) {
        let socket = MockWebServerSocket::create_fake(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            80,
        );
        let request_line = decode_request_line("OPTIONS * HTTP/1.1");
        let request = RecordedRequest::new(
            request_line,
            self.headers.clone(),
            vec![],
            0,
            vec![],
            0,
            0,
            socket,
        );
        assert_eq!(request.target(), "*");
        assert_eq!(request.url().to_string(), "http://127.0.0.1/");
    }

    #[test]
    pub fn test_ipv6(&self) {
        let socket = MockWebServerSocket::create_fake(
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
            80,
        );
        let request = RecordedRequest::new(
            DEFAULT_REQUEST_LINE_HTTP_1.clone(),
            self.headers.clone(),
            vec![],
            0,
            vec![],
            0,
            0,
            socket,
        );
        assert_eq!(request.url().to_string(), "http://[::1]/");
    }

    #[test]
    pub fn test_uses_local(&self) {
        let socket = MockWebServerSocket::create_fake(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            80,
        );
        let request = RecordedRequest::new(
            DEFAULT_REQUEST_LINE_HTTP_1.clone(),
            self.headers.clone(),
            vec![],
            0,
            vec![],
            0,
            0,
            socket,
        );
        assert_eq!(request.url().to_string(), "http://127.0.0.1/");
    }

    #[test]
    pub fn test_hostname(&self) {
        let headers = Headers::headers_of(&[("Host", "host-from-header.com")]);
        let socket = MockWebServerSocket::create_fake(
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
            80,
        );
        let request = RecordedRequest::new(
            DEFAULT_REQUEST_LINE_HTTP_1.clone(),
            headers,
            vec![],
            0,
            vec![],
            0,
            0,
            socket,
        );
        assert_eq!(request.url().to_string(), "http://host-from-header.com/");
    }
}