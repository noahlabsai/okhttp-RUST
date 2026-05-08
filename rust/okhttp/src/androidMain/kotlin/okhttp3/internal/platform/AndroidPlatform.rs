use std::sync::Arc;
use std::io::{Error as IoError, ErrorKind};
use std::any::Any;

// Import paths as specified in the translation directives
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::SuppressSignatureCheck::*;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::android::AndroidCertificateChainCleaner::*;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::android::AndroidSocketAdapter::*;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::android::BouncyCastleSocketAdapter::*;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::android::ConscryptSocketAdapter::*;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::android::DeferredSocketAdapter::*;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::android::StandardAndroidSocketAdapter::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::tls::BasicTrustRootIndex::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::tls::CertificateChainCleaner::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::tls::TrustRootIndex::*;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::ContextAwarePlatform::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::platform::Platform;
use crate::android_test::src::test::kotlin::okhttp::android::test::DisabledInitialiserTest::*;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::CallTagsTest::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::ProtocolTest::*;
use crate::okhttp_testing_support::build_gradle::*;

// Mocking Android-specific types that would be provided by the Android NDK/Framework bindings
pub struct Build;
impl Build {
    pub struct Version;
}
pub struct StrictMode;
impl StrictMode {
    pub fn note_slow_call(_name: &str) {}
}
pub struct NetworkSecurityPolicy;
impl NetworkSecurityPolicy {
    pub fn get_instance() -> Self { NetworkSecurityPolicy }
    pub fn is_cleartext_traffic_permitted(&self, _hostname: &str) -> bool { true }
    pub fn is_cleartext_traffic_permitted_global(&self) -> bool { true }
}
pub struct Log;
impl Log {
    pub fn w(_tag: &str, _msg: &str, _t: Option<&dyn std::error::Error>) {}
    pub fn i(_tag: &str, _msg: &str, _t: Option<&dyn std::error::Error>) {}
}
pub struct Socket;
impl Socket {
    pub fn connect(&self, _address: &InetSocketAddress, _timeout: i32) -> Result<(), IoError> { Ok(()) }
}
pub struct InetSocketAddress;
pub struct SSLContext;
pub struct SSLSocketFactory;
pub struct SSLSocket;
pub struct X509TrustManager;
pub struct X509Certificate;
pub struct TrustAnchor {
    pub trusted_cert: X509Certificate,
}
pub struct Method;
impl Method {
    pub fn set_accessible(&mut self, _accessible: bool) {}
    pub fn invoke(&self, _target: &Any, _args: Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, Box<dyn std::error::Error>> {
        Err(Box::new(IoError::new(ErrorKind::Other, "Not implemented")))
    }
}

const TAG: &str = "OkHttp";
const WARN: i32 = 1;

#[derive(Debug, Clone)]
pub struct AndroidPlatform {
    pub application_context: Option<Context>,
    socket_adapters: Vec<Arc<dyn SocketAdapter>>,
}

impl AndroidPlatform {
    pub fn new() -> Self {
        let mut adapters = Vec::new();
        
        if let Some(adapter) = StandardAndroidSocketAdapter::build_if_supported() {
            adapters.push(Arc::new(adapter));
        }
        adapters.push(Arc::new(DeferredSocketAdapter::new(AndroidSocketAdapter::play_provider_factory())));
        adapters.push(Arc::new(DeferredSocketAdapter::new(ConscryptSocketAdapter::factory())));
        adapters.push(Arc::new(DeferredSocketAdapter::new(BouncyCastleSocketAdapter::factory())));

        let socket_adapters = adapters.into_iter()
            .filter(|a| a.is_supported())
            .collect();

        AndroidPlatform {
            application_context: None,
            socket_adapters,
        }
    }

    pub fn is_supported() -> bool {
        // isAndroid is assumed to be a global check in the original context
        let is_android = true; 
        is_android && Build::SDK_INT >= 21 && Build::SDK_INT < 29
    }

    pub fn build_if_supported() -> Option<Self> {
        if Self::is_supported() {
            Some(Self::new())
        } else {
            None
        }
    }
}

impl ContextAwarePlatform for AndroidPlatform {
    fn application_context(&self) -> Option<Context> {
        self.application_context.clone()
    }

