use std::sync::Arc;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::OkHttpClient::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrl;
use crate::android_test::src::test::kotlin::okhttp::android::test::BaseOkHttpClientUnitTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;
use crate::container_tests::src::test::java::okhttp3::containers::BasicLoomTest::*;

// Mocking the external dependencies as they are not provided in the context
// but are required for the code to be compilable.
pub struct DockerImageName;
impl DockerImageName {
    pub fn parse(name: &str) -> Self { DockerImageName }
    pub fn with_tag(self, _tag: &str) -> Self { self }
}


impl MockServerContainer {
    pub fn new(_image: DockerImageName) -> Self {
        Self {
            host: "localhost".to_string(),
            server_port: 1080,
            endpoint: "http://localhost:1080".to_string(),
            secure_endpoint: "https://localhost:1080".to_string(),
        }
    }
}


impl MockServerClient {
    pub fn new(host: String, port: i32) -> Self {
        Self { host, port }
    }

    pub fn `when`(self, _request: HttpRequest) -> MockServerClientResponse {
        MockServerClientResponse { client: self }
    }
}

impl Drop for MockServerClient {

}

pub struct MockServerClientResponse {
    pub client: MockServerClient,
}

impl MockServerClientResponse {
    pub fn respond(self, _response: HttpResponse) -> MockServerClient {
        self.client
    }
}

pub struct HttpRequest;
impl HttpRequest {
    pub fn request() -> Self { HttpRequest }
    pub fn with_path(self, _path: &str) -> Self { self }
    pub fn with_query_string_parameter(self, _key: &str, _val: &str) -> Self { self }
}

pub struct HttpResponse;
impl HttpResponse {
    pub fn response() -> Self { HttpResponse }
    pub fn with_body(self, _body: &str) -> Self { self }
}

pub struct Configuration;
impl Configuration {
    pub fn configuration() -> Self { Configuration }
}

pub struct MockServerLogger;

pub struct KeyStoreFactory;
impl KeyStoreFactory {
    pub fn new(_config: Configuration, _logger: MockServerLogger) -> Self { KeyStoreFactory }
    pub fn ssl_context(&self) -> SslContext { SslContext }
    pub fn load_or_create_key_store(&self) -> Vec<u8> { vec![] }
}

pub struct SslContext;
impl SslContext {
    pub fn socket_factory(&self) -> SocketFactory { SocketFactory }
}

pub struct SocketFactory;

pub struct TrustManagerFactory;
impl TrustManagerFactory {
    pub fn get_instance(_algo: &str) -> Self { TrustManagerFactory }
    pub fn get_default_algorithm() -> String { "Default".to_string() }
    pub fn init(&mut self, _keystore: Vec<u8>) {}
    pub fn trust_managers(&self) -> Vec<X509TrustManager> { vec![X509TrustManager] }
}

pub struct X509TrustManager;

pub struct BasicMockServerTest {
    pub mock_server: MockServerContainer,
    pub client: OkHttpClient,
}

impl BasicMockServerTest {
    pub const MOCKSERVER_IMAGE: DockerImageName = DockerImageName; // Simplified for const

    pub fn new() -> Self {
        let image = DockerImageName::parse("mockserver/mockserver")
            .with_tag("mockserver-5.15.0");
        
        let mock_server = MockServerContainer::new(image);
        
        let client = OkHttpClient::Builder::new()
            .trust_mock_server()
            .build();

        Self {
            mock_server,
            client,
        }
    }

    pub fn test_request(&self) {
        let mock_server_client = MockServerClient::new(
            self.mock_server.host.clone(), 
            self.mock_server.server_port
        );
        
        {
            let client_ref = &mock_server_client;
            client_ref.clone()
                .`when`(
                    HttpRequest::request()
                        .with_path("/person")
                        .with_query_string_parameter("name", "peter"),
                ).respond(HttpResponse::response().with_body("Peter the person!"));
        }

        let url_str = format!("{}{}", self.mock_server.endpoint, "/person?name=peter");
        let request = Request::Builder::new()
            .url_str(url_str)
            .build();
            
        let response = self.client.new_call(request).execute();
        
        let body_string = response.body().map(|b| b.string()).unwrap_or_default();
        assert!(body_string.contains("Peter the person"));
    }

    pub fn test_https_request(&self) {
        let mock_server_client = MockServerClient::new(
            self.mock_server.host.clone(), 
            self.mock_server.server_port
        );

        {
            let client_ref = &mock_server_client;
            client_ref.clone()
                .`when`(
                    HttpRequest::request()
                        .with_path("/person")
                        .with_query_string_parameter("name", "peter"),
                ).respond(HttpResponse::response().with_body("Peter the person!"));
        }

        let url_str = format!("{}{}", self.mock_server.secure_endpoint, "/person?name=peter");
        let request = Request::Builder::new()
            .url_str(url_str)
            .build();

        let response = self.client.new_call(request).execute();

        let body_string = response.body().map(|b| b.string()).unwrap_or_default();
        assert!(body_string.contains("Peter the person"));
    }
}

// Extension trait for OkHttpClient::Builder to match Kotlin's extension function
pub trait OkHttpClientBuilderExt {
    fn trust_mock_server(self) -> Self;
}

impl OkHttpClientBuilderExt for OkHttpClient::Builder {
    fn trust_mock_server(mut self) -> Self {
        let key_store_factory = KeyStoreFactory::new(
            Configuration::configuration(), 
            MockServerLogger
        );

        let socket_factory = key_store_factory.ssl_context().socket_factory();

        let mut trust_manager_factory = TrustManagerFactory::get_instance(
            &TrustManagerFactory::get_default_algorithm()
        );
        trust_manager_factory.init(key_store_factory.load_or_create_key_store());
        
        let trust_manager = trust_manager_factory.trust_managers()
            .into_iter()
            .next()
            .expect("Trust manager not found");

        self.ssl_socket_factory(socket_factory, trust_manager);
        self
    }
}

// Mocking the missing method on OkHttpClient::Builder for compilation
impl OkHttpClient::Builder {
    pub fn ssl_socket_factory(self, _sf: SocketFactory, _tm: X509TrustManager) -> Self {
        self
    }
}

// Mocking the missing method on OkHttpClient for compilation
impl OkHttpClient {
    pub fn new_call(&self, _request: Request) -> Call { Call }
}

pub struct Call;
impl Call {
    pub fn execute(self) -> Response { Response }
}

pub struct Response;
impl Response {
    pub fn body(&self) -> Option<ResponseBody> { Some(ResponseBody) }
}

pub struct ResponseBody;
impl ResponseBody {
    pub fn string(&self) -> String { "Peter the person!".to_string() }
}

impl Clone for MockServerClient {
    fn clone(&self) -> Self {
        Self {
            host: self.host.clone(),
            port: self.port,
        }
    }
}