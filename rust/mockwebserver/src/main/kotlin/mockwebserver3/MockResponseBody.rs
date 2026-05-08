use std::io::{Result, Write};

// The body of a [MockResponse].
//
// Unlike [okhttp3::ResponseBody], this trait is designed to be implemented by writers and not
// called by readers.
pub trait MockResponseBody {
    // The length of this response in bytes, or -1 if unknown.
    fn content_length(&self) -> i64;

    // Writes the response body to the provided sink.
    // 
    // In Rust, `BufferedSink` from Okio is typically represented by a type implementing `std::io::Write`.
    fn write_to(&self, sink: &mut dyn Write) -> Result<()>;
}