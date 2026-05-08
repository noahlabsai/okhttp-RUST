use std::collections::HashMap;
use std::time::Duration;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::SocketEffect::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::{
    Dispatcher as Dispatcher3, MockResponse as MockResponse3, PushPromise as PushPromise3,
    RecordedRequest as RecordedRequest3,
};
use okio::{Buffer, ByteString};
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;

// Note: The following types (Dispatcher, QueueDispatcher, MockResponse, PushPromise, 
// RecordedRequest, SocketPolicy) are assumed to be defined in the okhttp3.mockwebserver package 
// as per the Kotlin source.

pub trait Dispatcher {
    fn dispatch(&self, request: RecordedRequest) -> MockResponse;
    fn peek(&self) -> MockResponse;
    fn shutdown(&self);
}

pub struct QueueDispatcher {
    pub delegate: Box<dyn Dispatcher3>,
}

impl Dispatcher for QueueDispatcher {
    fn dispatch(&self, request: RecordedRequest) -> MockResponse {
        // Implementation details would be here
        panic!("Not implemented")
    }
    fn peek(&self) -> MockResponse {
        panic!("Not implemented")
    }
    fn shutdown(&self) {
        self.delegate.close();
    }
}

pub struct MockResponse {
    pub web_socket_listener: Option<Box<dyn std::any::Any>>,
    pub body: Option<Vec<u8>>,
    pub push_promises: Vec<PushPromise>,
    pub settings: HashMap<String, String>,
    pub status: i32,
    pub headers: HashMap<String, String>,
    pub trailers: HashMap<String, String>,
    pub socket_policy: SocketPolicy,
    pub throttle_bytes_per_period: i64,
    pub throttle_period_ms: i64,
    pub body_delay_ms: i64,
    pub headers_delay_ms: i64,
    pub http2_error_code: i32,
}

impl MockResponse {
    pub fn get_body(&self) -> Option<&Vec<u8>> {
        self.body.as_ref()
    }
    pub fn get_throttle_period(&self) -> i64 {
        self.throttle_period_ms
    }
    pub fn get_body_delay(&self) -> i64 {
        self.body_delay_ms
    }
    pub fn get_headers_delay(&self) -> i64 {
        self.headers_delay_ms
    }
}

pub struct PushPromise {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub response: MockResponse,
}

