use std::net::{IpAddr, SocketAddr};
use std::time::Instant;
use std::sync::{Arc, Mutex};

// Assuming these types are defined in the okhttp3 crate as per the provided context
use crate::okhttp3::{
    Call, Dispatcher, EventListener, EventListenerFactory, Handshake, HttpUrl, 
    Proxy, Protocol, Connection, Request, Response
};

pub struct ClientRuleEventListener {
    pub logger: Box<dyn Fn(String) + Send + Sync>,
    start_ns: Mutex<Option<Instant>>,
}

impl ClientRuleEventListener {
    pub fn new(logger: impl Fn(String) + Send + Sync + 'static) -> Self {
        Self {
            logger: Box::new(logger),
            start_ns: Mutex::new(None),
        }
    }

    fn log_with_time(&self, message: String) {
        let start_ns_lock = self.start_ns.lock().unwrap();
        let time_ms = if let Some(start_time) = *start_ns_lock {
            start_time.elapsed().as_millis() as i64
        } else {
            // Event occurred before start, for an example an early cancel.
            0i64
        };

        (self.logger)(format!("[{} ms] {}", time_ms, message));
    }
}

impl EventListenerFactory for ClientRuleEventListener {
    fn create(&self, _call: &Call) -> Arc<dyn EventListener> {
        // In Kotlin, 'this' is returned. In Rust, the factory must return an Arc<dyn EventListener>.
        // Since ClientRuleEventListener is the listener itself, we wrap it in an Arc.
        // This assumes the factory is called on an Arc<ClientRuleEventListener>.
        // To maintain the 1:1 architecture where the factory is the listener, 
        // we must be able to return an Arc of self.
        unimplemented!("The factory pattern in Rust requires the instance to be Arc-managed to return Arc<Self>")
    }
}

impl EventListener for ClientRuleEventListener {
    fn call_start(&self, call: &Call) {
        {
            let mut start_ns_lock = self.start_ns.lock().unwrap();
            *start_ns_lock = Some(Instant::now());
        }
        self.log_with_time(format!("callStart: {}", call.request()));
    }

    fn dispatcher_queue_start(&self, _call: &Call, dispatcher: &Dispatcher) {
        self.log_with_time(format!(
            "dispatcherQueueStart: queuedCallsCount={}",
            dispatcher.queued_calls_count()
        ));
    }

    fn dispatcher_queue_end(&self, _call: &Call, dispatcher: &Dispatcher) {
        self.log_with_time(format!(
            "dispatcherQueueEnd: queuedCallsCount={}",
            dispatcher.queued_calls_count()
        ));
    }

    fn proxy_select_start(&self, _call: &Call, url: &HttpUrl) {
        self.log_with_time(format!("proxySelectStart: {}", url));
    }

    fn proxy_select_end(&self, _call: &Call, _url: &HttpUrl, proxies: &[Proxy]) {
        self.log_with_time(format!("proxySelectEnd: {:?}", proxies));
    }

    fn dns_start(&self, _call: &Call, domain_name: &str) {
        self.log_with_time(format!("dnsStart: {}", domain_name));
    }

    fn dns_end(&self, _call: &Call, _domain_name: &str, inet_address_list: &[IpAddr]) {
        self.log_with_time(format!("dnsEnd: {:?}", inet_address_list));
    }

    fn connect_start(&self, _call: &Call, inet_socket_address: &SocketAddr, proxy: &Proxy) {
        self.log_with_time(format!("connectStart: {} {}", inet_socket_address, proxy));
    }

    fn secure_connect_start(&self, _call: &Call) {
        self.log_with_time("secureConnectStart".to_string());
    }

    fn secure_connect_end(&self, _call: &Call, handshake: Option<&Handshake>) {
        self.log_with_time(format!("secureConnectEnd: {:?}", handshake));
    }

    fn connect_end(&self, _call: &Call, _inet_socket_address: &SocketAddr, _proxy: &Proxy, protocol: Option<&Protocol>) {
        self.log_with_time(format!("connectEnd: {:?}", protocol));
    }

    fn connect_failed(&self, _call: &Call, _inet_socket_address: &SocketAddr, _proxy: &Proxy, protocol: Option<&Protocol>, ioe: &dyn std::error::Error) {
        self.log_with_time(format!("connectFailed: {:?} {}", protocol, ioe));
    }

    fn connection_acquired(&self, _call: &Call, connection: &Connection) {
        self.log_with_time(format!("connectionAcquired: {}", connection));
    }

    fn connection_released(&self, _call: &Call, _connection: &Connection) {
        self.log_with_time("connectionReleased".to_string());
    }

    fn request_headers_start(&self, _call: &Call) {
        self.log_with_time("requestHeadersStart".to_string());
    }

    fn request_headers_end(&self, _call: &Call, _request: &Request) {
        self.log_with_time("requestHeadersEnd".to_string());
    }

    fn request_body_start(&self, _call: &Call) {
        self.log_with_time("requestBodyStart".to_string());
    }

    fn request_body_end(&self, _call: &Call, byte_count: i64) {
        self.log_with_time(format!("requestBodyEnd: byteCount={}", byte_count));
    }

    fn request_failed(&self, _call: &Call, ioe: &dyn std::error::Error) {
        self.log_with_time(format!("requestFailed: {}", ioe));
    }

    fn response_headers_start(&self, _call: &Call) {
        self.log_with_time("responseHeadersStart".to_string());
    }

    fn response_headers_end(&self, _call: &Call, response: &Response) {
        self.log_with_time(format!("responseHeadersEnd: {}", response));
    }

    fn response_body_start(&self, _call: &Call) {
        self.log_with_time("responseBodyStart".to_string());
    }

    fn response_body_end(&self, _call: &Call, byte_count: i64) {
        self.log_with_time(format!("responseBodyEnd: byteCount={}", byte_count));
    }

    fn response_failed(&self, _call: &Call, ioe: &dyn std::error::Error) {
        self.log_with_time(format!("responseFailed: {}", ioe));
    }

    fn call_end(&self, _call: &Call) {
        self.log_with_time("callEnd".to_string());
    }

    fn call_failed(&self, _call: &Call, ioe: &dyn std::error::Error) {
        self.log_with_time(format!("callFailed: {}", ioe));
    }

    fn canceled(&self, _call: &Call) {
        self.log_with_time("canceled".to_string());
    }

    fn satisfaction_failure(&self, _call: &Call, _response: &Response) {
        self.log_with_time("satisfactionFailure".to_string());
    }

    fn cache_miss(&self, _call: &Call) {
        self.log_with_time("cacheMiss".to_string());
    }

    fn cache_hit(&self, _call: &Call, _response: &Response) {
        self.log_with_time("cacheHit".to_string());
    }

    fn cache_conditional_hit(&self, _call: &Call, _cached_response: &Response) {
        self.log_with_time("cacheConditionalHit".to_string());
    }

    fn retry_decision(&self, _call: &Call, _exception: &dyn std::error::Error, _retry: bool) {
        self.log_with_time("retryDecision".to_string());
    }

    fn follow_up_decision(&self, _call: &Call, _network_response: &Response, _next_request: Option<&Request>) {
        self.log_with_time("followUpDecision".to_string());
    }
}
