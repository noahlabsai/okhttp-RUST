use std::io::{self, Read};
use std::sync::{Arc, Mutex};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{Headers, Protocol, Request, Response, ResponseBody};
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody as Buffer;

/// Trait representing the TrailersSource interface from Kotlin
pub trait TrailersSource {
    fn get(&self) -> Headers;
}

/// Implementation of the ResponseJvmTest class
pub struct ResponseJvmTest;

impl ResponseJvmTest {
    #[test]
    pub fn test_empty_by_default_if_trailers_not_set() {
        let body = "".to_response_body();
        let response = Self::new_response(body, 200, |_builder| {});

        assert!(response.trailers().is_empty());
    }

    #[test]
    pub fn works_if_trailers_set() {
        struct MyTrailers;
        impl TrailersSource for MyTrailers {
            fn get(&self) -> Headers {
                Headers::headers_of(&["a", "b"])
            }
        }

        let body = "".to_response_body();
        let response = Self::new_response(body, 200, |builder| {
            builder.trailers(Box::new(MyTrailers));
        });

        assert_eq!(response.trailers().get("a"), Some("b"));
    }

    #[test]
    pub fn peek_after_reading_response() {
        let body = Self::response_body("abc");
        let response = Self::new_response(body, 200, |_builder| {});
        
        assert_eq!(response.body().string(), "abc");

        let result = std::panic::catch_unwind(|| {
            response.peek_body(3);
        });
        
        assert!(result.is_err(), "Expected IllegalStateException (panic) when peeking after body is consumed");
    }

    /// Returns a new response body that refuses to be read once it has been closed.
    fn response_body(content: &str) -> ResponseBody {
        let mut data = Buffer::new();
        data.write_utf8(content);
        let data = Arc::new(Mutex::new(data));

        struct ClosingSource {
            data: Arc<Mutex<Buffer>>,
            closed: Arc<Mutex<bool>>,
        }

        impl Read for ClosingSource {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                let is_closed = self.closed.lock().unwrap();
                if *is_closed {
                    return Err(io::Error::new(io::ErrorKind::Other, "IllegalStateException: Source closed"));
                }
                
                let mut data_lock = self.data.lock().unwrap();
                // Simulate okio Buffer.read(sink, byteCount)
                let bytes_read = data_lock.read(buf);
                Ok(bytes_read)
            }
        }

        impl Drop for ClosingSource {
            fn drop(&mut self) {
                let mut closed = self.closed.lock().unwrap();
                *closed = true;
            }
        }

        let source = ClosingSource {
            data,
            closed: Arc::new(Mutex::new(false)),
        };

        // In Rust, we wrap the source into the ResponseBody. 
        // .as_response_body(null, -1) equivalent:
        ResponseBody::from_source(Box::new(source), None, -1)
    }

    fn new_response<F>(
        response_body: ResponseBody,
        code: i32,
        fn_block: F,
    ) -> Response 
    where 
        F: FnOnce(&mut Response::Builder) 
    {
        let mut builder = Response::Builder::new();
        builder.request(
            Request::builder()
                .url("https://example.com/")
                .build(),
        );
        builder.protocol(Protocol::HTTP_1_1);
        builder.code(code);
        builder.message("OK");
        builder.body(response_body);
        
        fn_block(&mut builder);
        
        builder.build()
    }
}

/// Extension trait to mimic Kotlin's String.toResponseBody()
pub trait StringResponseBodyExt {
    fn to_response_body(&self) -> ResponseBody;
}

impl StringResponseBodyExt for str {
    fn to_response_body(&self) -> ResponseBody {
        let mut buffer = Buffer::new();
        buffer.write_utf8(self);
        ResponseBody::from_buffer(buffer, None, -1)
    }
}

/// Mocking the ResponseBody companion methods for compilability
impl ResponseBody {
    pub fn from_source<S: Read + 'static>(source: Box<S>, content_type: Option<&str>, content_length: i64) -> Self {
        // Implementation provided by okhttp crate
        unimplemented!("Provided by okhttp crate")
    }
    pub fn from_buffer(buffer: Buffer, content_type: Option<&str>, content_length: i64) -> Self {
        // Implementation provided by okhttp crate
        unimplemented!("Provided by okhttp crate")
    }
}

/// Mocking Response Builder for the DSL-like behavior
impl Response::Builder {
    pub fn new() -> Self {
        unimplemented!("Provided by okhttp crate")
    }
    pub fn request(&mut self, request: Request) -> &mut Self {
        unimplemented!("Provided by okhttp crate")
    }
    pub fn protocol(&mut self, protocol: Protocol) -> &mut Self {
        unimplemented!("Provided by okhttp crate")
    }
    pub fn code(&mut self, code: i32) -> &mut Self {
        unimplemented!("Provided by okhttp crate")
    }
    pub fn message(&mut self, message: &str) -> &mut Self {
        unimplemented!("Provided by okhttp crate")
    }
    pub fn body(&mut self, body: ResponseBody) -> &mut Self {
        unimplemented!("Provided by okhttp crate")
    }
    pub fn trailers(&mut self, source: Box<dyn TrailersSource>) -> &mut Self {
        unimplemented!("Provided by okhttp crate")
    }
    pub fn build(self) -> Response {
        unimplemented!("Provided by okhttp crate")
    }
}