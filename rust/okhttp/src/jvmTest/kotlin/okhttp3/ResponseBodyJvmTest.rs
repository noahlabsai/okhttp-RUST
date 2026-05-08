use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::ResponseBody::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::ResponseBodyTest::*;

// The following imports are based on the project structure and the provided context.
// We use the paths that align with the intended final rust path and common okhttp structure.
use crate::okhttp::src::jvmTest::kotlin::okhttp3::{
    MediaType, ResponseBody, ResponseBodyCompanion, 
    Source, BufferedSource, Buffer, ByteString, ForwardingSource,
    SourceExt
};

// Custom source used to track if the source was closed.
struct CustomSource<S: Source> {
    inner: S,
    closed: Arc<AtomicBool>,
}

impl<S: Source> Read for CustomSource<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<S: Source> Source for CustomSource<S> {
    fn timeout(&self) -> crate::okhttp::src::jvmTest::kotlin::okhttp3::Timeout {
        self.inner.timeout()
    }
    fn close(&mut self) -> io::Result<()> {
        self.closed.store(true, Ordering::SeqCst);
        self.inner.close()
    }
}

pub struct ResponseBodyJvmTest;

impl ResponseBodyJvmTest {
    pub fn string_empty() {
        let body = Self::body("".to_string(), None);
        assert_eq!(body.string(), "");
    }

    pub fn string_looks_like_bom_but_too_short() {
        let body = Self::body("000048".to_string(), None);
        assert_eq!(body.string(), "\u{0000}\u{0000}H");
    }

    pub fn string_defaults_to_utf8() {
        let body = Self::body("68656c6c6f".to_string(), None);
        assert_eq!(body.string(), "hello");
    }

    pub fn string_explicit_charset() {
        let body = Self::body("00000068000000650000006c0000006c0000006f".to_string(), Some("utf-32be".to_string()));
        assert_eq!(body.string(), "hello");
    }

    pub fn string_bom_overrides_explicit_charset() {
        let body = Self::body("0000feff00000068000000650000006c0000006c0000006f".to_string(), Some("utf-8".to_string()));
        assert_eq!(body.string(), "hello");
    }

    pub fn string_bom_utf8() {
        let body = Self::body("efbbbf68656c6c6f".to_string(), None);
        assert_eq!(body.string(), "hello");
    }

    pub fn string_bom_utf16_be() {
        let body = Self::body("feff00680065006c006c006f".to_string(), None);
        assert_eq!(body.string(), "hello");
    }

    pub fn string_bom_utf16_le() {
        let body = Self::body("fffe680065006c006c006f00".to_string(), None);
        assert_eq!(body.string(), "hello");
    }

    pub fn string_bom_utf32_be() {
        let body = Self::body("0000feff00000068000000650000006c0000006c0000006f".to_string(), None);
        assert_eq!(body.string(), "hello");
    }

    pub fn string_bom_utf32_le() {
        let body = Self::body("fffe000068000000650000006c0000006c0000006f000000".to_string(), None);
        assert_eq!(body.string(), "hello");
    }

    pub fn string_closes_underlying_source() {
        let closed = Arc::new(AtomicBool::new(false));
        let closed_clone = Arc::clone(&closed);

        struct ClosingBody {
            closed: Arc<AtomicBool>,
        }
        impl ResponseBody for ClosingBody {
            fn content_type(&self) -> Option<MediaType> { None }
            fn content_length(&self) -> i64 { 5 }
            fn source(&self) -> Box<dyn Source> {
                let mut buf = Buffer::new();
                buf.write_utf8("hello");
                Box::new(CustomSource {
                    inner: buf,
                    closed: Arc::clone(&self.closed),
                })
            }
            fn close(&mut self) -> io::Result<()> { Ok(()) }
        }

        let body = ClosingBody { closed: closed_clone };
        assert_eq!(body.string(), "hello");
        assert!(closed.load(Ordering::SeqCst));
    }

    pub fn reader_empty() {
        let body = Self::body("".to_string(), None);
        assert_eq!(Self::exhaust(body.char_stream()), "");
    }

    pub fn reader_looks_like_bom_but_too_short() {
        let body = Self::body("000048".to_string(), None);
        assert_eq!(Self::exhaust(body.char_stream()), "\u{0000}\u{0000}H");
    }

