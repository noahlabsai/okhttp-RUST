use std::io::{self};
use std::time::Duration;
use okio::{Buffer, BufferedSource};
use okio::ByteString;

// Assuming these constants are defined in a WebSocketProtocol module
mod web_socket_protocol {
    pub const B0_FLAG_FIN: i32 = 0x80;
    pub const B0_FLAG_RSV1: i32 = 0x40;
    pub const B0_FLAG_RSV2: i32 = 0x20;
    pub const B0_FLAG_RSV3: i32 = 0x10;
    pub const B0_MASK_OPCODE: i32 = 0x0F;
    pub const B1_FLAG_MASK: i32 = 0x80;
    pub const B1_MASK_LENGTH: i32 = 0x7F;
    pub const CLOSE_NO_STATUS_CODE: i32 = 1000;
    pub const OPCODE_BINARY: i32 = 2;
    pub const OPCODE_CONTINUATION: i32 = 0;
    pub const OPCODE_CONTROL_CLOSE: i32 = 8;
    pub const OPCODE_CONTROL_PING: i32 = 9;
    pub const OPCODE_CONTROL_PONG: i32 = 10;
    pub const OPCODE_FLAG_CONTROL: i32 = 0x80;
    pub const OPCODE_TEXT: i32 = 1;
    pub const PAYLOAD_BYTE_MAX: i64 = 125;
    pub const PAYLOAD_SHORT: i32 = 126;
    pub const PAYLOAD_LONG: i32 = 127;

    pub fn toggle_mask(cursor: &mut Buffer::UnsafeCursor, mask_key: &[u8]) {
        // Implementation of mask toggling logic
    }

    pub fn close_code_exception_message(code: i32) -> Option<String> {
        // Implementation of close code validation
        None
    }
}

use web_socket_protocol::*;

// Corrected imports based on project structure and common OkHttp Rust translation patterns
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::ws::MessageInflater;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Reader::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::ws::MessageInflater::*;

pub trait FrameCallback {
    fn on_read_message_text(&mut self, text: String) -> io::Result<()>;
    fn on_read_message_bytes(&mut self, bytes: ByteString) -> io::Result<()>;
    fn on_read_ping(&mut self, payload: ByteString);
    fn on_read_pong(&mut self, payload: ByteString);
    fn on_read_close(&mut self, code: i32, reason: String);
}

pub struct WebSocketReader<S: BufferedSource> {
    is_client: bool,
    pub source: S,
    frame_callback: Box<dyn FrameCallback>,
    per_message_deflate: bool,
    no_context_takeover: bool,
    closed: bool,
    received_close_frame: bool,
    opcode: i32,
    frame_length: i64,
    is_final_frame: bool,
    is_control_frame: bool,
    reading_compressed_message: bool,
    control_frame_buffer: Buffer,
    message_frame_buffer: Buffer,
    message_inflater: Option<MessageInflater>,
    mask_key: Option<Vec<u8>>,
    mask_cursor: Option<Buffer::UnsafeCursor>,
}

impl<S: BufferedSource> WebSocketReader<S> {
    pub fn new(
        is_client: bool,
        source: S,
        frame_callback: Box<dyn FrameCallback>,
        per_message_deflate: bool,
        no_context_takeover: bool,
    ) -> Self {
        let mut mask_key = if is_client { None } else { Some(vec![0u8; 4]) };
        let mut mask_cursor = if is_client { None } else { Some(Buffer::UnsafeCursor::new()) };

        Self {
            is_client,
            source,
            frame_callback,
            per_message_deflate,
            no_context_takeover,
            closed: false,
            received_close_frame: false,
            opcode: 0,
            frame_length: 0,
            is_final_frame: false,
            is_control_frame: false,
            reading_compressed_message: false,
            control_frame_buffer: Buffer::new(),
            message_frame_buffer: Buffer::new(),
            message_inflater: None,
            mask_key,
            mask_cursor,
        }
    }

    pub fn process_next_frame(&mut self) -> io::Result<()> {
        if self.closed {
            return Err(io::Error::new(io::ErrorKind::Other, "closed"));
        }

        self.read_header()?;
        if self.is_control_frame {
            self.read_control_frame()?;
        } else {
            self.read_message_frame()?;
        }
        Ok(())
    }

    fn read_header(&mut self) -> io::Result<()> {
        if self.received_close_frame {
            return Err(io::Error::new(io::ErrorKind::Other, "closed"));
        }

        let timeout_before = self.source.timeout().timeout_nanos();
        self.source.timeout().clear_timeout();
        let b0 = match self.source.read_byte() {
            Ok(b) => (b as i32) & 0xff,
            Err(e) => {
                self.source.timeout().timeout(timeout_before, Duration::from_nanos(1));
                return Err(e);
            }
        };
        self.source.timeout().timeout(timeout_before, Duration::from_nanos(1));

        self.opcode = b0 & B0_MASK_OPCODE;
        self.is_final_frame = (b0 & B0_FLAG_FIN) != 0;
        self.is_control_frame = (b0 & OPCODE_FLAG_CONTROL) != 0;

        if self.is_control_frame && !self.is_final_frame {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Control frames must be final."));
        }

        let reserved_flag1 = (b0 & B0_FLAG_RSV1) != 0;
        match self.opcode {
            OPCODE_TEXT | OPCODE_BINARY => {
                self.reading_compressed_message = if reserved_flag1 {
                    if !self.per_message_deflate {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Unexpected rsv1 flag"));
                    }
                    true
                } else {
                    false
                };
            }
            _ => {
                if reserved_flag1 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Unexpected rsv1 flag"));
                }
            }
        }

