use std::io::{Read, Write, BufReader, BufRead};
use std::net::{HttpURLConnection, HttpsURLConnection, ConnectException};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use std::thread;

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::RecordingHostnameVerifier::*;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::testing::PlatformRule::*;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::HandshakeCertificates::*;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::HeldCertificate::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::SingleAndroidTest::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::MockResponse::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::RecordedRequest::*;
use crate::mockwebserver::src::test::java::mockwebserver3::CustomDispatcherTest::*;
use crate::mockwebserver_deprecated::src::main::kotlin::okhttp3::mockwebserver::Dispatcher::*;
use crate::mockwebserver_deprecated::src::main::kotlin::okhttp3::mockwebserver::SocketPolicy::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::ProtocolTest::*;

use crate::mockwebserver::src::main::kotlin::mockwebserver3::{
    MockWebServer, MockResponse, SocketPolicy, RecordedRequest
};

pub struct MockWebServerTest {
    platform: PlatformRule,
    server: MockWebServer,
}

impl MockWebServerTest {
    pub fn new() -> Self {
        Self {
            platform: PlatformRule::new(),
            server: MockWebServer::new(),
        }
    }

    pub fn set_up(&mut self) {
        self.server.start();
    }

    pub fn tear_down(&mut self) {
        self.server.shutdown();
    }

    fn headers_to_list(&self, response: &MockResponse) -> Vec<String> {
        let headers = response.headers();
        let size = headers.size();
        let mut header_list = Vec::with_capacity(size);
        for i in 0..size {
            header_list.push(format!("{}: {}", headers.name(i), headers.value(i)));
        }
        header_list
    }

    pub fn default_mock_response(&self) {
        let response = MockResponse::new();
        assert_eq!(self.headers_to_list(&response), vec!["Content-Length: 0"]);
        assert_eq!(response.status(), "HTTP/1.1 200 OK");
    }

    pub fn set_response_mock_reason(&self) {
        let reasons = [
            "Mock Response",
            "Informational",
            "OK",
            "Redirection",
            "Client Error",
            "Server Error",
            "Mock Response",
        ];
        for i in 0..600 {
            let response = MockResponse::new().set_response_code(i);
            let expected_reason = reasons[(i / 100) as usize];
            assert_eq!(response.status(), format!("HTTP/1.1 {} {}", i, expected_reason));
            assert_eq!(self.headers_to_list(&response), vec!["Content-Length: 0"]);
        }
    }

    pub fn set_status_controls_whole_status_line(&self) {
        let response = MockResponse::new().set_status("HTTP/1.1 202 That'll do pig");
        assert_eq!(self.headers_to_list(&response), vec!["Content-Length: 0"]);
        assert_eq!(response.status(), "HTTP/1.1 202 That'll do pig");
    }

    pub fn set_body_adjusts_headers(&self) {
        let response = MockResponse::new().set_body("ABC");
        assert_eq!(self.headers_to_list(&response), vec!["Content-Length: 3"]);
        let body = response.get_body().expect("Body should be present");
        let mut reader = BufReader::new(body);
        let mut content = String::new();
        reader.read_to_string(&mut content).unwrap();
        assert_eq!(content, "ABC");
    }

    pub fn mock_response_add_header(&self) {
        let response = MockResponse::new()
            .clear_headers()
            .add_header("Cookie: s=square")
            .add_header("Cookie", "a=android");
        assert_eq!(
            self.headers_to_list(&response),
            vec!["Cookie: s=square", "Cookie: a=android"]
        );
    }

    pub fn mock_response_set_header(&self) {
        let response = MockResponse::new()
            .clear_headers()
            .add_header("Cookie: s=square")
            .add_header("Cookie: a=android")
            .add_header("Cookies: delicious");
        
        let mut response = response;
        response.set_header("cookie", "r=robot");
        assert_eq!(
            self.headers_to_list(&response),
            vec!["Cookies: delicious", "cookie: r=robot"]
        );
    }

