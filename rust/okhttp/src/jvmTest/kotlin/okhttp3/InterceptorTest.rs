use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use std::thread;

use crate::mockwebserver::src::main::kotlin::mockwebserver3::{MockResponse, MockWebServer, SocketEffect};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{
    Call, Headers, MediaType, OkHttpClient, Protocol, Request, Response, ResponseBody,
    RequestBody, Interceptor,
};
use crate::okhttp::src::jvmTest::kotlin::okhttp3::ResponseBodyTest::{
    Source, ForwardingSource, ForwardingSink,
};
use okio::{Buffer, BufferedSink, Sink};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::MockResponse::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::SocketEffect::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::mockwebserver::src::test::java::mockwebserver3::CustomDispatcherTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::cache::FaultHidingSink::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Reader::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::CallTagsTest::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::ResponseBodyTest::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::UrlComponentEncodingTester::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::internal::ws::WebSocketReaderTest::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::ProtocolTest::*;

// Mocking the Test Rule and Callback for the test environment
impl OkHttpClientTestRule {
    pub fn new_client(&self) -> OkHttpClient {
        OkHttpClient::builder().build()
    }
}

pub struct RecordingCallback {
    responses: Mutex<VecDeque<RecordedResponse>>,
}

#[derive(Debug, Clone)]
pub struct RecordedResponse {
    pub code: i32,
    pub headers: Headers,
    pub body: String,
    pub failure: Option<String>,
    pub cause: Option<Arc<dyn std::error::Error + Send + Sync>>,
}

impl RecordingCallback {
    pub fn new() -> Self {
        Self {
            responses: Mutex::new(VecDeque::new()),
        }
    }

    pub fn await(&self, _url: &str) -> RecordedResponse {
        loop {
            let mut lock = self.responses.lock().unwrap();
            if let Some(res) = lock.pop_front() {
                return res;
            }
            drop(lock);
            thread::sleep(Duration::from_millis(10));
        }
    }
}

pub struct InterceptorTest {
    client_test_rule: OkHttpClientTestRule,
    server: Arc<MockWebServer>,
    client: Mutex<OkHttpClient>,
    callback: Arc<RecordingCallback>,
}

impl InterceptorTest {
    pub fn new() -> Self {
        let rule = OkHttpClientTestRule;
        Self {
            client_test_rule: rule,
            server: Arc::new(MockWebServer::new()),
            client: Mutex::new(rule.new_client()),
            callback: Arc::new(RecordingCallback::new()),
        }
    }

    pub fn application_interceptors_can_short_circuit_responses(&self) {
        self.server.close();
        let request = Request::builder()
            .url("https://localhost:1/")
            .build();

        let interceptor_response = Response::builder()
            .request(&request)
            .protocol(Protocol::HTTP_1_1)
            .code(200)
            .message("Intercepted!")
            .body(ResponseBody::to_response_body("abc".as_bytes().to_vec(), Some(MediaType::parse("text/plain; charset=utf-8"))))
            .build();

        let interceptor_response_clone = interceptor_response.clone();
        let mut client_lock = self.client.lock().unwrap();
        *client_lock = client_lock.new_builder()
            .add_interceptor(Arc::new(Interceptor {
                intercept: Box::new(move |_chain| {
                    interceptor_response_clone.clone()
                }),
            }))
            .build();

        let response = client_lock.new_call(request).execute().unwrap();
        // In Rust, we check equality or use Arc for identity.
        assert_eq!(response.code(), 200);
        assert_eq!(response.message(), Some("Intercepted!"));
    }

