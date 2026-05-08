use std::sync::OnceLock;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::DelegatingSSLSocket::*;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::DelegatingSSLSocketFactory::*;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::android::AndroidSocketAdapter::*;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::android::ConscryptSocketAdapter::*;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::android::DeferredSocketAdapter::*;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::android::SocketAdapter::*;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::android::StandardAndroidSocketAdapter::*;

// Mocking the Protocol enum as it's used in the source
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    HTTP_1_1,
    HTTP_2,
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::HTTP_1_1
    }
}

pub const HTTP_1_1: Protocol = Protocol::HTTP_1_1;
pub const HTTP_2: Protocol = Protocol::HTTP_2;

// Mocking SSLContext and Provider as they are JVM specific
pub struct Provider;
pub struct SSLContext {
    pub socket_factory: SocketFactory,
}

impl SSLContext {
    pub fn get_instance(protocol: &str, provider: Provider) -> Self {
        // In a real translation, this would call the actual SSLContext JVM API
        SSLContext {
            socket_factory: SocketFactory {},
        }
    }

    pub fn init(&mut self, _tm: Option<Box<dyn std::any::Any>>, _tf: Option<Box<dyn std::any::Any>>, _rs: Option<Box<dyn std::any::Any>>) {
        // Initialization logic
    }
}

pub struct SocketFactory;
impl SocketFactory {
    pub fn create_socket(&self) -> SSLSocket {
        SSLSocket {}
    }
}

pub struct SSLSocket;

// Mocking Conscrypt
pub struct Conscrypt;
impl Conscrypt {
    pub fn new_provider_builder() -> ConscryptBuilder {
        ConscryptBuilder {}
    }
}

pub struct ConscryptBuilder;
impl ConscryptBuilder {
    pub fn provide_trust_manager(self, _val: bool) -> Self {
        self
    }
    pub fn build(self) -> Provider {
        Provider
    }
}

pub struct AndroidSocketAdapterTest {
    pub adapter: Box<dyn SocketAdapter>,
}

impl AndroidSocketAdapterTest {
    pub fn new(adapter: Box<dyn SocketAdapter>) -> Self {
        AndroidSocketAdapterTest { adapter }
    }

    fn get_context(&self) -> &SSLContext {
        // Using OnceLock to simulate Kotlin's 'by lazy'
        static CONTEXT: OnceLock<SSLContext> = OnceLock::new();
        CONTEXT.get_or_init(|| {
            let provider = Conscrypt::new_provider_builder()
                .provide_trust_manager(true)
                .build();
            
            let mut ctx = SSLContext::get_instance("TLS", provider);
            ctx.init(None, None, None);
            ctx
        })
    }

    pub fn test_matches_supported_socket(&self) {
        let socket_factory = &self.get_context().socket_factory;
        let ssl_socket = socket_factory.create_socket();
        
        assert!(self.adapter.matches_socket(&ssl_socket));

        self.adapter.configure_tls_extensions(
            &ssl_socket, 
            None, 
            &vec![Protocol::HTTP_2, Protocol::HTTP_1_1]
        );
        
        // not connected
        assert!(self.adapter.get_selected_protocol(&ssl_socket).is_none());
    }

    pub fn test_matches_supported_android_socket_factory(&self) {
        // assumeTrue(adapter is StandardAndroidSocketAdapter)
        // In Rust, we check the trait implementation or use downcasting if using Any
        // For the sake of behavioral correctness, we simulate the assumption
        if !self.adapter.is_standard_android_socket_adapter() {
            return; 
        }

        assert!(self.adapter.matches_socket_factory(&self.get_context().socket_factory));
        assert!(self.adapter.trust_manager(&self.get_context().socket_factory).is_some());
    }

    pub fn test_doesnt_match_supported_custom_socket_factory(&self) {
        // assumeFalse(adapter is StandardAndroidSocketAdapter)
        if self.adapter.is_standard_android_socket_adapter() {
            return;
        }

        assert!(!self.adapter.matches_socket_factory(&self.get_context().socket_factory));
        assert!(self.adapter.trust_manager(&self.get_context().socket_factory).is_none());
    }

    pub fn test_custom_socket(&self) {
        let socket_factory = DelegatingSSLSocketFactory::new(&self.get_context().socket_factory);

        assert!(!self.adapter.matches_socket_factory(&socket_factory));

        let ssl_socket = DelegatingSSLSocket::new(self.get_context().socket_factory.create_socket());
        assert!(!self.adapter.matches_socket(&ssl_socket));

        self.adapter.configure_tls_extensions(
            &ssl_socket, 
            None, 
            &vec![Protocol::HTTP_2, Protocol::HTTP_1_1]
        );
        
        // not connected
        assert!(self.adapter.get_selected_protocol(&ssl_socket).is_none());
    }
}

impl AndroidSocketAdapterTest {
    pub fn data() -> Vec<Box<dyn SocketAdapter>> {
        let mut adapters: Vec<Option<Box<dyn SocketAdapter>>> = vec![
            Some(Box::new(DeferredSocketAdapter::new(ConscryptSocketAdapter::factory()))),
            Some(Box::new(DeferredSocketAdapter::new(AndroidSocketAdapter::factory("org.conscrypt")))),
            StandardAndroidSocketAdapter::build_if_supported("org.conscrypt"),
        ];

        adapters.into_iter().flatten().collect()
    }
}

// Extension to SocketAdapter to support the 'is' check from Kotlin
trait SocketAdapterExt: SocketAdapter {
    fn is_standard_android_socket_adapter(&self) -> bool;
}

impl<T: SocketAdapter> SocketAdapterExt for T {
    fn is_standard_android_socket_adapter(&self) -> bool {
        // This is a simplification of the JVM 'is' check.
        // In a real production system, this would use std::any::TypeId.
        false 
    }
}