    pub fn reader_defaults_to_utf8() {
        let body = Self::body("68656c6c6f".to_string(), None);
        assert_eq!(Self::exhaust(body.char_stream()), "hello");
    }

    pub fn reader_explicit_charset() {
        let body = Self::body("00000068000000650000006c0000006c0000006f".to_string(), Some("utf-32be".to_string()));
        assert_eq!(Self::exhaust(body.char_stream()), "hello");
    }

    pub fn reader_bom_utf8() {
        let body = Self::body("efbbbf68656c6c6f".to_string(), None);
        assert_eq!(Self::exhaust(body.char_stream()), "hello");
    }

    pub fn reader_bom_utf16_be() {
        let body = Self::body("feff00680065006c006c006f".to_string(), None);
        assert_eq!(Self::exhaust(body.char_stream()), "hello");
    }

    pub fn reader_bom_utf16_le() {
        let body = Self::body("fffe680065006c006f00".to_string(), None);
        assert_eq!(Self::exhaust(body.char_stream()), "hello");
    }

    pub fn reader_bom_utf32_be() {
        let body = Self::body("0000feff00000068000000650000006c0000006c0000006f".to_string(), None);
        assert_eq!(Self::exhaust(body.char_stream()), "hello");
    }

    pub fn reader_bom_utf32_le() {
        let body = Self::body("fffe000068000000650000006c0000006c0000006f000000".to_string(), None);
        assert_eq!(Self::exhaust(body.char_stream()), "hello");
    }

    pub fn reader_closed_before_bom_closes_underlying_source() {
        let closed = Arc::new(AtomicBool::new(false));
        let closed_clone = Arc::clone(&closed);

        struct ClosingBody {
            closed: Arc<AtomicBool>,
        }
        impl ResponseBody for ClosingBody {
            fn content_type(&self) -> Option<MediaType> { None }
            fn content_length(&self) -> i64 { 5 }
            fn source(&self) -> Box<dyn Source> {
                let inner_body = Self::body("fffe680065006c006c006f00".to_string(), None);
                Box::new(CustomSource {
                    inner: inner_body.source(),
                    closed: Arc::clone(&self.closed),
                })
            }
            fn close(&mut self) -> io::Result<()> { Ok(()) }
        }

        let body = ClosingBody { closed: closed_clone };
        let mut reader = body.char_stream();
        let _ = reader.close();
        assert!(closed.load(Ordering::SeqCst));
    }

    pub fn reader_closed_after_bom_closes_underlying_source() {
        let closed = Arc::new(AtomicBool::new(false));
        let closed_clone = Arc::clone(&closed);

        struct ClosingBody {
            closed: Arc<AtomicBool>,
        }
        impl ResponseBody for ClosingBody {
            fn content_type(&self) -> Option<MediaType> { None }
            fn content_length(&self) -> i64 { 5 }
            fn source(&self) -> Box<dyn Source> {
                let inner_body = Self::body("fffe680065006c006c006f00".to_string(), None);
                Box::new(CustomSource {
                    inner: inner_body.source(),
                    closed: Arc::clone(&self.closed),
                })
            }
            fn close(&mut self) -> io::Result<()> { Ok(()) }
        }

        let body = ClosingBody { closed: closed_clone };
        let mut reader = body.char_stream();
        let _ = reader.read_char(); 
        let _ = reader.close();
        assert!(closed.load(Ordering::SeqCst));
    }

    pub fn source_sees_bom() {
        let bytes = ByteString::decode_hex("efbbbf68656c6c6f");
        let body = ResponseBodyCompanion::to_response_body(bytes, None);
        let mut source = body.source();
        
        let mut b = [0u8; 1];
        source.read(&mut b).unwrap();
        assert_eq!(b[0] & 0xff, 0xef);
        source.read(&mut b).unwrap();
        assert_eq!(b[0] & 0xff, 0xbb);
        source.read(&mut b).unwrap();
        assert_eq!(b[0] & 0xff, 0xbf);
        
        let mut buffered = source.buffer();
        assert_eq!(buffered.read_utf8(), "hello");
    }

    pub fn bytes_empty() {
        let body = Self::body("".to_string(), None);
        assert_eq!(body.bytes().len(), 0);
    }

