use std::io::{self, Read, Write};
use okio::{Buffer, ByteString};
use flate2::write::ZlibEncoder;
use flate2::Compression;

// Mocking the internal components as they are dependencies of the test
// In a real project, these would be imported from the actual implementation modules.


impl MessageDeflater {
    pub fn new(no_context_takeover: bool) -> Self {
        Self {
            closed: false,
            no_context_takeover,
        }
    }

    pub fn deflate(&mut self, buffer: &mut Buffer) -> io::Result<()> {
        if self.closed {
            return Err(io::Error::new(io::ErrorKind::Other, "Deflater closed"));
        }
        
        let mut input = buffer.clone();
        let mut output = Buffer::new();
        
        // In OkHttp, MessageDeflater uses a specific Zlib configuration.
        // no_context_takeover corresponds to the 'nowrap' or reset behavior.
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        
        // Simulate the deflate process
        let data = input.read_byte_string();
        encoder.write_all(&data)?;
        encoder.finish().map(|v| {
            output.write_all(&v).unwrap();
        })?;
        
        // The actual OkHttp implementation modifies the buffer in place or replaces it.
        // For the test, we simulate the result.
        buffer.clear();
        buffer.write_all(&output.read_byte_string());
        Ok(())
    }

    pub fn close(&mut self) {
        self.closed = true;
    }
}


impl MessageInflater {
    pub fn new(no_context_takeover: bool) -> Self {
        Self {
            closed: false,
            no_context_takeover,
        }
    }

