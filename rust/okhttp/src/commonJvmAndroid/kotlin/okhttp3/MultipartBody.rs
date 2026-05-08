use std::io::Write;
use std::sync::Mutex;
use uuid::Uuid;

// Assuming these types are defined in the crate as per OkHttp structure
use crate::okhttp3::{Headers, MediaType, RequestBody};
use okio::{Buffer, BufferedSink, ByteString};
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;

pub struct MultipartBody {
    boundary_byte_string: ByteString,
    pub r#type: MediaType,
    pub parts: Vec<Part>,
    content_type: MediaType,
    content_length: Mutex<i64>,
}

impl MultipartBody {
    pub fn new(boundary_byte_string: ByteString, r#type: MediaType, parts: Vec<Part>) -> Self {
        let boundary = boundary_byte_string.utf8();
        let content_type_str = format!("{}; boundary={}", r#type, boundary);
        let content_type = MediaType::parse(&content_type_str).expect("Invalid media type");

        Self {
            boundary_byte_string,
            r#type,
            parts,
            content_type,
            content_length: Mutex::new(-1),
        }
    }

    pub fn boundary(&self) -> String {
        self.boundary_byte_string.utf8()
    }

    pub fn size(&self) -> usize {
        self.parts.len()
    }

    pub fn part(&self, index: usize) -> &Part {
        &self.parts[index]
    }

    pub fn is_one_shot(&self) -> bool {
        self.parts.iter().any(|p| p.body.is_one_shot())
    }

    pub fn content_type(&self) -> &MediaType {
        &self.content_type
    }

    pub fn content_length(&self) -> Result<i64, Box<dyn std::error::Error>> {
        let mut len = self.content_length.lock().unwrap();
        if *len == -1 {
            let result = self.write_or_count_bytes(None, true)?;
            *len = result;
        }
        Ok(*len)
    }

    pub fn write_to(&self, sink: &mut dyn BufferedSink) -> Result<(), Box<dyn std::error::Error>> {
        self.write_or_count_bytes(Some(sink), false)?;
        Ok(())
    }

    fn write_or_count_bytes(
        &self,
        sink: Option<&mut dyn BufferedSink>,
        count_bytes: bool,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        let mut byte_count = 0i64;
        let mut byte_count_buffer: Option<Buffer> = None;

        if count_bytes {
            byte_count_buffer = Some(Buffer::new());
        }

        for part in &self.parts {
            let headers = &part.headers;
            let body = &part.body;

            self.write_to_sink(sink, count_bytes, &mut byte_count_buffer, 
                |s| {
                    s.write_all(Self::DASHDASH)?;
                    s.write_all(&self.boundary_byte_string)?;
                    s.write_all(Self::CRLF)?;
                    Ok(())
                })?;

            if let Some(h) = headers {
                for i in 0..h.size() {
                    self.write_to_sink(sink, count_bytes, &mut byte_count_buffer, 
                        |s| {
                            s.write_all(h.name(i).as_bytes())?;
                            s.write_all(Self::COLONSPACE)?;
                            s.write_all(h.value(i).as_bytes())?;
                            s.write_all(Self::CRLF)?;
                            Ok(())
                        })?;
                }
            }

            if let Some(ct) = body.content_type() {
                self.write_to_sink(sink, count_bytes, &mut byte_count_buffer, 
                    |s| {
                        s.write_all(b"Content-Type: ")?;
                        s.write_all(ct.to_string().as_bytes())?;
                        s.write_all(Self::CRLF)?;
                        Ok(())
                    })?;
            }

            let body_len = body.content_length()?;
            if body_len == -1 && count_bytes {
                if let Some(ref mut buf) = byte_count_buffer {
                    buf.clear();
                }
                return Ok(-1);
            }

            self.write_to_sink(sink, count_bytes, &mut byte_count_buffer, 
                |s| s.write_all(Self::CRLF)
            )?;

            if count_bytes {
                byte_count += body_len;
            } else {
                self.write_to_sink(sink, count_bytes, &mut byte_count_buffer, 
                    |s| body.write_to(s)
                )?;
            }

            self.write_to_sink(sink, count_bytes, &mut byte_count_buffer, 
                |s| s.write_all(Self::CRLF)
            )?;
        }

        self.write_to_sink(sink, count_bytes, &mut byte_count_buffer, 
            |s| {
                s.write_all(Self::DASHDASH)?;
                s.write_all(&self.boundary_byte_string)?;
                s.write_all(Self::DASHDASH)?;
                s.write_all(Self::CRLF)?;
                Ok(())
            })?;

        if count_bytes {
            if let Some(ref buf) = byte_count_buffer {
                byte_count += buf.size() as i64;
            }
        }

        Ok(byte_count)
    }

    fn write_to_sink<F>(&self, sink: Option<&mut dyn BufferedSink>, count_bytes: bool, buffer: &mut Option<Buffer>, f: F) -> Result<(), Box<dyn std::error::Error>> 
    where F: FnOnce(&mut dyn Write) -> Result<(), Box<dyn std::error::Error>> 
    {
        if count_bytes {
            if let Some(ref mut buf) = buffer {
                f(buf)?;
            }
        } else if let Some(ref mut s) = sink {
            f(s)?;
        }
        Ok(())
    }

    pub const MIXED: &'static str = "multipart/mixed";
    pub const ALTERNATIVE: &'static str = "multipart/alternative";
    pub const DIGEST: &'static str = "multipart/digest";
    pub const PARALLEL: &'static str = "multipart/parallel";
    pub const FORM: &'static str = "multipart/form-data";

    const COLONSPACE: &'static [u8] = b": ";
    const CRLF: &'static [u8] = b"\r\n";
    const DASHDASH: &'static [u8] = b"--";

    fn append_quoted_string(s: &mut String, key: &str) {
        s.push('\"');
        for ch in key.chars() {
            match ch {
                '\n' => s.push_str("%0A"),
                '\r' => s.push_str("%0D"),
                '\"' => s.push_str("%22"),
                _ => s.push(ch),
            }
        }
        s.push('\"');
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub headers: Option<Headers>,
    pub body: RequestBody,
}

impl Part {
    pub fn create(body: RequestBody) -> Self {
        Self::create_with_headers(None, body)
    }

    pub fn create_with_headers(headers: Option<Headers>, body: RequestBody) -> Self {
        if let Some(ref h) = headers {
            if h.get("Content-Type").is_some() {
                panic!("Unexpected header: Content-Type");
            }
            if h.get("Content-Length").is_some() {
                panic!("Unexpected header: Content-Length");
            }
        }
        Part { headers, body }
    }

    pub fn create_form_data(name: &str, value: &str) -> Self {
        Self::create_form_data_with_body(name, None, RequestBody::to_request_body(value))
    }

    pub fn create_form_data_with_body(name: &str, filename: Option<&str>, body: RequestBody) -> Self {
        let mut disposition = String::from("form-data; name=");
        MultipartBody::append_quoted_string(&mut disposition, name);

        if let Some(fname) = filename {
            disposition.push_str("; filename=");
            MultipartBody::append_quoted_string(&mut disposition, fname);
        }

        let headers = Headers::builder()
            .add_unsafe_non_ascii("Content-Disposition", disposition)
            .build();

        Self::create_with_headers(Some(headers), body)
    }
}


impl Builder {
    pub fn new(boundary: Option<String>) -> Self {
        let b = boundary.unwrap_or_else(|| Uuid::new_v4().to_string());
        Self {
            boundary: ByteString::encode_utf8(&b),
            r#type: MediaType::parse(MultipartBody::MIXED).expect("Invalid default type"),
            parts: Vec::new(),
        }
    }

    pub fn set_type(mut self, r#type: MediaType) -> Self {
        if r#type.r#type() != "multipart" {
            panic!("multipart != {}", r#type);
        }
        self.r#type = r#type;
        self
    }

    pub fn add_part(self, body: RequestBody) -> Self {
        self.add_part_with_headers(None, body)
    }

    pub fn add_part_with_headers(mut self, headers: Option<Headers>, body: RequestBody) -> Self {
        self.parts.push(Part::create_with_headers(headers, body));
        self
    }

    pub fn add_form_data_part(self, name: &str, value: &str) -> Self {
        self.add_part_instance(Part::create_form_data(name, value))
    }

    pub fn add_form_data_part_with_body(self, name: &str, filename: Option<&str>, body: RequestBody) -> Self {
        self.add_part_instance(Part::create_form_data_with_body(name, filename, body))
    }

    pub fn add_part_instance(mut self, part: Part) -> Self {
        self.parts.push(part);
        self
    }

    pub fn build(self) -> MultipartBody {
        if self.parts.is_empty() {
            panic!("Multipart body must have at least one part.");
        }
        MultipartBody::new(self.boundary, self.r#type, self.parts)
    }
}
