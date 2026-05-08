use std::io::{Result as IoResult, Write};

// Mocking necessary okhttp3 types and internal utilities as they are dependencies
// In a real project, these would be imported from the respective modules.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaType(pub String);

impl MediaType {
    pub fn new(value: &str) -> Self {
        MediaType(value.to_string())
    }
}

pub trait RequestBody {
    fn content_type(&self) -> MediaType;
    fn content_length(&self) -> i64;
    fn write_to(&self, sink: &mut dyn Write) -> IoResult<()>;
}

// Internal utility mocks for URL encoding/decoding
mod internal {
    pub const FORM_ENCODE_SET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~";

    pub fn canonicalize_with_charset(
        input: &str,
        _encode_set: &[u8],
        _already_encoded: bool,
        _plus_is_space: bool,
        _charset: Option<&str>,
    ) -> String {
        // Simplified implementation of canonicalizeWithCharset
        input.to_string()
    }

    pub fn percent_decode(input: &str, plus_is_space: bool) -> String {
        // Simplified implementation of percentDecode
        if plus_is_space {
            input.replace('+', " ")
        } else {
            input.to_string()
        }
    }
}

use internal::{canonicalize_with_charset, percent_decode, FORM_ENCODE_SET};
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;

pub struct FormBody {
    encoded_names: Vec<String>,
    encoded_values: Vec<String>,
}

impl FormBody {
    // Internal constructor
    pub(crate) fn new(encoded_names: Vec<String>, encoded_values: Vec<String>) -> Self {
        Self {
            encoded_names,
            encoded_values,
        }
    }

    // The number of key-value pairs in this form-encoded body.
    pub fn size(&self) -> i32 {
        self.encoded_names.len() as i32
    }

    pub fn encoded_name(&self, index: usize) -> &str {
        &self.encoded_names[index]
    }

    pub fn name(&self, index: usize) -> String {
        percent_decode(self.encoded_name(index), true)
    }

    pub fn encoded_value(&self, index: usize) -> &str {
        &self.encoded_values[index]
    }

    pub fn value(&self, index: usize) -> String {
        percent_decode(self.encoded_value(index), true)
    }

    fn write_or_count_bytes(&self, sink: Option<&mut dyn Write>, count_bytes: bool) -> i64 {
        let mut byte_count = 0i64;

        // In Rust, we use a Vec<u8> as a buffer to mimic okio.Buffer
        let mut buffer = Vec::new();

        for i in 0..self.encoded_names.len() {
            if i > 0 {
                buffer.push(b'&');
            }
            buffer.extend_from_slice(self.encoded_names[i].as_bytes());
            buffer.push(b'=');
            buffer.extend_from_slice(self.encoded_values[i].as_bytes());
        }

        if count_bytes {
            byte_count = buffer.len() as i64;
        } else if let Some(s) = sink {
            // If not counting, write the buffer to the sink
            let _ = s.write_all(&buffer);
            byte_count = buffer.len() as i64;
        }

        byte_count
    }

    fn content_type_static() -> MediaType {
        MediaType::new("application/x-www-form-urlencoded")
    }

    pub struct Builder {
        charset: Option<String>,
        names: Vec<String>,
        values: Vec<String>,
    }

    impl Builder {
        pub fn new(charset: Option<String>) -> Self {
            Self {
                charset,
                names: Vec::new(),
                values: Vec::new(),
            }
        }

        pub fn add(mut self, name: &str, value: &str) -> Self {
            let charset_ref = self.charset.as_deref();

            self.names.push(canonicalize_with_charset(
                name,
                FORM_ENCODE_SET,
                false, // alreadyEncoded
                false, // plusIsSpace
                charset_ref,
            ));

            self.values.push(canonicalize_with_charset(
                value,
                FORM_ENCODE_SET,
                false, // alreadyEncoded
                false, // plusIsSpace
                charset_ref,
            ));

            self
        }

        pub fn add_encoded(mut self, name: &str, value: &str) -> Self {
            let charset_ref = self.charset.as_deref();

            self.names.push(canonicalize_with_charset(
                name,
                FORM_ENCODE_SET,
                true, // alreadyEncoded
                true, // plusIsSpace
                charset_ref,
            ));

            self.values.push(canonicalize_with_charset(
                value,
                FORM_ENCODE_SET,
                true, // alreadyEncoded
                true, // plusIsSpace
                charset_ref,
            ));

            self
        }

        pub fn build(self) -> FormBody {
            FormBody::new(self.names, self.values)
        }
    }
}

impl RequestBody for FormBody {
    fn content_type(&self) -> MediaType {
        Self::content_type_static()
    }

    fn content_length(&self) -> i64 {
        self.write_or_count_bytes(None, true)
    }

    fn write_to(&self, sink: &mut dyn Write) -> IoResult<()> {
        self.write_or_count_bytes(Some(sink), false);
        Ok(())
    }
}
