use std::io::{Result, Write};
use std::cmp::min;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;

// MockWebServerSocket trait to represent the socket dependency.
// Based on the Kotlin source, it must provide a method to sleep while the socket is open.
pub trait MockWebServerSocket: Send + Sync {
    fn sleep_while_open(&self, nanos: i64);
}

// Sink trait equivalent to okio.Sink.
pub trait Sink: Write {
    fn write_buffer(&mut self, source: &mut Vec<u8>, byte_count: i64) -> Result<()>;
    fn flush_sink(&mut self) -> Result<()>;
}

// A sink that sleeps [period_delay_nanos] every [bytes_per_period] bytes. 
// Unlike [okio.Throttler], this permits any interval to be used.
pub struct ThrottledSink<S: Sink, Sock: MockWebServerSocket> {
    socket: Sock,
    delegate: S,
    bytes_per_period: i64,
    period_delay_nanos: i64,
    bytes_written_since_last_delay: i64,
}

impl<S: Sink, Sock: MockWebServerSocket> ThrottledSink<S, Sock> {
    pub fn new(socket: Sock, delegate: S, bytes_per_period: i64, period_delay_nanos: i64) -> Self {
        Self {
            socket,
            delegate,
            bytes_per_period,
            period_delay_nanos,
            bytes_written_since_last_delay: 0,
        }
    }
}

impl<S: Sink, Sock: MockWebServerSocket> Sink for ThrottledSink<S, Sock> {
    fn write_buffer(&mut self, source: &mut Vec<u8>, byte_count: i64) -> Result<()> {
        let mut bytes_left = byte_count;

        while bytes_left > 0 {
            if self.bytes_written_since_last_delay == self.bytes_per_period {
                self.flush_sink()?;
                self.socket.sleep_while_open(self.period_delay_nanos);
                self.bytes_written_since_last_delay = 0;
            }

            let to_write = min(bytes_left, self.bytes_per_period - self.bytes_written_since_last_delay);
            self.bytes_written_since_last_delay += to_write;
            bytes_left -= to_write;
            self.delegate.write_buffer(source, to_write)?;
        }
        Ok(())
    }

    fn flush_sink(&mut self) -> Result<()> {
        self.delegate.flush_sink()
    }
}

impl<S: Sink, Sock: MockWebServerSocket> Write for ThrottledSink<S, Sock> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        // To maintain behavioral correctness with the Kotlin 'Sink by delegate' 
        // and the specific 'write(Buffer, Long)' override, we delegate the standard 
        // Write trait to the underlying sink.
        self.delegate.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.delegate.flush()
    }
}