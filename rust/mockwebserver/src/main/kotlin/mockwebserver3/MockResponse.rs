use std::sync::Arc;
use std::time::Duration;
use okio::Buffer;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::WebSocketListener;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::ErrorCode;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Settings;
use crate::mockwebserver3::SocketEffect;
use crate::mockwebserver3::MockResponseBody;
use crate::mockwebserver3::SocketHandler;
use crate::mockwebserver3::PushPromise;
use crate::mockwebserver3::internal::to_mock_response_body;

#[derive(Debug, Clone, PartialEq)]
pub struct MockResponse {
    pub status: String,
    pub headers: Headers,
    pub trailers: Headers,
    pub body: Option<MockResponseBody>,
    pub web_socket_listener: Option<Arc<dyn WebSocketListener>>,
    pub socket_handler: Option<Arc<SocketHandler>>,
    pub in_tunnel: bool,
    pub informational_responses: Vec<MockResponse>,
    pub throttle_bytes_per_period: i64,
    pub throttle_period_nanos: i64,
    pub fail_handshake: bool,
    pub on_request_start: Option<SocketEffect>,
    pub do_not_read_request_body: bool,
    pub on_request_body: Option<SocketEffect>,
    pub on_response_start: Option<SocketEffect>,
    pub on_response_body: Option<SocketEffect>,
    pub on_response_end: Option<SocketEffect>,
    pub shutdown_server: bool,
    pub headers_delay_nanos: i64,
    pub body_delay_nanos: i64,
    pub trailers_delay_nanos: i64,
    pub push_promises: Vec<PushPromise>,
    pub settings: Settings,
}

impl MockResponse {
    pub fn new(code: i32, headers: Headers, body: String) -> Self {
        let mut builder = Self::Builder::new();
        builder.code(code);
        builder.headers(headers);
        builder.body_string(body);
        builder.build()
    }

    pub fn code(&self) -> i32 {
        let status_parts: Vec<&str> = self.status.splitn(3, ' ').collect();
        if status_parts.len() < 2 {
            panic!("Unexpected status: {}", self.status);
        }
        status_parts[1].parse::<i32>().expect("Invalid status code")
    }

    pub fn message(&self) -> String {
        let status_parts: Vec<&str> = self.status.splitn(3, ' ').collect();
        if status_parts.len() < 2 {
            panic!("Unexpected status: {}", self.status);
        }
        status_parts.get(2).unwrap_or(&"").to_string()
    }

    pub fn new_builder(&self) -> Self::Builder {
        Self::Builder::from_response(self)
    }

    pub struct Builder {
        pub in_tunnel: bool,
        informational_responses_: Vec<MockResponse>,
        pub status: String,
        pub code_val: i32,
        headers_: Headers::Builder,
        trailers_: Headers::Builder,
        body_var: Option<MockResponseBody>,
        socket_handler_var: Option<Arc<SocketHandler>>,
        web_socket_listener_var: Option<Arc<dyn WebSocketListener>>,
        pub throttle_bytes_per_period: i64,
        pub throttle_period_nanos: i64,
        pub fail_handshake: bool,
        pub on_request_start: Option<SocketEffect>,
        pub do_not_read_request_body: bool,
        pub on_request_body: Option<SocketEffect>,
        pub on_response_start: Option<SocketEffect>,
        pub on_response_body: Option<SocketEffect>,
        pub on_response_end: Option<SocketEffect>,
        pub shutdown_server: bool,
        pub headers_delay_nanos: i64,
        pub body_delay_nanos: i64,
        pub trailers_delay_nanos: i64,
        push_promises_: Vec<PushPromise>,
        settings_: Settings,
    }

    impl Self::Builder {
        pub fn new() -> Self {
            let mut headers_builder = Headers::Builder::new();
            headers_builder.add("Content-Length", "0");

            Self {
                in_tunnel: false,
                informational_responses_: Vec::new(),
                status: "HTTP/1.1 200 OK".to_string(),
                code_val: 200,
                headers_: headers_builder,
                trailers_: Headers::Builder::new(),
                body_var: None,
                socket_handler_var: None,
                web_socket_listener_var: None,
                throttle_bytes_per_period: i64::MAX,
                throttle_period_nanos: 0,
                fail_handshake: false,
                on_request_start: None,
                do_not_read_request_body: false,
                on_request_body: None,
                on_response_start: None,
                on_response_body: None,
                on_response_end: None,
                shutdown_server: false,
                headers_delay_nanos: 0,
                body_delay_nanos: 0,
                trailers_delay_nanos: 0,
                push_promises_: Vec::new(),
                settings_: Settings::new(),
            }
        }

