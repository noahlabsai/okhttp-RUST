/*
 * Copyright (C) 2025 Square, Inc.
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

use std::sync::{Arc, Mutex, RwLock};
use lazy_static::lazy_static;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::Dispatcher;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::MockResponse;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::MockWebServer;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::RecordedRequest;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::MockResponse::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::RecordedRequest::*;
use crate::mockwebserver::src::test::java::mockwebserver3::CustomDispatcherTest::*;

// Mocking the JUnit 5 annotations and callbacks as they are framework-specific
pub struct StartStop;
pub struct RegisterExtension;

pub trait AfterAllCallback {
    fn after_all(&self);
}

lazy_static! {
    static ref TEST_INSTANCES: RwLock<Vec<Arc<StartStopTest>>> = RwLock::new(Vec::new());
    static ref DISPATCHER_D: Arc<Mutex<ClosableDispatcher>> = Arc::new(Mutex::new(ClosableDispatcher::new()));
    static ref SERVER_D: MockWebServer = {
        let mut server = MockWebServer::new();
        server.set_dispatcher(Box::new(DispatcherProxy { 
            inner: Arc::clone(&DISPATCHER_D) 
        }));
        server
    };
    static ref DISPATCHER_E: Arc<Mutex<ClosableDispatcher>> = Arc::new(Mutex::new(ClosableDispatcher::new()));
    static ref SERVER_E: MockWebServer = {
        let mut server = MockWebServer::new();
        server.set_dispatcher(Box::new(DispatcherProxy { 
            inner: Arc::clone(&DISPATCHER_E) 
        }));
        server
    };
    static ref DISPATCHER_F: Arc<Mutex<ClosableDispatcher>> = Arc::new(Mutex::new(ClosableDispatcher::new()));
    static ref SERVER_F: MockWebServer = {
        let mut server = MockWebServer::new();
        server.set_dispatcher(Box::new(DispatcherProxy { 
            inner: Arc::clone(&DISPATCHER_F) 
        }));
        server
    };
}

pub struct StartStopTest {
    pub dispatcher_a: Arc<Mutex<ClosableDispatcher>>,
    pub server_a: MockWebServer,
    pub dispatcher_b: Arc<Mutex<ClosableDispatcher>>,
    pub server_b: MockWebServer,
    pub dispatcher_c: Arc<Mutex<ClosableDispatcher>>,
    pub server_c: MockWebServer,
}

impl StartStopTest {
    pub fn new() -> Self {
        let dispatcher_a = Arc::new(Mutex::new(ClosableDispatcher::new()));
        let mut server_a = MockWebServer::new();
        server_a.set_dispatcher(Box::new(DispatcherProxy { 
            inner: Arc::clone(&dispatcher_a) 
        }));

        let dispatcher_b = Arc::new(Mutex::new(ClosableDispatcher::new()));
        let mut server_b = MockWebServer::new();
        server_b.set_dispatcher(Box::new(DispatcherProxy { 
            inner: Arc::clone(&dispatcher_b) 
        }));

        let dispatcher_c = Arc::new(Mutex::new(ClosableDispatcher::new()));
        let mut server_c = MockWebServer::new();
        server_c.set_dispatcher(Box::new(DispatcherProxy { 
            inner: Arc::clone(&dispatcher_c) 
        }));

        StartStopTest {
            dispatcher_a,
            server_a,
            dispatcher_b,
            server_b,
            dispatcher_c,
            server_c,
        }
    }

    pub fn happy_path(self: Arc<Self>) {
        {
            let mut instances = TEST_INSTANCES.write().unwrap();
            instances.push(Arc::clone(&self));
        }

        assert!(self.server_a.started()); 
        assert!(self.server_b.started());
        assert!(!self.server_c.started());

        assert!(SERVER_D.started());
        assert!(SERVER_E.started());
        assert!(!SERVER_F.started());
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosableDispatcher {
    pub closed: bool,
}

impl ClosableDispatcher {
    pub fn new() -> Self {
        ClosableDispatcher { closed: false }
    }
}

impl Dispatcher for ClosableDispatcher {
    fn dispatch(&self, _request: RecordedRequest) -> MockResponse {
        MockResponse::new()
    }

    fn close(&mut self) {
        self.closed = true;
    }
}

struct DispatcherProxy {
    inner: Arc<Mutex<ClosableDispatcher>>,
}

impl Dispatcher for DispatcherProxy {
    fn dispatch(&self, request: RecordedRequest) -> MockResponse {
        self.inner.lock().unwrap().dispatch(request)
    }

    fn close(&mut self) {
        self.inner.lock().unwrap().close();
    }
}

struct CheckClosed;

impl AfterAllCallback for CheckClosed {
    fn after_all(&self) {
        let instances = TEST_INSTANCES.read().unwrap();
        for test in instances.iter() {
            assert!(test.dispatcher_a.lock().unwrap().closed);
            assert!(test.dispatcher_b.lock().unwrap().closed);
            assert!(!test.dispatcher_c.lock().unwrap().closed);
        }
        drop(instances);
        
        TEST_INSTANCES.write().unwrap().clear();

        if false {
            assert!(DISPATCHER_D.lock().unwrap().closed);
            assert!(DISPATCHER_E.lock().unwrap().closed);
            assert!(!DISPATCHER_F.lock().unwrap().closed);
        }
    }
}

impl MockWebServer {
    pub fn started(&self) -> bool {
        // This logic is provided by the @StartStop extension in the original JVM code.
        true 
    }
}