    pub fn mock_response_set_headers(&self) {
        let response = MockResponse::new()
            .clear_headers()
            .add_header("Cookie: s=square")
            .add_header("Cookies: delicious");
        
        let mut response = response;
        let mut builder = Builder::new();
        builder.add("Cookie", "a=android");
        response.set_headers(builder.build());
        assert_eq!(self.headers_to_list(&response), vec!["Cookie: a=android"]);
    }

    pub fn regular_response(&mut self) {
        self.server.enqueue(MockResponse::new().set_body("hello world"));
        let url = self.server.url("/").to_url();
        let connection = url.open_connection() as HttpURLConnection;
        connection.set_request_property("Accept-Language", "en-US");
        
        let mut reader = BufReader::new(connection.input_stream());
        assert_eq!(connection.get_response_code(), 200);
        
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert_eq!(line.trim(), "hello world");
        
        let request = self.server.take_request();
        assert_eq!(request.request_line, "GET / HTTP/1.1");
        assert_eq!(request.get_header("Accept-Language"), Some("en-US".to_string()));

        assert!(self.server.take_request_with_timeout(100, Duration::from_millis(100)).is_none());
    }

    pub fn redirect(&mut self) {
        self.server.enqueue(
            MockResponse::new()
                .set_response_code(302)
                .add_header(&format!("Location: {}", self.server.url("/new-path")))
                .set_body("This page has moved!"),
        );
        self.server.enqueue(MockResponse::new().set_body("This is the new location!"));
        
        let connection = self.server.url("/").to_url().open_connection();
        let mut reader = BufReader::new(connection.get_input_stream());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert_eq!(line.trim(), "This is the new location!");
        
        let first = self.server.take_request();
        assert_eq!(first.request_line, "GET / HTTP/1.1");
        let redirect = self.server.take_request();
        assert_eq!(redirect.request_line, "GET /new-path HTTP/1.1");
    }

