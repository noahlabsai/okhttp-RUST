use std::error::Error;
use std::net::{InetAddress, InetSocketAddress};
use std::sync::Arc;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;

// Assuming these types are defined elsewhere in the okhttp3 crate
use crate::okhttp3::{
    Call, Connection, Dispatcher, Handshake, HttpUrl, Protocol, Proxy, Request, Response,
};

// Listener for metrics events. Extend this class to monitor the quantity, size, and duration of
// your application's HTTP calls.
pub trait EventListener: Send + Sync {
    fn call_start(&self, _call: &Call) {}

    fn dispatcher_queue_start(&self, _call: &Call, _dispatcher: &Dispatcher) {}

    fn dispatcher_queue_end(&self, _call: &Call, _dispatcher: &Dispatcher) {}

    fn proxy_select_start(&self, _call: &Call, _url: &HttpUrl) {}

    fn proxy_select_end(&self, _call: &Call, _url: &HttpUrl, _proxies: &[Proxy]) {}

    fn dns_start(&self, _call: &Call, _domain_name: &str) {}

    fn dns_end(&self, _call: &Call, _domain_name: &str, _inet_address_list: &[InetAddress]) {}

    fn connect_start(&self, _call: &Call, _inet_socket_address: &InetSocketAddress, _proxy: &Proxy) {}

    fn secure_connect_start(&self, _call: &Call) {}

    fn secure_connect_end(&self, _call: &Call, _handshake: Option<&Handshake>) {}

    fn connect_end(&self, _call: &Call, _inet_socket_address: &InetSocketAddress, _proxy: &Proxy, _protocol: Option<&Protocol>) {}

    fn connect_failed(&self, _call: &Call, _inet_socket_address: &InetSocketAddress, _proxy: &Proxy, _protocol: Option<&Protocol>, _ioe: &(dyn Error + Send + Sync)) {}

    fn connection_acquired(&self, _call: &Call, _connection: &Connection) {}

    fn connection_released(&self, _call: &Call, _connection: &Connection) {}

    fn request_headers_start(&self, _call: &Call) {}

    fn request_headers_end(&self, _call: &Call, _request: &Request) {}

    fn request_body_start(&self, _call: &Call) {}

    fn request_body_end(&self, _call: &Call, _byte_count: i64) {}

    fn request_failed(&self, _call: &Call, _ioe: &(dyn Error + Send + Sync)) {}

    fn response_headers_start(&self, _call: &Call) {}

    fn response_headers_end(&self, _call: &Call, _response: &Response) {}

    fn response_body_start(&self, _call: &Call) {}

    fn response_body_end(&self, _call: &Call, _byte_count: i64) {}

    fn response_failed(&self, _call: &Call, _ioe: &(dyn Error + Send + Sync)) {}

    fn call_end(&self, _call: &Call) {}

    fn call_failed(&self, _call: &Call, _ioe: &(dyn Error + Send + Sync)) {}

    fn canceled(&self, _call: &Call) {}

    fn satisfaction_failure(&self, _call: &Call, _response: &Response) {}

    fn cache_hit(&self, _call: &Call, _response: &Response) {}

    fn cache_miss(&self, _call: &Call) {}

    fn cache_conditional_hit(&self, _call: &Call, _cached_response: &Response) {}

    fn retry_decision(&self, _call: &Call, _exception: &(dyn Error + Send + Sync), _retry: bool) {}

    fn follow_up_decision(&self, _call: &Call, _network_response: &Response, _next_request: Option<&Request>) {}
}

// Implementation of the NONE singleton
pub struct NoneEventListener;
impl EventListener for NoneEventListener {}

// AggregateEventListener allows multiple EventListeners to be combined.
struct AggregateEventListener {
    event_listeners: Vec<Arc<dyn EventListener>>,
}

