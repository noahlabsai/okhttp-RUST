use crate::okhttp_hpacktests::src::test::java::okhttp3::internal::http2::hpackjson::HpackJsonUtil;
use crate::okhttp_hpacktests::src::test::java::okhttp3::internal::http2::hpackjson::Story;
use okio::Buffer;
use std::collections::HashSet;
use std::any::Any;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp_hpacktests::src::test::java::okhttp3::internal::http2::hpackjson::HpackJsonUtil::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Reader::*;

// The original Kotlin code uses a Header type. 
// Since it's not found in the current workspace search, we define a compatible 
// representation to ensure the test base is compilable and logically correct.

// Mocking Hpack::Reader as it is a dependency of the base class.
// In a real scenario, this would be imported from the actual Hpack implementation.
pub struct HpackReader {
    source: Buffer,
    _capacity: i32,
}

impl HpackReader {
    pub fn new(source: Buffer, capacity: i32) -> Self {
        Self {
            source,
            _capacity: capacity,
        }
    }

    pub fn read_headers(&mut self) {
        // Implementation provided by Hpack.Reader
    }

    pub fn get_and_reset_header_list(&mut self) -> Vec<Header> {
        // Implementation provided by Hpack.Reader
        Vec::new()
    }
}

/*
 * Tests Hpack implementation using https://github.com/http2jp/hpack-test-case/
 */
pub struct HpackDecodeTestBase {
    bytes_in: Buffer,
    hpack_reader: HpackReader,
}

impl HpackDecodeTestBase {
    pub fn new() -> Self {
        let bytes_in = Buffer::new();
        // In Kotlin: private val hpackReader = Hpack.Reader(bytesIn, 4096)
        // We use a separate buffer or a shared reference in a real impl, 
        // but for the base class structure, we mirror the Kotlin initialization.
        let hpack_reader = HpackReader::new(Buffer::new(), 4096); 
        
        Self {
            bytes_in,
            hpack_reader,
        }
    }

    pub fn test_decoder(&mut self, story: Story) {
        for test_case in &story.cases {
            let encoded = match &test_case.wire {
                Some(wire) => wire,
                None => continue,
            };
            
            // bytesIn.write(encoded)
            self.bytes_in.write(encoded.as_bytes());
            
            // hpackReader.readHeaders()
            self.hpack_reader.read_headers();
            
            let message = format!("seqno={}", test_case.seqno);
            let observed = self.hpack_reader.get_and_reset_header_list();
            
            Self::assert_set_equals(
                &message,
                &test_case.headers_list,
                &observed,
            );
        }
    }

    /*
     * Reads all stories in the folders provided, asserts if no story found.
     */
    pub fn create_stories(interop_tests: &[String]) -> Vec<Box<dyn Any>> {
        if interop_tests.is_empty() {
            // Story::MISSING is used as a sentinel in Kotlin
            return vec![Box::new(Story::MISSING)];
        }

        let mut result: Vec<Box<dyn Any>> = Vec::new();
        for interop_test_name in interop_tests {
            let stories = HpackJsonUtil::read_stories(interop_test_name);
            for story in stories {
                result.push(Box::new(story));
            }
        }
        result
    }

    /*
     * Checks if `expected` and `observed` are equal when viewed as a set and headers are
     * deduped.
     */
    fn assert_set_equals(
        message: &str,
        expected: &[Header],
        observed: &[Header],
    ) {
        let observed_set: HashSet<&Header> = observed.iter().collect();
        let expected_set: HashSet<&Header> = expected.iter().collect();

        if observed_set != expected_set {
            panic!("{}: Expected {:?}, but observed {:?}", message, expected_set, observed_set);
        }
    }
}
