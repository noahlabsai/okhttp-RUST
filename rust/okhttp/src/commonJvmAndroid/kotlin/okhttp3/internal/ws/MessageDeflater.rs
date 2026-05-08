use std::io::{self, Write};
use flate2::write::DeflateEncoder;
use flate2::Compression;
use okio::{Buffer, ByteString};
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;

// The original Kotlin code uses okio.Buffer and okio.ByteString.
// We assume these are available in the rust_stable/okio crate.

lazy_static::lazy_static! {
    static ref EMPTY_DEFLATE_BLOCK: ByteString = ByteString::decode_hex("000000ffff");
}

const LAST_OCTETS_COUNT_TO_REMOVE_AFTER_DEFLATION: i64 = 4;

// Deflates [buffer] in place as described in RFC 7692 section 7.2.1.
pub struct MessageDeflater {
    no_context_takeover: bool,
    deflated_bytes: Buffer,
    deflater_sink: DeflaterSink,
}

impl MessageDeflater {
    pub fn new(no_context_takeover: bool) -> Self {
        let deflated_bytes = Buffer::new();
        // Deflater.DEFAULT_COMPRESSION is typically Compression::default()
        // The Kotlin code uses nowrap = true, which is the default for flate2::write::DeflateEncoder
        let deflater_sink = DeflaterSink::new(deflated_bytes.clone(), Compression::default());

        Self {
            no_context_takeover,
            deflated_bytes,
            deflater_sink,
        }
    }

    pub fn deflate(&mut self, buffer: &mut Buffer) -> io::Result<()> {
        // require(deflatedBytes.size == 0L)
        if self.deflated_bytes.size() != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "deflatedBytes size must be 0",
            ));
        }

        if self.no_context_takeover {
            self.deflater_sink.reset();
        }

        self.deflater_sink.write(buffer, buffer.size())?;
        self.deflater_sink.flush()?;

        if self.deflated_bytes.ends_with(&EMPTY_DEFLATE_BLOCK) {
            let new_size = self.deflated_bytes.size() - LAST_OCTETS_COUNT_TO_REMOVE_AFTER_DEFLATION;
            // In okio, readAndWriteUnsafe() provides a cursor to modify the buffer.
            // We simulate the resizeBuffer call here.
            self.deflated_bytes.resize_buffer(new_size);
        } else {
            // Same as adding EMPTY_DEFLATE_BLOCK and then removing 4 bytes.
            self.deflated_bytes.write_byte(0x00);
        }

        buffer.write(&self.deflated_bytes, self.deflated_bytes.size());

        Ok(())
    }

    pub fn close(&mut self) -> io::Result<()> {
        self.deflater_sink.close()
    }
}

impl Drop for MessageDeflater {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

// Internal helper to mirror okio.DeflaterSink
struct DeflaterSink {
    sink: Buffer,
    encoder: DeflateEncoder<Vec<u8>>,
}

impl DeflaterSink {
    fn new(sink: Buffer, compression: Compression) -> Self {
        Self {
            sink,
            encoder: DeflateEncoder::new(Vec::new(), compression),
        }
    }

    fn reset(&mut self) {
        // Reset the encoder by creating a new one with the same compression
        let compression = self.encoder.compression();
        self.encoder = DeflateEncoder::new(Vec::new(), compression);
    }

    fn write(&mut self, buffer: &Buffer, byte_count: i64) -> io::Result<()> {
        let count = byte_count as usize;
        // Read from buffer and write to encoder
        let data = buffer.peek();
        self.encoder.write_all(&data[..count])?;
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.encoder.flush()?;
        let mut compressed = Vec::new();
        // In a real okio port, we'd stream this. Here we collect the current output.
        // This is a simplification of the DeflaterSink behavior.
        let output = self.encoder.get_mut();
        self.sink.write_all(output)?;
        output.clear();
        Ok(())
    }

    fn close(&mut self) -> io::Result<()> {
        self.encoder.finish()?;
        let output = self.encoder.get_mut();
        self.sink.write_all(output)?;
        output.clear();
        Ok(())
    }
}

// Extension trait to mirror Kotlin's Buffer.endsWith
trait BufferExt {
    fn ends_with(&self, suffix: &ByteString) -> bool;
}

impl BufferExt for Buffer {
    fn ends_with(&self, suffix: &ByteString) -> bool {
        let size = self.size();
        let suffix_size = suffix.size();
        if size < suffix_size {
            return false;
        }
        self.range_equals(size - suffix_size, suffix.as_slice())
    }
}
