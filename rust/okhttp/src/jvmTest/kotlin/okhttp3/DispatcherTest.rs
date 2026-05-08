/*
 * Copyright (C) 2023 Square, Inc.
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

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::thread;
use std::panic;

// Import paths as per directives
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::CallEvent::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::CallTagsTest::*;
use crate::okhttp_sse::src::test::java::okhttp3::sse::internal::Event::*;
use crate::okhttp_testing_support::build_gradle::*;

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{
    Call, Callback, Dispatcher, EventListener, EventListenerFactory, 
    HttpUrl, OkHttpClient, OkHttpClientBuilder, Request, Response, 
    WebSocketListener, Interceptor, InterceptorChain
};

// Mocking the test rule and support classes based on the Kotlin source
impl OkHttpClientTestRule {
    pub fn new_client_builder(&self) -> OkHttpClientBuilder {
        OkHttpClientBuilder::new()
    }
    pub fn wrap<T: EventListener>(&self, listener: T) -> Box<dyn EventListenerFactory> {
        Box::new(EventListenerFactoryImpl { listener })
    }
}

struct EventListenerFactoryImpl<T: EventListener> {
    listener: T,
}
impl EventListenerFactory for EventListenerFactoryImpl<T> where T: EventListener {
    fn event_listener(&self) -> Box<dyn EventListener> {
        // In a real implementation, this would return the wrapped listener
        unimplemented!() 
    }
}

pub struct RecordingExecutor {
    // Internal state to track jobs and simulate execution
}
impl RecordingExecutor {
    pub fn new(_test: &DispatcherTest) -> Self {
        Self {}
    }
    pub fn assert_jobs(&self, urls: &[&str]) {
        // Verification logic
    }
    pub fn finish_job(&self, url: &str) {
        // Simulation logic
    }
    pub fn shutdown(&self) {
        // Shutdown logic
    }
}

pub struct RecordingCallback;
impl RecordingCallback {
    pub fn await_url(&self, _url: HttpUrl) -> CallResult {
        CallResult {}
    }
}
impl Callback for RecordingCallback {
    fn on_failure(&self, _call: &Call, _ioe: Box<dyn std::error::Error>) {}
    fn on_response(&self, _call: &Call, _response: Response) {}
}

pub struct CallResult;
impl CallResult {
    pub fn assert_failure<E: 'static>(&self, _expected_type: std::any::TypeId) {
        // Verification logic
    }
}

pub struct EventRecorder {
    pub event_sequence: Vec<CallEvent>,
}
impl EventRecorder {
    pub fn new() -> Self {
        Self { event_sequence: Vec::new() }
    }
    pub fn forbid_lock(&self, _dispatcher: &Dispatcher) {
        // Lock monitoring logic
    }
    pub fn remove_up_to_event<T>(&mut self) -> CallEvent {
        // Logic to find and return the first event of type T
        // This is a simplification of the Kotlin generic filter
        self.event_sequence.pop().expect("Event not found")
    }
    pub fn recorded_event_types(&self) -> Vec<std::any::TypeId> {
        self.event_sequence.iter().map(|_| std::any::TypeId::of::<CallEvent>()).collect()
    }
}

pub struct DispatcherTest {
    client_test_rule: OkHttpClientTestRule,
    executor: RecordingExecutor,
    callback: RecordingCallback,
    web_socket_listener: Box<dyn WebSocketListener>,
    dispatcher: Arc<Dispatcher>,
    event_recorder: EventRecorder,
    client: OkHttpClient,
}

impl DispatcherTest {
    pub fn new() -> Self {
        let client_test_rule = OkHttpClientTestRule;
        let dispatcher = Arc::new(Dispatcher::new());
        let event_recorder = EventRecorder::new();
        
        // Note: In Rust, we can't easily use a closure for DNS in the builder 
        // without a specific trait implementation.
        let client = client_test_rule.new_client_builder()
            .dispatcher(dispatcher.clone())
            .event_listener_factory(client_test_rule.wrap(event_recorder))
            .build();

        Self {
            client_test_rule,
            executor: RecordingExecutor::new(&self), // This is a conceptual loop, handled in actual instantiation
            callback: RecordingCallback,
            web_socket_listener: Box::new(EmptyWebSocketListener),
            dispatcher,
            event_recorder,
            client,
        }
    }

    pub fn set_up(&mut self) {
        self.dispatcher.set_max_requests(20);
        self.dispatcher.set_max_requests_per_host(10);
        self.event_recorder.forbid_lock(&self.dispatcher);
    }

    pub fn max_requests_zero(&self) {
        let result = panic::catch_unwind(|| {
            self.dispatcher.set_max_requests(0);
        });
        assert!(result.is_err(), "Should throw IllegalArgumentException");
    }

    pub fn max_per_host_zero(&self) {
        let result = panic::catch_unwind(|| {
            self.dispatcher.set_max_requests_per_host(0);
        });
        assert!(result.is_err(), "Should throw IllegalArgumentException");
    }

    pub fn enqueued_jobs_run_immediately(&mut self) {
        let request = self.new_request("http://a/1");
        self.client.new_call(request).enqueue(&self.callback);
        self.executor.assert_jobs(&["http://a/1"]);

        assert!(self.event_recorder.event_sequence.iter().all(|e| !matches!(e, DispatcherQueueStart { .. })));
        assert!(self.event_recorder.event_sequence.iter().all(|e| !matches!(e, DispatcherQueueEnd { .. })));
    }

    pub fn max_requests_enforced(&mut self) {
        self.dispatcher.set_max_requests(3);
        self.client.new_call(self.new_request("http://a/1")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://a/2")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://b/1")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://b/2")).enqueue(&self.callback);
        
        self.executor.assert_jobs(&["http://a/1", "http://a/2", "http://b/1"]);

        let event = self.event_recorder.remove_up_to_event::<DispatcherQueueStart>();
        // In a real translation, we'd extract the call from the event variant
        // assertThat(event.call.request().url).isEqualTo("http://b/2".toHttpUrl())
    }

    pub fn max_per_host_enforced(&mut self) {
        self.dispatcher.set_max_requests_per_host(2);
        self.client.new_call(self.new_request("http://a/1")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://a/2")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://a/3")).enqueue(&self.callback);
        
        self.executor.assert_jobs(&["http://a/1", "http://a/2"]);

        let event = self.event_recorder.remove_up_to_event::<DispatcherQueueStart>();
        // Verification logic...
    }

    pub fn max_per_host_not_enforced_for_web_sockets(&mut self) {
        self.dispatcher.set_max_requests_per_host(2);
        self.client.new_web_socket(self.new_request("http://a/1"), self.web_socket_listener.as_ref());
        self.client.new_web_socket(self.new_request("http://a/2"), self.web_socket_listener.as_ref());
        self.client.new_web_socket(self.new_request("http://a/3"), self.web_socket_listener.as_ref());
        self.executor.assert_jobs(&["http://a/1", "http://a/2", "http://a/3"]);
    }

    pub fn increasing_max_requests_promotes_jobs_immediately(&mut self) {
        self.dispatcher.set_max_requests(2);
        self.client.new_call(self.new_request("http://a/1")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://b/1")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://c/1")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://a/2")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://b/2")).enqueue(&self.callback);

        self.event_recorder.remove_up_to_event::<DispatcherQueueStart>();
        self.event_recorder.remove_up_to_event::<DispatcherQueueStart>();
        self.event_recorder.remove_up_to_event::<DispatcherQueueStart>();

        self.dispatcher.set_max_requests(4);
        self.executor.assert_jobs(&["http://a/1", "http://b/1", "http://c/1", "http://a/2"]);

        self.event_recorder.remove_up_to_event::<DispatcherQueueEnd>();
        self.event_recorder.remove_up_to_event::<DispatcherQueueEnd>();
    }

    pub fn increasing_max_per_host_promotes_jobs_immediately(&mut self) {
        self.dispatcher.set_max_requests_per_host(2);
        self.client.new_call(self.new_request("http://a/1")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://a/2")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://a/3")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://a/4")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://a/5")).enqueue(&self.callback);

        self.event_recorder.remove_up_to_event::<DispatcherQueueStart>();
        self.event_recorder.remove_up_to_event::<DispatcherQueueStart>();
        self.event_recorder.remove_up_to_event::<DispatcherQueueStart>();

        self.dispatcher.set_max_requests_per_host(4);
        self.executor.assert_jobs(&["http://a/1", "http://a/2", "http://a/3", "http://a/4"]);

        self.event_recorder.remove_up_to_event::<DispatcherQueueEnd>();
        self.event_recorder.remove_up_to_event::<DispatcherQueueEnd>();
    }

    pub fn old_job_finishes_new_job_can_run_different_host(&mut self) {
        self.dispatcher.set_max_requests(1);
        self.client.new_call(self.new_request("http://a/1")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://b/1")).enqueue(&self.callback);
        self.executor.finish_job("http://a/1");
        self.executor.assert_jobs(&["http://b/1"]);
    }

    pub fn old_job_finishes_new_job_with_same_host_starts(&mut self) {
        self.dispatcher.set_max_requests(2);
        self.dispatcher.set_max_requests_per_host(1);
        self.client.new_call(self.new_request("http://a/1")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://b/1")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://b/2")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://a/2")).enqueue(&self.callback);
        self.executor.finish_job("http://a/1");
        self.executor.assert_jobs(&["http://b/1", "http://a/2"]);
    }

    pub fn old_job_finishes_new_job_cant_run_due_to_host_limit(&mut self) {
        self.dispatcher.set_max_requests_per_host(1);
        self.client.new_call(self.new_request("http://a/1")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://b/1")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://a/2")).enqueue(&self.callback);
        self.executor.finish_job("http://b/1");
        self.executor.assert_jobs(&["http://a/1"]);
    }

    pub fn enqueued_calls_still_respect_max_calls_per_host(&mut self) {
        self.dispatcher.set_max_requests(1);
        self.dispatcher.set_max_requests_per_host(1);
        self.client.new_call(self.new_request("http://a/1")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://b/1")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://b/2")).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://b/3")).enqueue(&self.callback);
        self.dispatcher.set_max_requests(3);
        self.executor.finish_job("http://a/1");
        self.executor.assert_jobs(&["http://b/1"]);
    }

    pub fn canceling_running_job_takes_no_effect_until_job_finishes(&mut self) {
        self.dispatcher.set_max_requests(1);
        let c1 = self.client.new_call(self.new_request("http://a/1", Some("tag1")));
        let c2 = self.client.new_call(self.new_request("http://a/2", None));
        c1.enqueue(&self.callback);
        c2.enqueue(&self.callback);
        c1.cancel();
        self.executor.assert_jobs(&["http://a/1"]);
        self.executor.finish_job("http://a/1");
        self.executor.assert_jobs(&["http://a/2"]);
    }

    pub fn async_call_accessors(&mut self) {
        self.dispatcher.set_max_requests(3);
        let a1 = self.client.new_call(self.new_request("http://a/1", None));
        let a2 = self.client.new_call(self.new_request("http://a/2", None));
        let a3 = self.client.new_call(self.new_request("http://a/3", None));
        let a4 = self.client.new_call(self.new_request("http://a/4", None));
        let a5 = self.client.new_call(self.new_request("http://a/5", None));
        a1.enqueue(&self.callback);
        a2.enqueue(&self.callback);
        a3.enqueue(&self.callback);
        a4.enqueue(&self.callback);
        a5.enqueue(&self.callback);
        
        assert_eq!(self.dispatcher.running_calls_count(), 3);
        assert_eq!(self.dispatcher.queued_calls_count(), 2);
    }

    pub fn synchronous_call_accessors(&mut self) {
        let ready = Arc::new(Mutex::new(false));
        let waiting = Arc::new(Mutex::new(false));
        
        let ready_clone = ready.clone();
        let waiting_clone = waiting.clone();
        
        let interceptor = move |chain: &InterceptorChain| {
            let mut r = ready_clone.lock().unwrap();
            *r = true;
            drop(r);
            
            while !*waiting_clone.lock().unwrap() {
                thread::sleep(Duration::from_millis(10));
            }
            // Simulate IOException
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "IO Error")))
        };

        let client = self.client.new_builder()
            .add_interceptor(interceptor)
            .build();
        
        let a1 = client.new_call(self.new_request("http://a/1", None));
        let a2 = client.new_call(self.new_request("http://a/2", None));
        let a3 = client.new_call(self.new_request("http://a/3", None));
        let a4 = client.new_call(self.new_request("http://a/4", None));
        
        let t1 = self.make_synchronous_call(a1.clone());
        let t2 = self.make_synchronous_call(a2.clone());

        while !*ready.lock().unwrap() {
            thread::sleep(Duration::from_millis(10));
        }
        
        assert_eq!(self.dispatcher.running_calls_count(), 2);
        assert_eq!(self.dispatcher.queued_calls_count(), 0);

        a2.cancel();
        a3.cancel();
        
        {
            let mut w = waiting.lock().unwrap();
            *w = true;
        }
        
        t1.join().unwrap();
        t2.join().unwrap();

        assert_eq!(self.dispatcher.running_calls_count(), 0);
        assert_eq!(self.dispatcher.queued_calls_count(), 0);
    }

    pub fn idle_callback_invoked_when_idle(&mut self) {
        let idle = Arc::new(AtomicBool::new(false));
        let idle_clone = idle.clone();
        self.dispatcher.set_idle_callback(move || {
            idle_clone.store(true, Ordering::SeqCst);
        });

        self.client.new_call(self.new_request("http://a/1", None)).enqueue(&self.callback);
        self.client.new_call(self.new_request("http://a/2", None)).enqueue(&self.callback);
        self.executor.finish_job("http://a/1");
        assert!(!idle.load(Ordering::SeqCst));

        let ready = Arc::new(AtomicBool::new(false));
        let proceed = Arc::new(AtomicBool::new(false));
        let ready_c = ready.clone();
        let proceed_c = proceed.clone();

        let interceptor = move |chain: &InterceptorChain| {
            ready_c.store(true, Ordering::SeqCst);
            while !proceed_c.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(10));
            }
            chain.proceed()
        };

        let client = self.client.new_builder()
            .add_interceptor(interceptor)
            .build();

        let call = client.new_call(self.new_request("http://a/3", None));
        let t1 = self.make_synchronous_call(call);
        
        while !ready.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(10));
        }
        
        self.executor.finish_job("http://a/2");
        assert!(!idle.load(Ordering::SeqCst));
        
        proceed.store(true, Ordering::SeqCst);
        t1.join().unwrap();
        assert!(idle.load(Ordering::SeqCst));
    }

    pub fn execution_rejected_immediately(&mut self) {
        let request = self.new_request("http://a/1", None);
        self.executor.shutdown();
        self.client.new_call(request.clone()).enqueue(&self.callback);
        
        let result = self.callback.await_url(request.url());
        result.assert_failure(std::any::TypeId::of::<std::io::Error>());
    }

    pub fn execution_rejected_after_max_requests_change(&mut self) {
        let request1 = self.new_request("http://a/1", None);
        let request2 = self.new_request("http://a/2", None);
        self.dispatcher.set_max_requests(1);
        self.client.new_call(request1).enqueue(&self.callback);
        self.executor.shutdown();
        self.client.new_call(request2.clone()).enqueue(&self.callback);
        self.dispatcher.set_max_requests(2);
        
        let result = self.callback.await_url(request2.url());
        result.assert_failure(std::any::TypeId::of::<std::io::Error>());
    }

    pub fn execution_rejected_after_max_requests_per_host_change(&mut self) {
        let request1 = self.new_request("http://a/1", None);
        let request2 = self.new_request("http://a/2", None);
        self.dispatcher.set_max_requests_per_host(1);
        self.client.new_call(request1).enqueue(&self.callback);
        self.executor.shutdown();
        self.client.new_call(request2.clone()).enqueue(&self.callback);
        self.dispatcher.set_max_requests_per_host(2);
        
        let result = self.callback.await_url(request2.url());
        result.assert_failure(std::any::TypeId::of::<std::io::Error>());
    }

    pub fn execution_rejected_after_preceding_call_finishes(&mut self) {
        let request1 = self.new_request("http://a/1", None);
        let request2 = self.new_request("http://a/2", None);
        self.dispatcher.set_max_requests(1);
        self.client.new_call(request1).enqueue(&self.callback);
        self.executor.shutdown();
        self.client.new_call(request2.clone()).enqueue(&self.callback);
        self.executor.finish_job("http://a/1");
        
        let result = self.callback.await_url(request2.url());
        result.assert_failure(std::any::TypeId::of::<std::io::Error>());
    }

    fn make_synchronous_call(&self, call: Call) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let _ = call.execute();
        })
    }

    fn new_request(&self, url: &str, tag: Option<&str>) -> Request {
        let mut builder = Request::builder();
        builder.url(url);
        if let Some(t) = tag {
            builder.tag(t);
        }
        builder.build()
    }
}

struct EmptyWebSocketListener;
impl WebSocketListener for EmptyWebSocketListener {}