use std::io::{Result, Write};
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;

// Mocking okio::Buffer for the purpose of this translation.
// In a real production environment, this would be a reference to the actual okio-rust implementation.

impl Buffer {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn skip(&mut self, byte_count: i64) {
        let count = byte_count as usize;
        if count > self.data.len() {
            self.data.clear();
        } else {
            self.data.drain(0..count);
        }
    }

    pub fn read_bytes(&mut self, count: i64) -> Vec<u8> {
        let count = count as usize;
        if count > self.data.len() {
            std::mem::take(&mut self.data)
        } else {
            self.data.drain(0..count).collect()
        }
    }
}

// Mocking okio::Sink for the purpose of this translation.
pub trait Sink: Write {
    fn write_buffer(&mut self, source: &mut Buffer, byte_count: i64) -> Result<()>;
}

// A sink that executes [trigger] after [trigger_byte_count] bytes are written, and then skips all
// subsequent bytes.
pub struct TriggerSink<S: Sink, F: FnMut()> {
    delegate: S,
    trigger_byte_count: i64,
    trigger: F,
    bytes_written: i64,
}

impl<S: Sink, F: FnMut()> TriggerSink<S, F> {
    pub fn new(delegate: S, trigger_byte_count: i64, trigger: F) -> Self {
        Self {
            delegate,
            trigger_byte_count,
            trigger,
            bytes_written: 0,
        }
    }

    // This method implements the core logic of the Kotlin `write` override.
    pub fn write_from_buffer(&mut self, source: &mut Buffer, byte_count: i64) -> Result<()> {
        if byte_count == 0 {
            return Ok(()); // Avoid double-triggering.
        }

        if self.bytes_written == self.trigger_byte_count {
            source.skip(byte_count);
            return Ok(());
        }

        let to_write = std::cmp::min(byte_count, self.trigger_byte_count - self.bytes_written);
        self.bytes_written += to_write;

        self.delegate.write_buffer(source, to_write)?;

        if self.bytes_written == self.trigger_byte_count {
            (self.trigger)();
        }

        source.skip(byte_count - to_write);
        Ok(())
    }
}

// Implementation of std::io::Write to satisfy the Sink requirement and general Rust usage.
impl<S: Sink, F: FnMut()> Write for TriggerSink<S, F> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        // To maintain behavioral correctness with the Kotlin source which uses okio::Buffer,
        // we wrap the slice in a temporary Buffer.
        let mut source = Buffer { data: buf.to_vec() };
        let byte_count = buf.len() as i64;
        
        self.write_from_buffer(&mut source, byte_count)?;
        
        // In the original Kotlin code, the 'write' method returns Unit.
        // For std::io::Write, we return the number of bytes processed from the input slice.
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        self.delegate.flush()
    }
}

// Delegate the Sink trait to the internal delegate.
impl<S: Sink, F: FnMut()> Sink for TriggerSink<S, F> {
    fn write_buffer(&mut self, source: &mut Buffer, byte_count: i64) -> Result<()> {
        self.write_from_buffer(source, byte_count)
    }
}