use std::collections::HashMap;
use std::io::{self, Read, Write};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;

// Mocking okio-like types as per the provided context and common Rust patterns
pub type BufferedSource = Box<dyn Read + Send>;
pub type Buffer = Vec<u8>;
pub type ByteString = Vec<u8>;

// Constants from Http2 class
pub const CONNECTION_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
pub const FLAG_ACK: i32 = 0x1;
pub const FLAG_COMPRESSED: i32 = 0x2;
pub const FLAG_END_HEADERS: i32 = 0x4;
pub const FLAG_END_STREAM: i32 = 0x1;
pub const FLAG_PADDED: i32 = 0x8;
pub const FLAG_PRIORITY: i32 = 0x20;
pub const INITIAL_MAX_FRAME_SIZE: i32 = 16384;
pub const TYPE_CONTINUATION: i32 = 0x0;
pub const TYPE_DATA: i32 = 0x0;
pub const TYPE_GOAWAY: i32 = 0x7;
pub const TYPE_HEADERS: i32 = 0x1;
pub const TYPE_PING: i32 = 0x6;
pub const TYPE_PRIORITY: i32 = 0x2;
pub const TYPE_PUSH_PROMISE: i32 = 0x5;
pub const TYPE_RST_STREAM: i32 = 0x3;
pub const TYPE_SETTINGS: i32 = 0x4;
pub const TYPE_WINDOW_UPDATE: i32 = 0x8;

// Helper traits for reading specific types from the source
trait OkioReadExt: Read {
    fn read_medium(&mut self) -> io::Result<i32> {
        let mut buf = [0u8; 3];
        self.read_exact(&mut buf)?;
        Ok(((buf[0] as i32) << 16) | ((buf[1] as i32) << 8) | (buf[2] as i32))
    }

    fn read_byte(&mut self) -> io::Result<i8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0] as i8)
    }

    fn read_short(&mut self) -> io::Result<i16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_be_bytes(buf))
    }

    fn read_int(&mut self) -> io::Result<i32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_be_bytes(buf))
    }

    fn read_byte_string(&mut self, size: i64) -> io::Result<ByteString> {
        let mut buf = vec![0u8; size as usize];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn require(&mut self, _bytes: usize) -> io::Result<()> {
        // In a real BufferedSource, this checks the buffer.
        Ok(())
    }

    fn skip(&mut self, bytes: i64) -> io::Result<()> {
        let mut buf = vec![0u8; bytes as usize];
        self.read_exact(&mut buf)?;
        Ok(())
    }
}

impl<R: Read + ?Sized> OkioReadExt for R {}

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Settings(pub HashMap<i16, i32>);

impl std::ops::Index<i16> for Settings {
    type Output = i32;
    fn index(&self, index: i16) -> &Self::Output {
        self.0.get(&index).unwrap_or(&0)
    }
}

