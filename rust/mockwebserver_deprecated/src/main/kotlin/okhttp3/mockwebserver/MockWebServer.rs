/*
 * Copyright (C) 2020 Square, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::io;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::fmt;

// Import paths as specified in the translation directives
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrl;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::MockResponse::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::QueueDispatcher::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::RecordedRequest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;

use crate::mockwebserver::src::main::kotlin::mockwebserver3::{
use crate::mockwebserver::src::test::java::mockwebserver3::CustomDispatcherTest::*;
    Dispatcher, MockResponse, MockWebServer as MockWebServer3, 
    QueueDispatcher, RecordedRequest, ServerSocketFactory, SSLSocketFactory
};

// MockWebServer is a wrapper around mockwebserver3::MockWebServer to provide 
// compatibility with the deprecated API.

impl MockWebServer {
    pub fn new() -> Self {
        let delegate = Arc::new(Mutex::new(MockWebServer3::new()));
        let server = Self {
            delegate,
            started: Mutex::new(false),
        };
        
        // Initialize with QueueDispatcher as per Kotlin init block
        server.set_dispatcher(Box::new(QueueDispatcher::new()));
        server
    }

    pub fn request_count(&self) -> i32 {
        self.delegate.lock().unwrap().request_count()
    }

    pub fn body_limit(&self) -> i64 {
        self.delegate.lock().unwrap().body_limit()
    }

    pub fn set_body_limit(&self, body_limit: i64) {
        self.delegate.lock().unwrap().set_body_limit(body_limit);
    }

    pub fn server_socket_factory(&self) -> Option<ServerSocketFactory> {
        self.delegate.lock().unwrap().server_socket_factory().cloned()
    }

    pub fn set_server_socket_factory(&self, factory: ServerSocketFactory) {
        self.delegate.lock().unwrap().set_server_socket_factory(factory);
    }

    pub fn dispatcher(&self) -> Arc<Mutex<dyn Dispatcher>> {
        self.delegate.lock().unwrap().dispatcher().clone()
    }

    pub fn set_dispatcher(&self, dispatcher: Box<dyn Dispatcher>) {
        // In Kotlin: delegate.dispatcher = value.wrap()
        self.delegate.lock().unwrap().set_dispatcher(dispatcher);
    }

    pub fn port(&self) -> i32 {
        self.before(); // This implicitly starts the delegate.
        self.delegate.lock().unwrap().port()
    }

    pub fn host_name(&self) -> String {
        self.before(); // This implicitly starts the delegate.
        self.delegate.lock().unwrap().host_name()
    }

    pub fn protocol_negotiation_enabled(&self) -> bool {
        self.delegate.lock().unwrap().protocol_negotiation_enabled()
    }

    pub fn set_protocol_negotiation_enabled(&self, enabled: bool) {
        self.delegate.lock().unwrap().set_protocol_negotiation_enabled(enabled);
    }

    pub fn protocols(&self) -> Vec<Protocol> {
        self.delegate.lock().unwrap().protocols()
    }

    pub fn set_protocols(&self, protocols: Vec<Protocol>) {
        self.delegate.lock().unwrap().set_protocols(protocols);
    }

    pub fn before(&self) {
        let mut started = self.started.lock().unwrap();
        if *started {
            return;
        }
        if let Err(e) = self.start(0) {
            panic!("RuntimeException: {}", e);
        }
    }

    pub fn get_port(&self) -> i32 {
        self.port()
    }

    pub fn to_proxy_address(&self) -> SocketAddr {
        self.before();
        self.delegate.lock().unwrap().proxy_address()
    }

    pub fn url(&self, path: &str) -> HttpUrl {
        self.before();
        self.delegate.lock().unwrap().url(path)
    }

    pub fn use_https(&self, ssl_socket_factory: SSLSocketFactory, _tunnel_proxy: bool) {
        self.delegate.lock().unwrap().use_https(ssl_socket_factory);
    }

    pub fn no_client_auth(&self) {
        self.delegate.lock().unwrap().no_client_auth();
    }

    pub fn request_client_auth(&self) {
        self.delegate.lock().unwrap().request_client_auth();
    }

    pub fn require_client_auth(&self) {
        self.delegate.lock().unwrap().require_client_auth();
    }

    pub fn take_request(&self) -> RecordedRequest {
        self.delegate.lock().unwrap().take_request()
    }

    pub fn take_request_with_timeout(&self, timeout: i64, unit: Duration) -> Option<RecordedRequest> {
        self.delegate.lock().unwrap().take_request_with_timeout(timeout, unit)
    }

    pub fn get_request_count(&self) -> i32 {
        self.request_count()
    }

    pub fn enqueue(&self, response: MockResponse) {
        self.delegate.lock().unwrap().enqueue(response);
    }

    pub fn start(&self, port: i32) -> io::Result<()> {
        let mut started = self.started.lock().unwrap();
        *started = true;
        self.delegate.lock().unwrap().start(port)
    }

    pub fn start_with_address(&self, inet_address: IpAddr, port: i32) -> io::Result<()> {
        let mut started = self.started.lock().unwrap();
        *started = true;
        self.delegate.lock().unwrap().start_with_address(inet_address, port)
    }

    pub fn shutdown(&self) -> io::Result<()> {
        self.delegate.lock().unwrap().close()
    }

    pub fn after(&self) {
        if let Err(e) = self.shutdown() {
            // Equivalent to logger.log(Level.WARNING, ...)\n
            eprintln!("WARNING: MockWebServer shutdown failed: {}", e);
        }
    }

    pub fn close(&self) -> io::Result<()> {
        self.delegate.lock().unwrap().close()
    }
}

impl fmt::Debug for MockWebServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MockWebServer({})", self.delegate.lock().unwrap())
    }
}

impl fmt::Display for MockWebServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.delegate.lock().unwrap())
    }
}

impl Drop for MockWebServer {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