    pub fn bytes_sees_bom() {
        let body = Self::body("efbbbf68656c6c6f".to_string(), None);
        let bytes = body.bytes();
        assert_eq!(bytes[0] & 0xff, 0xef);
        assert_eq!(bytes[1] & 0xff, 0xbb);
        assert_eq!(bytes[2] & 0xff, 0xbf);
        assert_eq!(String::from_utf8_lossy(&bytes[3..]).to_string(), "hello");
    }

    pub fn bytes_closes_underlying_source() {
        let closed = Arc::new(AtomicBool::new(false));
        let closed_clone = Arc::clone(&closed);

        struct ClosingBody {
            closed: Arc<AtomicBool>,
        }
        impl ResponseBody for ClosingBody {
            fn content_type(&self) -> Option<MediaType> { None }
            fn content_length(&self) -> i64 { 5 }
            fn source(&self) -> Box<dyn Source> {
                let mut buf = Buffer::new();
                buf.write_utf8("hello");
                Box::new(CustomSource {
                    inner: buf,
                    closed: Arc::clone(&self.closed),
                })
            }
            fn close(&mut self) -> io::Result<()> { Ok(()) }
        }

        let body = ClosingBody { closed: closed_clone };
        assert_eq!(body.bytes().len(), 5);
        assert!(closed.load(Ordering::SeqCst));
    }

    pub fn bytes_throws_when_lengths_disagree() {
        struct DisagreeBody;
        impl ResponseBody for DisagreeBody {
            fn content_type(&self) -> Option<MediaType> { None }
            fn content_length(&self) -> i64 { 10 }
            fn source(&self) -> Box<dyn Source> {
                let mut buf = Buffer::new();
                buf.write_utf8("hello");
                Box::new(buf)
            }
            fn close(&mut self) -> io::Result<()> { Ok(()) }
        }

        let body = DisagreeBody;
        let result = std::panic::catch_unwind(|| {
            body.bytes();
        });
        assert!(result.is_err());
    }

    pub fn bytes_throws_more_than_int_max_value() {
        struct LargeBody;
        impl ResponseBody for LargeBody {
            fn content_type(&self) -> Option<MediaType> { None }
            fn content_length(&self) -> i64 { i32::MAX as i64 + 1 }
            fn source(&self) -> Box<dyn Source> {
                panic!("AssertionError");
            }
            fn close(&mut self) -> io::Result<()> { Ok(()) }
        }

        let body = LargeBody;
        let result = std::panic::catch_unwind(|| {
            body.bytes();
        });
        assert!(result.is_err());
    }

    pub fn byte_string_empty() {
        let body = Self::body("".to_string(), None);
        assert_eq!(body.byte_string(), ByteString(vec![]));
    }

    pub fn byte_string_sees_bom() {
        let body = Self::body("efbbbf68656c6c6f".to_string(), None);
        let actual = body.byte_string();
        let expected = ByteString::decode_hex("efbbbf68656c6c6f");
        assert_eq!(actual, expected);
    }

    pub fn byte_string_closes_underlying_source() {
        let closed = Arc::new(AtomicBool::new(false));
        let closed_clone = Arc::clone(&closed);

        struct ClosingBody {
            closed: Arc<AtomicBool>,
        }
        impl ResponseBody for ClosingBody {
            fn content_type(&self) -> Option<MediaType> { None }
            fn content_length(&self) -> i64 { 5 }
            fn source(&self) -> Box<dyn Source> {
                let mut buf = Buffer::new();
                buf.write_utf8("hello");
                Box::new(CustomSource {
                    inner: buf,
                    closed: Arc::clone(&self.closed),
                })
            }
            fn close(&mut self) -> io::Result<()> { Ok(()) }
        }

        let body = ClosingBody { closed: closed_clone };
        assert_eq!(body.byte_string().0.len(), 5);
        assert!(closed.load(Ordering::SeqCst));
    }

    pub fn byte_string_throws_when_lengths_disagree() {
        struct DisagreeBody;
        impl ResponseBody for DisagreeBody {
            fn content_type(&self) -> Option<MediaType> { None }
            fn content_length(&self) -> i64 { 10 }
            fn source(&self) -> Box<dyn Source> {
                let mut buf = Buffer::new();
                buf.write_utf8("hello");
                Box::new(buf)
            }
            fn close(&mut self) -> io::Result<()> { Ok(()) }
        }

        let body = DisagreeBody;
        let result = std::panic::catch_unwind(|| {
            body.byte_string();
        });
        assert!(result.is_err());
    }

