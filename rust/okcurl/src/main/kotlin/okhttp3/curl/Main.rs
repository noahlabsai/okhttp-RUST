use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::time::Duration;

// Import paths as specified in the translation rules
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Call;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::OkHttpClient;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request;
use crate::okcurl::src::main::kotlin::okhttp3::curl::logging::LoggingUtil;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::platform::Platform;
use crate::okhttp_logging_interceptor::src::main::kotlin::okhttp3::logging::HttpLoggingInterceptor;
use crate::okhttp_logging_interceptor::src::main::kotlin::okhttp3::logging::LoggingEventListener;

// Mocking internal common functions as they are not provided in the source but called
mod internal {
    use super::*;
    pub fn common_create_request(_main: &Main) -> Request {
        // This is a placeholder for the actual logic in okhttp3.curl.internal.commonCreateRequest
        Request::builder().url("http://localhost").build()
    }
    pub fn common_run(_main: &Main) {
        // This is a placeholder for the actual logic in okhttp3.curl.internal.commonRun
    }
}

pub struct Main {
    pub method: Option<String>,
    pub data: Option<String>,
    pub headers: Option<Vec<String>>,
    pub user_agent: String,
    pub connect_timeout: i32,
    pub read_timeout: i32,
    pub call_timeout: i32,
    pub follow_redirects: bool,
    pub allow_insecure: bool,
    pub show_headers: bool,
    pub show_http2_frames: bool,
    pub referer: Option<String>,
    pub verbose: bool,
    pub ssl_debug: bool,
    pub url: Option<String>,
    pub client: Option<Box<dyn Call::Factory>>,
}

impl Main {
    pub const NAME: &'static str = "okcurl";
    pub const DEFAULT_TIMEOUT: i32 = -1;

    pub fn new() -> Self {
        Self {
            method: None,
            data: None,
            headers: None,
            user_agent: format!("{}/{}", Self::NAME, Self::version_string()),
            connect_timeout: Self::DEFAULT_TIMEOUT,
            read_timeout: Self::DEFAULT_TIMEOUT,
            call_timeout: Self::DEFAULT_TIMEOUT,
            follow_redirects: false,
            allow_insecure: false,
            show_headers: false,
            show_http2_frames: false,
            referer: None,
            verbose: false,
            ssl_debug: false,
            url: None,
            client: None,
        }
    }

    pub fn help(&self) -> String {
        "A curl for the next-generation web.".to_string()
    }

    pub fn run(&mut self) {
        LoggingUtil::configure_logging(
            self.verbose,
            self.show_http2_frames,
            self.ssl_debug,
        );

        internal::common_run(self);
    }

    pub fn create_request(&self) -> Request {
        internal::common_create_request(self)
    }

    pub fn create_client(&self) -> Box<dyn Call::Factory> {
        let mut builder = OkHttpClient::builder();
        builder.follow_ssl_redirects(self.follow_redirects);

        if self.connect_timeout != Self::DEFAULT_TIMEOUT {
            builder.connect_timeout(self.connect_timeout as i64, Duration::from_secs(1));
        }
        if self.read_timeout != Self::DEFAULT_TIMEOUT {
            builder.read_timeout(self.read_timeout as i64, Duration::from_secs(1));
        }
        if self.call_timeout != Self::DEFAULT_TIMEOUT {
            builder.call_timeout(self.call_timeout as i64, Duration::from_secs(1));
        }

        if self.allow_insecure {
            let trust_manager = Self::create_insecure_trust_manager();
            let ssl_socket_factory = Self::create_insecure_ssl_socket_factory(trust_manager.clone());
            builder.ssl_socket_factory(ssl_socket_factory, trust_manager);
            builder.hostname_verifier(Self::create_insecure_hostname_verifier());
        }

        if self.verbose {
            let logger = HttpLoggingInterceptor::Logger(|msg| println!("{}", msg));
            builder.event_listener_factory(LoggingEventListener::Factory::new(logger));
        }

        Box::new(builder.build())
    }

    pub fn close(&mut self) {
        if let Some(ref mut factory) = self.client {
            if let Some(client) = factory.as_any().downcast_ref::<OkHttpClient>() {
                client.connection_pool().evict_all();
                client.dispatcher().executor_service().shutdown_now();
            }
        }
    }

    fn version_string() -> String {
        let mut prop = HashMap::new();
        if let Ok(file) = File::open("/okcurl-version.properties") {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let parts: Vec<&str> = line.splitn(2, '=').collect();
                if parts.len() == 2 {
                    prop.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
                }
            }
        }
        prop.get("version").cloned().unwrap_or_else(|| "dev".to_string())
    }

    fn create_insecure_trust_manager() -> Arc<dyn X509TrustManager> {
        Arc::new(InsecureTrustManager)
    }

    fn create_insecure_ssl_socket_factory(trust_manager: Arc<dyn X509TrustManager>) -> Box<dyn SslSocketFactory> {
        let platform = Platform::get();
        let ssl_context = platform.new_ssl_context();
        ssl_context.init(None, vec![trust_manager], None);
        ssl_context.socket_factory()
    }

    fn create_insecure_hostname_verifier() -> Box<dyn HostnameVerifier> {
        Box::new(InsecureHostnameVerifier)
    }
}

// Trait definitions to support the SSL/TLS logic
pub trait TrustManager: Send + Sync {}

pub trait X509TrustManager: TrustManager + Send + Sync {
    fn check_client_trusted(&self, chain: &[X509Certificate], auth_type: &str);
    fn check_server_trusted(&self, chain: &[X509Certificate], auth_type: &str);
    fn get_accepted_issuers(&self) -> Vec<X509Certificate>;
}

pub trait SslSocketFactory: Send + Sync {}

pub trait HostnameVerifier: Send + Sync {
    fn verify(&self, hostname: &str, session: &SslSession) -> bool;
}

pub struct SslSession;
pub struct X509Certificate;

struct InsecureTrustManager;
impl TrustManager for InsecureTrustManager {}
impl X509TrustManager for InsecureTrustManager {
    fn check_client_trusted(&self, _chain: &[X509Certificate], _auth_type: &str) {
        // No-op as per Kotlin source
    }
    fn check_server_trusted(&self, _chain: &[X509Certificate], _auth_type: &str) {
        // No-op as per Kotlin source
    }
    fn get_accepted_issuers(&self) -> Vec<X509Certificate> {
        vec![]
    }
}

struct InsecureHostnameVerifier;
impl HostnameVerifier for InsecureHostnameVerifier {
    fn verify(&self, _hostname: &str, _session: &SslSession) -> bool {
        true
    }
}

// Extension to allow downcasting of the Factory trait
trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl AsAny for OkHttpClient {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