        pub fn from_response(mock_response: &MockResponse) -> Self {
            let mut settings = Settings::new();
            settings.merge(&mock_response.settings);

            Self {
                in_tunnel: mock_response.in_tunnel,
                informational_responses_: mock_response.informational_responses.clone(),
                status: mock_response.status.clone(),
                code_val: mock_response.code(),
                headers_: mock_response.headers.new_builder(),
                trailers_: mock_response.trailers.new_builder(),
                body_var: mock_response.body.clone(),
                socket_handler_var: mock_response.socket_handler.clone(),
                web_socket_listener_var: mock_response.web_socket_listener.clone(),
                throttle_bytes_per_period: mock_response.throttle_bytes_per_period,
                throttle_period_nanos: mock_response.throttle_period_nanos,
                fail_handshake: mock_response.fail_handshake,
                on_request_start: mock_response.on_request_start.clone(),
                do_not_read_request_body: mock_response.do_not_read_request_body,
                on_request_body: mock_response.on_request_body.clone(),
                on_response_start: mock_response.on_response_start.clone(),
                on_response_body: mock_response.on_response_body.clone(),
                on_response_end: mock_response.on_response_end.clone(),
                shutdown_server: mock_response.shutdown_server,
                headers_delay_nanos: mock_response.headers_delay_nanos,
                body_delay_nanos: mock_response.body_delay_nanos,
                trailers_delay_nanos: mock_response.trailers_delay_nanos,
                push_promises_: mock_response.push_promises.clone(),
                settings_: settings,
            }
        }

        pub fn code(&mut self, code: i32) -> &mut Self {
            self.code_val = code;
            let reason = match code {
                100..=199 => "Informational",
                200..=299 => "OK",
                300..=399 => "Redirection",
                400..=499 => "Client Error",
                500..=599 => "Server Error",
                _ => "Mock Response",
            };
            self.status = format!("HTTP/1.1 {} {}", code, reason);
            self
        }

        pub fn status(&mut self, status: String) -> &mut Self {
            self.status = status;
            self
        }

        pub fn clear_headers(&mut self) -> &mut Self {
            self.headers_ = Headers::Builder::new();
            self
        }

        pub fn add_header_string(&mut self, header: String) -> &mut Self {
            self.headers_.add_line(&header);
            self
        }

        pub fn add_header(&mut self, name: String, value: String) -> &mut Self {
            self.headers_.add(&name, &value);
            self
        }

        pub fn add_header_lenient(&mut self, name: String, value: String) -> &mut Self {
            self.headers_.add_lenient(&name, &value);
            self
        }

        pub fn set_header(&mut self, name: String, value: String) -> &mut Self {
            self.remove_header(name.clone());
            self.add_header(name, value);
            self
        }

        pub fn remove_header(&mut self, name: String) -> &mut Self {
            self.headers_.remove_all(&name);
            self
        }

        pub fn body_buffer(&mut self, body: Buffer) -> &mut Self {
            self.body_mock_response_body(to_mock_response_body(body))
        }

        pub fn body_mock_response_body(&mut self, body: MockResponseBody) -> &mut Self {
            self.set_header("Content-Length".to_string(), body.content_length().to_string());
            self.body_var = Some(body);
            self.socket_handler_var = None;
            self.web_socket_listener_var = None;
            self
        }

        pub fn body_string(&mut self, body: String) -> &mut Self {
            let mut buffer = Buffer::new();
            buffer.write_utf8(&body);
            self.body_buffer(buffer)
        }

        pub fn socket_handler(&mut self, socket_handler: Arc<SocketHandler>) -> &mut Self {
            self.socket_handler_var = Some(socket_handler);
            self.body_var = None;
            self.web_socket_listener_var = None;
            self
        }

        pub fn chunked_body(&mut self, mut body: Buffer, max_chunk_size: i32) -> &mut Self {
            self.remove_header("Content-Length".to_string());
            self.headers_.add("Transfer-encoding", "chunked");

            let mut bytes_out = Buffer::new();
            while !body.is_empty() {
                let chunk_size = std::cmp::min(body.size(), max_chunk_size as i64);
                bytes_out.write_hexadecimal_unsigned_long(chunk_size);
                bytes_out.write_utf8("\r\n");
                
                let mut chunk = Buffer::new();
                body.read_into(&mut chunk, chunk_size);
                bytes_out.write(chunk);
                bytes_out.write_utf8("\r\n");
            }
            bytes_out.write_utf8("0\r\n");
            self.body_mock_response_body(to_mock_response_body(bytes_out))
        }

        pub fn chunked_body_string(&mut self, body: String, max_chunk_size: i32) -> &mut Self {
            let mut buffer = Buffer::new();
            buffer.write_utf8(&body);
            self.chunked_body(buffer, max_chunk_size)
        }

        pub fn headers(&mut self, headers: Headers) -> &mut Self {
            self.headers_ = headers.new_builder();
            self
        }

