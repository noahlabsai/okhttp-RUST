use std::io;
use std::net::{IpAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::mockwebserver::src::main::kotlin::mockwebserver3::{
    MockResponse, MockWebServer, SocketEffect,
};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{
    CallEvent, HttpUrl, OkHttpClient, OkHttpClientBuilder, Protocol, Request,
};
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::testing::Flaky;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::EventRecorder;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::ErrorCode;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::MockResponse::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::SocketEffect::*;
use crate::mockwebserver::src::test::java::mockwebserver3::CustomDispatcherTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::ErrorCode::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::CallTagsTest::*;
use crate::okhttp_testing_support::build_gradle::*;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::EventRecorder::*;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::DelegatingSocketFactory::*;

// Mocking the TestUtil for unreachable addresses as per Kotlin source
struct TestUtil;
impl TestUtil {
    struct UnreachableAddress {
        address: IpAddr,
    }
    const UNREACHABLE_ADDRESS_IPV4: UnreachableAddress = UnreachableAddress {
        address: "192.0.2.1".parse().unwrap(), // TEST-NET-1
    };
    const UNREACHABLE_ADDRESS_IPV6: UnreachableAddress = UnreachableAddress {
        address: "2001:db8::1".parse().unwrap(), // Documentation range
    };
}

// Mocking the DelegatingSocketFactory and SocketFactory logic
pub trait SocketFactory: Send + Sync {
    fn create_socket(&self) -> io::Result<TcpStream>;
}

pub struct DefaultSocketFactory;
impl SocketFactory for DefaultSocketFactory {
    fn create_socket(&self) -> io::Result<TcpStream> {
        // In a real implementation, this would create a socket
        TcpStream::connect("127.0.0.1:0")
    }
}


impl DelegatingSocketFactory {
    pub fn new(delegate: Box<dyn SocketFactory>) -> Self {
        Self { delegate }
    }
}

impl SocketFactory for DelegatingSocketFactory {
    fn create_socket(&self) -> io::Result<TcpStream> {
        self.delegate.create_socket()
    }
}

// Interceptor trait equivalent
pub trait Interceptor: Send + Sync {
    fn intercept(&self, chain: &mut Chain) -> Result<okhttp3::Response, Box<dyn std::error::Error>>;
}

pub struct Chain {
    pub request: Request,
}

impl Chain {
    pub fn proceed(&mut self) -> Result<okhttp3::Response, Box<dyn std::error::Error>> {
        // In a real test environment, this would call the next interceptor or the network
        Err(Box::new(io::Error::new(io::ErrorKind::Other, "Mock chain proceed not implemented")))
    }
    pub fn request(&self) -> &Request {
        &self.request
    }
}

pub struct FastFallbackTest {
    client_test_rule: OkHttpClientTestRule,
    localhost_ipv4: Option<IpAddr>,
    localhost_ipv6: Option<IpAddr>,
    server_ipv4: Option<MockWebServer>,
    server_ipv6: Option<MockWebServer>,
    event_recorder: Arc<EventRecorder>,
    client: Option<OkHttpClient>,
    url: Option<HttpUrl>,
    dns_results: Vec<IpAddr>,
}

impl FastFallbackTest {
    pub fn new() -> Self {
        Self {
            client_test_rule: OkHttpClientTestRule::new(),
            localhost_ipv4: None,
            localhost_ipv6: None,
            server_ipv4: None,
            server_ipv6: None,
            event_recorder: Arc::new(EventRecorder::new()),
            client: None,
            url: None,
            dns_results: Vec::new(),
        }
    }

    pub fn set_up(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Simulate InetAddress.getAllByName("localhost")
        let inet_addresses: Vec<IpAddr> = vec![
            "127.0.0.1".parse().unwrap(),
            "::1".parse().unwrap(),
        ];

        self.localhost_ipv4 = inet_addresses.iter().find(|&&ip| ip.is_ipv4()).cloned();
        self.localhost_ipv6 = inet_addresses.iter().find(|&&ip| ip.is_ipv6()).cloned();

        let ipv4 = self.localhost_ipv4.ok_or("IPv4 localhost not found")?;
        let ipv6 = self.localhost_ipv6.ok_or("IPv6 localhost not found")?;

        let mut s4 = MockWebServer::new();
        s4.start_with_address(ipv4, 0)?;
        let port = s4.port();

        let mut s6 = MockWebServer::new();
        s6.start_with_address(ipv6, port)?;

        self.server_ipv4 = Some(s4);
        self.server_ipv6 = Some(s6);

        self.dns_results = vec![ipv4, ipv6];

        let dns_results_clone = self.dns_results.clone();
        let event_recorder_clone = self.event_recorder.clone();
        
        let client = self.client_test_rule
            .new_client_builder()
            .event_listener_factory(self.client_test_rule.wrap(event_recorder_clone))
            .connect_timeout(Duration::from_secs(60))
            .dns(move || dns_results_clone.clone())
            .fast_fallback(true)
            .build();

        self.client = Some(client);
        
        let s4_ref = self.server_ipv4.as_ref().unwrap();
        self.url = Some(s4_ref.url("/").new_builder().host("localhost").build());

        Ok(())
    }

    pub fn tear_down(&mut self) {
        if let Some(s4) = self.server_ipv4.take() {
            let _ = s4.close();
        }
        if let Some(s6) = self.server_ipv6.take() {
            let _ = s6.close();
        }
    }

    pub fn call_ipv6_first_even_when_ipv4_ip_is_listed_first(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.dns_results = vec![
            self.localhost_ipv4.unwrap(),
            self.localhost_ipv6.unwrap(),
        ];
        
        self.server_ipv4.as_ref().unwrap().enqueue(MockResponse::new().body("unexpected call to IPv4"));
        self.server_ipv6.as_ref().unwrap().enqueue(MockResponse::new().body("hello from IPv6"));

        let client = self.client.as_ref().unwrap();
        let call = client.new_call(Request::new(self.url.as_ref().unwrap().clone()));
        let response = call.execute()?;
        
        assert_eq!(response.body().string(), "hello from IPv6");

        let events = self.event_recorder.recorded_event_types();
        assert_eq!(events.iter().filter(|&&e| e == &CallEvent::ConnectStart).count(), 1);
        assert_eq!(events.iter().filter(|&&e| e == &CallEvent::ConnectFailed).count(), 0);
        
        Ok(())
    }

    pub fn call_ipv6_when_both_servers_are_reachable(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.dns_results = vec![
            self.localhost_ipv6.unwrap(),
            self.localhost_ipv4.unwrap(),
        ];
        
        self.server_ipv4.as_ref().unwrap().enqueue(MockResponse::new().body("unexpected call to IPv4"));
        self.server_ipv6.as_ref().unwrap().enqueue(MockResponse::new().body("hello from IPv6"));

        let client = self.client.as_ref().unwrap();
        let call = client.new_call(Request::new(self.url.as_ref().unwrap().clone()));
        let response = call.execute()?;
        
        assert_eq!(response.body().string(), "hello from IPv6");

        let events = self.event_recorder.recorded_event_types();
        assert_eq!(events.iter().filter(|&&e| e == &CallEvent::ConnectStart).count(), 1);
        assert_eq!(events.iter().filter(|&&e| e == &CallEvent::ConnectFailed).count(), 0);
        
        Ok(())
    }

    pub fn reaches_ipv4_when_ipv6_is_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.server_ipv6.as_ref().unwrap().close()?;
        self.server_ipv4.as_ref().unwrap().enqueue(MockResponse::new().body("hello from IPv4"));

        let client = self.client.as_ref().unwrap();
        let call = client.new_call(Request::new(self.url.as_ref().unwrap().clone()));
        let response = call.execute()?;
        
        assert_eq!(response.body().string(), "hello from IPv4");

        let events = self.event_recorder.recorded_event_types();
        assert_eq!(events.iter().filter(|&&e| e == &CallEvent::ConnectStart).count(), 2);
        assert_eq!(events.iter().filter(|&&e| e == &CallEvent::ConnectFailed).count(), 1);
        assert_eq!(events.iter().filter(|&&e| e == &CallEvent::ConnectEnd).count(), 1);
        
        Ok(())
    }

    pub fn reaches_ipv6_when_ipv4_is_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.server_ipv4.as_ref().unwrap().close()?;
        self.server_ipv6.as_ref().unwrap().enqueue(MockResponse::new().body("hello from IPv6"));

        let client = self.client.as_ref().unwrap();
        let call = client.new_call(Request::new(self.url.as_ref().unwrap().clone()));
        let response = call.execute()?;
        
        assert_eq!(response.body().string(), "hello from IPv6");

        let events = self.event_recorder.recorded_event_types();
        assert_eq!(events.iter().filter(|&&e| e == &CallEvent::ConnectStart).count(), 1);
        assert_eq!(events.iter().filter(|&&e| e == &CallEvent::ConnectEnd).count(), 1);
        assert_eq!(events.iter().filter(|&&e| e == &CallEvent::ConnectFailed).count(), 0);
        
        Ok(())
    }

    pub fn fails_when_both_servers_are_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.server_ipv4.as_ref().unwrap().close()?;
        self.server_ipv6.as_ref().unwrap().close()?;

        let client = self.client.as_ref().unwrap();
        let call = client.new_call(Request::new(self.url.as_ref().unwrap().clone()));
        
        let result = call.execute();
        assert!(result.is_err());

        let events = self.event_recorder.recorded_event_types();
        assert_eq!(events.iter().filter(|&&e| e == &CallEvent::ConnectStart).count(), 2);
        assert_eq!(events.iter().filter(|&&e| e == &CallEvent::ConnectFailed).count(), 2);
        
        Ok(())
    }

    pub fn reaches_ipv4_after_unreachable_ipv6_address(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.dns_results = vec![
            TestUtil::UNREACHABLE_ADDRESS_IPV6.address,
            self.localhost_ipv4.unwrap(),
        ];
        self.server_ipv6.as_ref().unwrap().close()?;
        self.server_ipv4.as_ref().unwrap().enqueue(MockResponse::new().body("hello from IPv4"));

        let client = self.client.as_ref().unwrap();
        let call = client.new_call(Request::new(self.url.as_ref().unwrap().clone()));
        let response = call.execute()?;
        
        assert_eq!(response.body().string(), "hello from IPv4");

        let events = self.event_recorder.recorded_event_types();
        assert_eq!(events.iter().filter(|&&e| e == &CallEvent::ConnectStart).count(), 2);
        assert_eq!(events.iter().filter(|&&e| e == &CallEvent::ConnectFailed).count(), 1);
        
        Ok(())
    }

    pub fn times_out_with_fast_fallback_disabled(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.dns_results = vec![
            TestUtil::UNREACHABLE_ADDRESS_IPV4.address,
            self.localhost_ipv6.unwrap(),
        ];
        self.server_ipv4.as_ref().unwrap().close()?;
        self.server_ipv6.as_ref().unwrap().enqueue(MockResponse::new().body("hello from IPv6"));

        let client = self.client.as_ref().unwrap().new_builder()
            .fast_fallback(false)
            .call_timeout(Duration::from_millis(1000))
            .build();
        
        self.client = Some(client);
        
        let client_ref = self.client.as_ref().unwrap();
        let call = client_ref.new_call(Request::new(self.url.as_ref().unwrap().clone()));
        
        let result = call.execute();
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("timeout"));
        }
        
        Ok(())
    }

    pub fn prefer_call_connection_over_deferred_connection(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.dns_results = vec![
            self.localhost_ipv4.unwrap(),
            self.localhost_ipv6.unwrap(),
            TestUtil::UNREACHABLE_ADDRESS_IPV4.address,
        ];
        
        self.server_ipv4.as_ref().unwrap().set_protocols(vec![Protocol::H2_PRIOR_KNOWLEDGE]);
        self.server_ipv6.as_ref().unwrap().set_protocols(vec![Protocol::H2_PRIOR_KNOWLEDGE]);

        let first_connect_latch = Arc::new(Mutex::new(false));
        
        struct CustomSocketFactory {
            latch: Arc<Mutex<bool>>,
            first: Mutex<bool>,
        }
        impl SocketFactory for CustomSocketFactory {
            fn create_socket(&self) -> io::Result<TcpStream> {
                let mut is_first = self.first.lock().unwrap();
                if *is_first {
                    *is_first = false;
                    while !*self.latch.lock().unwrap() {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                }
                DefaultSocketFactory.create_socket()
            }
        }

        let socket_factory = Arc::new(CustomSocketFactory {
            latch: first_connect_latch.clone(),
            first: Mutex::new(true),
        });

        let latch_for_interceptor = first_connect_latch.clone();
        struct MyInterceptor {
            latch: Arc<Mutex<bool>>,
        }
        impl Interceptor for MyInterceptor {
            fn intercept(&self, chain: &mut Chain) -> Result<okhttp3::Response, Box<dyn std::error::Error>> {
                let res = chain.proceed();
                let mut l = self.latch.lock().unwrap();
                *l = true;
                res
            }
        }

        let client = self.client.as_ref().unwrap().new_builder()
            .protocols(vec![Protocol::H2_PRIOR_KNOWLEDGE])
            .socket_factory(socket_factory)
            .add_network_interceptor(Box::new(MyInterceptor { latch: latch_for_interceptor }))
            .build();
        
        self.client = Some(client);

        self.server_ipv4.as_ref().unwrap().enqueue(
            MockResponse::builder()
                .on_request_start(SocketEffect::CloseStream(ErrorCode::REFUSED_STREAM.http_code()))
                .build()
        );
        self.server_ipv4.as_ref().unwrap().enqueue(MockResponse::new().body("this was the 2nd request on IPv4"));
        self.server_ipv6.as_ref().unwrap().enqueue(MockResponse::new().body("unexpected call to IPv6"));

        let client_ref = self.client.as_ref().unwrap();
        let call = client_ref.new_call(Request::new(self.url.as_ref().unwrap().clone()));
        let response = call.execute()?;
        
        assert_eq!(response.body().string(), "this was the 2nd request on IPv4");
        
        let s4 = self.server_ipv4.as_ref().unwrap();
        assert_eq!(s4.take_request().exchange_index, 0);
        assert_eq!(s4.take_request().exchange_index, 1);
        
        Ok(())
    }
}

// Helper mock for OkHttpClientTestRule
impl OkHttpClientTestRule {
    pub fn new() -> Self { Self }
    pub fn new_client_builder(&self) -> OkHttpClientBuilder { OkHttpClientBuilder::new() }
    pub fn wrap(&self, recorder: Arc<EventRecorder>) -> Box<dyn okhttp3::EventListener> {
        Box::new(recorder)
    }
}
