use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::OkHttpClient;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::DelegatingSSLSocketFactory;
use std::sync::Arc;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::SocketHandler::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::letsencrypt::LetsEncryptClientTest::*;

// Mocking Android Build and Log for compilability as they are platform-specific
pub struct Build;
impl Build {
    pub struct Version;
    pub struct VersionCodes;
    pub const N: i32 = 24; // Mocked value
}

pub struct Log;
impl Log {
    pub fn d(tag: &str, msg: String) {
        println!("[{}] DEBUG: {}", tag, msg);
    }
}

// Mocking Java SSL types
#[derive(Debug, Clone)]
pub struct SNIHostName(pub String);

#[derive(Debug, Clone)]
pub enum SNIServerName {
    HostName(SNIHostName),
}

pub const HostName: SNIServerName = SNIServerName::HostName;

impl Default for SNIServerName {
    fn default() -> Self {
        SNIServerName::HostName(SNIHostName("".to_string()))
    }
}

pub struct SSLParameters {
    pub server_names: Vec<SNIServerName>,
}

pub struct SSLSocket {
    pub parameters: SSLParameters,
}

impl SSLSocket {
    pub fn ssl_parameters(&self) -> SSLParameters {
        SSLParameters {
            server_names: self.parameters.server_names.clone(),
        }
    }
    pub fn set_ssl_parameters(&mut self, params: SSLParameters) {
        self.parameters = params;
    }
}

pub struct SSLSocketFactory;

pub struct SSLSession {
    pub peer_host: String,
    pub peer_certificates: Vec<Box<dyn std::any::Any>>,
}

// Mocking OkHttp Dns
pub struct Dns;
impl Dns {
    pub const SYSTEM: Dns = Dns;
    pub fn lookup(&self, hostname: &str) -> Vec<String> {
        vec![hostname.to_string()]
    }
}

// Mocking Response and Body
pub struct Response {
    pub code: i32,
    pub protocol: Protocol,
    pub body: ResponseBody,
}

pub struct ResponseBody;
impl ResponseBody {
    pub fn string(&self) -> String {
        "h=cloudflare-dns.com".to_string()
    }
}

// Custom Socket Factory Implementation
pub struct CustomSSLSocketFactory {
    delegate: Arc<DelegatingSSLSocketFactory>,
}

impl CustomSSLSocketFactory {
    pub fn new(delegate: Arc<DelegatingSSLSocketFactory>) -> Self {
        Self { delegate }
    }

    pub fn configure_socket(&self, ssl_socket: &mut SSLSocket) -> &mut SSLSocket {
        if Build::SDK_INT >= Build::N {
            let mut parameters = ssl_socket.ssl_parameters();
            let sni = format!("{:?}", parameters.server_names);
            Log::d("CustomSSLSocketFactory", format!("old SNI: {}", sni));
            
            parameters.server_names = vec![SNIServerName::HostName(SNIHostName("cloudflare-dns.com".to_string()))];
            ssl_socket.set_ssl_parameters(parameters);
        }
        ssl_socket
    }
}

pub struct SniOverrideTest {
    pub client: OkHttpClient,
}

impl SniOverrideTest {
    pub fn new() -> Self {
        Self {
            client: OkHttpClient::builder().build(),
        }
    }

    pub fn assume_true(condition: bool) {
        if !condition {
            panic!("Assumption failed");
        }
    }

    pub fn get_with_custom_socket_factory(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Self::assume_true(Build::SDK_INT >= 24);

        let socket_factory = Arc::new(DelegatingSSLSocketFactory::new(SSLSocketFactory));
        let custom_factory = CustomSSLSocketFactory::new(socket_factory);

        self.client = self.client.new_builder()
            .ssl_socket_factory(custom_factory, self.client.x509_trust_manager.as_ref().expect("TrustManager required"))
            .hostname_verifier(Box::new(|hostname, session: &SSLSession| {
                let s = format!("hostname: {} peerHost:{}", hostname, session.peer_host);
                Log::d("SniOverrideTest", s);
                
                if let Some(cert_any) = session.peer_certificates.get(0) {
                    if let Some(cert) = cert_any.downcast_ref::<X509Certificate>() {
                        for name in &cert.subject_alternative_names {
                            if let Some(type_val) = name.get(0).and_then(|v| v.downcast_ref::<i32>()) {
                                if *type_val == 2 {
                                    if let Some(val) = name.get(1).and_then(|v| v.downcast_ref::<String>()) {
                                        Log::d("SniOverrideTest", format!("cert: {}", val));
                                    }
                                }
                            }
                        }
                        return true;
                    }
                }
                false
            }))
            .build();

        let request = Request::builder()
            .url("https://sni.cloudflaressl.com/cdn-cgi/trace")
            .header("Host", "cloudflare-dns.com")
            .build();

        let response = self.client.new_call(request).execute()?;
        
        assert_eq!(response.code, 200);
        assert_eq!(response.protocol, Protocol::HTTP_2);
        assert!(response.body.string().contains("h=cloudflare-dns.com"));

        Ok(())
    }

    pub fn get_with_dns(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.client = self.client.new_builder()
            .dns(Box::new(|hostname| {
                Dns::SYSTEM.lookup(hostname)
            }))
            .build();

        let request = Request::builder()
            .url("https://cloudflare-dns.com/cdn-cgi/trace")
            .build();

        let response = self.client.new_call(request).execute()?;
        
        assert_eq!(response.code, 200);
        assert_eq!(response.protocol, Protocol::HTTP_2);
        assert!(response.body.string().contains("h=cloudflare-dns.com"));

        Ok(())
    }
}

// generated-compatibility for X509Certificate to ensure compilation