        pub fn trailers(&mut self, trailers: Headers) -> &mut Self {
            self.trailers_ = trailers.new_builder();
            self
        }

        pub fn fail_handshake(&mut self) -> &mut Self {
            self.fail_handshake = true;
            self
        }

        pub fn on_request_start(&mut self, socket_effect: Option<SocketEffect>) -> &mut Self {
            self.on_request_start = socket_effect;
            self
        }

        pub fn do_not_read_request_body(&mut self) -> &mut Self {
            self.do_not_read_request_body = true;
            self.on_response_end = Some(SocketEffect::close_stream(ErrorCode::NoError.http_code()));
            self
        }

        pub fn on_request_body(&mut self, socket_effect: Option<SocketEffect>) -> &mut Self {
            self.on_request_body = socket_effect;
            self
        }

        pub fn on_response_start(&mut self, socket_effect: Option<SocketEffect>) -> &mut Self {
            self.on_response_start = socket_effect;
            self
        }

        pub fn on_response_body(&mut self, socket_effect: Option<SocketEffect>) -> &mut Self {
            self.on_response_body = socket_effect;
            self
        }

        pub fn on_response_end(&mut self, socket_effect: Option<SocketEffect>) -> &mut Self {
            self.on_response_end = socket_effect;
            self
        }

        pub fn shutdown_server(&mut self, shutdown_server: bool) -> &mut Self {
            self.shutdown_server = shutdown_server;
            self
        }

        pub fn throttle_body(&mut self, bytes_per_period: i64, period: i64, unit: Duration) -> &mut Self {
            self.throttle_bytes_per_period = bytes_per_period;
            self.throttle_period_nanos = unit.as_nanos() as i64;
            self
        }

        pub fn headers_delay(&mut self, delay: i64, unit: Duration) -> &mut Self {
            self.headers_delay_nanos = unit.as_nanos() as i64;
            self
        }

        pub fn body_delay(&mut self, delay: i64, unit: Duration) -> &mut Self {
            self.body_delay_nanos = unit.as_nanos() as i64;
            self
        }

        pub fn trailers_delay(&mut self, delay: i64, unit: Duration) -> &mut Self {
            self.trailers_delay_nanos = unit.as_nanos() as i64;
            self
        }

        pub fn add_push(&mut self, promise: PushPromise) -> &mut Self {
            self.push_promises_.push(promise);
            self
        }

        pub fn settings(&mut self, settings: Settings) -> &mut Self {
            self.settings_.clear();
            self.settings_.merge(&settings);
            self
        }

        pub fn web_socket_upgrade(&mut self, listener: Arc<dyn WebSocketListener>) -> &mut Self {
            self.status = "HTTP/1.1 101 Switching Protocols".to_string();
            self.set_header("Connection".to_string(), "Upgrade".to_string());
            self.set_header("Upgrade".to_string(), "websocket".to_string());
            self.web_socket_listener_var = Some(listener);
            self.body_var = None;
            self.socket_handler_var = None;
            self
        }

        pub fn in_tunnel(&mut self) -> &mut Self {
            self.remove_header("Content-Length".to_string());
            self.in_tunnel = true;
            self
        }

        pub fn add_informational_response(&mut self, response: MockResponse) -> &mut Self {
            self.informational_responses_.push(response);
            self
        }

        pub fn add_100_continue(&mut self) -> &mut Self {
            self.add_informational_response(MockResponse::new(100, Headers::empty().clone(), "".to_string()))
        }

        pub fn build(&self) -> MockResponse {
            let mut settings = Settings::new();
            settings.merge(&self.settings_);

            MockResponse {
                status: self.status.clone(),
                headers: self.headers_.build(),
                trailers: self.trailers_.build(),
                body: self.body_var.clone(),
                web_socket_listener: self.web_socket_listener_var.clone(),
                socket_handler: self.socket_handler_var.clone(),
                in_tunnel: self.in_tunnel,
                informational_responses: self.informational_responses_.clone(),
                throttle_bytes_per_period: self.throttle_bytes_per_period,
                throttle_period_nanos: self.throttle_period_nanos,
                fail_handshake: self.fail_handshake,
                on_request_start: self.on_request_start.clone(),
                do_not_read_request_body: self.do_not_read_request_body,
                on_request_body: self.on_request_body.clone(),
                on_response_start: self.on_response_start.clone(),
                on_response_body: self.on_response_body.clone(),
                on_response_end: self.on_response_end.clone(),
                shutdown_server: self.shutdown_server,
                headers_delay_nanos: self.headers_delay_nanos,
                body_delay_nanos: self.body_delay_nanos,
                trailers_delay_nanos: self.trailers_delay_nanos,
                push_promises: self.push_promises_.clone(),
                settings,
            }
        }
    }
}

impl std::fmt::Display for MockResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.status)
    }
}
