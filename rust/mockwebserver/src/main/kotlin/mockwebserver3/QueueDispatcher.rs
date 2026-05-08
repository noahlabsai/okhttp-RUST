use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::sync::Condvar;
use log::info;

// Import from the specified path
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Dispatcher;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::mockwebserver_deprecated::src::main::kotlin::okhttp3::mockwebserver::Dispatcher::*;

// Constants for HTTP status codes as per java.net.HttpURLConnection
const HTTP_NOT_FOUND: i32 = 404;
const HTTP_UNAVAILABLE: i32 = 503;

// MockResponse data class equivalent

impl MockResponse {
    pub fn new(code: i32) -> Self {
        Self { code }
    }
}

// RecordedRequest data class equivalent

// A BlockingQueue implementation for Rust to mimic LinkedBlockingQueue
struct BlockingQueue<T> {
    queue: Mutex<VecDeque<T>>,
    condvar: Condvar,
}

impl<T> BlockingQueue<T> {
    fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            condvar: Condvar::new(),
        }
    }

    fn add(&self, item: T) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(item);
        self.condvar.notify_one();
    }

    fn take(&self) -> T {
        let mut queue = self.queue.lock().unwrap();
        while queue.is_empty() {
            queue = self.condvar.wait(queue).unwrap();
        }
        queue.pop_front().unwrap()
    }

    fn peek(&self) -> Option<T> 
    where T: Clone {
        let queue = self.queue.lock().unwrap();
        queue.front().cloned()
    }

    fn clear(&self) {
        let mut queue = self.queue.lock().unwrap();
        queue.clear();
    }
}

// Default dispatcher that processes a script of responses. Populate the script by calling
// [enqueue].
pub struct QueueDispatcher {
    response_queue: BlockingQueue<MockResponse>,
    fail_fast_response: Mutex<Option<MockResponse>>,
}

impl QueueDispatcher {
    pub fn new() -> Self {
        Self {
            response_queue: BlockingQueue::new(),
            fail_fast_response: Mutex::new(None),
        }
    }

    // Enqueued on shutdown to release threads waiting on [dispatch].
    fn dead_letter() -> MockResponse {
        MockResponse::new(HTTP_UNAVAILABLE)
    }

    pub fn enqueue(&self, response: MockResponse) {
        self.response_queue.add(response);
    }

    pub fn clear(&self) {
        self.response_queue.clear();
    }

    pub fn set_fail_fast_bool(&self, fail_fast: bool) {
        let response = if fail_fast {
            Some(MockResponse::new(HTTP_NOT_FOUND))
        } else {
            None
        };
        self.set_fail_fast_response(response);
    }

    pub fn set_fail_fast_response(&self, fail_fast_response: Option<MockResponse>) {
        let mut lock = self.fail_fast_response.lock().unwrap();
        *lock = fail_fast_response;
    }

    pub fn peek(&self) -> Option<MockResponse> {
        // responseQueue.peek() ?: failFastResponse ?: super.peek()
        if let Some(resp) = self.response_queue.peek() {
            return Some(resp);
        }
        if let Some(resp) = self.fail_fast_response.lock().unwrap().clone() {
            return Some(resp);
        }
        // super.peek() is handled by the Dispatcher trait/base logic
        // Since we are implementing the logic here, we return None if not found in queue or fail-fast
        None
    }

    pub fn close(&self) {
        self.response_queue.add(Self::dead_letter());
    }
}

impl Dispatcher for QueueDispatcher {
    fn dispatch(&self, request: RecordedRequest) -> MockResponse {
        // To permit interactive/browser testing, ignore requests for favicons.
        let request_line = &request.request_line;
        if request_line == "GET /favicon.ico HTTP/1.1" {
            info!("served {}", request_line);
            return MockResponse::new(HTTP_NOT_FOUND);
        }

        {
            let fail_fast = self.fail_fast_response.lock().unwrap();
            if fail_fast.is_some() && self.response_queue.peek().is_none() {
                // Fail fast if there's no response queued up.
                return fail_fast.clone().unwrap();
            }
        }

        let result = self.response_queue.take();

        // If take() returned because we're shutting down, then enqueue another dead letter so that any
        // other threads waiting on take() will also return.
        if result == Self::dead_letter() {
            self.response_queue.add(Self::dead_letter());
        }

        result
    }
}