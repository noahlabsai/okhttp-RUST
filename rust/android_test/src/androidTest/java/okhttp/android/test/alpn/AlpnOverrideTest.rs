use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Call::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Connection::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::ConnectionSpec::*;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::DelegatingSSLSocketFactory::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::EventListener::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::OkHttpClient::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;

// Mocking Android Build and Log as they are platform-specific JVM dependencies
pub struct BuildVersion;
impl BuildVersion {
    pub const SDK_INT: i32 = 30; // Example value representing a modern Android version
    pub const VERSION_CODES_N: i32 = 24;
}

pub struct Log;
impl Log {
    pub fn d(tag: &str, msg: String) {
        println!("[DEBUG] {}: {}", tag, msg);
    }
}

// Mocking javax.net.ssl types as they are JVM specific
#[derive(Debug, Clone, PartialEq)]
pub struct SSLParameters {
    pub application_protocols: Vec<String>,
}

pub struct SSLSocket {
    pub parameters: SSLParameters,
    pub negotiated_protocol: Option<String>,
}

impl SSLSocket {
    pub fn ssl_parameters(&self) -> &SSLParameters {
        &self.parameters
    }
    pub fn set_ssl_parameters(&mut self, params: SSLParameters) {
        self.parameters = params;
    }
    pub fn application_protocol(&self) -> Option<String> {
        self.negotiated_protocol.clone()
    }
}

pub struct SSLSocketFactory;

// Tests for ALPN overriding on Android.
pub struct AlpnOverrideTest {
    pub client: OkHttpClient,
}

impl AlpnOverrideTest {
    pub fn new() -> Self {
        Self {
            client: OkHttpClient::new(),
        }
    }

    pub fn get_with_custom_socket_factory(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let custom_factory = CustomSSLSocketFactory::new(self.client.ssl_socket_factory());
        
        // Kotlin: client.x509TrustManager!!
        let trust_manager = self.client.x509_trust_manager()
            .expect("x509TrustManager should not be null");

        let connection_spec = ConnectionSpec::builder(ConnectionSpec::MODERN_TLS)
            .supports_tls_extensions(false)
            .build();

        let event_listener = CustomEventListener {};

        self.client = self.client
            .new_builder()
            .ssl_socket_factory(custom_factory, trust_manager)
            .connection_specs(vec![connection_spec])
            .event_listener(event_listener)
            .build();

        let request = Request::builder()
            .url("https://www.google.com")
            .build();

        let response = self.client.new_call(request).execute()?;
        
        // assertThat(response.code).isEqualTo(200)
        if response.code != 200 {
            return Err(format!("Expected 200, got {}", response.code).into());
        }

        Ok(())
    }
}

pub struct CustomSSLSocketFactory {
    delegate: SSLSocketFactory,
}

impl CustomSSLSocketFactory {
    pub fn new(delegate: SSLSocketFactory) -> Self {
        Self { delegate }
    }
}

impl DelegatingSSLSocketFactory for CustomSSLSocketFactory {
    fn configure_socket(&self, mut ssl_socket: SSLSocket) -> SSLSocket {
        if BuildVersion::SDK_INT >= BuildVersion::VERSION_CODES_N {
            let mut parameters = ssl_socket.ssl_parameters().clone();
            
            Log::d(
                "CustomSSLSocketFactory", 
                format!("old applicationProtocols: {:?}", parameters.application_protocols)
            );
            
            parameters.application_protocols = vec!["x-amzn-http-ca".to_string()];
            ssl_socket.set_ssl_parameters(parameters);
        }
        ssl_socket
    }
}

struct CustomEventListener;

impl EventListener for CustomEventListener {
    fn connection_acquired(&self, _call: &Call, connection: &Connection) {
        // Kotlin: val sslSocket = connection.socket() as SSLSocket
        // In Rust, we assume the socket() returns a trait object or we use a safe cast pattern
        if let Some(ssl_socket) = connection.socket_as_ssl() {
            let protocols = ssl_socket.ssl_parameters().application_protocols.join(", ");
            println!("Requested {}", protocols);
            println!("Negotiated {}", ssl_socket.application_protocol().unwrap_or_default());
        }
    }
}

// Extension trait to handle the 'as SSLSocket' cast from Kotlin
trait ConnectionExt {
    fn socket_as_ssl(&self) -> Option<&SSLSocket>;
}

impl ConnectionExt for Connection {
    fn socket_as_ssl(&self) -> Option<&SSLSocket> {
        // This is a simplified representation of the JVM cast
        self.socket().downcast_ref::<SSLSocket>()
    }
}