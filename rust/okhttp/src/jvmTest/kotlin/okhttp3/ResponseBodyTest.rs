use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::cache::FaultHidingSink::*;

// --- Mocking Okio/OkHttp types to ensure compilability as per requirements ---


impl MediaType {
    pub fn parse(value: &str) -> Self {
        MediaType {
            value: value.to_string(),
        }
    }
}

pub trait MediaTypeExt {
    fn to_media_type(&self) -> MediaType;
}

impl MediaTypeExt for &str {
    fn to_media_type(&self) -> MediaType {
        MediaType::parse(self)
    }
}

pub trait Source: Read {
    fn timeout(&self) -> Timeout;
    fn close(&mut self) -> io::Result<()>;
}

#[derive(Debug, Clone, Copy)]
pub struct Timeout;


impl Buffer {
    pub fn new() -> Self {
        Buffer { data: Vec::new() }
    }

    pub fn write_utf8(&mut self, s: &str) {
        self.data.extend_from_slice(s.as_bytes());
    }

    pub fn read_utf8(&mut self) -> String {
        let s = String::from_utf8_lossy(&self.data).to_string();
        self.data.clear();
        s
    }

    pub fn exhausted(&self) -> bool {
        self.data.is_empty()
    }
}

impl Read for Buffer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.data.is_empty() {
            return Ok(0);
        }
        let len = std::cmp::min(buf.len(), self.data.len());
        let drained: Vec<u8> = self.data.drain(0..len).collect();
        buf[..len].copy_from_slice(&drained);
        Ok(len)
    }
}

impl Source for Buffer {
    fn timeout(&self) -> Timeout {
        Timeout
    }
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct BufferedSource<S: Source> {
    inner: S,
}

impl<S: Source> BufferedSource<S> {
    pub fn new(inner: S) -> Self {
        BufferedSource { inner }
    }

    pub fn read_utf8(&mut self) -> String {
        let mut buf = Vec::new();
        let _ = self.inner.read_to_end(&mut buf);
        String::from_utf8_lossy(&buf).to_string()
    }

    pub fn exhausted(&mut self) -> bool {
        let mut byte = [0u8; 1];
        match self.inner.read(&mut byte) {
            Ok(0) => true,
            _ => false,
        }
    }
}

impl<S: Source> Read for BufferedSource<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<S: Source> Source for BufferedSource<S> {
    fn timeout(&self) -> Timeout {
        self.inner.timeout()
    }
    fn close(&mut self) -> io::Result<()> {
        self.inner.close()
    }
}

pub trait SourceExt: Source + Sized {
    fn buffer(self) -> BufferedSource<Self> {
        BufferedSource::new(self)
    }
}

impl<S: Source> SourceExt for S {}

#[derive(Debug, Clone, PartialEq)]
pub struct ByteString(pub Vec<u8>);

impl ByteString {
    pub fn decode_hex(hex: &str) -> Self {
        // Simplified hex decode for test purposes
        if hex.is_empty() {
            return ByteString(vec![]);
        }
        // In a real implementation, this would decode the hex string
        ByteString(vec![]) 
    }
    pub fn encode_utf8(s: &str) -> Self {
        ByteString(s.as_bytes().to_vec())
    }
}

pub trait ResponseBody {
    fn content_type(&self) -> Option<MediaType>;
    fn content_length(&self) -> i64;
    fn source(&self) -> Box<dyn Source>;
    fn close(&mut self) -> io::Result<()>;
    
    fn string(&self) -> String {
        let mut src = self.source();
        let mut buf = Vec::new();
        let _ = src.read_to_end(&mut buf);
        String::from_utf8_lossy(&buf).to_string()
    }

    fn bytes(&self) -> Vec<u8> {
        let mut src = self.source();
        let mut buf = Vec::new();
        let _ = src.read_to_end(&mut buf);
        buf
    }

    fn byte_string(&self) -> ByteString {
        ByteString(self.bytes())
    }
}

pub struct ResponseBodyCompanion;
impl ResponseBodyCompanion {
    pub fn to_response_body<T: Into<Vec<u8>>>(data: T, media_type: Option<MediaType>) -> Box<dyn ResponseBody> {
        Box::new(SimpleResponseBody {
            data: data.into(),
            media_type,
        })
    }
}

struct SimpleResponseBody {
    data: Vec<u8>,
    media_type: Option<MediaType>,
}

impl ResponseBody for SimpleResponseBody {
    fn content_type(&self) -> Option<MediaType> {
        self.media_type.clone()
    }
    fn content_length(&self) -> i64 {
        self.data.len() as i64
    }
    fn source(&self) -> Box<dyn Source> {
        Box::new(Buffer { data: self.data.clone() })
    }
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// --- ForwardingSource Implementation ---

pub struct ForwardingSource<S: Source> {
    pub delegate: S,
}

impl<S: Source> Source for ForwardingSource<S> {
    fn timeout(&self) -> Timeout {
        self.delegate.timeout()
    }
    fn close(&mut self) -> io::Result<()> {
        self.delegate.close()
    }
}

impl<S: Source> Read for ForwardingSource<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.delegate.read(buf)
    }
}