    pub fn dispatch_blocks_waiting_for_enqueue(&mut self) {
        let server_ref = self.server.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(1000));
            server_ref.enqueue(MockResponse::new().set_body("enqueued in the background"));
        });
        
        let connection = self.server.url("/").to_url().open_connection();
        let mut reader = BufReader::new(connection.get_input_stream());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert_eq!(line.trim(), "enqueued in the background");
    }

    pub fn non_hexadecimal_chunk_size(&mut self) {
        self.server.enqueue(
            MockResponse::new()
                .set_body("G\r\nxxxxxxxxxxxxxxxx\r\n0\r\n\r\n")
                .clear_headers()
                .add_header("Transfer-encoding: chunked"),
        );
        let connection = self.server.url("/").to_url().open_connection();
        let mut input_stream = connection.get_input_stream();
        
        let result = input_stream.read(&mut [0u8; 1]);
        assert!(result.is_err(), "Expected IOException for non-hex chunk size");
    }

    pub fn response_timeout(&mut self) {
        self.server.enqueue(
            MockResponse::new()
                .set_body("ABC")
                .clear_headers()
                .add_header("Content-Length: 4"),
        );
        self.server.enqueue(MockResponse::new().set_body("DEF"));
        
        let url_connection = self.server.url("/").to_url().open_connection();
        url_connection.set_read_timeout(Duration::from_millis(1000));
        let mut input_stream = url_connection.get_input_stream();
        
        let mut buf = [0u8; 1];
        assert_eq!(input_stream.read_exact(&mut buf).unwrap()[0], b'A');
        assert_eq!(input_stream.read_exact(&mut buf).unwrap()[0], b'B');
        assert_eq!(input_stream.read_exact(&mut buf).unwrap()[0], b'C');
        
        let result = input_stream.read_exact(&mut buf);
        assert!(result.is_err(), "Expected timeout");

        let url_connection2 = self.server.url("/").to_url().open_connection();
        let mut in2 = url_connection2.get_input_stream();
        assert_eq!(in2.read_exact(&mut buf).unwrap()[0], b'D');
        assert_eq!(in2.read_exact(&mut buf).unwrap()[0], b'E');
        assert_eq!(in2.read_exact(&mut buf).unwrap()[0], b'F');
        assert_eq!(in2.read(&mut buf).unwrap(), 0);
        
        assert_eq!(self.server.take_request().sequence_number, 0);
        assert_eq!(self.server.take_request().sequence_number, 0);
    }

    pub fn disconnect_at_start(&mut self) {
        self.server.enqueue(MockResponse::new().set_socket_policy(SocketPolicy::DisconnectAtStart));
        self.server.enqueue(MockResponse::new());
        self.server.enqueue(MockResponse::new());
        
        let result = self.server.url("/a").to_url().open_connection().get_input_stream().read(&mut [0u8; 1]);
        assert!(result.is_err());
        
        let result_ok = self.server.url("/b").to_url().open_connection().get_input_stream().read(&mut [0u8; 1]);
        assert!(result_ok.is_ok());
    }

    pub fn throttle_request(&mut self) {
        self.server.enqueue(
            MockResponse::new().throttle_body(3, 500, Duration::from_millis(500)),
        );
        let start = Instant::now();
        let connection = self.server.url("/").to_url().open_connection() as HttpURLConnection;
        connection.set_do_output(true);
        connection.output_stream().write_all(b"ABCDEF").unwrap();
        
        let mut input_stream = connection.input_stream();
        let mut buf = [0u8; 1];
        assert_eq!(input_stream.read(&mut buf).unwrap(), 0);
        
        let elapsed = start.elapsed().as_millis() as i64;
        assert!(elapsed >= 500 && elapsed <= 1000);
    }

    pub fn throttle_response(&mut self) {
        self.server.enqueue(
            MockResponse::new()
                .set_body("ABCDEF")
                .throttle_body(3, 500, Duration::from_millis(500)),
        );
        let start = Instant::now();
        let connection = self.server.url("/").to_url().open_connection();
        let mut input_stream = connection.get_input_stream();
        
        let mut buf = [0u8; 1];
        for &expected in b"ABCDEF" {
            input_stream.read_exact(&mut buf).unwrap();
            assert_eq!(buf[0], expected);
        }
        assert_eq!(input_stream.read(&mut buf).unwrap(), 0);
        
        let elapsed = start.elapsed().as_millis() as i64;
        assert!(elapsed >= 500 && elapsed <= 1000);
    }

    pub fn delay_response(&mut self) {
        self.server.enqueue(
            MockResponse::new()
                .set_body("ABCDEF")
                .set_body_delay(1, Duration::from_secs(1)),
        );
        let start = Instant::now();
        let connection = self.server.url("/").to_url().open_connection();
        let mut input_stream = connection.get_input_stream();
        
        let mut buf = [0u8; 1];
        input_stream.read_exact(&mut buf).unwrap();
        
        let elapsed = start.elapsed().as_millis() as i64;
        assert!(elapsed >= 1000);
    }

    pub fn disconnect_request_halfway(&mut self) {
        self.server.enqueue(MockResponse::new().set_socket_policy(SocketPolicy::DisconnectDuringRequestBody));
        self.server.set_body_limit(7 * 512 * 1024);
        
        let connection = self.server.url("/").to_url().open_connection() as HttpURLConnection;
        connection.set_request_method("POST");
        connection.set_do_output(true);
        connection.set_fixed_length_streaming_mode(1024 * 1024 * 1024);
        connection.connect();
        
        let mut out = connection.output_stream();
        let data = vec![0u8; 1024 * 1024];
        let mut i = 0;
        while i < 1024 {
            if let Err(_) = out.write_all(&data).and_then(|_| out.flush()) {
                break;
            }
            if i == 513 {
                thread::sleep(Duration::from_millis(100));
            }
            i += 1;
        }
        let i_float = i as f32;
        assert!((i_float - 512.0).abs() < 5.0);
    }

    pub fn disconnect_response_halfway(&mut self) {
        self.server.enqueue(
            MockResponse::new()
                .set_body("ab")
                .set_socket_policy(SocketPolicy::DisconnectDuringResponseBody),
        );
        let connection = self.server.url("/").to_url().open_connection();
        assert_eq!(connection.get_content_length(), 2);
        
        let mut input_stream = connection.get_input_stream();
        let mut buf = [0u8; 1];
        assert_eq!(input_stream.read_exact(&mut buf).unwrap()[0], b'a');
        
        let result = input_stream.read(&mut buf);
        match result {
            Ok(0) => {}, // OpenJDK behavior
            Err(e) => {
                let msg = e.to_string();
                assert!(msg.contains("Premature EOF") || msg.contains("Protocol"));
            }
            _ => panic!("Unexpected read result"),
        }
    }

    pub fn shutdown_without_start(&self) {
        let server = MockWebServer::new();
        server.shutdown();
    }

    pub fn close_via_closable(&self) {
        let server = MockWebServer::new();
        server.shutdown();
    }

    pub fn shutdown_without_enqueue(&self) {
        let mut server = MockWebServer::new();
        server.start();
        server.shutdown();
    }

    pub fn port_implicitly_starts(&self) {
        assert!(self.server.port() > 0);
    }

    pub fn hostname_implicitly_starts(&self) {
        assert!(self.server.host_name().is_some());
    }

    pub fn to_proxy_address_implicitly_starts(&self) {
        assert!(self.server.to_proxy_address().is_some());
    }

    pub fn different_instances_get_different_ports(&self) {
        let other = MockWebServer::new();
        assert_ne!(other.port(), self.server.port());
        other.shutdown();
    }

    pub fn statement_starts_and_stops(&mut self) {
        let called = std::sync::Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();
        
        {
            called_clone.store(true, Ordering::SeqCst);
            let connection = self.server.url("/").to_url().open_connection();
            connection.connect();
        }
        
        assert!(called.load(Ordering::SeqCst));
        
        self.server.shutdown();
        let result = self.server.url("/").to_url().open_connection().connect();
        assert!(result.is_err());
    }

    pub fn shutdown_while_blocked_dispatching(&mut self) {
        let connection = self.server.url("/").to_url().open_connection() as HttpURLConnection;
        connection.set_read_timeout(Duration::from_millis(500));
        
        let result = connection.get_response_code();
        assert!(result.is_err());

        self.server.shutdown();
    }

    pub fn request_url_reconstructed(&mut self) {
        self.server.enqueue(MockResponse::new().set_body("hello world"));
        let url = self.server.url("/a/deep/path?key=foo%20bar").to_url();
        let connection = url.open_connection() as HttpURLConnection;
        
        let mut reader = BufReader::new(connection.input_stream());
        assert_eq!(connection.get_response_code(), 200);
        
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert_eq!(line.trim(), "hello world");
        
        let request = self.server.take_request();
        assert_eq!(request.request_line, "GET /a/deep/path?key=foo%20bar HTTP/1.1");
        assert_eq!(request.path, "/a/deep/path?key=foo%20bar");
        
        let request_url = request.request_url.expect("URL should be present");
        assert_eq!(request_url.scheme, "http");
        assert_eq!(request_url.host, self.server.host_name().unwrap());
        assert_eq!(request_url.port, self.server.port());
        assert_eq!(request_url.encoded_path, "/a/deep/path");
        assert_eq!(request_url.query_parameter("key"), Some("foo bar".to_string()));
    }

    pub fn shutdown_server_after_request(&mut self) {
        self.server.enqueue(MockResponse::new().set_socket_policy(SocketPolicy::ShutdownServerAfterResponse));
        let url = self.server.url("/").to_url();
        let connection = url.open_connection() as HttpURLConnection;
        assert_eq!(connection.get_response_code(), 200);
        
        let refused_connection = url.open_connection() as HttpURLConnection;
        let result = refused_connection.get_response_code();
        assert!(result.is_err());
    }

    pub fn http_100_continue(&mut self) {
        self.server.enqueue(MockResponse::new().set_body("response"));
        let url = self.server.url("/").to_url();
        let connection = url.open_connection() as HttpURLConnection;
        connection.set_do_output(true);
        connection.set_request_property("Expect", "100-Continue");
        
        connection.output_stream().write_all(b"request").unwrap();
        
        let mut reader = BufReader::new(connection.input_stream());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert_eq!(line.trim(), "response");
        
        let request = self.server.take_request();
        let mut body_reader = BufReader::new(request.body);
        let mut body_content = String::new();
        body_reader.read_to_string(&mut body_content).unwrap();
        assert_eq!(body_content, "request");
    }

    pub fn test_h2_prior_knowledge_server_fallback(&mut self) {
        let protocols = vec![Protocol::H2PriorKnowledge, Protocol::Http11];
        let result = self.server.set_protocols(protocols);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("protocols containing h2_prior_knowledge cannot use other protocols"));
    }

    pub fn test_h2_prior_knowledge_server_duplicates(&mut self) {
        let protocols = vec![Protocol::H2PriorKnowledge, Protocol::H2PriorKnowledge];
        let result = self.server.set_protocols(protocols);
        assert!(result.is_err());
    }

    pub fn test_mock_web_server_h2_prior_knowledge_protocol(&mut self) {
        self.server.set_protocols(vec![Protocol::H2PriorKnowledge]).unwrap();
        assert_eq!(self.server.protocols().len(), 1);
        assert_eq!(self.server.protocols()[0], Protocol::H2PriorKnowledge);
    }

    pub fn https(&mut self) {
        let handshake_certificates = self.platform.localhost_handshake_certificates();
        self.server.use_https(handshake_certificates.ssl_socket_factory(), false);
        self.server.enqueue(MockResponse::new().set_body("abc"));
        
        let url = self.server.url("/").to_url();
        let connection = url.open_connection() as HttpsURLConnection;
        connection.set_ssl_socket_factory(handshake_certificates.ssl_socket_factory());
        connection.set_hostname_verifier(RecordingHostnameVerifier::new());
        
        assert_eq!(connection.get_response_code(), 200);
        let mut reader = BufReader::new(connection.input_stream());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert_eq!(line.trim(), "abc");
        
        let request = self.server.take_request();
        assert_eq!(request.request_url.unwrap().scheme, "https");
        let handshake = request.handshake.expect("Handshake should be present");
        assert!(handshake.tls_version.is_some());
        assert!(handshake.cipher_suite.is_some());
        assert!(handshake.local_principal.is_some());
        assert_eq!(handshake.local_certificates.len(), 1);
        assert!(handshake.peer_principal.is_none());
        assert_eq!(handshake.peer_certificates.len(), 0);
    }

    pub fn https_with_client_auth(&mut self) {
        self.platform.assume_not_bouncy_castle();
        self.platform.assume_not_conscrypt();
        
        let client_ca = HeldCertificate::builder().certificate_authority(0).build();
        let server_ca = HeldCertificate::builder().certificate_authority(0).build();
        let server_cert = HeldCertificate::builder()
            .signed_by(server_ca.clone())
            .add_subject_alternative_name(self.server.host_name().unwrap())
            .build();
            
        let server_handshake = HandshakeCertificates::builder()
            .add_trusted_certificate(client_ca.certificate().clone())
            .held_certificate(server_cert)
            .build();
            
        self.server.use_https(server_handshake.ssl_socket_factory(), false);
        self.server.enqueue(MockResponse::new().set_body("abc"));
        self.server.request_client_auth();
        
        let client_cert = HeldCertificate::builder().signed_by(client_ca).build();
        let client_handshake = HandshakeCertificates::builder()
            .add_trusted_certificate(server_ca.certificate().clone())
            .held_certificate(client_cert)
            .build();
            
        let url = self.server.url("/").to_url();
        let connection = url.open_connection() as HttpsURLConnection;
        connection.set_ssl_socket_factory(client_handshake.ssl_socket_factory());
        connection.set_hostname_verifier(RecordingHostnameVerifier::new());
        
        assert_eq!(connection.get_response_code(), 200);
        let mut reader = BufReader::new(connection.input_stream());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert_eq!(line.trim(), "abc");
        
        let request = self.server.take_request();
        let handshake = request.handshake.expect("Handshake should be present");
        assert!(handshake.peer_principal.is_some());
        assert_eq!(handshake.peer_certificates.len(), 1);
    }

    pub fn shutdown_twice(&self) {
        let mut server2 = MockWebServer::new();
        server2.start();
        server2.shutdown();
        
        let result = server2.start();
        assert!(result.is_err());
        server2.shutdown();
    }
}
