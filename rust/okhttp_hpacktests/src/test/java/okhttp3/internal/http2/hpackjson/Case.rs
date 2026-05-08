use std::collections::HashMap;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Header;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::ByteString;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Reader::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

/*
 * Representation of an individual case (set of headers and wire format). There are many cases for a
 * single story.  This class is used reflectively with Moshi to parse stories.
 */
#[derive(Debug, Clone, PartialEq)]
pub struct Case {
    pub seqno: i32,
    pub wire: Option<ByteString>,
    pub headers: Vec<HashMap<String, String>>,
}

impl Case {
    // Returns the headers as a list of Header objects.
    // This corresponds to the `headersList` getter in Kotlin.
    pub fn headers_list(&self) -> Vec<Header> {
        let mut result = Vec::with_capacity(self.headers.len());
        for input_header in &self.headers {
            // Kotlin: val (key, value) = inputHeader.entries.iterator().next()
            // This assumes each map in the list contains exactly one entry.
            if let Some((key, value)) = input_header.iter().next() {
                result.push(Header::from_strings(key, value));
            }
        }
        result
    }

    // Creates a new Case instance.
    pub fn new(seqno: i32, wire: Option<ByteString>, headers: Vec<HashMap<String, String>>) -> Self {
        Self {
            seqno,
            wire,
            headers,
        }
    }
}

impl Default for Case {
    fn default() -> Self {
        Self {
            seqno: 0,
            wire: None,
            headers: Vec::new(),
        }
    }
}

// Note: Kotlin's `clone()` is handled by the `Clone` derive macro in Rust.