impl EventListener for AggregateEventListener {
    fn call_start(&self, call: &Call) {
        for delegate in &self.event_listeners {
            delegate.call_start(call);
        }
    }
    fn dispatcher_queue_start(&self, call: &Call, dispatcher: &Dispatcher) {
        for delegate in &self.event_listeners {
            delegate.dispatcher_queue_start(call, dispatcher);
        }
    }
    fn dispatcher_queue_end(&self, call: &Call, dispatcher: &Dispatcher) {
        for delegate in &self.event_listeners {
            delegate.dispatcher_queue_end(call, dispatcher);
        }
    }
    fn proxy_select_start(&self, call: &Call, url: &HttpUrl) {
        for delegate in &self.event_listeners {
            delegate.proxy_select_start(call, url);
        }
    }
    fn proxy_select_end(&self, call: &Call, url: &HttpUrl, proxies: &[Proxy]) {
        for delegate in &self.event_listeners {
            delegate.proxy_select_end(call, url, proxies);
        }
    }
    fn dns_start(&self, call: &Call, domain_name: &str) {
        for delegate in &self.event_listeners {
            delegate.dns_start(call, domain_name);
        }
    }
    fn dns_end(&self, call: &Call, domain_name: &str, inet_address_list: &[InetAddress]) {
        for delegate in &self.event_listeners {
            delegate.dns_end(call, domain_name, inet_address_list);
        }
    }
    fn connect_start(&self, call: &Call, inet_socket_address: &InetSocketAddress, proxy: &Proxy) {
        for delegate in &self.event_listeners {
            delegate.connect_start(call, inet_socket_address, proxy);
        }
    }
    fn secure_connect_start(&self, call: &Call) {
        for delegate in &self.event_listeners {
            delegate.secure_connect_start(call);
        }
    }
    fn secure_connect_end(&self, call: &Call, handshake: Option<&Handshake>) {
        for delegate in &self.event_listeners {
            delegate.secure_connect_end(call, handshake);
        }
    }
    fn connect_end(&self, call: &Call, inet_socket_address: &InetSocketAddress, proxy: &Proxy, protocol: Option<&Protocol>) {
        for delegate in &self.event_listeners {
            delegate.connect_end(call, inet_socket_address, proxy, protocol);
        }
    }
    fn connect_failed(&self, call: &Call, inet_socket_address: &InetSocketAddress, proxy: &Proxy, protocol: Option<&Protocol>, ioe: &(dyn Error + Send + Sync)) {
        for delegate in &self.event_listeners {
            delegate.connect_failed(call, inet_socket_address, proxy, protocol, ioe);
        }
    }
    fn connection_acquired(&self, call: &Call, connection: &Connection) {
        for delegate in &self.event_listeners {
            delegate.connection_acquired(call, connection);
        }
    }
    fn connection_released(&self, call: &Call, connection: &Connection) {
        for delegate in &self.event_listeners {
            delegate.connection_released(call, connection);
        }
    }
    fn request_headers_start(&self, call: &Call) {
        for delegate in &self.event_listeners {
            delegate.request_headers_start(call);
        }
    }
    fn request_headers_end(&self, call: &Call, request: &Request) {
        for delegate in &self.event_listeners {
            delegate.request_headers_end(call, request);
        }
    }
    fn request_body_start(&self, call: &Call) {
        for delegate in &self.event_listeners {
            delegate.request_body_start(call);
        }
    }
    fn request_body_end(&self, call: &Call, byte_count: i64) {
        for delegate in &self.event_listeners {
            delegate.request_body_end(call, byte_count);
        }
    }
    fn request_failed(&self, call: &Call, ioe: &(dyn Error + Send + Sync)) {
        for delegate in &self.event_listeners {
            delegate.request_failed(call, ioe);
        }
    }
    fn response_headers_start(&self, call: &Call) {
        for delegate in &self.event_listeners {
            delegate.response_headers_start(call);
        }
    }
    fn response_headers_end(&self, call: &Call, response: &Response) {
        for delegate in &self.event_listeners {
            delegate.response_headers_end(call, response);
        }
    }
    fn response_body_start(&self, call: &Call) {
        for delegate in &self.event_listeners {
            delegate.response_body_start(call);
        }
    }
    fn response_body_end(&self, call: &Call, byte_count: i64) {
        for delegate in &self.event_listeners {
            delegate.response_body_end(call, byte_count);
        }
    }
    fn response_failed(&self, call: &Call, ioe: &(dyn Error + Send + Sync)) {
        for delegate in &self.event_listeners {
            delegate.response_failed(call, ioe);
        }
    }
    fn call_end(&self, call: &Call) {
        for delegate in &self.event_listeners {
            delegate.call_end(call);
        }
    }
    fn call_failed(&self, call: &Call, ioe: &(dyn Error + Send + Sync)) {
        for delegate in &self.event_listeners {
            delegate.call_failed(call, ioe);
        }
    }
    fn canceled(&self, call: &Call) {
        for delegate in &self.event_listeners {
            delegate.canceled(call);
        }
    }
    fn satisfaction_failure(&self, call: &Call, response: &Response) {
        for delegate in &self.event_listeners {
            delegate.satisfaction_failure(call, response);
        }
    }
    fn cache_hit(&self, call: &Call, response: &Response) {
        for delegate in &self.event_listeners {
            delegate.cache_hit(call, response);
        }
    }
    fn cache_miss(&self, call: &Call) {
        for delegate in &self.event_listeners {
            delegate.cache_miss(call);
        }
    }
    fn cache_conditional_hit(&self, call: &Call, cached_response: &Response) {
        for delegate in &self.event_listeners {
            delegate.cache_conditional_hit(call, cached_response);
        }
    }
    fn retry_decision(&self, call: &Call, exception: &(dyn Error + Send + Sync), retry: bool) {
        for delegate in &self.event_listeners {
            delegate.retry_decision(call, exception, retry);
        }
    }
    fn follow_up_decision(&self, call: &Call, network_response: &Response, next_request: Option<&Request>) {
        for delegate in &self.event_listeners {
            delegate.follow_up_decision(call, network_response, next_request);
        }
    }
}

// Helper to implement the `plus` operator logic for EventListeners.
pub fn combine_event_listeners(left: Arc<dyn EventListener>, right: Arc<dyn EventListener>) -> Arc<dyn EventListener> {
    // In Rust, we can't easily check if a trait object is a specific struct without downcasting.
    // To preserve the logic of the Kotlin `plus` operator:
    
    // Check if either is "NONE" (NoneEventListener)
    // Note: In a real production system, we'd use a more robust way to identify the NONE singleton.
    // For this translation, we treat them as general listeners unless we can identify them.
    
    let mut listeners = Vec::new();
    
    // This is a simplification of the Kotlin `when` logic for AggregateEventListener
    // Since we are using Arc<dyn EventListener>, we treat them as individual units 
    // unless we implement a way to flatten them.
    listeners.push(left);
    listeners.push(right);
    
    Arc::new(AggregateEventListener {
        event_listeners: listeners,
    })
}

pub trait EventListenerFactory: Send + Sync {
    fn create(&self, call: &Call) -> Arc<dyn EventListener>;
}