    fn set_application_context(&mut self, context: Option<Context>) {
        self.application_context = context;
    }
}

impl Platform for AndroidPlatform {
    fn connect_socket(&self, socket: &Socket, address: &InetSocketAddress, connect_timeout: i32) -> Result<(), IoError> {
        match socket.connect(address, connect_timeout) {
            Ok(_) => Ok(()),
            Err(e) => {
                // In Rust, ClassCastException doesn't exist in the same way, 
                // but we preserve the logic for SDK 26.
                if Build::SDK_INT == 26 {
                    Err(IoError::new(ErrorKind::Other, format!("Exception in connect: {}", e)))
                } else {
                    Err(e)
                }
            }
        }
    }

    fn new_ssl_context(&self) -> SSLContext {
        StrictMode::note_slow_call("newSSLContext");
        // Call super.newSSLContext() equivalent
        SSLContext
    }

    fn trust_manager(&self, ssl_socket_factory: &SSLSocketFactory) -> Option<X509TrustManager> {
        self.socket_adapters.iter()
            .find(|adapter| adapter.matches_socket_factory(ssl_socket_factory))
            .and_then(|adapter| adapter.trust_manager(ssl_socket_factory))
    }

    fn configure_tls_extensions(&self, ssl_socket: &SSLSocket, hostname: Option<String>, protocols: Vec<Protocol>) {
        if let Some(adapter) = self.socket_adapters.iter().find(|a| a.matches_socket(ssl_socket)) {
            adapter.configure_tls_extensions(ssl_socket, hostname, protocols);
        }
    }

    fn get_selected_protocol(&self, ssl_socket: &SSLSocket) -> Option<String> {
        self.socket_adapters.iter()
            .find(|a| a.matches_socket(ssl_socket))
            .and_then(|a| a.get_selected_protocol(ssl_socket))
    }

    fn is_cleartext_traffic_permitted(&self, hostname: String) -> bool {
        if Build::SDK_INT >= 24 {
            NetworkSecurityPolicy::get_instance().is_cleartext_traffic_permitted(&hostname)
        } else if Build::SDK_INT >= 23 {
            NetworkSecurityPolicy::get_instance().is_cleartext_traffic_permitted_global()
        } else {
            true
        }
    }

    fn build_certificate_chain_cleaner(&self, trust_manager: X509TrustManager) -> Box<dyn CertificateChainCleaner> {
        AndroidCertificateChainCleaner::build_if_supported(trust_manager.clone())
            .map(|cleaner| Box::new(cleaner) as Box<dyn CertificateChainCleaner>)
            .unwrap_or_else(|| {
                // super.buildCertificateChainCleaner(trustManager)
                Box::new(AndroidCertificateChainCleaner::default()) 
            })
    }

    fn build_trust_root_index(&self, trust_manager: X509TrustManager) -> Box<dyn TrustRootIndex> {
        StrictMode::note_slow_call("buildTrustRootIndex");
        
        // Reflection simulation: In Rust, we cannot dynamically get a method by name from a class.
        // We simulate the try-catch block by attempting to construct the CustomTrustRootIndex.
        // In a real production Rust port, this would be replaced by a trait method or a specific implementation.
        let method_found = false; // Simulation of NoSuchMethodException
        if method_found {
            // This part is unreachable in this simulation but preserves the logic flow
            let method = Method; 
            Box::new(CustomTrustRootIndex {
                trust_manager,
                find_by_issuer_and_signature_method: method,
            })
        } else {
            // super.buildTrustRootIndex(trustManager)
            Box::new(BasicTrustRootIndex::new(Vec::new()))
        }
    }

    fn get_handshake_server_names(&self, ssl_socket: &SSLSocket) -> Vec<String> {
        if Build::SDK_INT <= 24 {
            return Vec::new();
        }
        // super.getHandshakeServerNames(sslSocket)
        Vec::new()
    }

    fn log(&self, message: String, level: i32, t: Option<&dyn std::error::Error>) {
        if level == WARN {
            Log::w(TAG, &message, t);
        } else {
            Log::i(TAG, &message, t);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct CustomTrustRootIndex {
    trust_manager: X509TrustManager,
    find_by_issuer_and_signature_method: Method,
}

impl TrustRootIndex for CustomTrustRootIndex {
    fn find_by_issuer_and_signature(&self, cert: X509Certificate) -> Option<X509Certificate> {
        let args: Vec<Box<dyn Any>> = vec![Box::new(self.trust_manager.clone()), Box::new(cert)];
        match self.find_by_issuer_and_signature_method.invoke(&self.trust_manager, args) {
            Ok(result) => {
                // Cast result to TrustAnchor and return trustedCert
                // This is a simulation of the Java reflection cast
                None 
            }
            Err(_) => {
                // InvocationTargetException maps to None in the original code
                None
            }
        }
    }
}