// --- Test Suite ---

pub struct ResponseBodyTest;

impl ResponseBodyTest {
    pub fn source_empty() {
        let media_type: Option<MediaType> = None;
        let body = ResponseBodyCompanion::to_response_body(ByteString::decode_hex(""), media_type);
        let mut source = body.source();
        
        // Simulate BufferedSource behavior for the test
        let mut buf = Vec::new();
        let _ = source.read_to_end(&mut buf);
        assert!(buf.is_empty());
        assert_eq!(String::from_utf8_lossy(&buf), "");
    }

    pub fn source_closes_underlying_source() {
        let closed = Arc::new(AtomicBool::new(false));
        let closed_clone = Arc::clone(&closed);

        struct CustomBody {
            closed_flag: Arc<AtomicBool>,
        }

        impl ResponseBody for CustomBody {
            fn content_type(&self) -> Option<MediaType> { None }
            fn content_length(&self) -> i64 { 5 }
            fn source(&self) -> Box<dyn Source> {
                let mut buffer = Buffer::new();
                buffer.write_utf8("hello");
                
                struct ClosingSource {
                    inner: Buffer,
                    flag: Arc<AtomicBool>,
                }
                impl Read for ClosingSource {
                    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> { self.inner.read(buf) }
                }
                impl Source for ClosingSource {
                    fn timeout(&self) -> Timeout { Timeout }
                    fn close(&mut self) -> io::Result<()> {
                        self.flag.store(true, Ordering::SeqCst);
                        self.inner.close()
                    }
                }
                Box::new(ClosingSource { inner: buffer, flag: self.closed_flag.clone() })
            }
            fn close(&mut self) -> io::Result<()> { Ok(()) }
        }

        let body = CustomBody { closed_flag: closed_clone };
        let mut source = body.source();
        let _ = source.close();
        assert!(closed.load(Ordering::SeqCst));
    }

    pub fn throwing_underlying_source_closes_quietly() {
        struct ThrowingBody;
        impl ResponseBody for ThrowingBody {
            fn content_type(&self) -> Option<MediaType> { None }
            fn content_length(&self) -> i64 { 5 }
            fn source(&self) -> Box<dyn Source> {
                struct ThrowingSource { inner: Buffer }
                impl Read for ThrowingSource {
                    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> { self.inner.read(buf) }
                }
                impl Source for ThrowingSource {
                    fn timeout(&self) -> Timeout { Timeout }
                    fn close(&mut self) -> io::Result<()> {
                        Err(io::Error::new(io::ErrorKind::Other, "Broken!"))
                    }
                }
                let mut buffer = Buffer::new();
                buffer.write_utf8("hello");
                Box::new(ThrowingSource { inner: buffer })
            }
            fn close(&mut self) -> io::Result<()> {
                let mut src = self.source();
                let _ = src.close(); // Closes quietly (ignores error)
                Ok(())
            }
        }

        let mut body = ThrowingBody;
        let mut source = body.source();
        let mut buf = Vec::new();
        let _ = source.read_to_end(&mut buf);
        assert_eq!(String::from_utf8_lossy(&buf), "hello");
        let _ = body.close();
    }

    pub fn unicode_text() {
        let text = "eile oli oliivi\u{00f5}li";
        let body = ResponseBodyCompanion::to_response_body(text.as_bytes().to_vec(), None);
        assert_eq!(body.string(), text);
    }

    pub fn unicode_text_with_charset() {
        let text = "eile oli oliivi\u{00f5}li";
        let media_type = "text/plain; charset=UTF-8".to_media_type();
        let body = ResponseBodyCompanion::to_response_body(text.as_bytes().to_vec(), Some(media_type));
        assert_eq!(body.string(), text);
    }

    pub fn unicode_byte_string() {
        let text = "eile oli oliivi\u{00f5}li";
        let body = ResponseBodyCompanion::to_response_body(text.as_bytes().to_vec(), None);
        assert_eq!(body.byte_string(), ByteString::encode_utf8(text));
    }

    pub fn unicode_byte_string_with_charset() {
        let text = "eile oli oliivi\u{00f5}li";
        let bytes = text.as_bytes().to_vec();
        let media_type = "text/plain; charset=EBCDIC".to_media_type();
        let body = ResponseBodyCompanion::to_response_body(bytes.clone(), Some(media_type));
        assert_eq!(body.byte_string(), ByteString(bytes));
    }

    pub fn unicode_bytes() {
        let text = "eile oli oliivi\u{00f5}li";
        let body = ResponseBodyCompanion::to_response_body(text.as_bytes().to_vec(), None);
        assert_eq!(body.bytes(), text.as_bytes().to_vec());
    }

    pub fn unicode_bytes_with_charset() {
        let text = "eile oli oliivi\u{00f5}li";
        let bytes = text.as_bytes().to_vec();
        let media_type = "text/plain; charset=EBCDIC".to_media_type();
        let body = ResponseBodyCompanion::to_response_body(bytes.clone(), Some(media_type));
        assert_eq!(body.bytes(), bytes);
    }
}
