use std::sync::Arc;
use crate::mockwebserver_junit5::src::main::kotlin::mockwebserver3::junit5::StartStop;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::OkHttpClientTestRule;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::testing::PlatformRule;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{Request, HttpUrl, Headers, Dns};
use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::{HandshakeCertificates, HeldCertificate};
use crate::mockwebserver::src::main::kotlin::mockwebserver3::{MockWebServer, MockResponse, RecordedRequest};

// Mocking the TlsUtil.localhost() helper as it's a common utility in okhttp-tls tests
mod tls_util {
    use super::* ;
    pub fn localhost() -> HandshakeCertificates {
        // In a real production translation, this would call the actual TlsUtil implementation
        HandshakeCertificates::builder().build()
    }
}

pub struct MockResponseSniTest {
    client_test_rule: OkHttpClientTestRule,
    platform: PlatformRule,
    server: MockWebServer,
}

impl MockResponseSniTest {
    pub fn new() -> Self {
        Self {
            client_test_rule: OkHttpClientTestRule::new(),
            platform: PlatformRule::new(),
            server: MockWebServer::new(),
        }
    }

    pub async fn client_sends_server_name_and_server_receives_it(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // java.net.ConnectException: Connection refused
        self.platform.assume_not_conscrypt();

        let handshake_certificates = tls_util::localhost();
        self.server.use_https(handshake_certificates.ssl_socket_factory());

        let server_host_name = self.server.host_name().to_string();
        let dns = Dns::new(move |_ -> Result<Vec<std::net::IpAddress>, Box<dyn std::error::Error>> {
            Dns::SYSTEM.lookup(&server_host_name)
        });

        let client = self.client_test_rule
            .new_client_builder()
            .ssl_socket_factory(
                handshake_certificates.ssl_socket_factory(),
                handshake_certificates.trust_manager,
            )
            .dns(dns)
            .build();

        self.server.enqueue(MockResponse::new());

        let url = self.server
            .url("/")
            .new_builder()
            .host("localhost.localdomain")
            .build();
            
        let request = Request::builder()
            .url(url)
            .build();
            
        let call = client.new_call(request);
        let response = call.execute()?;
        assert!(response.is_successful());

        let recorded_request = self.server.take_request();
        
        // https://github.com/bcgit/bc-java/issues/1773
        if !self.platform.is_bouncy_castle() {
            assert_eq!(recorded_request.handshake_server_names, vec![url.host().to_string()]);
        }
        
        Ok(())
    }

    pub async fn domain_fronting(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let held_certificate = HeldCertificate::builder()
            .common_name("server name")
            .add_subject_alternative_name("url-host.com")
            .build();
            
        let handshake_certificates = HandshakeCertificates::builder()
            .held_certificate(held_certificate.clone())
            .add_trusted_certificate(held_certificate.certificate())
            .build();
            
        self.server.use_https(handshake_certificates.ssl_socket_factory());

        let server_host_name = self.server.host_name().to_string();
        let dns = Dns::new(move |_ -> Result<Vec<std::net::IpAddress>, Box<dyn std::error::Error>> {
            Dns::SYSTEM.lookup(&server_host_name)
        });

        let client = self.client_test_rule
            .new_client_builder()
            .ssl_socket_factory(
                handshake_certificates.ssl_socket_factory(),
                handshake_certificates.trust_manager,
            )
            .dns(dns)
            .build();

        self.server.enqueue(MockResponse::new());

        let url_str = format!("https://url-host.com:{}/", self.server.port());
        let url = HttpUrl::parse(&url_str).expect("Invalid URL");
        
        let request = Request::builder()
            .url(url)
            .header("Host", "header-host")
            .build();
            
        let call = client.new_call(request);
        let response = call.execute()?;
        assert!(response.is_successful());

        let recorded_request = self.server.take_request();
        assert_eq!(recorded_request.url.host(), "header-host");

        // https://github.com/bcgit/bc-java/issues/1773
        if !self.platform.is_bouncy_castle() {
            assert_eq!(recorded_request.handshake_server_names, vec!["url-host.com".to_string()]);
        }
        
        Ok(())
    }

    pub async fn ipv6(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let recorded_request = self.request_to_hostname_via_proxy("2607:f8b0:400b:804::200e").await?;
        assert_eq!(recorded_request.url.host(), "2607:f8b0:400b:804::200e");
        assert!(recorded_request.handshake_server_names.is_empty());
        Ok(())
    }

    pub async fn ipv4(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let recorded_request = self.request_to_hostname_via_proxy("76.223.91.57").await?;
        assert_eq!(recorded_request.url.host(), "76.223.91.57");
        assert!(recorded_request.handshake_server_names.is_empty());
        Ok(())
    }

    pub async fn regular_hostname(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let recorded_request = self.request_to_hostname_via_proxy("cash.app").await?;
        assert_eq!(recorded_request.url.host(), "cash.app");
        
        // https://github.com/bcgit/bc-java/issues/1773
        if !self.platform.is_bouncy_castle() {
            assert_eq!(recorded_request.handshake_server_names, vec!["cash.app".to_string()]);
        }
        Ok(())
    }

    async fn request_to_hostname_via_proxy(&mut self, hostname_or_ip_address: &str) -> Result<RecordedRequest, Box<dyn std::error::Error>> {
        let held_certificate = HeldCertificate::builder()
            .common_name("server name")
            .add_subject_alternative_name(hostname_or_ip_address)
            .build();
            
        let handshake_certificates = HandshakeCertificates::builder()
            .held_certificate(held_certificate.clone())
            .add_trusted_certificate(held_certificate.certificate())
            .build();
            
        self.server.use_https(handshake_certificates.ssl_socket_factory());

        let client = self.client_test_rule
            .new_client_builder()
            .ssl_socket_factory(
                handshake_certificates.ssl_socket_factory(),
                handshake_certificates.trust_manager,
            )
            .proxy(self.server.proxy_address())
            .build();

        self.server.enqueue(
            MockResponse::builder()
                .in_tunnel()
                .build(),
        );
        self.server.enqueue(MockResponse::new());

        let url = self.server
            .url("/")
            .new_builder()
            .host(hostname_or_ip_address)
            .build();
            
        let request = Request::builder()
            .url(url)
            .build();
            
        let call = client.new_call(request);
        let response = call.execute()?;
        assert!(response.is_successful());

        self.server.take_request(); // Discard the CONNECT tunnel.
        Ok(self.server.take_request())
    }
}
