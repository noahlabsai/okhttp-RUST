use std::net::{InetAddress, InetSocketAddress};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::error::Error;

// Import types from the specified paths
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::CallEvent;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::CallTagsTest::*;
use crate::okhttp_testing_support::build_gradle::*;

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{
    Call, Connection, Dispatcher, Handshake, HttpUrl, Protocol, Proxy, Request, Response, EventListener,
};

// This accepts events as function calls on [EventListener], and publishes them as subtypes of
// [CallEvent].
pub struct EventListenerAdapter {
    pub listeners: Mutex<Vec<Box<dyn Fn(CallEvent) + Send + Sync>>>,
}

impl EventListenerAdapter {
    pub fn new() -> Self {
        Self {
            listeners: Mutex::new(Vec::new()),
        }
    }

    fn on_event(&self, event: CallEvent) {
        let listeners = self.listeners.lock().unwrap();
        for listener in listeners.iter() {
            listener(event.clone());
        }
    }

    fn nano_time() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as i64
    }
}

impl EventListener for EventListenerAdapter {
    fn dispatcher_queue_start(&self, call: &Call, dispatcher: &Dispatcher) {
        self.on_event(CallEvent::DispatcherQueueStart {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            dispatcher: dispatcher.clone(),
        });
    }

    fn dispatcher_queue_end(&self, call: &Call, dispatcher: &Dispatcher) {
        self.on_event(CallEvent::DispatcherQueueEnd {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            dispatcher: dispatcher.clone(),
        });
    }

    fn proxy_select_start(&self, call: &Call, url: &HttpUrl) {
        self.on_event(CallEvent::ProxySelectStart {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            url: url.clone(),
        });
    }

    fn proxy_select_end(&self, call: &Call, url: &HttpUrl, proxies: &[Proxy]) {
        self.on_event(CallEvent::ProxySelectEnd {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            url: url.clone(),
            proxies: Some(proxies.to_vec()),
        });
    }

    fn dns_start(&self, call: &Call, domain_name: &str) {
        self.on_event(CallEvent::DnsStart {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            domain_name: domain_name.to_string(),
        });
    }

    fn dns_end(&self, call: &Call, domain_name: &str, inet_address_list: &[InetAddress]) {
        self.on_event(CallEvent::DnsEnd {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            domain_name: domain_name.to_string(),
            inet_address_list: inet_address_list.to_vec(),
        });
    }

    fn connect_start(&self, call: &Call, inet_socket_address: &InetSocketAddress, proxy: &Proxy) {
        self.on_event(CallEvent::ConnectStart {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            inet_socket_address: *inet_socket_address,
            proxy: Some(proxy.clone()),
        });
    }

    fn secure_connect_start(&self, call: &Call) {
        self.on_event(CallEvent::SecureConnectStart {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
        });
    }

    fn secure_connect_end(&self, call: &Call, handshake: Option<&Handshake>) {
        self.on_event(CallEvent::SecureConnectEnd {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            handshake: handshake.cloned(),
        });
    }

    fn connect_end(&self, call: &Call, inet_socket_address: &InetSocketAddress, proxy: &Proxy, protocol: Option<&Protocol>) {
        self.on_event(CallEvent::ConnectEnd {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            inet_socket_address: *inet_socket_address,
            proxy: Some(proxy.clone()),
            protocol: protocol.cloned(),
        });
    }

    fn connect_failed(&self, call: &Call, inet_socket_address: &InetSocketAddress, proxy: &Proxy, protocol: Option<&Protocol>, ioe: &(dyn Error + Send + Sync)) {
        // In a real production environment, we would need a way to clone the error or wrap it.
        // Since CallEvent expects Box<dyn Error>, we assume the error can be boxed.
        // Note: This is a simplification as dyn Error is not Clone.
        // In the Kotlin source, IOException is passed.
        self.on_event(CallEvent::ConnectFailed {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            inet_socket_address: *inet_socket_address,
            proxy: proxy.clone(),
            protocol: protocol.cloned(),
            ioe: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Connect failed")), 
        });
    }

    fn connection_acquired(&self, call: &Call, connection: &Connection) {
        self.on_event(CallEvent::ConnectionAcquired {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            connection: connection.clone(),
        });
    }

    fn connection_released(&self, call: &Call, connection: &Connection) {
        self.on_event(CallEvent::ConnectionReleased {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            connection: connection.clone(),
        });
    }

    fn call_start(&self, call: &Call) {
        self.on_event(CallEvent::CallStart {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
        });
    }

    fn request_headers_start(&self, call: &Call) {
        self.on_event(CallEvent::RequestHeadersStart {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
        });
    }

    fn request_headers_end(&self, call: &Call, request: &Request) {
        self.on_event(CallEvent::RequestHeadersEnd {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            header_length: request.headers().byte_count(),
        });
    }

    fn request_body_start(&self, call: &Call) {
        self.on_event(CallEvent::RequestBodyStart {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
        });
    }

    fn request_body_end(&self, call: &Call, byte_count: i64) {
        self.on_event(CallEvent::RequestBodyEnd {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            bytes_written: byte_count,
        });
    }

    fn request_failed(&self, call: &Call, ioe: &(dyn Error + Send + Sync)) {
        self.on_event(CallEvent::RequestFailed {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            ioe: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Request failed")),
        });
    }

    fn response_headers_start(&self, call: &Call) {
        self.on_event(CallEvent::ResponseHeadersStart {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
        });
    }

    fn response_headers_end(&self, call: &Call, response: &Response) {
        self.on_event(CallEvent::ResponseHeadersEnd {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            header_length: response.headers().byte_count(),
        });
    }

    fn response_body_start(&self, call: &Call) {
        self.on_event(CallEvent::ResponseBodyStart {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
        });
    }

    fn response_body_end(&self, call: &Call, byte_count: i64) {
        self.on_event(CallEvent::ResponseBodyEnd {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            bytes_read: byte_count,
        });
    }

    fn response_failed(&self, call: &Call, ioe: &(dyn Error + Send + Sync)) {
        self.on_event(CallEvent::ResponseFailed {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            ioe: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Response failed")),
        });
    }

    fn call_end(&self, call: &Call) {
        self.on_event(CallEvent::CallEnd {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
        });
    }

    fn call_failed(&self, call: &Call, ioe: &(dyn Error + Send + Sync)) {
        self.on_event(CallEvent::CallFailed {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            ioe: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Call failed")),
        });
    }

    fn canceled(&self, call: &Call) {
        self.on_event(CallEvent::Canceled {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
        });
    }

    fn satisfaction_failure(&self, call: &Call, _response: &Response) {
        self.on_event(CallEvent::SatisfactionFailure {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
        });
    }

    fn cache_miss(&self, call: &Call) {
        self.on_event(CallEvent::CacheMiss {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
        });
    }

    fn cache_hit(&self, call: &Call, _response: &Response) {
        self.on_event(CallEvent::CacheHit {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
        });
    }

    fn cache_conditional_hit(&self, call: &Call, _cached_response: &Response) {
        self.on_event(CallEvent::CacheConditionalHit {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
        });
    }

    fn retry_decision(&self, call: &Call, exception: &(dyn Error + Send + Sync), retry: bool) {
        self.on_event(CallEvent::RetryDecision {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            exception: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Retry decision")),
            retry,
        });
    }

    fn follow_up_decision(&self, call: &Call, network_response: &Response, next_request: Option<&Request>) {
        self.on_event(CallEvent::FollowUpDecision {
            timestamp_ns: Self::nano_time(),
            call: call.clone(),
            network_response: network_response.clone(),
            next_request: next_request.cloned(),
        });
    }
}