impl Settings {
    pub fn set(&mut self, id: i16, value: i32) {
        self.0.insert(id, value);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorCode {
    NoError = 0x0,
    ProtocolError = 0x1,
    InternalError = 0x2,
    FlowControlError = 0x3,
    SettingsTimeout = 0x4,
    StreamClosed = 0x5,
    FrameSizeError = 0x6,
    RefusedStream = 0x7,
    Cancel = 0x8,
    CompressionError = 0x9,
    ConnectError = 0xa,
    EnhanceYourCalm = 0xb,
    IncorrectLength = 0xc,
    StreamIdError = 0xd,
    FramePayloadError = 0xe,
    ProtocolError_Custom = 0xf,
}

pub const NoError: ErrorCode = ErrorCode::NoError;
pub const ProtocolError: ErrorCode = ErrorCode::ProtocolError;
pub const InternalError: ErrorCode = ErrorCode::InternalError;
pub const FlowControlError: ErrorCode = ErrorCode::FlowControlError;
pub const SettingsTimeout: ErrorCode = ErrorCode::SettingsTimeout;
pub const StreamClosed: ErrorCode = ErrorCode::StreamClosed;
pub const FrameSizeError: ErrorCode = ErrorCode::FrameSizeError;
pub const RefusedStream: ErrorCode = ErrorCode::RefusedStream;
pub const Cancel: ErrorCode = ErrorCode::Cancel;
pub const CompressionError: ErrorCode = ErrorCode::CompressionError;
pub const ConnectError: ErrorCode = ErrorCode::ConnectError;
pub const EnhanceYourCalm: ErrorCode = ErrorCode::EnhanceYourCalm;
pub const IncorrectLength: ErrorCode = ErrorCode::IncorrectLength;
pub const StreamIdError: ErrorCode = ErrorCode::StreamIdError;
pub const FramePayloadError: ErrorCode = ErrorCode::FramePayloadError;
pub const ProtocolError_Custom: ErrorCode = ErrorCode::ProtocolError_Custom;

impl Default for ErrorCode {
    fn default() -> Self {
        ErrorCode::NoError
    }
}

impl ErrorCode {
    pub fn from_http2(code: i32) -> Option<Self> {
        match code {
            0x0 => Some(ErrorCode::NoError),
            0x1 => Some(ErrorCode::ProtocolError),
            0x2 => Some(ErrorCode::InternalError),
            0x3 => Some(ErrorCode::FlowControlError),
            0x4 => Some(ErrorCode::SettingsTimeout),
            0x5 => Some(ErrorCode::StreamClosed),
            0x6 => Some(ErrorCode::FrameSizeError),
            0x7 => Some(ErrorCode::RefusedStream),
            0x8 => Some(ErrorCode::Cancel),
            0x9 => Some(ErrorCode::CompressionError),
            0xa => Some(ErrorCode::ConnectError),
            0xb => Some(ErrorCode::EnhanceYourCalm),
            0xc => Some(ErrorCode::IncorrectLength),
            0xd => Some(ErrorCode::StreamIdError),
            0xe => Some(ErrorCode::FramePayloadError),
            _ => None,
        }
    }
}

pub trait Handler {

impl HpackReader {
    fn new(_source: &mut ContinuationSource) -> Self {
        Self {}
    }
    fn read_headers(&mut self, _source: &mut ContinuationSource) -> io::Result<()> {
        // Logic to read from source and decode HPACK
        Ok(())
    }
    fn get_and_reset_header_list(&mut self) -> Vec<Header> {
        Vec::new()
    }
}

pub struct Http2Reader {
    source: BufferedSource,
    client: bool,
    continuation: ContinuationSource,
    hpack_reader: HpackReader,
}

impl Http2Reader {
    pub fn new(source: BufferedSource, client: bool) -> Self {
        let continuation = ContinuationSource {
            flags: 0,
            stream_id: 0,
            left: 0,
            padding: 0,
        };
        // We pass a dummy reference or handle it via the reader's source
        let hpack_reader = HpackReader::new(&mut continuation);
        
        Self {
            source,
            client,
            continuation,
            hpack_reader,
        }
    }

    pub fn read_connection_preface<H: Handler>(&mut self, handler: &mut H) -> io::Result<()> {
        if self.client {
            if !self.next_frame(true, handler)? {
                return Err(io::Error::new(io::ErrorKind::Other, "Required SETTINGS preface not received"));
            }
        } else {
            let connection_preface = self.source.read_byte_string(CONNECTION_PREFACE.len() as i64)?;
            if CONNECTION_PREFACE != connection_preface.as_slice() {
                return Err(io::Error::new(io::ErrorKind::Other, format!("Expected a connection header but was {:?}", String::from_utf8_lossy(&connection_preface))));
            }
        }
        Ok(())
    }

    pub fn next_frame<H: Handler>(&mut self, require_settings: bool, handler: &mut H) -> io::Result<bool> {
        if let Err(e) = self.source.require(9) {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                return Ok(false);
            }
            return Err(e);
        }

        let length = self.source.read_medium()?;
        if length > INITIAL_MAX_FRAME_SIZE {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("FRAME_SIZE_ERROR: {}", length)));
        }
        let frame_type = self.source.read_byte()? as u8 as i32;
        let flags = self.source.read_byte()? as u8 as i32;
        let stream_id = self.source.read_int()? & 0x7fffffff;

        if frame_type != TYPE_WINDOW_UPDATE {
            // Log frame (simulated)
        }

        if require_settings && frame_type != TYPE_SETTINGS {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Expected a SETTINGS frame but was {}", frame_type)));
        }

        match frame_type {
            TYPE_DATA => self.read_data(handler, length, flags, stream_id)?,
            TYPE_HEADERS => self.read_headers(handler, length, flags, stream_id)?,
            TYPE_PRIORITY => self.read_priority_frame(handler, length, flags, stream_id)?,
            TYPE_RST_STREAM => self.read_rst_stream(handler, length, flags, stream_id)?,
            TYPE_SETTINGS => self.read_settings(handler, length, flags, stream_id)?,
            TYPE_PUSH_PROMISE => self.read_push_promise(handler, length, flags, stream_id)?,
            TYPE_PING => self.read_ping(handler, length, flags, stream_id)?,
            TYPE_GOAWAY => self.read_go_away(handler, length, flags, stream_id)?,
            TYPE_WINDOW_UPDATE => self.read_window_update(handler, length, flags, stream_id)?,
            _ => { self.source.skip(length as i64)?; }
        }

        Ok(true)
    }