pub struct RecordedRequest {
    pub request_line: String,
    pub headers: HashMap<String, String>,
    pub chunk_sizes: Vec<i32>,
    pub body_size: i64,
    pub body: Buffer,
    pub sequence_number: i32,
    pub failure: Option<Box<dyn std::error::Error>>,
    pub method: String,
    pub path: String,
    pub handshake: Option<Box<dyn std::any::Any>>,
    pub request_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SocketPolicy {
    EXPECT_CONTINUE,
    CONTINUE_ALWAYS,
    UPGRADE_TO_SSL_AT_END,
    SHUTDOWN_SERVER_AFTER_RESPONSE,
    KEEP_OPEN,
    DISCONNECT_AT_END,
    DISCONNECT_AT_START,
    DISCONNECT_AFTER_REQUEST,
    DISCONNECT_DURING_REQUEST_BODY,
    DISCONNECT_DURING_RESPONSE_BODY,
    DO_NOT_READ_REQUEST_BODY,
    FAIL_HANDSHAKE,
    SHUTDOWN_INPUT_AT_END,
    SHUTDOWN_OUTPUT_AT_END,
    STALL_SOCKET_AT_START,
    NO_RESPONSE,
    RESET_STREAM_AT_START,
}

impl Default for SocketPolicy {
    fn default() -> Self {
        SocketPolicy::EXPECT_CONTINUE
    }
}

pub const EXPECT_CONTINUE: SocketPolicy = SocketPolicy::EXPECT_CONTINUE;
pub const CONTINUE_ALWAYS: SocketPolicy = SocketPolicy::CONTINUE_ALWAYS;
pub const UPGRADE_TO_SSL_AT_END: SocketPolicy = SocketPolicy::UPGRADE_TO_SSL_AT_END;
pub const SHUTDOWN_SERVER_AFTER_RESPONSE: SocketPolicy = SocketPolicy::SHUTDOWN_SERVER_AFTER_RESPONSE;
pub const KEEP_OPEN: SocketPolicy = SocketPolicy::KEEP_OPEN;
pub const DISCONNECT_AT_END: SocketPolicy = SocketPolicy::DISCONNECT_AT_END;
pub const DISCONNECT_AT_START: SocketPolicy = SocketPolicy::DISCONNECT_AT_START;
pub const DISCONNECT_AFTER_REQUEST: SocketPolicy = SocketPolicy::DISCONNECT_AFTER_REQUEST;
pub const DISCONNECT_DURING_REQUEST_BODY: SocketPolicy = SocketPolicy::DISCONNECT_DURING_REQUEST_BODY;
pub const DISCONNECT_DURING_RESPONSE_BODY: SocketPolicy = SocketPolicy::DISCONNECT_DURING_RESPONSE_BODY;
pub const DO_NOT_READ_REQUEST_BODY: SocketPolicy = SocketPolicy::DO_NOT_READ_REQUEST_BODY;
pub const FAIL_HANDSHAKE: SocketPolicy = SocketPolicy::FAIL_HANDSHAKE;
pub const SHUTDOWN_INPUT_AT_END: SocketPolicy = SocketPolicy::SHUTDOWN_INPUT_AT_END;
pub const SHUTDOWN_OUTPUT_AT_END: SocketPolicy = SocketPolicy::SHUTDOWN_OUTPUT_AT_END;
pub const STALL_SOCKET_AT_START: SocketPolicy = SocketPolicy::STALL_SOCKET_AT_START;
pub const NO_RESPONSE: SocketPolicy = SocketPolicy::NO_RESPONSE;
pub const RESET_STREAM_AT_START: SocketPolicy = SocketPolicy::RESET_STREAM_AT_START;

// --- Translation of Bridge Functions ---

internal fn wrap_dispatcher(dispatcher: Box<dyn Dispatcher>) -> Box<dyn Dispatcher3> {
    // Check if it's a QueueDispatcher (simulating 'is' check via downcasting or pattern)
    // In Rust, we'd typically use an enum or trait. For this bridge:
    
    // This is a simplified representation of the anonymous object in Kotlin
    struct DispatcherWrapper {
        delegate: Box<dyn Dispatcher>,
    }

    impl Dispatcher3 for DispatcherWrapper {
        fn dispatch(&self, request: RecordedRequest3) -> MockResponse3 {
            let unwrapped = unwrap_recorded_request(request);
            let response = self.delegate.dispatch(unwrapped);
            wrap_mock_response(response)
        }

        fn peek(&self) -> MockResponse3 {
            let response = self.delegate.peek();
            wrap_mock_response(response)
        }

        fn close(&self) {
            self.delegate.shutdown();
        }
    }

    Box::new(DispatcherWrapper { delegate: dispatcher })
}

internal fn wrap_mock_response(response: MockResponse) -> MockResponse3 {
    let mut builder = MockResponse3::Builder::new();
    
    if let Some(ref listener) = response.web_socket_listener {
        builder.web_socket_upgrade(listener.clone());
    }

    if let Some(ref body) = response.get_body() {
        builder.body(body.clone());
    }

    for push_promise in &response.push_promises {
        builder.add_push(wrap_push_promise(push_promise));
    }

    builder.settings(response.settings.clone());
    builder.status(response.status);
    builder.headers(response.headers.clone());
    builder.trailers(response.trailers.clone());

    match response.socket_policy {
        SocketPolicy::EXPECT_CONTINUE | SocketPolicy::CONTINUE_ALWAYS => {
            builder.add_100_continue();
        }
        SocketPolicy::UPGRADE_TO_SSL_AT_END => {
            builder.in_tunnel();
        }
        SocketPolicy::SHUTDOWN_SERVER_AFTER_RESPONSE => {
            builder.shutdown_server(true);
        }
        SocketPolicy::KEEP_OPEN => {}
        SocketPolicy::DISCONNECT_AT_END => {
            builder.on_response_end(SocketEffect::ShutdownConnection);
        }
        SocketPolicy::DISCONNECT_AT_START => {
            builder.on_request_start(SocketEffect::CloseSocket {
                close_socket: true,
                shutdown_input: false,
                shutdown_output: false,
            });
        }
        SocketPolicy::DISCONNECT_AFTER_REQUEST => {
            builder.on_response_start(SocketEffect::CloseSocket {
                close_socket: true,
                shutdown_input: false,
                shutdown_output: false,
            });
        }
        SocketPolicy::DISCONNECT_DURING_REQUEST_BODY => {
            builder.on_request_body(SocketEffect::CloseSocket {
                close_socket: true,
                shutdown_input: false,
                shutdown_output: false,
            });
        }
        SocketPolicy::DISCONNECT_DURING_RESPONSE_BODY => {
            builder.on_response_body(SocketEffect::CloseSocket {
                close_socket: true,
                shutdown_input: false,
                shutdown_output: false,
            });
        }
        SocketPolicy::DO_NOT_READ_REQUEST_BODY => {
            builder.do_not_read_request_body();
        }
        SocketPolicy::FAIL_HANDSHAKE => {
            builder.fail_handshake();
        }
        SocketPolicy::SHUTDOWN_INPUT_AT_END => {
            builder.on_response_end(SocketEffect::CloseSocket {
                close_socket: false,
                shutdown_input: true,
                shutdown_output: false,
            });
        }
        SocketPolicy::SHUTDOWN_OUTPUT_AT_END => {
            builder.on_response_end(SocketEffect::CloseSocket {
                close_socket: false,
                shutdown_input: false,
                shutdown_output: true,
            });
        }
        SocketPolicy::STALL_SOCKET_AT_START => {
            builder.on_request_start(SocketEffect::Stall);
        }
        SocketPolicy::NO_RESPONSE => {
            builder.on_response_start(SocketEffect::Stall);
        }
        SocketPolicy::RESET_STREAM_AT_START => {
            builder.on_request_start(SocketEffect::CloseStream {
                http2_error_code: response.http2_error_code,
            });
        }
    }

    builder.throttle_body(
        response.throttle_bytes_per_period,
        Duration::from_millis(response.get_throttle_period() as u64),
    );
    builder.body_delay(Duration::from_millis(response.get_body_delay() as u64));
    builder.headers_delay(Duration::from_millis(response.get_headers_delay() as u64));

    builder.build()
}

fn wrap_push_promise(push: &PushPromise) -> PushPromise3 {
    PushPromise3 {
        method: push.method.clone(),
        path: push.path.clone(),
        headers: push.headers.clone(),
        response: wrap_mock_response(push.response.clone()),
    }
}

internal fn unwrap_recorded_request(request: RecordedRequest3) -> RecordedRequest {
    let body_bytes = request.body.unwrap_or(ByteString::EMPTY);
    let mut buffer = Buffer::new();
    buffer.write(body_bytes);

    RecordedRequest {
        request_line: request.request_line,
        headers: request.headers,
        chunk_sizes: request.chunk_sizes.unwrap_or_else(Vec::new),
        body_size: request.body_size,
        body: buffer,
        sequence_number: request.exchange_index,
        failure: request.failure,
        method: request.method,
        path: request.target,
        handshake: request.handshake,
        request_url: request.url,
    }
}