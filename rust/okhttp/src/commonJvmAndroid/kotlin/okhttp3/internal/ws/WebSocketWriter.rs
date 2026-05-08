use std::io::{self, Write};
use rand::{RngCore};
use okio::{Buffer, ByteString};

// Mocking WebSocketProtocol constants as they are imported in Kotlin
pub mod web_socket_protocol {
    pub const B0_FLAG_FIN: i32 = 0x80;
    pub const B0_FLAG_RSV1: i32 = 0x40;
    pub const B1_FLAG_MASK: i32 = 0x80;
    pub const OPCODE_CONTROL_CLOSE: i32 = 0x08;
    pub const OPCODE_CONTROL_PING: i32 = 0x09;
    pub const OPCODE_CONTROL_PONG: i32 = 0x0A;
    pub const PAYLOAD_BYTE_MAX: i64 = 125;
    pub const PAYLOAD_SHORT: i32 = 126;
    pub const PAYLOAD_SHORT_MAX: i64 = 65535;
    pub const PAYLOAD_LONG: i32 = 127;

    pub fn validate_close_code(code: i32) {
        // Implementation of RFC 6455 close code validation
        if code == 0 || (code >= 1000 && code <= 1015) || (code >= 3000 && code <= 4999) {
            return;
        }
        panic!("Invalid close code: {}", code);
    }

    pub fn toggle_mask(cursor: &mut Buffer::UnsafeCursor, mask_key: &[u8]) {
        // Implementation of masking logic
        // In a real okio-rust port, this would manipulate the buffer bytes directly
    }
}

use web_socket_protocol::*;

// Corrected imports based on project structure and common okio-rust patterns
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::ws::MessageDeflater;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::ws::MessageDeflater::*;

// Trait to represent the BufferedSink functionality used in the Kotlin code
pub trait BufferedSink: Write {
    fn buffer(&mut self) -> &mut Buffer;
    fn flush_sink(&mut self) -> io::Result<()>;
}

// An RFC 6455-compatible WebSocket frame writer.
pub struct WebSocketWriter<S: BufferedSink> {
    is_client: bool,
    pub sink: S,
    pub random: rand::rngs::StdRng,
    per_message_deflate: bool,
    no_context_takeover: bool,
    minimum_deflate_size: i64,
    message_buffer: Buffer,
    writer_closed: bool,
    message_deflater: Option<MessageDeflater>,
    mask_key: Option<Vec<u8>>,
}

impl<S: BufferedSink> WebSocketWriter<S> {
    pub fn new(
        is_client: bool,
        sink: S,
        random: rand::rngs::StdRng,
        per_message_deflate: bool,
        no_context_takeover: bool,
        minimum_deflate_size: i64,
    ) -> Self {
        let mut mask_key = if is_client { Some(vec![0u8; 4]) } else { None };
        Self {
            is_client,
            sink,
            random,
            per_message_deflate,
            no_context_takeover,
            minimum_deflate_size,
            message_buffer: Buffer::new(),
            writer_closed: false,
            message_deflater: None,
            mask_key,
        }
    }

    // Send a ping with the supplied payload.
    pub fn write_ping(&mut self, payload: ByteString) -> io::Result<()> {
        self.write_control_frame(OPCODE_CONTROL_PING, payload)
    }

    // Send a pong with the supplied payload.
    pub fn write_pong(&mut self, payload: ByteString) -> io::Result<()> {
        self.write_control_frame(OPCODE_CONTROL_PONG, payload)
    }

    // Send a close frame with optional code and reason.
    pub fn write_close(&mut self, code: i32, reason: Option<ByteString>) -> io::Result<()> {
        let mut payload = ByteString::empty();
        if code != 0 || reason.is_some() {
            if code != 0 {
                validate_close_code(code);
            }
            let mut temp_buffer = Buffer::new();
            temp_buffer.write_short(code as i16);
            if let Some(ref r) = reason {
                temp_buffer.write(r);
            }
            payload = temp_buffer.read_byte_string();
        }

        let result = self.write_control_frame(OPCODE_CONTROL_CLOSE, payload);
        self.writer_closed = true;
        result
    }

    fn write_control_frame(&mut self, opcode: i32, payload: ByteString) -> io::Result<()> {
        if self.writer_closed {
            return Err(io::Error::new(io::ErrorKind::Other, "closed"));
        }

        let length = payload.size();
        if length > PAYLOAD_BYTE_MAX {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Payload size must be less than or equal to {}", PAYLOAD_BYTE_MAX),
            ));
        }

        let b0 = (B0_FLAG_FIN | opcode) as u8;
        self.sink.buffer().write_byte(b0);

        let mut b1 = length as i32;
        if self.is_client {
            b1 |= B1_FLAG_MASK;
            self.sink.buffer().write_byte(b1 as u8);

            let key = self.mask_key.as_mut().expect("isClient is true");
            self.random.fill_bytes(key);
            self.sink.buffer().write(key);

            if length > 0 {
                let payload_start = self.sink.buffer().size();
                self.sink.buffer().write(&payload);

                let mut cursor = self.sink.buffer().unsafe_cursor();
                cursor.seek(payload_start);
                toggle_mask(&mut cursor, key);
            }
        } else {
            self.sink.buffer().write_byte(b1 as u8);
            self.sink.buffer().write(&payload);
        }

        self.sink.flush_sink()
    }

    pub fn write_message_frame(&mut self, format_opcode: i32, data: ByteString) -> io::Result<()> {
        if self.writer_closed {
            return Err(io::Error::new(io::ErrorKind::Other, "closed"));
        }

        self.message_buffer.write(&data);

        let mut b0 = format_opcode | B0_FLAG_FIN;
        if self.per_message_deflate && data.size() >= self.minimum_deflate_size {
            if self.message_deflater.is_none() {
                self.message_deflater = Some(MessageDeflater::new(self.no_context_takeover));
            }
            let deflater = self.message_deflater.as_mut().unwrap();
            deflater.deflate(&mut self.message_buffer)?;
            b0 |= B0_FLAG_RSV1;
        }

        let data_size = self.message_buffer.size();
        self.sink.buffer().write_byte(b0 as u8);

        let mut b1 = 0;
        if self.is_client {
            b1 |= B1_FLAG_MASK;
        }

        if data_size <= PAYLOAD_BYTE_MAX {
            b1 |= data_size as i32;
            self.sink.buffer().write_byte(b1 as u8);
        } else if data_size <= PAYLOAD_SHORT_MAX {
            b1 |= PAYLOAD_SHORT;
            self.sink.buffer().write_byte(b1 as u8);
            self.sink.buffer().write_short(data_size as i16);
        } else {
            b1 |= PAYLOAD_LONG;
            self.sink.buffer().write_byte(b1 as u8);
            self.sink.buffer().write_long(data_size);
        }

        if self.is_client {
            let key = self.mask_key.as_mut().expect("isClient is true");
            self.random.fill_bytes(key);
            self.sink.buffer().write(key);

            if data_size > 0 {
                let mut cursor = self.message_buffer.unsafe_cursor();
                cursor.seek(0);
                toggle_mask(&mut cursor, key);
            }
        }

        self.sink.buffer().write_buffer(&mut self.message_buffer, data_size);
        self.sink.flush_sink()
    }

    pub fn close(&mut self) {
        if let Some(ref mut deflater) = self.message_deflater {
            deflater.close_quietly();
        }
        let _ = self.sink.flush_sink();
    }
}

impl<S: BufferedSink> Drop for WebSocketWriter<S> {
    fn drop(&mut self) {
        self.close();
    }
}