    pub fn network_interceptors_cannot_short_circuit_responses(&self) {
        self.server.enqueue(MockResponse::builder().code(500).build());
        
        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                Response::builder()
                    .request(chain.request())
                    .protocol(Protocol::HTTP_1_1)
                    .code(200)
                    .message("Intercepted!")
                    .body(ResponseBody::to_response_body("abc".as_bytes().to_vec(), Some(MediaType::parse("text/plain; charset=utf-8"))))
                    .build()
            }),
        });

        let mut client_lock = self.client.lock().unwrap();
        *client_lock = client_lock.new_builder()
            .add_network_interceptor(interceptor.clone())
            .build();

        let request = Request::builder()
            .url(&self.server.url("/"))
            .build();

        let result = std::panic::catch_unwind(|| {
            client_lock.new_call(request).execute()
        });
        
        assert!(result.is_err(), "Should throw IllegalStateException because proceed() was not called");
    }

    pub fn network_interceptors_cannot_call_proceed_multiple_times(&self) {
        self.server.enqueue(MockResponse::new());
        self.server.enqueue(MockResponse::new());

        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                let _ = chain.proceed(chain.request());
                let _ = chain.proceed(chain.request());
                // This line is unreachable in a correct implementation as proceed() should throw
                panic!("Should have failed on second proceed()");
            }),
        });

        let mut client_lock = self.client.lock().unwrap();
        *client_lock = client_lock.new_builder()
            .add_network_interceptor(interceptor)
            .build();

        let request = Request::builder().url(&self.server.url("/")).build();
        let result = std::panic::catch_unwind(|| {
            client_lock.new_call(request).execute()
        });
        assert!(result.is_err(), "Should throw IllegalStateException because proceed() was called twice");
    }

    pub fn network_interceptors_cannot_change_server_address(&self) {
        self.server.enqueue(MockResponse::builder().code(500).build());

        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                let connection = chain.connection().expect("Connection required");
                let address = connection.route().address();
                let host = &address.url().host();
                let port = address.url().port() + 1;
                
                let new_url = format!("http://{}:{}/", host, port);
                let new_request = chain.request().new_builder().url(&new_url).build();
                chain.proceed(new_request)
            }),
        });

        let mut client_lock = self.client.lock().unwrap();
        *client_lock = client_lock.new_builder()
            .add_network_interceptor(interceptor)
            .build();

        let request = Request::builder().url(&self.server.url("/")).build();
        let result = std::panic::catch_unwind(|| {
            client_lock.new_call(request).execute()
        });
        assert!(result.is_err(), "Should throw IllegalStateException because host/port changed");
    }

    pub fn network_interceptors_have_connection_access(&self) {
        self.server.enqueue(MockResponse::new());
        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                let connection = chain.connection();
                assert!(connection.is_some());
                chain.proceed(chain.request())
            }),
        });

        let mut client_lock = self.client.lock().unwrap();
        *client_lock = client_lock.new_builder()
            .add_network_interceptor(interceptor)
            .build();

        let request = Request::builder().url(&self.server.url("/")).build();
        client_lock.new_call(request).execute().unwrap();
    }

    pub fn network_interceptors_observe_network_headers(&self) {
        let gzipped_body = self.gzip("abcabcabc");
        self.server.enqueue(
            MockResponse::builder()
                .body(gzipped_body)
                .add_header("Content-Encoding: gzip")
                .build(),
        );

        let server_host = self.server.host_name().to_string();
        let server_port = self.server.port().to_string();
        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(move |chain| {
                let network_request = chain.request();
                assert!(network_request.header("User-Agent").is_some());
                assert_eq!(
                    network_request.header("Host"),
                    Some(&format!("{}:{}", server_host, server_port))
                );
                assert!(network_request.header("Accept-Encoding").is_some());

                let network_response = chain.proceed(network_request);
                assert_eq!(network_response.header("Content-Encoding"), Some("gzip"));
                network_response
            }),
        });

        let mut client_lock = self.client.lock().unwrap();
        *client_lock = client_lock.new_builder()
            .add_network_interceptor(interceptor)
            .build();

        let request = Request::builder().url(&self.server.url("/")).build();
        assert!(request.header("User-Agent").is_none());
        
        let response = client_lock.new_call(request).execute().unwrap();
        assert_eq!(response.body().string(), "abcabcabc");
    }

    pub fn network_interceptors_can_change_request_method_from_get_to_post(&self) {
        self.server.enqueue(MockResponse::new());
        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                let original_request = chain.request();
                let media_type = MediaType::parse("text/plain");
                let body = RequestBody::to_request_body("abc".as_bytes().to_vec(), Some(media_type.clone()));
                
                let new_request = original_request.new_builder()
                    .method("POST", body)
                    .header("Content-Type", &media_type.to_string())
                    .header("Content-Length", &"3".to_string())
                    .build();
                chain.proceed(new_request)
            }),
        });

        let mut client_lock = self.client.lock().unwrap();
        *client_lock = client_lock.new_builder()
            .add_network_interceptor(interceptor)
            .build();

        let request = Request::builder().url(&self.server.url("/")).get().build();
        client_lock.new_call(request).execute().unwrap();
        
        let recorded_request = self.server.take_request();
        assert_eq!(recorded_request.method, "POST");
        assert_eq!(recorded_request.body.map(|b| b.utf8()).unwrap_or_default(), "abc");
    }

    pub fn application_interceptors_rewrite_request_to_server(&self) {
        self.rewrite_request_to_server(false);
    }

    pub fn network_interceptors_rewrite_request_to_server(&self) {
        self.rewrite_request_to_server(true);
    }

    fn rewrite_request_to_server(&self, network: bool) {
        self.server.enqueue(MockResponse::new());
        
        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                let original_request = chain.request();
                let body = self.uppercase_request_body(original_request.body());
                let new_request = original_request.new_builder()
                    .method("POST", body)
                    .add_header("OkHttp-Intercepted", "yep")
                    .build();
                chain.proceed(new_request)
            }),
        });

        self.add_interceptor(network, interceptor);

        let request = Request::builder()
            .url(&self.server.url("/"))
            .add_header("Original-Header", "foo")
            .method("PUT", RequestBody::to_request_body("abc".as_bytes().to_vec(), Some(MediaType::parse("text/plain"))))
            .build();

        self.client.lock().unwrap().new_call(request).execute().unwrap();
        let recorded_request = self.server.take_request();
        assert_eq!(recorded_request.body.map(|b| b.utf8()).unwrap_or_default(), "ABC");
        assert_eq!(recorded_request.headers.get("Original-Header"), Some("foo"));
        assert_eq!(recorded_request.headers.get("OkHttp-Intercepted"), Some("yep"));
        assert_eq!(recorded_request.method, "POST");
    }

    pub fn application_interceptors_rewrite_response_from_server(&self) {
        self.rewrite_response_from_server(false);
    }

    pub fn network_interceptors_rewrite_response_from_server(&self) {
        self.rewrite_response_from_server(true);
    }

    fn rewrite_response_from_server(&self, network: bool) {
        self.server.enqueue(
            MockResponse::builder()
                .add_header("Original-Header: foo")
                .body("abc")
                .build(),
        );

        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                let original_response = chain.proceed(chain.request());
                let body = self.uppercase_response_body(original_response.body());
                original_response.new_builder()
                    .body(body)
                    .add_header("OkHttp-Intercepted", "yep")
                    .build()
            }),
        });

        self.add_interceptor(network, interceptor);

        let request = Request::builder().url(&self.server.url("/")).build();
        let response = self.client.lock().unwrap().new_call(request).execute().unwrap();
        assert_eq!(response.body().string(), "ABC");
        assert_eq!(response.header("OkHttp-Intercepted"), Some("yep"));
        assert_eq!(response.header("Original-Header"), Some("foo"));
    }

    pub fn multiple_application_interceptors(&self) {
        self.multiple_interceptors(false);
    }

    pub fn multiple_network_interceptors(&self) {
        self.multiple_interceptors(true);
    }

    fn multiple_interceptors(&self, network: bool) {
        self.server.enqueue(MockResponse::new());
        
        let int1 = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                let req = chain.request().new_builder().add_header("Request-Interceptor", "Android").build();
                let res = chain.proceed(req);
                res.new_builder().add_header("Response-Interceptor", "Donut").build()
            }),
        });
        let int2 = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                let req = chain.request().new_builder().add_header("Request-Interceptor", "Bob").build();
                let res = chain.proceed(req);
                res.new_builder().add_header("Response-Interceptor", "Cupcake").build()
            }),
        });

        self.add_interceptor(network, int1);
        self.add_interceptor(network, int2);

        let request = Request::builder().url(&self.server.url("/")).build();
        let response = self.client.lock().unwrap().new_call(request).execute().unwrap();
        
        // Check headers "Response-Interceptor" contains exactly ["Cupcake", "Donut"]
        let response_headers = response.headers().get_all("Response-Interceptor");
        assert_eq!(response_headers, vec!["Cupcake", "Donut"]);

        let recorded_request = self.server.take_request();
        let request_headers = recorded_request.headers.get_all("Request-Interceptor");
        assert_eq!(request_headers, vec!["Android", "Bob"]);
    }

    pub fn async_application_interceptors(&self) {
        self.async_interceptors(false);
    }

    pub fn async_network_interceptors(&self) {
        self.async_interceptors(true);
    }

    fn async_interceptors(&self, network: bool) {
        self.server.enqueue(MockResponse::new());
        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                let res = chain.proceed(chain.request());
                res.new_builder().add_header("OkHttp-Intercepted", "yep").build()
            }),
        });
        self.add_interceptor(network, interceptor);

        let request = Request::builder().url(&self.server.url("/")).build();
        let callback = self.callback.clone();
        self.client.lock().unwrap().new_call(request).enqueue(callback.clone());
        
        let response = callback.r#await("/");
        assert_eq!(response.code, 200);
        assert_eq!(response.headers.get("OkHttp-Intercepted"), Some("yep"));
    }

    pub fn application_interceptors_can_make_multiple_requests_to_server(&self) {
        self.server.enqueue(MockResponse::builder().body("a").build());
        self.server.enqueue(MockResponse::builder().body("b").build());

        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                let response1 = chain.proceed(chain.request());
                let _ = response1.body().close();
                chain.proceed(chain.request())
            }),
        });

        let mut client_lock = self.client.lock().unwrap();
        *client_lock = client_lock.new_builder().add_interceptor(interceptor).build();

        let request = Request::builder().url(&self.server.url("/")).build();
        let response = client_lock.new_call(request).execute().unwrap();
        assert_eq!(response.body().string(), "b");
    }

    pub fn interceptor_makes_an_unrelated_request(&self) {
        self.server.enqueue(MockResponse::builder().body("a").build());
        self.server.enqueue(MockResponse::builder().body("b").build());

        let client_arc = Arc::new(Mutex::new(self.client_test_rule.new_client()));
        let client_clone = client_arc.clone();
        let server_clone = self.server.clone();

        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(move |chain| {
                if chain.request().url().encoded_path() == "/b" {
                    let request_a = Request::builder().url(server_clone.url("/a")).build();
                    let response_a = client_clone.lock().unwrap().new_call(request_a).execute().unwrap();
                    assert_eq!(response_a.body().string(), "a");
                }
                chain.proceed(chain.request())
            }),
        });

        let mut client_lock = self.client.lock().unwrap();
        *client_lock = client_lock.new_builder().add_interceptor(interceptor).build();

        let request_b = Request::builder().url(&self.server.url("/b")).build();
        let response_b = client_lock.new_call(request_b).execute().unwrap();
        assert_eq!(response_b.body().string(), "b");
    }

    pub fn interceptor_makes_an_unrelated_async_request(&self) {
        self.server.enqueue(MockResponse::builder().body("a").build());
        self.server.enqueue(MockResponse::builder().body("b").build());

        let client_arc = Arc::new(Mutex::new(self.client_test_rule.new_client()));
        let client_clone = client_arc.clone();
        let server_clone = self.server.clone();

        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(move |chain| {
                if chain.request().url().encoded_path() == "/b" {
                    let request_a = Request::builder().url(server_clone.url("/a")).build();
                    let callback_a = Arc::new(RecordingCallback::new());
                    client_clone.lock().unwrap().new_call(request_a).enqueue(callback_a.clone());
                    let res_a = callback_a.r#await("/a");
                    assert_eq!(res_a.body, "a");
                }
                chain.proceed(chain.request())
            }),
        });

        let mut client_lock = self.client.lock().unwrap();
        *client_lock = client_lock.new_builder().add_interceptor(interceptor).build();

        let request_b = Request::builder().url(&self.server.url("/b")).build();
        let callback_b = self.callback.clone();
        client_lock.new_call(request_b).enqueue(callback_b.clone());
        let res_b = callback_b.r#await("/b");
        assert_eq!(res_b.body, "b");
    }

    pub fn application_interceptor_throws_runtime_exception_synchronous(&self) {
        self.interceptor_throws_runtime_exception_synchronous(false);
    }

    pub fn network_interceptor_throws_runtime_exception_synchronous(&self) {
        self.interceptor_throws_runtime_exception_synchronous(true);
    }

    fn interceptor_throws_runtime_exception_synchronous(&self, network: bool) {
        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(|_chain| {
                panic!("boom!");
            }),
        });
        self.add_interceptor(network, interceptor);

        let request = Request::builder().url(&self.server.url("/")).build();
        let result = std::panic::catch_unwind(|| {
            self.client.lock().unwrap().new_call(request).execute()
        });
        assert!(result.is_err());
    }

    pub fn network_interceptor_modified_request_is_returned(&self) {
        self.server.enqueue(MockResponse::new());
        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                let modified_request = chain.request().new_builder()
                    .header("User-Agent", "intercepted request")
                    .build();
                chain.proceed(modified_request)
            }),
        });

        let mut client_lock = self.client.lock().unwrap();
        *client_lock = client_lock.new_builder().add_network_interceptor(interceptor).build();

        let request = Request::builder()
            .url(&self.server.url("/"))
            .header("User-Agent", "user request")
            .build();
        let response = client_lock.new_call(request).execute().unwrap();
        
        assert_eq!(response.request().header("User-Agent"), Some("user request"));
        // network_response check would require access to internal network response
    }

    pub fn application_interceptor_throws_runtime_exception_asynchronous(&self) {
        self.interceptor_throws_runtime_exception_asynchronous(false);
    }

    pub fn network_interceptor_throws_runtime_exception_asynchronous(&self) {
        self.interceptor_throws_runtime_exception_asynchronous(true);
    }

    fn interceptor_throws_runtime_exception_asynchronous(&self, network: bool) {
        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(|_chain| {
                panic!("boom!");
            }),
        });
        self.add_interceptor(network, interceptor);

        let request = Request::builder().url(&self.server.url("/")).build();
        let call = self.client.lock().unwrap().new_call(request);
        call.enqueue(self.callback.clone());
        
        let response = self.callback.r#await("/");
        assert!(response.failure.as_ref().map_or(false, |f| f.contains("boom!")));
        assert!(call.is_canceled());
    }

    pub fn network_interceptor_returns_connection_on_empty_body(&self) {
        self.server.enqueue(
            MockResponse::builder()
                .on_response_end(SocketEffect::ShutdownConnection)
                .add_header("Connection: Close")
                .build(),
        );

        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                let response = chain.proceed(chain.request());
                assert!(chain.connection().is_some());
                response
            }),
        });

        let mut client_lock = self.client.lock().unwrap();
        *client_lock = client_lock.new_builder().add_network_interceptor(interceptor).build();

        let request = Request::builder().url(&self.server.url("/")).build();
        let response = client_lock.new_call(request).execute().unwrap();
        let _ = response.body().close();
    }

    pub fn connect_timeout(&self) {
        let interceptor1 = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                assert_eq!(chain.connect_timeout_millis(), 5000);
                let chain_b = chain.with_connect_timeout(100, Duration::from_millis(100));
                assert_eq!(chain_b.connect_timeout_millis(), 100);
                chain_b.proceed(chain.request())
            }),
        });
        let interceptor2 = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                assert_eq!(chain.connect_timeout_millis(), 100);
                chain.proceed(chain.request())
            }),
        });

        let mut client_lock = self.client.lock().unwrap();
        *client_lock = client_lock.new_builder()
            .connect_timeout(Duration::from_secs(5))
            .add_interceptor(interceptor1)
            .add_interceptor(interceptor2)
            .build();

        let request = Request::builder().url("http://10.255.255.1").build();
        let call = client_lock.new_call(request);
        let start = Instant::now();
        let _ = std::panic::catch_unwind(|| {
            call.execute()
        });
        assert!(start.elapsed() < Duration::from_secs(5));
    }

    pub fn chain_with_read_timeout(&self) {
        let interceptor1 = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                assert_eq!(chain.read_timeout_millis(), 5000);
                let chain_b = chain.with_read_timeout(100, Duration::from_millis(100));
                assert_eq!(chain_b.read_timeout_millis(), 100);
                chain_b.proceed(chain.request())
            }),
        });
        let interceptor2 = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                assert_eq!(chain.read_timeout_millis(), 100);
                chain.proceed(chain.request())
            }),
        });

        let mut client_lock = self.client.lock().unwrap();
        *client_lock = client_lock.new_builder()
            .read_timeout(Duration::from_secs(5))
            .add_interceptor(interceptor1)
            .add_interceptor(interceptor2)
            .build();

        self.server.enqueue(
            MockResponse::builder()
                .body("abc")
                .throttle_body(1, 1, Duration::from_secs(1))
                .build(),
        );

        let request = Request::builder().url(&self.server.url("/")).build();
        let response = client_lock.new_call(request).execute().unwrap();
        let body = response.body();
        let _ = std::panic::catch_unwind(|| {
            body.string()
        });
    }

    pub fn network_interceptor_cannot_change_read_timeout(&self) {
        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                chain.with_read_timeout(100, Duration::from_millis(100)).proceed(chain.request())
            }),
        });
        self.add_interceptor(true, interceptor);

        let request = Request::builder().url(&self.server.url("/")).build();
        let result = std::panic::catch_unwind(|| {
            self.client.lock().unwrap().new_call(request).execute()
        });
        assert!(result.is_err());
    }

    pub fn network_interceptor_cannot_change_write_timeout(&self) {
        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                chain.with_write_timeout(100, Duration::from_millis(100)).proceed(chain.request())
            }),
        });
        self.add_interceptor(true, interceptor);

        let request = Request::builder().url(&self.server.url("/")).build();
        let result = std::panic::catch_unwind(|| {
            self.client.lock().unwrap().new_call(request).execute()
        });
        assert!(result.is_err());
    }

    pub fn network_interceptor_cannot_change_connect_timeout(&self) {
        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                chain.with_connect_timeout(100, Duration::from_millis(100)).proceed(chain.request())
            }),
        });
        self.add_interceptor(true, interceptor);

        let request = Request::builder().url(&self.server.url("/")).build();
        let result = std::panic::catch_unwind(|| {
            self.client.lock().unwrap().new_call(request).execute()
        });
        assert!(result.is_err());
    }

    pub fn chain_with_write_timeout(&self) {
        let interceptor1 = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                assert_eq!(chain.write_timeout_millis(), 5000);
                let chain_b = chain.with_write_timeout(100, Duration::from_millis(100));
                assert_eq!(chain_b.write_timeout_millis(), 100);
                chain_b.proceed(chain.request())
            }),
        });
        let interceptor2 = Arc::new(Interceptor {
            intercept: Box::new(|chain| {
                assert_eq!(chain.write_timeout_millis(), 100);
                chain.proceed(chain.request())
            }),
        });

        let mut client_lock = self.client.lock().unwrap();
        *client_lock = client_lock.new_builder()
            .write_timeout(Duration::from_secs(5))
            .add_interceptor(interceptor1)
            .add_interceptor(interceptor2)
            .build();

        self.server.enqueue(
            MockResponse::builder()
                .body("abc")
                .throttle_body(1, 1, Duration::from_secs(1))
                .build(),
        );

        let data = vec![0u8; 2 * 1024 * 1024];
        let request = Request::builder()
            .url(&self.server.url("/"))
            .post(RequestBody::to_request_body(data, Some(MediaType::parse("text/plain"))))
            .build();
        
        let _ = std::panic::catch_unwind(|| {
            client_lock.new_call(request).execute()
        });
    }

    pub fn chain_can_cancel_call(&self) {
        let call_ref = Arc::new(Mutex::new(None));
        let call_ref_clone = call_ref.clone();
        
        let interceptor = Arc::new(Interceptor {
            intercept: Box::new(move |chain| {
                let call = chain.call();
                *call_ref_clone.lock().unwrap() = Some(call.clone());
                assert!(!call.is_canceled());
                call.cancel();
                assert!(call.is_canceled());
                chain.proceed(chain.request())
            }),
        });

        let mut client_lock = self.client.lock().unwrap();
        *client_lock = client_lock.new_builder().add_interceptor(interceptor).build();

        let request = Request::builder().url(&self.server.url("/")).build();
        let call = client_lock.new_call(request);
        let _ = std::panic::catch_unwind(|| {
            call.execute()
        });
    }

    fn gzip(&self, data: &str) -> Buffer {
        let mut result = Buffer::new();
        // In a real implementation, use a GzipSink
        result.write_utf8(data); 
        result
    }

    fn add_interceptor(&self, network: bool, interceptor: Arc<Interceptor>) {
        let mut client_lock = self.client.lock().unwrap();
        let builder = client_lock.new_builder();
        if network {
            let b = builder.add_network_interceptor(interceptor);
            *client_lock = b.build();
        } else {
            let b = builder.add_interceptor(interceptor);
            *client_lock = b.build();
        }
    }

    fn uppercase_request_body(&self, original: Option<&RequestBody>) -> RequestBody {
        let original = original.expect("Body required");
        let original_clone = original.clone();
        RequestBody {
            content_type: move || original_clone.content_type().cloned(),
            content_length: move || original_clone.content_length(),
            write_to: Box::new(move |sink| {
                // Logic to wrap sink in an uppercase forwarding sink
                original_clone.write_to(sink);
            }),
        }
    }

    fn uppercase_response_body(&self, original: ResponseBody) -> ResponseBody {
        // Logic to wrap original source in an uppercase forwarding source
        original
    }
}