    fn read_headers<H: Handler>(&mut self, handler: &mut H, length: i32, flags: i32, stream_id: i32) -> io::Result<()> {
        if stream_id == 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "PROTOCOL_ERROR: TYPE_HEADERS streamId == 0"));
        }

        let end_stream = (flags & FLAG_END_STREAM) != 0;
        let padding = if (flags & FLAG_PADDED) != 0 {
            self.source.read_byte()? as u8 as i32
        } else {
            0
        };

        let mut header_block_length = length;
        if (flags & FLAG_PRIORITY) != 0 {
            self.read_priority_logic(handler, stream_id)?;
            header_block_length -= 5;
        }
        
        let header_block_length = Self::length_without_padding(header_block_length, flags, padding)?;
        let header_block = self.read_header_block(header_block_length, padding, flags, stream_id)?;

        handler.headers(end_stream, stream_id, -1, header_block)?;
        Ok(())
    }

    fn read_header_block(&mut self, length: i32, padding: i32, flags: i32, stream_id: i32) -> io::Result<Vec<Header>> {
        self.continuation.left = length;
        self.continuation.padding = padding;
        self.continuation.flags = flags;
        self.continuation.stream_id = stream_id;

        // In a real implementation, hpack_reader would use the source via continuation
        self.hpack_reader.read_headers(&mut self.continuation)?;
        Ok(self.hpack_reader.get_and_reset_header_list())
    }

    fn read_data<H: Handler>(&mut self, handler: &mut H, length: i32, flags: i32, stream_id: i32) -> io::Result<()> {
        if stream_id == 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "PROTOCOL_ERROR: TYPE_DATA streamId == 0"));
        }

        let in_finished = (flags & FLAG_END_STREAM) != 0;
        let gzipped = (flags & FLAG_COMPRESSED) != 0;
        if gzipped {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "PROTOCOL_ERROR: FLAG_COMPRESSED without SETTINGS_COMPRESS_DATA"));
        }

        let padding = if (flags & FLAG_PADDED) != 0 {
            self.source.read_byte()? as u8 as i32
        } else {
            0
        };
        let data_length = Self::length_without_padding(length, flags, padding)?;

        handler.data(in_finished, stream_id, &mut self.source, data_length)?;
        self.source.skip(padding as i64)?;
        Ok(())
    }

    fn read_priority_frame<H: Handler>(&mut self, handler: &mut H, length: i32, _flags: i32, stream_id: i32) -> io::Result<()> {
        if length != 5 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("TYPE_PRIORITY length: {} != 5", length)));
        }
        if stream_id == 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "TYPE_PRIORITY streamId == 0"));
        }
        self.read_priority_logic(handler, stream_id)
    }

    fn read_priority_logic<H: Handler>(&mut self, handler: &mut H, stream_id: i32) -> io::Result<()> {
        let w1 = self.source.read_int()?;
        let exclusive = (w1 & 0x80000000) != 0;
        let stream_dependency = w1 & 0x7fffffff;
        let weight = (self.source.read_byte()? as u8 as i32) + 1;
        handler.priority(stream_id, stream_dependency, weight, exclusive);
        Ok(())
    }

    fn read_rst_stream<H: Handler>(&mut self, handler: &mut H, length: i32, _flags: i32, stream_id: i32) -> io::Result<()> {
        if length != 4 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("TYPE_RST_STREAM length: {} != 4", length)));
        }
        if stream_id == 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "TYPE_RST_STREAM streamId == 0"));
        }
        let error_code_int = self.source.read_int()?;
        let error_code = ErrorCode::from_http2(error_code_int)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, format!("TYPE_RST_STREAM unexpected error code: {}", error_code_int)))?;
        handler.rst_stream(stream_id, error_code);
        Ok(())
    }

    fn read_settings<H: Handler>(&mut self, handler: &mut H, length: i32, flags: i32, stream_id: i32) -> io::Result<()> {
        if stream_id != 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "TYPE_SETTINGS streamId != 0"));
        }
        if (flags & FLAG_ACK) != 0 {
            if length != 0 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "FRAME_SIZE_ERROR ack frame should be empty!"));
            }
            handler.ack_settings();
            return Ok(());
        }

        if length % 6 != 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("TYPE_SETTINGS length % 6 != 0: {}", length)));
        }
        
        let mut settings = Settings(HashMap::new());
        for _ in (0..length).step_by(6) {
            let id = self.source.read_short()? as u16 as i16;
            let value = self.source.read_int()?;

            match id {
                1 => {}, // SETTINGS_HEADER_TABLE_SIZE
                2 => {
                    if value != 0 && value != 1 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "PROTOCOL_ERROR SETTINGS_ENABLE_PUSH != 0 or 1"));
                    }
                },
                3 => {}, // SETTINGS_MAX_CONCURRENT_STREAMS
                4 => {
                    if value < 0 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "PROTOCOL_ERROR SETTINGS_INITIAL_WINDOW_SIZE > 2^31 - 1"));
                    }
                },
                5 => {
                    if value < INITIAL_MAX_FRAME_SIZE || value > 16777215 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("PROTOCOL_ERROR SETTINGS_MAX_FRAME_SIZE: {}", value)));
                    }
                },
                6 => {}, // SETTINGS_MAX_HEADER_LIST_SIZE
                _ => {},
            }
            settings.set(id, value);
        }
        handler.settings(false, settings);
        Ok(())
    }

    fn read_push_promise<H: Handler>(&mut self, handler: &mut H, length: i32, flags: i32, stream_id: i32) -> io::Result<()> {
        if stream_id == 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "PROTOCOL_ERROR: TYPE_PUSH_PROMISE streamId == 0"));
        }
        let padding = if (flags & FLAG_PADDED) != 0 {
            self.source.read_byte()? as u8 as i32
        } else {
            0
        };
        let promised_stream_id = self.source.read_int()? & 0x7fffffff;
        let header_block_length = Self::length_without_padding(length - 4, flags, padding)?;
        let header_block = self.read_header_block(header_block_length, padding, flags, stream_id)?;
        handler.push_promise(stream_id, promised_stream_id, header_block)?;
        Ok(())
    }

    fn read_ping<H: Handler>(&mut self, handler: &mut H, length: i32, flags: i32, stream_id: i32) -> io::Result<()> {
        if length != 8 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("TYPE_PING length != 8: {}", length)));
        }
        if stream_id != 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "TYPE_PING streamId != 0"));
        }
        let payload1 = self.source.read_int()?;
        let payload2 = self.source.read_int()?;
        let ack = (flags & FLAG_ACK) != 0;
        handler.ping(ack, payload1, payload2);
        Ok(())
    }

    fn read_go_away<H: Handler>(&mut self, handler: &mut H, length: i32, flags: i32, stream_id: i32) -> io::Result<()> {
        if length < 8 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("TYPE_GOAWAY length < 8: {}", length)));
        }
        if stream_id != 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "TYPE_GOAWAY streamId != 0"));
        }
        let last_stream_id = self.source.read_int()?;
        let error_code_int = self.source.read_int()?;
        let opaque_data_length = length - 8;
        let error_code = ErrorCode::from_http2(error_code_int)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, format!("TYPE_GOAWAY unexpected error code: {}", error_code_int)))?;
        
        let mut debug_data = ByteString::new();
        if opaque_data_length > 0 {
            debug_data = self.source.read_byte_string(opaque_data_length as i64)?;
        }
        handler.go_away(last_stream_id, error_code, debug_data);
        Ok(())
    }

    fn read_window_update<H: Handler>(&mut self, handler: &mut H, length: i32, flags: i32, stream_id: i32) -> io::Result<()> {
        let increment: i64 = match (|| -> io::Result<i64> {
            if length != 4 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("TYPE_WINDOW_UPDATE length !=4: {}", length)));
            }
            let inc = (self.source.read_int()? & 0x7fffffff) as i64;
            if inc == 0 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "windowSizeIncrement was 0"));
            }
            Ok(inc)
        })() {
            Ok(val) => val,
            Err(e) => {
                // Log frame (simulated)
                return Err(e);
            }
        };

        handler.window_update(stream_id, increment);
        Ok(())
    }

    pub fn length_without_padding(length: i32, flags: i32, padding: i32) -> io::Result<i32> {
        let mut result = length;
        if (flags & FLAG_PADDED) != 0 {
            result -= 1;
        }
        if padding > result {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("PROTOCOL_ERROR padding {} > remaining length {}", padding, result)));
        }
        result -= padding;
        Ok(result)
    }

    pub fn close(&mut self) -> io::Result<()> {
        // In a real implementation, we would close the source
        Ok(())
    }
}

pub struct ContinuationSource {
    pub flags: i32,
    pub stream_id: i32,
    pub left: i32,
    pub padding: i32,
}

impl ContinuationSource {
    fn read_continuation_header<R: Read + OkioReadExt>(&mut self, source: &mut R) -> io::Result<()> {
        let previous_stream_id = self.stream_id;
        let length = source.read_medium()?;
        self.left = length;
        let frame_type = source.read_byte()? as u8 as i32;
        self.flags = source.read_byte()? as u8 as i32;
        self.stream_id = source.read_int()? & 0x7fffffff;
        if frame_type != TYPE_CONTINUATION {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("{} != TYPE_CONTINUATION", frame_type)));
        }
        if self.stream_id != previous_stream_id {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "TYPE_CONTINUATION streamId changed"));
        }
        Ok(())
    }
}
}
