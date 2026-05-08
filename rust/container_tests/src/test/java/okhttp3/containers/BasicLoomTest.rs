use std::io::{Cursor};
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Dispatcher;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::OkHttpClient;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::testing::PlatformRule;

// Mocking external dependencies for MockServer and Testcontainers as they are Java-specific
#[derive(Debug, Clone, PartialEq)]
pub struct MockServerContainer {
    pub image: String,
    pub host: String,
    pub server_port: u16,
    pub secure_endpoint: String,
}

impl MockServerContainer {
    pub fn new(image: &str) -> Self {
        Self {
            image: image.to_string(),
            host: "localhost".to_string(),
            server_port: 1080,
            secure_endpoint: "https://localhost:8443".to_string(),
        }
    }
}

pub struct MockServerClient {
    _host: String,
    _port: u16,
}

impl MockServerClient {
    pub fn new(host: String, port: u16) -> Self {
        Self { _host: host, _port: port }
    }

    pub fn when(&self, request: HttpRequest) -> MockServerExpectation {
        MockServerExpectation { _client: self }
    }
}

impl Drop for MockServerClient {
    fn drop(&mut self) {}
}

pub struct HttpRequest {
    _path: String,
    _query_params: Vec<(String, String)>,
}

impl HttpRequest {
    pub fn request() -> Self {
        Self { _path: "".to_string(), _query_params: Vec::new() }
    }
    pub fn with_path(mut self, path: &str) -> Self {
        self._path = path.to_string();
        self
    }
    pub fn with_query_string_parameter(mut self, key: &str, value: &str) -> Self {
        self._query_params.push((key.to_string(), value.to_string()));
        self
    }
}

pub struct HttpResponse {
    _body: String,
}

impl HttpResponse {
    pub fn response() -> Self {
        Self { _body: "".to_string() }
    }
    pub fn with_body(mut self, body: &str) -> Self {
        self._body = body.to_string();
        self
    }
}

pub struct MockServerExpectation<'a> {
    _client: &'a MockServerClient,
}

impl<'a> MockServerExpectation<'a> {
    pub fn respond(self, _response: HttpResponse) {
        // Logic to register expectation with mock server
    }
}

pub trait HttpUrlExt {
    fn to_http_url(&self) -> String;
}

impl HttpUrlExt for String {
    fn to_http_url(&self) -> String {
        self.clone()
    }
}

pub struct BasicLoomTest {
    pub platform: PlatformRule,
    pub mock_server: MockServerContainer,
    pub captured_out: Arc<Mutex<Cursor<Vec<u8>>>>,
    pub executor: Option<tokio::runtime::Runtime>,
    pub client: Option<OkHttpClient>,
}

impl BasicLoomTest {
    pub const MOCKSERVER_IMAGE: &'static str = "mockserver/mockserver";

    pub fn new() -> Self {
        Self {
            platform: PlatformRule::new(),
            mock_server: MockServerContainer::new(Self::MOCKSERVER_IMAGE),
            captured_out: Arc::new(Mutex::new(Cursor::new(Vec::new()))),
            executor: None,
            client: None,
        }
    }

    pub async fn set_up(&mut self) {
        self.platform.assume_loom();
        
        let trace_pinned = std::env::var("jdk.tracePinnedThreads").unwrap_or_default();
        assert!(!trace_pinned.is_empty(), "jdk.tracePinnedThreads should not be empty");

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime");

        let dispatcher = Dispatcher::new();

        self.client = Some(
            OkHttpClient::builder()
                .trust_mock_server()
                .dispatcher(dispatcher)
                .build()
        );

        self.executor = Some(rt);
    }

    pub fn check_for_pinning(&self) {
        let out = self.captured_out.lock().unwrap();
        let content = String::from_utf8_lossy(out.get_ref()).to_string();
        assert!(content.is_empty(), "Captured output should be empty: {}", content);
    }

    fn new_virtual_thread_per_task_executor(&self) -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime")
    }

    pub async fn test_https_request(&self) {
        let mock_server_client = MockServerClient::new(
            self.mock_server.host.clone(),
            self.mock_server.server_port,
        );

        mock_server_client.when(
            HttpRequest::request()
                .with_path("/person")
                .with_query_string_parameter("name", "peter"),
        ).respond(HttpResponse::response().with_body("Peter the person!"));

        let mut handles: Vec<JoinHandle<()>> = Vec::new();

        for _ in 1..=20 {
            let client = self.client.clone().expect("Client not initialized");
            let endpoint = self.mock_server.secure_endpoint.clone();
            
            let handle = tokio::spawn(async move {
                let url = format!("{}/person?name=peter", endpoint).to_http_url();
                let request = Request::new(url);
                
                let response = client.new_call(request).execute().await.expect("Request failed");
                let body = response.body().string().await.expect("Body read failed");
                
                assert!(body.contains("Peter the person"), "Body did not contain expected text");
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.expect("Task panicked");
        }
    }
}
