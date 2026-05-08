use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::cache::FaultHidingSink::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Reader::*;
    Protocol, Request, Response, ResponseBody,
};

// Mocking Okio types as they are dependencies not fully provided in the snippet
// but required for the logic to compile and behave as the Kotlin source.
#[derive(Debug, Clone, PartialEq)]
pub struct ByteString(pub Vec<u8>);

impl ByteString {
    pub const EMPTY: ByteString = ByteString(Vec::new());
}


impl Buffer {
    pub fn new() -> Self {
        Buffer { data: Vec::new() }
    }

    pub fn write_utf8(&mut self, s: &str) -> &mut Self {
        self.data.extend_from_slice(s.as_bytes());
        self
    }

    pub fn read(&mut self, sink: &mut Buffer, byte_count: i64) -> i64 {
        let count = std::cmp::min(byte_count as usize, self.data.len());
        let drained: Vec<u8> = self.data.drain(0..count).collect();
        sink.data.extend(drained);
        count as i64
    }

    pub fn string(&self) -> String {
        String::from_utf8_lossy(&self.data).into_owned()
    }
}

pub struct Timeout;
impl Timeout {
    pub const NONE: Timeout = Timeout;
}

pub trait Source: Read {
    fn timeout(&self) -> Timeout;
    fn close(&mut self) -> io::Result<()>;
}

pub trait SourceExt: Source {
    fn buffer(self) -> BufferedSource {
        BufferedSource {
            inner: Box::new(self),
        }
    }
}

impl<T: Source> SourceExt for T {}


impl Read for BufferedSource {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl BufferedSource {
    pub fn string(&mut self) -> String {
        let mut s = String::new();
        self.read_to_string(&mut s).unwrap_or(0);
        s
    }

    pub fn byte_string(&mut self) -> ByteString {
        let mut buf = Vec::new();
        self.read_to_end(&mut buf).unwrap_or(0);
        ByteString(buf)
    }

    pub fn content_length(&self) -> i64 {
        // In a real implementation, this would check the source length
        -1
    }

    pub fn content_type(&self) -> Option<String> {
        None
    }
}

// Extension to mimic Kotlin's .asResponseBody()
trait ResponseBodyExt {
    fn as_response_body(content_type: Option<&str>, length: i64) -> ResponseBody;
}

impl ResponseBodyExt for BufferedSource {
    fn as_response_body(_content_type: Option<&str>, _length: i64) -> ResponseBody {
        // This is a mock implementation of the ResponseBody creation
        ResponseBody::new(Box::new(BufferedSource {
            inner: Box::new(BufferSourceMock {}),
        }))
    }
}

struct BufferSourceMock;
impl Read for BufferSourceMock {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Ok(0)
    }
}
impl Source for BufferSourceMock {
    fn timeout(&self) -> Timeout { Timeout::NONE }
    fn close(&mut self) -> io::Result<()> { Ok(()) }
}

pub struct ResponseCommonTest;

impl ResponseCommonTest {
    #[test]
    pub fn peek_shorter_than_response() {
        let response = Self::new_response(Self::response_body("abcdef"), 200);
        let peeked_body = response.peek_body(3);
        assert_eq!(peeked_body.string(), "abc");
        assert_eq!(response.body().string(), "abcdef");
    }

    #[test]
    pub fn peek_longer_than_response() {
        let response = Self::new_response(Self::response_body("abc"), 200);
        let peeked_body = response.peek_body(6);
        assert_eq!(peeked_body.string(), "abc");
        assert_eq!(response.body().string(), "abc");
    }

    #[test]
    pub fn each_peak_is_independent() {
        let response = Self::new_response(Self::response_body("abcdef"), 200);
        let p1 = response.peek_body(4);
        let p2 = response.peek_body(2);
        assert_eq!(response.body().string(), "abcdef");
        assert_eq!(p1.string(), "abcd");
        assert_eq!(p2.string(), "ab");
    }

    #[test]
    pub fn negative_status_code_throws_illegal_state_exception() {
        let result = std::panic::catch_unwind(|| {
            Self::new_response(Self::response_body("set status code -1"), -1);
        });
        assert!(result.is_err());
    }

    #[test]
    pub fn zero_status_code_is_valid() {
        let response = Self::new_response(Self::response_body("set status code 0"), 0);
        assert_eq!(response.code(), 0);
    }

    #[test]
    pub fn default_response_body_is_empty() {
        let response = Response::builder()
            .request(
                Request::builder()
                    .url("https://example.com/")
                    .build(),
            )
            .protocol(Protocol::HTTP_1_1)
            .code(200)
            .message("OK")
            .build();

        let body = response.body().unwrap();
        assert!(body.content_type().is_none());
        assert_eq!(body.content_length(), 0);
        assert_eq!(body.byte_string(), ByteString::EMPTY);
    }

    fn response_body(content: &str) -> ResponseBody {
        let data = Arc::new(Mutex::new(Buffer::new().write_utf8(content).clone()));
        
        struct ClosedSource {
            data: Arc<Mutex<Buffer>>,
            closed: Mutex<bool>,
        }

        impl Read for ClosedSource {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                if *self.closed.lock().unwrap() {
                    return Err(io::Error::new(io::ErrorKind::Other, "IllegalStateException: closed"));
                }
                let mut internal_buffer = Buffer::new();
                let mut lock = self.data.lock().unwrap();
                let read_bytes = lock.read(&mut internal_buffer, buf.len() as i64);
                let bytes = internal_buffer.data;
                let len = std::cmp::min(bytes.len(), buf.len());
                buf[..len].copy_from_slice(&bytes[..len]);
                Ok(len)
            }
        }

        impl Source for ClosedSource {
            fn timeout(&self) -> Timeout { Timeout::NONE }
            fn close(&mut self) -> io::Result<()> {
                *self.closed.lock().unwrap() = true;
                Ok(())
            }
        }

        let source = ClosedSource {
            data: data.clone(),
            closed: Mutex::new(false),
        };

        // In a real scenario, we'd use the .buffer().as_response_body() chain
        // Here we simulate the result of that chain
        ResponseBody::new(Box::new(source))
    }

    fn new_response(response_body: ResponseBody, code: i32) -> Response {
        Response::builder()
            .request(
                Request::builder()
                    .url("https://example.com/")
                    .build(),
            )
            .protocol(Protocol::HTTP_1_1)
            .code(code)
            .message("OK")
            .body(response_body)
            .build()
    }
}