    pub fn byte_string_throws_more_than_int_max_value() {
        struct LargeBody;
        impl ResponseBody for LargeBody {
            fn content_type(&self) -> Option<MediaType> { None }
            fn content_length(&self) -> i64 { i32::MAX as i64 + 1 }
            fn source(&self) -> Box<dyn Source> {
                panic!("AssertionError");
            }
            fn close(&mut self) -> io::Result<()> { Ok(()) }
        }

        let body = LargeBody;
        let result = std::panic::catch_unwind(|| {
            body.byte_string();
        });
        assert!(result.is_err());
    }

    pub fn byte_stream_empty() {
        let body = Self::body("".to_string(), None);
        let mut bytes = body.source();
        let mut b = [0u8; 1];
        assert_eq!(bytes.read(&mut b).unwrap(), 0);
    }

    pub fn byte_stream_sees_bom() {
        let body = Self::body("efbbbf68656c6c6f".to_string(), None);
        let mut bytes = body.source();
        let mut b = [0u8; 1];
        
        bytes.read(&mut b).unwrap();
        assert_eq!(b[0], 0xef);
        bytes.read(&mut b).unwrap();
        assert_eq!(b[0], 0xbb);
        bytes.read(&mut b).unwrap();
        assert_eq!(b[0], 0xbf);
        
        let mut buffered = bytes.buffer();
        assert_eq!(buffered.read_utf8(), "hello");
    }

    pub fn byte_stream_closes_underlying_source() {
        let closed = Arc::new(AtomicBool::new(false));
        let closed_clone = Arc::clone(&closed);

        struct ClosingBody {
            closed: Arc<AtomicBool>,
        }
        impl ResponseBody for ClosingBody {
            fn content_type(&self) -> Option<MediaType> { None }
            fn content_length(&self) -> i64 { 5 }
            fn source(&self) -> Box<dyn Source> {
                let mut buf = Buffer::new();
                buf.write_utf8("hello");
                Box::new(CustomSource {
                    inner: buf,
                    closed: Arc::clone(&self.closed),
                })
            }
            fn close(&mut self) -> io::Result<()> { Ok(()) }
        }

        let body = ClosingBody { closed: closed_clone };
        let mut source = body.source();
        let _ = source.close();
        assert!(closed.load(Ordering::SeqCst));
    }

    pub fn unicode_text_with_unsupported_encoding() {
        let text = "eile oli oliivi\u{00f5}li";
        let media_type = MediaType::parse("text/plain; charset=unknown");
        let body = ResponseBodyCompanion::to_response_body(text.as_bytes().to_vec(), Some(media_type));
        assert_eq!(body.string(), text);
    }

    fn body(hex: String, charset: Option<String>) -> Box<dyn ResponseBody> {
        let media_type = charset.map(|c| MediaType::parse(&format!("any/thing; charset={}", c)));
        let bytes = ByteString::decode_hex(&hex);
        ResponseBodyCompanion::to_response_body(bytes, media_type)
    }

    fn exhaust<R: Read>(mut reader: R) -> String {
        let mut result = String::new();
        let mut buf = [0u8; 10];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    result.push_str(&String::from_utf8_lossy(&buf[..n]));
                }
                Err(_) => break,
            }
        }
        result
    }
}

// Helper trait to simulate Kotlin's charStream() and Reader behavior
pub trait CharStreamExt {
    fn char_stream(&self) -> Box<dyn CharReader>;
}

pub trait CharReader: Read {
    fn read_char(&mut self) -> Option<char>;
    fn close(&mut self) -> io::Result<()>;
}

impl CharStreamExt for Box<dyn ResponseBody> {
    fn char_stream(&self) -> Box<dyn CharReader> {
        Box::new(CharReaderImpl { source: self.source() })
    }
}

struct CharReaderImpl {
    source: Box<dyn Source>,
}

impl Read for CharReaderImpl {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.source.read(buf)
    }
}

impl CharReader for CharReaderImpl {
    fn read_char(&mut self) -> Option<char> {
        let mut b = [0u8; 1];
        if self.source.read(&mut b).unwrap_or(0) > 0 {
            Some(b[0] as char)
        } else {
            None
        }
    }
    fn close(&mut self) -> io::Result<()> {
        self.source.close()
    }
}
