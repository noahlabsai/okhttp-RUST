use std::time::Duration;
use okio::Buffer;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::WebSocketListener;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Settings;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::add_header_lenient;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::settings_gradle::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::PushPromise::*;

// Assuming SocketPolicy and PushPromise are defined in the same package/module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketPolicy {
    KEEP_OPEN,
    CLOSE_AFTER_REQUEST,
    CLOSE_AFTER_RESPONSE,
    DISCONNECT_AT_START,
}

impl Default for SocketPolicy {
    fn default() -> Self {
        SocketPolicy::KEEP_OPEN
    }
}

pub const KEEP_OPEN: SocketPolicy = SocketPolicy::KEEP_OPEN;
pub const CLOSE_AFTER_REQUEST: SocketPolicy = SocketPolicy::CLOSE_AFTER_REQUEST;
pub const CLOSE_AFTER_RESPONSE: SocketPolicy = SocketPolicy::CLOSE_AFTER_RESPONSE;
pub const DISCONNECT_AT_START: SocketPolicy = SocketPolicy::DISCONNECT_AT_START;


#[derive(Debug, Clone)]
pub struct MockResponse {
    pub status: String,
    headers_builder: Headers::Builder,
    trailers_builder: Headers::Builder,
    body: Option<Buffer>,
    pub throttle_bytes_per_period: i64,
    throttle_period_amount: i64,
    throttle_period_unit: DurationUnit,
    pub socket_policy: SocketPolicy,
    pub http2_error_code: i32,
    body_delay_amount: i64,
    body_delay_unit: DurationUnit,
    headers_delay_amount: i64,
    headers_delay_unit: DurationUnit,
    promises: Vec<PushPromise>,
    pub settings: Settings,
    pub web_socket_listener: Option<Box<dyn WebSocketListener>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurationUnit {
    MILLISECONDS,
    SECONDS,
    MINUTES,
    HOURS,
    DAYS,
}

impl Default for DurationUnit {
    fn default() -> Self {
        DurationUnit::MILLISECONDS
    }
}

pub const MILLISECONDS: DurationUnit = DurationUnit::MILLISECONDS;
pub const SECONDS: DurationUnit = DurationUnit::SECONDS;
pub const MINUTES: DurationUnit = DurationUnit::MINUTES;
pub const HOURS: DurationUnit = DurationUnit::HOURS;
pub const DAYS: DurationUnit = DurationUnit::DAYS;

impl DurationUnit {
    fn convert(&self, amount: i64, from: DurationUnit) -> i64 {
        let millis = match from {
            DurationUnit::MILLISECONDS => amount,
            DurationUnit::SECONDS => amount * 1000,
            DurationUnit::MINUTES => amount * 60000,
            DurationUnit::HOURS => amount * 3600000,
            DurationUnit::DAYS => amount * 86400000,
        };
        match self {
            DurationUnit::MILLISECONDS => millis,
            DurationUnit::SECONDS => millis / 1000,
            DurationUnit::MINUTES => millis / 60000,
            DurationUnit::HOURS => millis / 3600000,
            DurationUnit::DAYS => millis / 86400000,
        }
    }
}

impl MockResponse {
    const CHUNKED_BODY_HEADER: &'static str = "Transfer-encoding: chunked";

    pub fn new() -> Self {
        let mut response = Self {
            status: String::new(),
            headers_builder: Headers::Builder::new(),
            trailers_builder: Headers::Builder::new(),
            body: None,
            throttle_bytes_per_period: i64::MAX,
            throttle_period_amount: 1,
            throttle_period_unit: DurationUnit::SECONDS,
            socket_policy: SocketPolicy::KEEP_OPEN,
            http2_error_code: -1,
            body_delay_amount: 0,
            body_delay_unit: DurationUnit::MILLISECONDS,
            headers_delay_amount: 0,
            headers_delay_unit: DurationUnit::MILLISECONDS,
            promises: Vec::new(),
            settings: Settings::new(),
            web_socket_listener: None,
        };
        response.set_response_code(200);
        response.set_header("Content-Length", 0);
        response
    }

    pub fn headers(&self) -> Headers {
        self.headers_builder.build()
    }

    pub fn set_headers(&mut self, headers: Headers) {
        self.headers_builder = headers.new_builder();
    }

    pub fn trailers(&self) -> Headers {
        self.trailers_builder.build()
    }

    pub fn set_trailers(&mut self, trailers: Headers) {
        self.trailers_builder = trailers.new_builder();
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }

    pub fn set_response_code(&mut self, code: i32) -> &mut Self {
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

    pub fn clear_headers(&mut self) -> &mut Self {
        self.headers_builder = Headers::Builder::new();
        self
    }

    pub fn add_header_string(&mut self, header: String) -> &mut Self {
        self.headers_builder.add(header);
        self
    }

    pub fn add_header<V: std::fmt::Display>(&mut self, name: &str, value: V) -> &mut Self {
        self.headers_builder.add(name, &value.to_string());
        self
    }

    pub fn add_header_lenient<V: std::fmt::Display>(&mut self, name: &str, value: V) -> &mut Self {
        add_header_lenient(&mut self.headers_builder, name, &value.to_string());
        self
    }

    pub fn set_header<V: std::fmt::Display>(&mut self, name: &str, value: V) -> &mut Self {
        self.remove_header(name);
        self.add_header(name, value);
        self
    }

    pub fn remove_header(&mut self, name: &str) -> &mut Self {
        self.headers_builder.remove_all(name);
        self
    }

    pub fn get_body(&self) -> Option<Buffer> {
        self.body.as_ref().map(|b| b.clone())
    }

    pub fn set_body_buffer(&mut self, body: Buffer) -> &mut Self {
        self.set_header("Content-Length", body.size());
        self.body = Some(body.clone());
        self
    }

    pub fn set_body_string(&mut self, body: String) -> &mut Self {
        let mut buffer = Buffer::new();
        buffer.write_utf8(&body);
        self.set_body_buffer(buffer)
    }

    pub fn set_chunked_body_buffer(&mut self, body: Buffer, max_chunk_size: i32) -> &mut Self {
        self.remove_header("Content-Length");
        self.headers_builder.add(Self::CHUNKED_BODY_HEADER);

        let mut bytes_out = Buffer::new();
        let mut body_cursor = body;
        while !body_cursor.is_empty() {
            let chunk_size = std::cmp::min(body_cursor.size(), max_chunk_size as i64);
            bytes_out.write_hexadecimal_unsigned_long(chunk_size);
            bytes_out.write_utf8("\r\n");
            
            // In okio-rust, we read from the buffer
            let chunk = body_cursor.read_bytes(chunk_size);
            bytes_out.write(&chunk);
            bytes_out.write_utf8("\r\n");
        }
        bytes_out.write_utf8("0\r\n");
        self.body = Some(bytes_out);
        self
    }

    pub fn set_chunked_body_string(&mut self, body: String, max_chunk_size: i32) -> &mut Self {
        let mut buffer = Buffer::new();
        buffer.write_utf8(&body);
        self.set_chunked_body_buffer(buffer, max_chunk_size)
    }

    pub fn set_socket_policy(&mut self, socket_policy: SocketPolicy) -> &mut Self {
        self.socket_policy = socket_policy;
        self
    }

    pub fn set_http2_error_code(&mut self, http2_error_code: i32) -> &mut Self {
        self.http2_error_code = http2_error_code;
        self
    }

    pub fn throttle_body(&mut self, bytes_per_period: i64, period: i64, unit: DurationUnit) -> &mut Self {
        self.throttle_bytes_per_period = bytes_per_period;
        self.throttle_period_amount = period;
        self.throttle_period_unit = unit;
        self
    }

    pub fn get_throttle_period(&self, unit: DurationUnit) -> i64 {
        unit.convert(self.throttle_period_amount, self.throttle_period_unit)
    }

    pub fn set_body_delay(&mut self, delay: i64, unit: DurationUnit) -> &mut Self {
        self.body_delay_amount = delay;
        self.body_delay_unit = unit;
        self
    }

    pub fn get_body_delay(&self, unit: DurationUnit) -> i64 {
        unit.convert(self.body_delay_amount, self.body_delay_unit)
    }

    pub fn set_headers_delay(&mut self, delay: i64, unit: DurationUnit) -> &mut Self {
        self.headers_delay_amount = delay;
        self.headers_delay_unit = unit;
        self
    }

    pub fn get_headers_delay(&self, unit: DurationUnit) -> i64 {
        unit.convert(self.headers_delay_amount, self.headers_delay_unit)
    }

    pub fn with_push(&mut self, promise: PushPromise) -> &mut Self {
        self.promises.push(promise);
        self
    }

    pub fn with_settings(&mut self, settings: Settings) -> &mut Self {
        self.settings = settings;
        self
    }

    pub fn with_web_socket_upgrade(&mut self, listener: Box<dyn WebSocketListener>) -> &mut Self {
        self.status = "HTTP/1.1 101 Switching Protocols".to_string();
        self.set_header("Connection", "Upgrade");
        self.set_header("Upgrade", "websocket");
        self.body = None;
        self.web_socket_listener = Some(listener);
        self
    }

    pub fn push_promises(&self) -> &[PushPromise] {
        &self.promises
    }
}

impl std::fmt::Display for MockResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.status)
    }
}

impl Clone for MockResponse {
    fn clone(&self) -> Self {
        Self {
            status: self.status.clone(),
            headers_builder: self.headers.new_builder(),
            trailers_builder: self.trailers().new_builder(),
            body: self.body.as_ref().map(|b| b.clone()),
            throttle_bytes_per_period: self.throttle_bytes_per_period,
            throttle_period_amount: self.throttle_period_amount,
            throttle_period_unit: self.throttle_period_unit,
            socket_policy: self.socket_policy,
            http2_error_code: self.http2_error_code,
            body_delay_amount: self.body_delay_amount,
            body_delay_unit: self.body_delay_unit,
            headers_delay_amount: self.headers_delay_amount,
            headers_delay_unit: self.headers_delay_unit,
            promises: self.promises.clone(),
            settings: self.settings.clone(),
            web_socket_listener: None, // WebSocketListener is a trait, cannot be cloned easily
        }
    }
}