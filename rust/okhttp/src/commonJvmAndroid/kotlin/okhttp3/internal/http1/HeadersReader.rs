use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::HEADER_LIMIT;
// Assuming BufferedSource is a trait or struct provided by the okio translation
// In a real project, this would be imported from the okio crate translation.
use crate::okio::BufferedSource;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::HeadersCommon::*;

/*
 * Parse all headers delimited by "\r\n" until an empty line. This throws if headers exceed 256 KiB.
 */
pub struct HeadersReader<S: BufferedSource> {
    pub source: S,
    header_limit: i64,
}

impl<S: BufferedSource> HeadersReader<S> {
    pub fn new(source: S) -> Self {
        Self {
            source,
            header_limit: HEADER_LIMIT,
        }
    }

    /* Read a single line counted against the header size limit. */
    pub fn read_line(&mut self) -> String {
        // read_utf8_line_strict is expected to be part of the BufferedSource trait/impl
        let line = self.source.read_utf8_line_strict(self.header_limit);
        self.header_limit -= line.len() as i64;
        line
    }

    /* Reads headers or trailers. */
    pub fn read_headers(&mut self) -> Headers {
        let mut result = Builder::new();
        loop {
            let line = self.read_line();
            if line.is_empty() {
                break;
            }
            result.add_lenient(&line);
        }
        result.build()
    }
}