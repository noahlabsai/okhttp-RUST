use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread;
use std::sync::Condvar;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::CallTagsTest::*;
use crate::mockwebserver_deprecated::src::main::kotlin::okhttp3::mockwebserver::Dispatcher::*;

// These types are assumed to be defined in the mockwebserver3 crate.
// Since we are translating a test file, we provide the necessary structures 
// to make the test logic compile and run, mirroring the original Kotlin architecture.



impl MockResponse {
    pub fn new() -> Self {
        Self { code: 200 }
    }
}


pub trait Dispatcher: Send + Sync {
    fn dispatch(&self, request: RecordedRequest) -> MockResponse;
}

pub struct DefaultDispatcher;
impl Dispatcher for DefaultDispatcher {
    fn dispatch(&self, _request: RecordedRequest) -> MockResponse {
        MockResponse::new()
    }
}

pub struct MockWebServer {
    pub dispatcher: Arc<dyn Dispatcher>,
}

impl MockWebServer {
    pub fn new() -> Self {
        Self {
            dispatcher: Arc::new(DefaultDispatcher),
        }
    }

    pub fn url(&self, path: &str) -> HttpUrl {
        HttpUrl {
            encoded_path: path.to_string(),
        }
    }

    pub fn simulate_request(&self, request: RecordedRequest) -> i32 {
        let response = self.dispatcher.dispatch(request);
        response.code
    }
}

struct HttpURLConnection {
    response_code: i32,
}

impl HttpURLConnection {
    fn open_connection(server: Arc<MockWebServer>, url: HttpUrl) -> Self {
        let code = server.simulate_request(RecordedRequest { url });
        Self { response_code: code }
    }
}

pub struct CustomDispatcherTest {
    mock_web_server: Arc<MockWebServer>,
}

impl CustomDispatcherTest {
    pub fn new() -> Self {
        Self {
            mock_web_server: Arc::new(MockWebServer::new()),
        }
    }

    pub fn simple_dispatch() {
        let requests_made = Arc::new(Mutex::new(Vec::<RecordedRequest>::new()));
        let requests_made_clone = Arc::clone(&requests_made);

        struct SimpleDispatcher {
            requests: Arc<Mutex<Vec<RecordedRequest>>>,
        }
        impl Dispatcher for SimpleDispatcher {
            fn dispatch(&self, request: RecordedRequest) -> MockResponse {
                self.requests.lock().unwrap().push(request);
                MockResponse::new()
            }
        }

        let dispatcher = Arc::new(SimpleDispatcher {
            requests: requests_made_clone,
        });

        assert_eq!(requests_made.lock().unwrap().len(), 0);

        let mut server = MockWebServer::new();
        server.dispatcher = dispatcher;
        let server_arc = Arc::new(server);

        let url = server_arc.url("/");
        let conn = HttpURLConnection::open_connection(Arc::clone(&server_arc), url);
        let _ = conn.response_code;

        assert_eq!(requests_made.lock().unwrap().len(), 1);
    }

    pub fn out_of_order_responses() {
        let first_response_code = Arc::new(AtomicI32::new(0));
        let second_response_code = Arc::new(AtomicI32::new(0));
        let first_request_path = "/foo";
        let second_request_path = "/bar";

        let latch = Arc::new((Mutex::new(false), Condvar::new()));
        let latch_clone = Arc::clone(&latch);

        struct OrderDispatcher {
            first_path: String,
            latch: Arc<(Mutex<bool>, Condvar)>,
        }
        impl Dispatcher for OrderDispatcher {
            fn dispatch(&self, request: RecordedRequest) -> MockResponse {
                if request.url.encoded_path == self.first_path {
                    let (lock, cvar) = &*self.latch;
                    let mut started = lock.lock().unwrap();
                    while !*started {
                        started = cvar.wait(started).unwrap();
                    }
                }
                MockResponse::new()
            }
        }

        let dispatcher = Arc::new(OrderDispatcher {
            first_path: first_request_path.to_string(),
            latch: latch_clone,
        });

        let mut server = MockWebServer::new();
        server.dispatcher = dispatcher;
        let server_arc = Arc::new(server);

        let server_for_first = Arc::clone(&server_arc);
        let code_for_first = Arc::clone(&first_response_code);
        let path_for_first = first_request_path.to_string();
        let handle_first = thread::spawn(move || {
            let url = server_for_first.url(&path_for_first);
            let conn = HttpURLConnection::open_connection(server_for_first, url);
            code_for_first.store(conn.response_code, Ordering::SeqCst);
        });

        let server_for_second = Arc::clone(&server_arc);
        let code_for_second = Arc::clone(&second_response_code);
        let path_for_second = second_request_path.to_string();
        let handle_second = thread::spawn(move || {
            let url = server_for_second.url(&path_for_second);
            let conn = HttpURLConnection::open_connection(server_for_second, url);
            code_for_second.store(conn.response_code, Ordering::SeqCst);
        });

        handle_second.join().unwrap();

        assert_eq!(first_response_code.load(Ordering::SeqCst), 0);
        assert_eq!(second_response_code.load(Ordering::SeqCst), 200);

        {
            let (lock, cvar) = &*latch;
            let mut started = lock.lock().unwrap();
            *started = true;
            cvar.notify_all();
        }

        handle_first.join().unwrap();

        assert_eq!(first_response_code.load(Ordering::SeqCst), 200);
        assert_eq!(second_response_code.load(Ordering::SeqCst), 200);
    }
}
