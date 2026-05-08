use std::error::Error;
use std::io::{Read, Write};
use std::sync::Arc;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{Headers, MediaType, MultipartBody};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::MultipartBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::cache::FaultHidingSink::*;

// Mocking the Buffer from okio for the test environment
// In a real production translation, this would be a wrapper around a Vec<u8> or a real Okio port.

impl Buffer {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn size(&self) -> i64 {
        self.data.len() as i64
    }

    pub fn read_utf8(&mut self) -> String {
        let s = String::from_utf8_lossy(&self.data).into_owned();
        self.data.clear();
        s
    }

    pub fn write_utf8(&mut self, s: &str) {
        self.data.extend_from_slice(s.as_bytes());
    }
}

impl Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.data.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct MultipartBodyTest;

impl MultipartBodyTest {
    #[test]
    fn one_part_required() {
        let result = std::panic::catch_unwind(|| {
            MultipartBody::Builder::new().build();
        });
        
        assert!(result.is_err());
        // In Rust, we'd typically check the panic message if using a specific test framework,
        // but here we simulate the Kotlin assertFailsWith behavior.
    }

    #[test]
    fn single_part() {
        let expected = r#"--123

Hello, World!--123--
"#
        .replace("\n", "\r\n");

        let body = MultipartBody::Builder::new("123")
            .add_part("Hello, World!".to_request_body(None))
            .build();

        assert_eq!(body.boundary(), "123");
        assert_eq!(body.body_type(), MultipartBody::Type::Mixed);
        assert_eq!(body.content_type().unwrap().to_string(), "multipart/mixed; boundary=123");
        assert_eq!(body.parts().len(), 1);
        assert_eq!(body.content_length(), 33);

        let mut buffer = Buffer::new();
        body.write_to(&mut buffer).expect("Write failed");
        assert_eq!(body.content_length(), buffer.size());
        assert_eq!(buffer.read_utf8(), expected);
    }

    #[test]
    fn three_parts() {
        let expected = r#"--123

Quick
--123

Brown
--123

Fox
--123--
"#
        .replace("\n", "\r\n");

        let body = MultipartBody::Builder::new("123")
            .add_part("Quick".to_request_body(None))
            .add_part("Brown".to_request_body(None))
            .add_part("Fox".to_request_body(None))
            .build();

        assert_eq!(body.boundary(), "123");
        assert_eq!(body.body_type(), MultipartBody::Type::Mixed);
        assert_eq!(body.content_type().unwrap().to_string(), "multipart/mixed; boundary=123");
        assert_eq!(body.parts().len(), 3);
        assert_eq!(body.content_length(), 55);

        let mut buffer = Buffer::new();
        body.write_to(&mut buffer).expect("Write failed");
        assert_eq!(body.content_length(), buffer.size());
        assert_eq!(buffer.read_utf8(), expected);
    }

    #[test]
    fn field_and_two_files() {
        let expected = r#"--AaB03x
Content-Disposition: form-data; name="submit-name"

Larry
--AaB03x
Content-Disposition: form-data; name="files"
Content-Type: multipart/mixed; boundary=BbC04y

--BbC04y
Content-Disposition: file; filename="file1.txt"
Content-Type: text/plain; charset=utf-8

... contents of file1.txt ...
--BbC04y
Content-Disposition: file; filename="file2.gif"
Content-Transfer-Encoding: binary
Content-Type: image/gif

... contents of file2.gif ...
--BbC04y--

--AaB03x--
"#
        .replace("\n", "\r\n");

        let inner_body = MultipartBody::Builder::new("BbC04y")
            .add_part(
                Headers::headers_of(&[("Content-Disposition", "file; filename=\"file1.txt\"")]),
                "... contents of file1.txt ...".to_request_body(Some(MediaType::parse("text/plain").unwrap())),
            )
            .add_part(
                Headers::headers_of(&[
                    ("Content-Disposition", "file; filename=\"file2.gif\""),
                    ("Content-Transfer-Encoding", "binary"),
                ]),
                "... contents of file2.gif ...".as_bytes().to_request_body(Some(MediaType::parse("image/gif").unwrap()), 0, "... contents of file2.gif ...".len()),
            )
            .build();

        let body = MultipartBody::Builder::new("AaB03x")
            .set_type(MultipartBody::Type::Form)
            .add_form_data_part("submit-name", "Larry")
            .add_form_data_part("files", None, Arc::new(inner_body) as Arc<dyn RequestBody>)
            .build();

        assert_eq!(body.boundary(), "AaB03x");
        assert_eq!(body.body_type(), MultipartBody::Type::Form);
        assert_eq!(body.content_type().unwrap().to_string(), "multipart/form-data; boundary=AaB03x");
        assert_eq!(body.parts().len(), 2);
        assert_eq!(body.content_length(), 488);

        let mut buffer = Buffer::new();
        body.write_to(&mut buffer).expect("Write failed");
        assert_eq!(body.content_length(), buffer.size());
        assert_eq!(buffer.read_utf8(), expected);
    }

    #[test]
    fn string_escaping_is_weird() {
        let expected = r#"--AaB03x
Content-Disposition: form-data; name="field with spaces"; filename="filename with spaces.txt"
Content-Type: text/plain; charset=utf-8

okay
--AaB03x
Content-Disposition: form-data; name="field with %22"

"
--AaB03x
Content-Disposition: form-data; name="field with %22"

%22
--AaB03x
Content-Disposition: form-data; name="field with ~"

Alpha
--AaB03x--
"#
        .replace("\n", "\r\n");

        let body = MultipartBody::Builder::new("AaB03x")
            .set_type(MultipartBody::Type::Form)
            .add_form_data_part(
                "field with spaces",
                "filename with spaces.txt",
                "okay".to_request_body(Some(MediaType::parse("text/plain; charset=utf-8").unwrap())),
            )
            .add_form_data_part("field with \"", "\"")
            .add_form_data_part("field with %22", "%22")
            .add_form_data_part("field with \u{007e}", "Alpha")
            .build();

        let mut buffer = Buffer::new();
        body.write_to(&mut buffer).expect("Write failed");
        assert_eq!(buffer.read_utf8(), expected);
    }