        if (b0 & B0_FLAG_RSV2) != 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Unexpected rsv2 flag"));
        }
        if (b0 & B0_FLAG_RSV3) != 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Unexpected rsv3 flag"));
        }

        let b1 = (self.source.read_byte()? as i32) & 0xff;
        let is_masked = (b1 & B1_FLAG_MASK) != 0;

        if is_masked == self.is_client {
            let msg = if self.is_client {
                "Server-sent frames must not be masked."
            } else {
                "Client-sent frames must be masked."
            };
            return Err(io::Error::new(io::ErrorKind::InvalidData, msg));
        }

        self.frame_length = (b1 & B1_MASK_LENGTH) as i64;
        if self.frame_length == PAYLOAD_SHORT as i64 {
            self.frame_length = (self.source.read_short()? as i32 & 0xffff) as i64;
        } else if self.frame_length == PAYLOAD_LONG as i64 {
            self.frame_length = self.source.read_long()?;
            if self.frame_length < 0 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Frame length 0x{:x} > 0x7FFFFFFFFFFFFFFF", self.frame_length)));
            }
        }

        if self.is_control_frame && self.frame_length > PAYLOAD_BYTE_MAX {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Control frame must be less than {}B.", PAYLOAD_BYTE_MAX)));
        }

        if is_masked {
            let key = self.mask_key.as_mut().expect("maskKey must be present for server");
            self.source.read_fully(key)?;
        }

        Ok(())
    }

    fn read_control_frame(&mut self) -> io::Result<()> {
        if self.frame_length > 0 {
            self.source.read_fully(&mut self.control_frame_buffer, self.frame_length)?;

            if !self.is_client {
                let cursor = self.mask_cursor.as_mut().expect("maskCursor must be present for server");
                self.control_frame_buffer.read_and_write_unsafe(cursor);
                cursor.seek(0);
                let key = self.mask_key.as_ref().expect("maskKey must be present for server");
                toggle_mask(cursor, key);
                cursor.close();
            }
        }

        match self.opcode {
            OPCODE_CONTROL_PING => {
                self.frame_callback.on_read_ping(self.control_frame_buffer.read_byte_string());
            }
            OPCODE_CONTROL_PONG => {
                self.frame_callback.on_read_pong(self.control_frame_buffer.read_byte_string());
            }
            OPCODE_CONTROL_CLOSE => {
                let mut code = CLOSE_NO_STATUS_CODE;
                let mut reason = String::new();
                let buffer_size = self.control_frame_buffer.size();
                if buffer_size == 1 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Malformed close payload length of 1."));
                } else if buffer_size != 0 {
                    code = self.control_frame_buffer.read_short()? as i32;
                    reason = self.control_frame_buffer.read_utf8()?;
                    if let Some(msg) = close_code_exception_message(code) {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, msg));
                    }
                }
                self.frame_callback.on_read_close(code, reason);
                self.received_close_frame = true;
            }
            _ => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unknown control opcode: {:x}", self.opcode)));
            }
        }
        Ok(())
    }

    fn read_message_frame(&mut self) -> io::Result<()> {
        let opcode = self.opcode;
        if opcode != OPCODE_TEXT && opcode != OPCODE_BINARY {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unknown opcode: {:x}", opcode)));
        }

        self.read_message()?;

        if self.reading_compressed_message {
            if self.message_inflater.is_none() {
                self.message_inflater = Some(MessageInflater::new(self.no_context_takeover));
            }
            let inflater = self.message_inflater.as_mut().unwrap();
            inflater.inflate(&mut self.message_frame_buffer)?;
        }

        if opcode == OPCODE_TEXT {
            self.frame_callback.on_read_message_text(self.message_frame_buffer.read_utf8()?)?;
        } else {
            self.frame_callback.on_read_message_bytes(self.message_frame_buffer.read_byte_string())?;
        }
        Ok(())
    }

    fn read_until_non_control_frame(&mut self) -> io::Result<()> {
        while !self.received_close_frame {
            self.read_header()?;
            if !self.is_control_frame {
                break;
            }
            self.read_control_frame()?;
        }
        Ok(())
    }

    fn read_message(&mut self) -> io::Result<()> {
        loop {
            if self.received_close_frame {
                return Err(io::Error::new(io::ErrorKind::Other, "closed"));
            }

            if self.frame_length > 0 {
                self.source.read_fully(&mut self.message_frame_buffer, self.frame_length)?;

                if !self.is_client {
                    let cursor = self.mask_cursor.as_mut().expect("maskCursor must be present for server");
                    self.message_frame_buffer.read_and_write_unsafe(cursor);
                    cursor.seek(self.message_frame_buffer.size() - self.frame_length);
                    let key = self.mask_key.as_ref().expect("maskKey must be present for server");
                    toggle_mask(cursor, key);
                    cursor.close();
                }
            }

            if self.is_final_frame {
                break;
            }

            self.read_until_non_control_frame()?;
            if self.opcode != OPCODE_CONTINUATION {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Expected continuation opcode. Got: {:x}", self.opcode)));
            }
        }
        Ok(())
    }

    pub fn close(&mut self) -> io::Result<()> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;
        if let Some(mut inflater) = self.message_inflater.take() {
            inflater.close_quietly();
        }
        // source.close_quietly() is handled by the source's own Drop or explicit close
        Ok(())
    }
}

impl<S: BufferedSource> Drop for WebSocketReader<S> {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
