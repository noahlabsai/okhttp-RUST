use std::io::{self, Write};
use std::sync::Arc;

/// Mocking okio::Buffer for the purpose of this translation.
/// In a real okhttp Rust port, this would be a shared Buffer type.
pub struct Buffer(Vec<u8>);

impl Buffer {
    pub fn skip(&mut self, byte_count: i64) {
        // In a real Buffer, this would advance the read pointer.
        // Here we simulate by removing bytes from the front.
        let count = byte_count as usize;
        if self.0.len() > count {
            self.0.drain(0..count);
        } else {
            self.0.clear();
        }
    }
}

/// Mocking okio::Sink.
pub trait Sink: Write {
    fn write_buffer(&mut self, source: &mut Buffer, byte_count: i64) -> io::Result<()>;
}

/// Mocking okio::ForwardingSink.
/// In Rust, this is typically implemented as a wrapper struct.
pub struct ForwardingSink {
    delegate: Box<dyn Sink>,
}

impl ForwardingSink {
    pub fn new(delegate: Box<dyn Sink>) -> Self {
        Self { delegate }
    }
}

impl Write for ForwardingSink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.delegate.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.delegate.flush()
    }
}

impl Sink for ForwardingSink {
    fn write_buffer(&mut self, source: &mut Buffer, byte_count: i64) -> io::Result<()> {
        self.delegate.write_buffer(source, byte_count)
    }
}

/// A sink that never throws IOExceptions, even if the underlying sink does.
pub struct FaultHidingSink {
    delegate: ForwardingSink,
    on_exception: Arc<dyn Fn(io::Error) + Send + Sync>,
    has_errors: bool,
}

impl FaultHidingSink {
    pub fn new(
        delegate: Box<dyn Sink>,
        on_exception: Arc<dyn Fn(io::Error) + Send + Sync>,
    ) -> Self {
        Self {
            delegate: ForwardingSink::new(delegate),
            on_exception,
            has_errors: false,
        }
    }

    pub fn on_exception(&self) -> Arc<dyn Fn(io::Error) + Send + Sync> {
        Arc::clone(&self.on_exception)
    }
}

impl Sink for FaultHidingSink {
    fn write_buffer(&mut self, source: &mut Buffer, byte_count: i64) -> io::Result<()> {
        if self.has_errors {
            source.skip(byte_count);
            return Ok(());
        }

        if let Err(e) = self.delegate.write_buffer(source, byte_count) {
            self.has_errors = true;
            (self.on_exception)(e);
        }
        Ok(())
    }
}

impl Write for FaultHidingSink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // The Kotlin source specifically overrides 'write(source: Buffer, byteCount: Long)'.
        // Standard Write::write is handled by the delegate or treated as a fault-hiding operation.
        if self.has_errors {
            return Ok(buf.len());
        }

        match self.delegate.write(buf) {
            Ok(n) => Ok(n),
            Err(e) => {
                self.has_errors = true;
                (self.on_exception)(e);
                Ok(0) // Return 0 to indicate no bytes written without throwing
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.has_errors {
            return Ok(());
        }

        if let Err(e) = self.delegate.flush() {
            self.has_errors = true;
            (self.on_exception)(e);
        }
        Ok(())
    }
}

impl Drop for FaultHidingSink {
    fn drop(&mut self) {
        // Kotlin's close() is called explicitly or via 'use'. 
        // In Rust, we implement the logic in a close method or Drop.
        // Since the Kotlin class overrides close(), we simulate that behavior.
        // Note: In a real production system, we'd use a dedicated close() method 
        // because Drop cannot return a Result or easily handle the logic if 
        // the user wants to catch the error.
    }
}

impl FaultHidingSink {
    pub fn close(&mut self) -> io::Result<()> {
        // We use a manual close to mirror Kotlin's override fun close()
        // because we need to access the delegate's close logic.
        // Since ForwardingSink doesn't have a specific close() in the trait 
        // (it's usually part of the Sink/Write trait in okio), we call flush or 
        // a hypothetical close.
        
        // Assuming Sink/Write in this context handles closing via the underlying stream.
        // For the sake of the translation, we treat the delegate's flush/write as the target.
        if let Err(e) = self.delegate.flush() {
            self.has_errors = true;
            (self.on_exception)(e);
        }
        Ok(())
    }
}