    pub fn inflate(&mut self, buffer: &mut Buffer) -> io::Result<()> {
        if self.closed {
            return Err(io::Error::new(io::ErrorKind::Other, "Inflater closed"));
        }
        
        // This is a simplified simulation of the OkHttp MessageInflater logic
        // which handles the 0x00 suffix and Zlib decompression.
        let mut input = buffer.clone();
        let compressed_data = input.read_byte_string();
        
        // In reality, this would use flate2::read::ZlibDecoder
        // and handle the specific WebSocket compression framing.
        let mut decoder = flate2::read::ZlibDecoder::new(&compressed_data[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        
        buffer.clear();
        buffer.write_all(&decompressed);
        Ok(())
    }

    pub fn close(&mut self) {
        self.closed = true;
    }
}

// Helper trait to implement the private extension functions from Kotlin
trait MessageDeflaterExt {
    fn deflate_bytes(&mut self, byte_string: ByteString) -> io::Result<ByteString>;
}

impl MessageDeflaterExt for MessageDeflater {
    fn deflate_bytes(&mut self, byte_string: ByteString) -> io::Result<ByteString> {
        let mut buffer = Buffer::new();
        buffer.write_all(&byte_string)?;
        self.deflate(&mut buffer)?;
        Ok(buffer.read_byte_string())
    }
}

trait MessageInflaterExt {
    fn inflate_bytes(&mut self, byte_string: ByteString) -> io::Result<ByteString>;
}

impl MessageInflaterExt for MessageInflater {
    fn inflate_bytes(&mut self, byte_string: ByteString) -> io::Result<ByteString> {
        let mut buffer = Buffer::new();
        buffer.write_all(&byte_string)?;
        self.inflate(&mut buffer)?;
        Ok(buffer.read_byte_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::ws::MessageInflater::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::ws::MessageDeflater::*;

    #[test]
    fn test_inflate_golden_value() {
        let mut inflater = MessageInflater::new(false);
        let message = ByteString::decode_hex("f248cdc9c957c8cc4bcb492cc9cccf530400").unwrap();
        let result = inflater.inflate_bytes(message).unwrap();
        assert_eq!(result, ByteString::encode_utf8("Hello inflation!"));
    }

    #[test]
    fn test_inflate_returns_finished_before_bytes_read_reaches_input_length() {
        let mut inflater = MessageInflater::new(false);
        let message = ByteString::decode_hex("53621260020000").unwrap();
        let result = inflater.inflate_bytes(message).unwrap();
        assert_eq!(result, ByteString::decode_hex("22021002").unwrap());
    }

    #[test]
    fn test_deflate_golden_value() {
        let mut deflater = MessageDeflater::new(false);
        let deflated = deflater.deflate_bytes(ByteString::encode_utf8("Hello deflate!")).unwrap();
        // Note: Exact hex may vary based on zlib version/implementation, 
        // but we preserve the logic.
        assert_eq!(deflated.hex(), "f248cdc9c95748494dcb492c49550400");
    }

    #[test]
    fn test_inflate_deflate() {
        let mut deflater = MessageDeflater::new(false);
        let mut inflater = MessageInflater::new(false);

        let golden_value = ByteString::encode_utf8(&"Hello deflate!".repeat(100));

        let deflated = deflater.deflate_bytes(golden_value.clone()).unwrap();
        assert!(deflated.len() < golden_value.len());
        let inflated = inflater.inflate_bytes(deflated).unwrap();

        assert_eq!(inflated, golden_value);
    }

    #[test]
    fn test_inflate_deflate_empty_message() {
        let mut deflater = MessageDeflater::new(false);
        let mut inflater = MessageInflater::new(false);

        let golden_value = ByteString::encode_utf8("");

        let deflated = deflater.deflate_bytes(golden_value.clone()).unwrap();
        assert_eq!(deflated, ByteString::decode_hex("00").unwrap());
        let inflated = inflater.inflate_bytes(deflated).unwrap();

        assert_eq!(inflated, golden_value);
    }

    #[test]
    fn test_inflate_deflate_with_context_takeover() {
        let mut deflater = MessageDeflater::new(false);
        let mut inflater = MessageInflater::new(false);

        let golden_value1 = ByteString::encode_utf8(&"Hello deflate!".repeat(100));
        let deflated_value1 = deflater.deflate_bytes(golden_value1.clone()).unwrap();
        assert_eq!(inflater.inflate_bytes(deflated_value1.clone()).unwrap(), golden_value1);

        let golden_value2 = ByteString::encode_utf8(&"Hello deflate?".repeat(100));
        let deflated_value2 = deflater.deflate_bytes(golden_value2.clone()).unwrap();
        assert_eq!(inflater.inflate_bytes(deflated_value2.clone()).unwrap(), golden_value2);

        assert!(deflated_value2.len() < deflated_value1.len());
    }

    #[test]
    fn test_inflate_deflate_with_no_context_takeover() {
        let mut deflater = MessageDeflater::new(true);
        let mut inflater = MessageInflater::new(true);

        let golden_value1 = ByteString::encode_utf8(&"Hello deflate!".repeat(100));
        let deflated_value1 = deflater.deflate_bytes(golden_value1.clone()).unwrap();
        assert_eq!(inflater.inflate_bytes(deflated_value1.clone()).unwrap(), golden_value1);

        let golden_value2 = ByteString::encode_utf8(&"Hello deflate!".repeat(100));
        let deflated_value2 = deflater.deflate_bytes(golden_value2.clone()).unwrap();
        assert_eq!(inflater.inflate_bytes(deflated_value2.clone()).unwrap(), golden_value2);

        assert_eq!(deflated_value2, deflated_value1);
    }

    #[test]
    fn test_deflate_after_close() {
        let mut deflater = MessageDeflater::new(true);
        deflater.close();

        let result = deflater.deflate_bytes(ByteString::encode_utf8("Hello deflate!"));
        assert!(result.is_err());
    }

    #[test]
    fn test_inflate_after_close() {
        let mut inflater = MessageInflater::new(false);
        inflater.close();

        let result = inflater.inflate_bytes(ByteString::decode_hex("f240e30300").unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_inflate_golden_value_in_buffer_that_has_been_fragmented() {
        let mut inflater = MessageInflater::new(false);
        let mut buffer = Buffer::new();
        buffer.write_all(&ByteString::decode_hex("f248cdc9c957c8cc4bcb492cc9cccf530400").unwrap()).unwrap();
        
        // fragment_buffer simulation: in Rust, we just use the buffer as is 
        // since Buffer is already a segmented structure.
        inflater.inflate(&mut buffer).unwrap();
        assert_eq!(buffer.read_utf8(), "Hello inflation!");
    }

    #[test]
    fn test_deflated_data_has_too_many_bytes() {
        let mut inflater = MessageInflater::new(true);
        let mut buffer = Buffer::new();

        let message1 = ByteString::encode_utf8("hello");
        let message2 = ByteString::encode_utf8("hello 2");

        {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&message1).unwrap();
            let compressed = encoder.finish().unwrap();
            buffer.write_all(&compressed).unwrap();
        }
        buffer.write_all(&[0x00]).unwrap();
        // Trailing data
        buffer.write_all(&vec![0u8; 8192]).unwrap();
        
        inflater.inflate(&mut buffer).unwrap();
        assert_eq!(buffer.read_byte_string(), message1);

        {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&message2).unwrap();
            let compressed = encoder.finish().unwrap();
            buffer.write_all(&compressed).unwrap();
        }
        buffer.write_all(&[0x00]).unwrap();
        inflater.inflate(&mut buffer).unwrap();
        assert_eq!(buffer.read_byte_string(), message2);
    }
}