    #[test]
    fn streaming_part_has_no_length() {
        struct StreamingBody {
            body: String,
        }
        impl RequestBody for StreamingBody {
            fn content_type(&self) -> Option<Arc<MediaType>> { None }
            fn content_length(&self) -> i64 { -1 }
            fn write_to(&self, sink: &mut dyn Write) -> Result<(), Box<dyn Error + Send + Sync>> {
                sink.write_all(self.body.as_bytes())?;
                Ok(())
            }
        }

        let expected = r#"--123

Quick
--123

Brown
--123

Fox
--123--
"#
        .replace("\n", "\r\n");

        let body = MultipartBody::Builder::new("123")
            .add_part("Quick".to_request_body(None))
            .add_part(Arc::new(StreamingBody { body: "Brown".to_string() }) as Arc<dyn RequestBody>)
            .add_part("Fox".to_request_body(None))
            .build();

        assert_eq!(body.boundary(), "123");
        assert_eq!(body.body_type(), MultipartBody::Type::Mixed);
        assert_eq!(body.content_type().unwrap().to_string(), "multipart/mixed; boundary=123");
        assert_eq!(body.parts().len(), 3);
        assert_eq!(body.content_length(), -1);

        let mut buffer = Buffer::new();
        body.write_to(&mut buffer).expect("Write failed");
        assert_eq!(buffer.read_utf8(), expected);
    }

    #[test]
    fn content_type_header_is_forbidden() {
        let mut multipart = MultipartBody::Builder::new("boundary");
        let result = std::panic::catch_unwind(move || {
            multipart.add_part(
                Headers::headers_of(&[("Content-Type", "text/plain")]),
                "Hello, World!".to_request_body(None),
            );
        });
        assert!(result.is_err());
    }

    #[test]
    fn content_length_header_is_forbidden() {
        let mut multipart = MultipartBody::Builder::new("boundary");
        let result = std::panic::catch_unwind(move || {
            multipart.add_part(
                Headers::headers_of(&[("Content-Length", "13")]),
                "Hello, World!".to_request_body(None),
            );
        });
        assert!(result.is_err());
    }

    #[test]
    fn part_accessors() {
        let body = MultipartBody::Builder::new("boundary")
            .add_part(Headers::headers_of(&[("Foo", "Bar")]), "Baz".to_request_body(None))
            .build();

        assert_eq!(body.parts().len(), 1);
        let mut part1_buffer = Buffer::new();
        let part1 = body.part(0);
        part1.body.write_to(&mut part1_buffer).expect("Write failed");
        assert_eq!(part1.headers, Headers::headers_of(&[("Foo", "Bar")]));
        assert_eq!(part1_buffer.read_utf8(), "Baz");
    }

    #[test]
    fn non_ascii_filename() {
        let expected = r#"--AaB03x
Content-Disposition: form-data; name="attachment"; filename="resumé.pdf"
Content-Type: application/pdf; charset=utf-8

Jesse's Resumé
--AaB03x--
"#
        .replace("\n", "\r\n");

        let body = MultipartBody::Builder::new("AaB03x")
            .set_type(MultipartBody::Type::Form)
            .add_form_data_part(
                "attachment",
                "resumé.pdf",
                "Jesse's Resumé".to_request_body(Some(MediaType::parse("application/pdf").unwrap())),
            )
            .build();

        let mut buffer = Buffer::new();
        body.write_to(&mut buffer).expect("Write failed");
        assert_eq!(buffer.read_utf8(), expected);
    }

    #[test]
    fn write_twice() {
        let expected = r#"--123

Hello, World!
--123--
"#
        .replace("\n", "\r\n");

        let body = MultipartBody::Builder::new("123")
            .add_part("Hello, World!".to_request_body(None))
            .build();

        assert_eq!(body.is_one_shot(), false);

        let mut buffer = Buffer::new();
        body.write_to(&mut buffer).expect("Write failed");
        assert_eq!(body.content_length(), buffer.size());
        assert_eq!(buffer.read_utf8(), expected);

        let mut buffer2 = Buffer::new();
        body.write_to(&mut buffer2).expect("Write failed");
        assert_eq!(body.content_length(), buffer2.size());
        assert_eq!(buffer2.read_utf8(), expected);
    }

    #[test]
    fn write_twice_with_one_shot() {
        let expected = r#"--123

Hello, World!
--123--
"#
        .replace("\n", "\r\n");

        let body = MultipartBody::Builder::new("123")
            .add_part(Arc::new(OneShotRequestBody {
                content: "Hello, World!".to_string(),
            }) as Arc<dyn RequestBody>)
            .build();

        assert_eq!(body.is_one_shot(), true);

        let mut buffer = Buffer::new();
        body.write_to(&mut buffer).expect("Write failed");
        assert_eq!(body.content_length(), buffer.size());
        assert_eq!(buffer.read_utf8(), expected);
    }
}

struct OneShotRequestBody {
    content: String,
}

impl RequestBody for OneShotRequestBody {
    fn content_type(&self) -> Option<Arc<MediaType>> { None }
    fn is_one_shot(&self) -> bool { true }
    fn content_length(&self) -> i64 { self.content.len() as i64 }
    fn write_to(&self, sink: &mut dyn Write) -> Result<(), Box<dyn Error + Send + Sync>> {
        sink.write_all(self.content.as_bytes())?;
        Ok(())
    }
}