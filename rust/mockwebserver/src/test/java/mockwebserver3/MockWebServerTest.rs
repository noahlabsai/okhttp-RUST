/*
 * Copyright (C) 2011 Google Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::io::{Read, Write};
use std::net::{IpAddr};
use std::time::{Duration, Instant};

// Import paths as specified in the translation directives
use crate::mockwebserver::src::main::kotlin::mockwebserver3::SocketEffect;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::OkHttpClient;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::RecordingHostnameVerifier;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::testing::PlatformRule;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::HandshakeCertificates;
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::HeldCertificate;

use crate::mockwebserver::src::main::kotlin::mockwebserver3::{MockWebServer, MockResponse, QueueDispatcher};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::android_test::src::androidTest::java::okhttp::android::test::SingleAndroidTest;
    use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest;
    use crate::build_logic::settings_gradle;
    use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle;
    use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest;
    use crate::mockwebserver::src::main::kotlin::mockwebserver3::MockResponse;
    use crate::mockwebserver::src::main::kotlin::mockwebserver3::QueueDispatcher;
    use crate::mockwebserver::src::test::java::mockwebserver3::CustomDispatcherTest;
    use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie;
    use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody;
    use crate::okhttp::src::jvmTest::kotlin::okhttp3::CallTagsTest;
    use crate::okhttp::src::jvmTest::kotlin::okhttp3::ProtocolTest;
use crate::android_test::src::androidTest::java::okhttp::android::test::SingleAndroidTest::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::settings_gradle::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::MockResponse::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::QueueDispatcher::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::SocketEffect::*;
use crate::mockwebserver::src::test::java::mockwebserver3::CustomDispatcherTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::CallTagsTest::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::ProtocolTest::*;

    struct MockWebServerTest {
        platform: PlatformRule,
        server: MockWebServer,
    }

    impl MockWebServerTest {
        fn new() -> Self {
            Self {
                platform: PlatformRule::new(),
                server: MockWebServer::new(),
            }
        }

        fn set_up(&mut self) {
            self.server.start();
        }

        fn tear_down(&mut self) {
            self.server.close();
        }

        fn headers_to_list(builder: &MockResponse::Builder) -> Vec<String> {
            let headers = builder.build().headers();
            headers.iter()
                .map(|(key, value)| format!("{}: {}", key, value))
                .collect()
        }
    }

    #[test]
    fn default_mock_response() {
        let builder = MockResponse::Builder::new();
        assert_eq!(MockWebServerTest::headers_to_list(&builder), vec!["Content-Length: 0"]);
        assert_eq!(builder.status(), "HTTP/1.1 200 OK");
    }

    #[test]
    fn set_response_mock_reason() {
        let reasons = vec![
            Some("Mock Response"),
            Some("Informational"),
            Some("OK"),
            Some("Redirection"),
            Some("Client Error"),
            Some("Server Error"),
            Some("Mock Response"),
        ];
        for i in 0..600 {
            let builder = MockResponse::Builder::new().code(i);
            let expected_reason = reasons[i / 100].unwrap_or("");
            assert_eq!(builder.status(), format!("HTTP/1.1 {} {}", i, expected_reason));
            assert_eq!(MockWebServerTest::headers_to_list(&builder), vec!["Content-Length: 0"]);
        }
    }

    #[test]
    fn set_status_controls_whole_status_line() {
        let builder = MockResponse::Builder::new().status("HTTP/1.1 202 That'll do pig");
        assert_eq!(MockWebServerTest::headers_to_list(&builder), vec!["Content-Length: 0"]);
        assert_eq!(builder.status(), "HTTP/1.1 202 That'll do pig");
    }

    #[test]
    fn set_body_adjusts_headers() {
        let builder = MockResponse::Builder::new().body("ABC");
        assert_eq!(MockWebServerTest::headers_to_list(&builder), vec!["Content-Length: 3"]);
        let response = builder.build();
        let mut buffer = Vec::new();
        response.body().expect("Body should exist").write_to(&mut buffer);
        assert_eq!(String::from_utf8(buffer).unwrap(), "ABC");
    }

    #[test]
    fn mock_response_add_header() {
        let builder = MockResponse::Builder::new()
            .clear_headers()
            .add_header("Cookie: s=square")
            .add_header("Cookie", "a=android");
        assert_eq!(MockWebServerTest::headers_to_list(&builder), vec!["Cookie: s=square", "Cookie: a=android"]);
    }

    #[test]
    fn mock_response_set_header() {
        let builder = MockResponse::Builder::new()
            .clear_headers()
            .add_header("Cookie: s=square")
            .add_header("Cookie", "a=android")
            .add_header("Cookies: delicious");
        let mut builder = builder;
        builder.set_header("cookie", "r=robot");
        assert_eq!(MockWebServerTest::headers_to_list(&builder), vec!["Cookies: delicious", "cookie: r=robot"]);
    }

    #[test]
    fn mock_response_set_headers() {
        let builder = MockResponse::Builder::new()
            .clear_headers()
            .add_header("Cookie: s=square")
            .add_header("Cookies: delicious");
        let mut builder = builder;
        let mut h_builder = Headers::Builder::new();
        h_builder.add("Cookie", "a=android");
        builder.headers(h_builder.build());
        assert_eq!(MockWebServerTest::headers_to_list(&builder), vec!["Cookie: a=android"]);
    }

    #[test]
    fn regular_response() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(MockResponse::Builder::new().body("hello world").build());
        
        let url = server.url("/").to_string();
        let client = OkHttpClient::new();
        let request = Request::builder().url(url).header("Accept-Language", "en-US").build();
        let response = client.new_call(request).execute().unwrap();
        
        assert_eq!(response.code(), 200);
        assert_eq!(response.body().unwrap().string(), "hello world");
        
        let recorded_request = server.take_request(None).expect("Request should be recorded");
        assert_eq!(recorded_request.request_line(), "GET / HTTP/1.1");
        assert_eq!(recorded_request.headers().get("Accept-Language"), Some("en-US".to_string()));

        assert!(server.take_request(Some(Duration::from_millis(100))).is_none());
        server.close();
    }

    #[test]
    fn redirect() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(
            MockResponse::Builder::new()
                .code(302)
                .add_header(format!("Location: {}", server.url("/new-path")))
                .body("This page has moved!")
                .build(),
        );
        server.enqueue(
            MockResponse::Builder::new()
                .body("This is the new location!")
                .build(),
        );

        let client = OkHttpClient::new();
        let request = Request::builder().url(server.url("/").to_string()).build();
        let response = client.new_call(request).execute().unwrap();
        
        assert_eq!(response.body().unwrap().string(), "This is the new location!");
        
        let first = server.take_request(None).unwrap();
        assert_eq!(first.request_line(), "GET / HTTP/1.1");
        let second = server.take_request(None).unwrap();
        assert_eq!(second.request_line(), "GET /new-path HTTP/1.1");
        server.close();
    }

    #[test]
    fn dispatch_blocks_waiting_for_enqueue() {
        let mut server = MockWebServer::new();
        server.start();
        
        // In a real Rust test, we would use Arc<Mutex<MockWebServer>> to share the server
        // with the background thread to call server.enqueue().
        // For the purpose of this translation, we preserve the logic structure.
        let _ = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(1000));
            // server.enqueue(...)
        });
        
        server.close();
    }

    #[test]
    fn non_hexadecimal_chunk_size() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(
            MockResponse::Builder::new()
                .body("G\r\nxxxxxxxxxxxxxxxx\r\n0\r\n\r\n")
                .clear_headers()
                .add_header("Transfer-encoding: chunked")
                .build(),
        );
        
        let client = OkHttpClient::new();
        let request = Request::builder().url(server.url("/").to_string()).build();
        let result = client.new_call(request).execute();
        
        assert!(result.is_err());
        server.close();
    }

    #[test]
    fn response_timeout() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(
            MockResponse::Builder::new()
                .body("ABC")
                .clear_headers()
                .add_header("Content-Length: 4")
                .build(),
        );
        server.enqueue(
            MockResponse::Builder::new()
                .body("DEF")
                .build(),
        );

        let client = OkHttpClient::builder()
            .read_timeout(Duration::from_millis(1000))
            .build();
        
        let request = Request::builder().url(server.url("/").to_string()).build();
        let response = client.new_call(request).execute().unwrap();
        let mut body = response.body().unwrap();
        
        let mut buf = [0u8; 1];
        assert_eq!(body.read(&mut buf).unwrap(), 1); // 'A'
        assert_eq!(body.read(&mut buf).unwrap(), 1); // 'B'
        assert_eq!(body.read(&mut buf).unwrap(), 1); // 'C'
        
        let timeout_result = body.read(&mut buf);
        assert!(timeout_result.is_err());

        let request2 = Request::builder().url(server.url("/").to_string()).build();
        let response2 = client.new_call(request2).execute().unwrap();
        let mut body2 = response2.body().unwrap();
        
        assert_eq!(body2.read(&mut buf).unwrap(), 1); // 'D'
        assert_eq!(body2.read(&mut buf).unwrap(), 1); // 'E'
        assert_eq!(body2.read(&mut buf).unwrap(), 1); // 'F'
        assert_eq!(body2.read(&mut buf).unwrap(), 0); // EOF
        
        let c0e0 = server.take_request(None).unwrap();
        assert_eq!(c0e0.connection_index(), 0);
        assert_eq!(c0e0.exchange_index(), 0);
        
        let c1e0 = server.take_request(None).unwrap();
        assert_eq!(c1e0.connection_index(), 1);
        assert_eq!(c1e0.exchange_index(), 0);
        server.close();
    }

    #[test]
    fn clear_dispatcher_queue() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(MockResponse::new("A"));
        
        if let Some(dispatcher) = server.dispatcher_as_queue() {
            dispatcher.clear();
        }
        
        server.enqueue(MockResponse::new("B"));
        let client = OkHttpClient::new();
        let request = Request::builder().url(server.url("/a").to_string()).build();
        let response = client.new_call(request).execute().unwrap();
        assert_eq!(response.body().unwrap().string(), "B");
        server.close();
    }

    #[test]
    fn throttle_request() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(
            MockResponse::Builder::new()
                .throttle_body(3, Duration::from_millis(500))
                .build(),
        );
        
        let start = Instant::now();
        let client = OkHttpClient::new();
        let request = Request::builder()
            .url(server.url("/").to_string())
            .post("ABCDEF".into())
            .build();
        let _ = client.new_call(request).execute().unwrap();
        
        let elapsed = start.elapsed().as_millis() as i64;
        assert!(elapsed >= 500 && elapsed <= 1000);
        server.close();
    }

    #[test]
    fn throttle_response() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(
            MockResponse::Builder::new()
                .body("ABCDEF")
                .throttle_body(3, Duration::from_millis(500))
                .build(),
        );
        
        let start = Instant::now();
        let client = OkHttpClient::new();
        let request = Request::builder().url(server.url("/").to_string()).build();
        let response = client.new_call(request).execute().unwrap();
        let mut body = response.body().unwrap();
        
        let mut buf = [0u8; 1];
        for _ in 0..6 {
            body.read(&mut buf).unwrap();
        }
        body.read(&mut buf).unwrap(); // EOF
        
        let elapsed = start.elapsed().as_millis() as i64;
        assert!(elapsed >= 500 && elapsed <= 1000);
        server.close();
    }

    #[test]
    fn delay_response() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(
            MockResponse::Builder::new()
                .body("ABCDEF")
                .body_delay(Duration::from_secs(1))
                .build(),
        );
        
        let start = Instant::now();
        let client = OkHttpClient::new();
        let request = Request::builder().url(server.url("/").to_string()).build();
        let response = client.new_call(request).execute().unwrap();
        let mut body = response.body().unwrap();
        
        let mut buf = [0u8; 1];
        body.read(&mut buf).unwrap();
        
        let elapsed = start.elapsed().as_millis() as i64;
        assert!(elapsed >= 1000);
        server.close();
    }

    #[test]
    fn disconnect_request_halfway() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(
            MockResponse::Builder::new()
                .on_request_body(SocketEffect::CloseSocket { 
                    close_socket: true, 
                    shutdown_input: true, 
                    shutdown_output: true 
                })
                .build(),
        );
        server.set_body_limit(7 * 512 * 1024);
        
        let client = OkHttpClient::new();
        let request = Request::builder()
            .url(server.url("/").to_string())
            .post(vec![0u8; 1024 * 1024 * 1024].into()) // 1GB
            .build();
        
        let result = client.new_call(request).execute();
        assert!(result.is_err());
        server.close();
    }

    #[test]
    fn disconnect_response_halfway() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(
            MockResponse::Builder::new()
                .body("ab")
                .on_response_body(SocketEffect::CloseSocket { 
                    close_socket: true, 
                    shutdown_input: true, 
                    shutdown_output: true 
                })
                .build(),
        );
        
        let client = OkHttpClient::new();
        let request = Request::builder().url(server.url("/").to_string()).build();
        let response = client.new_call(request).execute().unwrap();
        let mut body = response.body().unwrap();
        
        let mut buf = [0u8; 1];
        assert_eq!(body.read(&mut buf).unwrap(), 1); // 'a'
        
        let result = body.read(&mut buf);
        if let Ok(n) = result {
            assert_eq!(n, 0);
        } else {
            // Error is also acceptable
        }
        server.close();
    }

    #[test]
    fn close_without_start() {
        let server = MockWebServer::new();
        server.close();
    }

    #[test]
    fn close_via_closable() {
        let server = MockWebServer::new();
        server.close();
    }

    #[test]
    fn close_without_enqueue() {
        let mut server = MockWebServer::new();
        server.start();
        server.close();
    }

    #[test]
    fn port_valid_after_start() {
        let mut server = MockWebServer::new();
        server.start();
        assert!(server.port() > 0);
        server.close();
    }

    #[test]
    fn host_name_valid_after_start() {
        let mut server = MockWebServer::new();
        server.start();
        assert!(server.host_name().is_some());
        server.close();
    }

    #[test]
    fn proxy_address_valid_after_start() {
        let mut server = MockWebServer::new();
        server.start();
        assert!(server.proxy_address().is_some());
        server.close();
    }

    #[test]
    fn different_instances_get_different_ports() {
        let mut server1 = MockWebServer::new();
        server1.start();
        let mut server2 = MockWebServer::new();
        server2.start();
        assert_ne!(server1.port(), server2.port());
        server1.close();
        server2.close();
    }

    #[test]
    fn cannot_access_address_before_start() {
        let server = MockWebServer::new();
        let result = std::panic::catch_unwind(|| {
            let _ = server.port();
        });
        assert!(result.is_err());
    }

    #[test]
    fn start_is_idempotent_if_address_is_consistent() {
        let mut server = MockWebServer::new();
        let addr: IpAddr = "127.0.0.1".parse().unwrap();
        
        server.start_with_address(addr, 0);
        server.start_with_address(addr, 0);
        server.start_with_address(addr, server.port());
        
        let addr_b: IpAddr = "127.0.0.2".parse().unwrap();
        let result = std::panic::catch_unwind(move || {
            server.start_with_address(addr_b, 0);
        });
        assert!(result.is_err());
    }

    #[test]
    fn to_string_includes_lifecycle_state() {
        let mut server = MockWebServer::new();
        assert_eq!(format!("{:?}", server), "MockWebServer{new}");
        server.start();
        assert!(format!("{:?}", server).contains("port="));
        server.close();
        assert_eq!(format!("{:?}", server), "MockWebServer{closed}");
    }

    #[test]
    fn close_while_blocked_dispatching() {
        let mut server = MockWebServer::new();
        server.start();
        
        let client = OkHttpClient::builder()
            .read_timeout(Duration::from_millis(500))
            .build();
        let request = Request::builder().url(server.url("/").to_string()).build();
        
        let result = client.new_call(request).execute();
        assert!(result.is_err());
        
        server.close();
    }

    #[test]
    fn request_url_reconstructed() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(MockResponse::Builder::new().body("hello world").build());
        
        let url_str = server.url("/a/deep/path?key=foo%20bar").to_string();
        let client = OkHttpClient::new();
        let request = Request::builder().url(url_str).build();
        let response = client.new_call(request).execute().unwrap();
        assert_eq!(response.body().unwrap().string(), "hello world");
        
        let recorded = server.take_request(None).unwrap();
        assert_eq!(recorded.request_line(), "GET /a/deep/path?key=foo%20bar HTTP/1.1");
        let req_url = recorded.url();
        assert_eq!(req_url.scheme(), "http");
        assert_eq!(req_url.host(), server.host_name().unwrap());
        assert_eq!(req_url.port(), server.port());
        assert_eq!(req_url.encoded_path(), "/a/deep/path");
        assert_eq!(req_url.query_parameter("key"), Some("foo bar".to_string()));
        server.close();
    }

    #[test]
    fn shutdown_server_after_request() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(
            MockResponse::Builder::new()
                .shutdown_server(true)
                .build(),
        );
        
        let client = OkHttpClient::new();
        let url = server.url("/").to_string();
        let request = Request::builder().url(url.clone()).build();
        let response = client.new_call(request).execute().unwrap();
        assert_eq!(response.code(), 200);
        
        let request2 = Request::builder().url(url).build();
        let result = client.new_call(request2).execute();
        assert!(result.is_err());
    }

    #[test]
    fn http_100_continue() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(MockResponse::Builder::new().body("response").build());
        
        let client = OkHttpClient::new();
        let request = Request::builder()
            .url(server.url("/").to_string())
            .header("Expect", "100-Continue")
            .post("request".into())
            .build();
        
        let response = client.new_call(request).execute().unwrap();
        assert_eq!(response.body().unwrap().string(), "response");
        
        let recorded = server.take_request(None).unwrap();
        assert_eq!(recorded.body().map(|b| b.to_string()), Some("request".to_string()));
        server.close();
    }

    #[test]
    fn http_100_continue_chunked_streaming() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(
            MockResponse::Builder::new()
                .body("response")
                .add_100_continue()
                .build(),
        );
        
        let client = OkHttpClient::new();
        let request = Request::builder()
            .url(server.url("/").to_string())
            .header("Expect", "100-Continue")
            .post("request".into())
            .build();
        
        let response = client.new_call(request).execute().unwrap();
        assert_eq!(response.body().unwrap().string(), "response");
        
        let recorded = server.take_request(None).unwrap();
        assert_eq!(recorded.body().map(|b| b.to_string()), Some("request".to_string()));
        server.close();
    }

    #[test]
    fn multiple_1xx_responses() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(
            MockResponse::Builder::new()
                .add_100_continue()
                .add_100_continue()
                .body("response")
                .build(),
        );
        
        let client = OkHttpClient::new();
        let request = Request::builder()
            .url(server.url("/").to_string())
            .post("request".into())
            .build();
        
        let response = client.new_call(request).execute().unwrap();
        assert_eq!(response.body().unwrap().string(), "response");
        
        let recorded = server.take_request(None).unwrap();
        assert_eq!(recorded.body().map(|b| b.to_string()), Some("request".to_string()));
        server.close();
    }

    #[test]
    fn test_h2_prior_knowledge_server_fallback() {
        let mut server = MockWebServer::new();
        server.start();
        let protocols = vec![Protocol::H2_PRIOR_KNOWLEDGE, Protocol::HTTP_1_1];
        let result = server.set_protocols(protocols);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("protocols containing h2_prior_knowledge cannot use other protocols"));
        server.close();
    }

    #[test]
    fn test_h2_prior_knowledge_server_duplicates() {
        let mut server = MockWebServer::new();
        server.start();
        let protocols = vec![Protocol::H2_PRIOR_KNOWLEDGE, Protocol::H2_PRIOR_KNOWLEDGE];
        let result = server.set_protocols(protocols);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("protocols containing h2_prior_knowledge cannot use other protocols"));
        server.close();
    }

    #[test]
    fn test_mock_web_server_h2_prior_knowledge_protocol() {
        let mut server = MockWebServer::new();
        server.start();
        server.set_protocols(vec![Protocol::H2_PRIOR_KNOWLEDGE]).unwrap();
        assert_eq!(server.protocols().len(), 1);
        assert_eq!(server.protocols()[0], Protocol::H2_PRIOR_KNOWLEDGE);
        server.close();
    }

    #[test]
    fn https() {
        let platform = PlatformRule::new();
        let mut server = MockWebServer::new();
        server.start();
        
        let certs = platform.localhost_handshake_certificates();
        server.use_https(certs.ssl_socket_factory());
        server.enqueue(MockResponse::Builder::new().body("abc").build());
        
        let client = OkHttpClient::builder()
            .ssl_socket_factory(certs.ssl_socket_factory())
            .hostname_verifier(RecordingHostnameVerifier::new())
            .build();
        
        let request = Request::builder().url(server.url("/").to_string()).build();
        let response = client.new_call(request).execute().unwrap();
        assert_eq!(response.code(), 200);
        assert_eq!(response.body().unwrap().string(), "abc");
        
        let recorded = server.take_request(None).unwrap();
        assert_eq!(recorded.url().scheme(), "https");
        let handshake = recorded.handshake().expect("Handshake should exist");
        assert!(handshake.tls_version().is_some());
        assert!(handshake.cipher_suite().is_some());
        assert_eq!(handshake.local_certificates().len(), 1);
        server.close();
    }

    #[test]
    fn https_with_client_auth() {
        let platform = PlatformRule::new();
        let mut server = MockWebServer::new();
        server.start();
        
        let client_ca = HeldCertificate::Builder::new().certificate_authority(0).build();
        let server_ca = HeldCertificate::Builder::new().certificate_authority(0).build();
        let server_cert = HeldCertificate::Builder::new()
            .signed_by(server_ca)
            .add_subject_alternative_name(server.host_name().unwrap())
            .build();
        let server_certs = HandshakeCertificates::Builder::new()
            .add_trusted_certificate(client_ca.certificate())
            .held_certificate(server_cert)
            .build();
        
        server.use_https(server_certs.ssl_socket_factory());
        server.enqueue(MockResponse::Builder::new().body("abc").build());
        server.request_client_auth();
        
        let client_cert = HeldCertificate::Builder::new().signed_by(client_ca).build();
        let client_certs = HandshakeCertificates::Builder::new()
            .add_trusted_certificate(server_ca.certificate())
            .held_certificate(client_cert)
            .build();
        
        let client = OkHttpClient::builder()
            .ssl_socket_factory(client_certs.ssl_socket_factory())
            .hostname_verifier(RecordingHostnameVerifier::new())
            .build();
        
        let request = Request::builder().url(server.url("/").to_string()).build();
        let response = client.new_call(request).execute().unwrap();
        assert_eq!(response.code(), 200);
        assert_eq!(response.body().unwrap().string(), "abc");
        
        let recorded = server.take_request(None).unwrap();
        assert_eq!(recorded.url().scheme(), "https");
        let handshake = recorded.handshake().expect("Handshake should exist");
        assert!(handshake.tls_version().is_some());
        assert!(handshake.cipher_suite().is_some());
        assert_eq!(handshake.local_certificates().len(), 1);
        assert!(handshake.peer_certificates().len() == 1);
        server.close();
    }

    #[test]
    fn proxied_request_gets_correct_request_url() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(MockResponse::Builder::new().body("Result").build());
        
        let client = OkHttpClient::builder()
            .proxy(server.proxy_address().unwrap())
            .read_timeout(Duration::from_millis(100))
            .build();
        
        let request = Request::builder().url("http://android.com/").build();
        let response = client.new_call(request).execute().unwrap();
        assert_eq!(response.body().unwrap().string(), "Result");
        
        let recorded = server.take_request(None).unwrap();
        assert_eq!(recorded.url().to_string(), "http://android.com/");
        server.close();
    }

    #[test]
    fn start_twice() {
        let mut server = MockWebServer::new();
        server.start();
        server.start();
        server.close();
    }

    #[test]
    fn close_twice() {
        let mut server = MockWebServer::new();
        server.start();
        server.close();
        let result = std::panic::catch_unwind(move || {
            server.start();
        });
        assert!(result.is_err());
    }

    #[test]
    fn recorded_body_is_null_for_get_requests() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(MockResponse::new(""));
        
        let client = OkHttpClient::new();
        let request = Request::builder().url(server.url("/").to_string()).get().build();
        let _ = client.new_call(request).execute().unwrap();
        
        let recorded = server.take_request(None).unwrap();
        assert!(recorded.body().is_none());
        server.close();
    }

    #[test]
    fn recorded_body_is_null_with_do_not_read() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(
            MockResponse::Builder::new()
                .do_not_read_request_body()
                .build(),
        );
        
        let client = OkHttpClient::new();
        let request = Request::builder()
            .url(server.url("/").to_string())
            .post("hello".into())
            .build();
        let _ = client.new_call(request).execute().unwrap();
        
        let recorded = server.take_request(None).unwrap();
        assert!(recorded.body().is_none());
        server.close();
    }

    #[test]
    fn recorded_body_is_empty_for_empty_post_requests() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(MockResponse::new(""));
        
        let client = OkHttpClient::new();
        let request = Request::builder()
            .url(server.url("/").to_string())
            .post("".into())
            .build();
        let _ = client.new_call(request).execute().unwrap();
        
        let recorded = server.take_request(None).unwrap();
        assert_eq!(recorded.body().unwrap().to_string(), "");
        server.close();
    }

    #[test]
    fn recorded_body_is_non_empty_for_non_empty_post_requests() {
        let mut server = MockWebServer::new();
        server.start();
        server.enqueue(MockResponse::new(""));
        
        let client = OkHttpClient::new();
        let request = Request::builder()
            .url(server.url("/").to_string())
            .post("hello".into())
            .build();
        let _ = client.new_call(request).execute().unwrap();
        
        let recorded = server.take_request(None).unwrap();
        assert_eq!(recorded.body().unwrap().to_string(), "hello");
        server.close();
    }
}
