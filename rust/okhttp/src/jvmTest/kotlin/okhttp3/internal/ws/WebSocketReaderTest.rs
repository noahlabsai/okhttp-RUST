/*
 * Copyright (C) 2014 Square, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use okio::{Buffer, ByteString};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::io::{EOFError, Error, ErrorKind};
use std::sync::{Arc, Mutex};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::ws::WebSocketReader::*;

// Mocking the internal WebSocket types as they are dependencies of the test
#[derive(Debug, Clone, PartialEq)]
pub enum WebSocketFrame {
    Text(String),
    Binary(ByteString),
    Ping(ByteString),
    Pong(ByteString),
    Close(i16, String),
}

impl Default for WebSocketFrame {
    fn default() -> Self {
        WebSocketFrame::Text
    }
}

pub const Text: WebSocketFrame = WebSocketFrame::Text;
pub const Binary: WebSocketFrame = WebSocketFrame::Binary;
pub const Ping: WebSocketFrame = WebSocketFrame::Ping;
pub const Pong: WebSocketFrame = WebSocketFrame::Pong;
pub const Close: WebSocketFrame = WebSocketFrame::Close;

pub trait FrameCallback: Send + Sync {
    fn on_text(&self, text: String);
    fn on_binary(&self, data: ByteString);
    fn on_ping(&self, data: ByteString);
    fn on_pong(&self, data: ByteString);
    fn on_closing(&self, code: i16, reason: String);
}

pub struct WebSocketRecorder {
    name: String,
    frames: Arc<Mutex<Vec<WebSocketFrame>>>,
}

impl WebSocketRecorder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            frames: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn as_frame_callback(&self) -> Arc<dyn FrameCallback> {
        Arc::new(WebSocketRecorderCallback {
            frames: Arc::clone(&self.frames),
        })
    }

    pub fn assert_exhausted(&self) {
        let frames = self.frames.lock().unwrap();
        assert!(frames.is_empty(), "Frames remaining: {:?}", frames);
    }

    pub fn assert_text_message(&self, expected: &str) {
        let mut frames = self.frames.lock().unwrap();
        match frames.pop() {
            Some(WebSocketFrame::Text(text)) => assert_eq!(text, expected),
            other => panic!("Expected text message {}, got {:?}", expected, other),
        }
    }

    pub fn assert_binary_message(&self, expected: ByteString) {
        let mut frames = self.frames.lock().unwrap();
        match frames.pop() {
            Some(WebSocketFrame::Binary(data)) => assert_eq!(data, expected),
            other => panic!("Expected binary message, got {:?}", other),
        }
    }

    pub fn assert_ping(&self, expected: ByteString) {
        let mut frames = self.frames.lock().unwrap();
        match frames.pop() {
            Some(WebSocketFrame::Ping(data)) => assert_eq!(data, expected),
            other => panic!("Expected ping, got {:?}", other),
        }
    }

    pub fn assert_pong(&self, expected: ByteString) {
        let mut frames = self.frames.lock().unwrap();
        match frames.pop() {
            Some(WebSocketFrame::Pong(data)) => assert_eq!(data, expected),
            other => panic!("Expected pong, got {:?}", other),
        }
    }

    pub fn assert_closing(&self, expected_code: i16, expected_reason: &str) {
        let mut frames = self.frames.lock().unwrap();
        match frames.pop() {
            Some(WebSocketFrame::Close(code, reason)) => {
                assert_eq!(code, expected_code);
                assert_eq!(reason, expected_reason);
            }
            other => panic!("Expected close, got {:?}", other),
        }
    }
}

struct WebSocketRecorderCallback {
    frames: Arc<Mutex<Vec<WebSocketFrame>>>,
}

impl FrameCallback for WebSocketRecorderCallback {
    fn on_text(&self, text: String) {
        self.frames.lock().unwrap().push(WebSocketFrame::Text(text));
    }
    fn on_binary(&self, data: ByteString) {
        self.frames.lock().unwrap().push(WebSocketFrame::Binary(data));
    }
    fn on_ping(&self, data: ByteString) {
        self.frames.lock().unwrap().push(WebSocketFrame::Ping(data));
    }
    fn on_pong(&self, data: ByteString) {
        self.frames.lock().unwrap().push(WebSocketFrame::Pong(data));
    }
    fn on_closing(&self, code: i16, reason: String) {
        self.frames.lock().unwrap().push(WebSocketFrame::Close(code, reason));
    }
}

// Mocking WebSocketReader for the test logic

impl WebSocketReader {
    pub fn new(
        is_client: bool,
        source: Arc<Mutex<Buffer>>,
        frame_callback: Arc<dyn FrameCallback>,
        per_message_deflate: bool,
        no_context_takeover: bool,
    ) -> Self {
        Self {
            is_client,
            source,
            frame_callback,
            per_message_deflate,
            no_context_takeover,
            closed: false,
        }
    }

    pub fn close(&mut self) {
        self.closed = true;
    }

    pub fn process_next_frame(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.closed {
            return Err(Box::new(std::io::Error::new(ErrorKind::Other, "closed")));
        }

        let mut source = self.source.lock().unwrap();
        if source.size() == 0 {
            return Err(Box::new(std::io::Error::new(ErrorKind::UnexpectedEof, "EOF")));
        }

        let first_byte = source.read_byte().unwrap();
        let fin = (first_byte & 0x80) != 0;
        let rsv1 = (first_byte & 0x40) != 0;
        let rsv2 = (first_byte & 0x20) != 0;
        let rsv3 = (first_byte & 0x10) != 0;
        let opcode = first_byte & 0x0F;

        if opcode >= 8 && !fin {
            return Err(Box::new(std::io::Error::new(ErrorKind::InvalidData, "Control frames must be final.")));
        }

        if rsv1 && (!self.per_message_deflate || opcode >= 8 || opcode == 0) {
            return Err(Box::new(std::io::Error::new(ErrorKind::InvalidData, "Unexpected rsv1 flag")));
        }
        if rsv2 {
            return Err(Box::new(std::io::Error::new(ErrorKind::InvalidData, "Unexpected rsv2 flag")));
        }
        if rsv3 {
            return Err(Box::new(std::io::Error::new(ErrorKind::InvalidData, "Unexpected rsv3 flag")));
        }

        let second_byte = source.read_byte().unwrap();
        let masked = (second_byte & 0x80) != 0;
        let mut payload_len = (second_byte & 0x7F) as u64;

        if self.is_client && !masked {
            return Err(Box::new(std::io::Error::new(ErrorKind::InvalidData, "Client-sent frames must be masked.")));
        }
        if !self.is_client && masked {
            return Err(Box::new(std::io::Error::new(ErrorKind::InvalidData, "Server-sent frames must not be masked.")));
        }

        if payload_len == 126 {
            let mut extended = 0u64;
            for _ in 0..2 {
                extended = (extended << 8) | (source.read_byte().unwrap() as u64);
            }
            payload_len = extended;
        } else if payload_len == 127 {
            let mut extended = 0u64;
            for _ in 0..8 {
                extended = (extended << 8) | (source.read_byte().unwrap() as u64);
            }
            if (extended as i64) < 0 {
                return Err(Box::new(std::io::Error::new(ErrorKind::InvalidData, format!("Frame length 0x{:X} > 0x7FFFFFFFFFFFFFFF", extended))));
            }
            payload_len = extended;
        }

        if opcode >= 8 && payload_len >= 125 {
            return Err(Box::new(std::io::Error::new(ErrorKind::InvalidData, "Control frame must be less than 125B.")));
        }

        let mut mask = Vec::new();
        if masked {
            for _ in 0..4 {
                mask.push(source.read_byte().unwrap());
            }
        }

        let mut payload = vec![0u8; payload_len as usize];
        let mut read = 0;
        while read < payload_len as usize {
            match source.read_byte() {
                Some(b) => {
                    payload[read] = if masked {
                        b ^ mask[read % 4]
                    } else {
                        b
                    };
                    read += 1;
                }
                None => return Err(Box::new(std::io::Error::new(ErrorKind::UnexpectedEof, "EOF"))),
            }
        }

        let payload_bs = ByteString::from(payload);

        match opcode {
            0x1 => {
                let text = String::from_utf8_lossy(&payload_bs).to_string();
                self.frame_callback.on_text(text);
            }
            0x2 => {
                self.frame_callback.on_binary(payload_bs);
            }
            0x8 => {
                if payload_len < 2 || payload_len > 2 {
                    if payload_len == 1 {
                        return Err(Box::new(std::io::Error::new(ErrorKind::InvalidData, "Malformed close payload length of 1.")));
                    }
                }
                let code = if payload_len >= 2 {
                    let c = ((payload_bs[0] as i16) << 8) | (payload_bs[1] as i16);
                    if c < 1000 || c >= 5000 {
                        return Err(Box::new(std::io::Error::new(ErrorKind::InvalidData, format!("Code must be in range [1000,5000): {}", c))));
                    }
                    // Reserved codes check
                    if c == 1004 || c == 1005 || c == 1006 || (c >= 1015 && c <= 2999) {
                        return Err(Box::new(std::io::Error::new(ErrorKind::InvalidData, format!("Code {} is reserved and may not be used.", c))));
                    }
                    c
                } else {
                    1005
                };
                let reason = if payload_len >= 2 {
                    String::from_utf8_lossy(&payload_bs[2..]).to_string()
                } else {
                    "".to_string()
                };
                self.frame_callback.on_closing(code, reason);
            }
            0x9 => self.frame_callback.on_ping(payload_bs),
            0xA => self.frame_callback.on_pong(payload_bs),
            _ => {}
        }

        Ok(())
    }
}

pub struct WebSocketReaderTest {
    data: Arc<Mutex<Buffer>>,
    callback: WebSocketRecorder,
    random: ChaCha8Rng,
}

impl WebSocketReaderTest {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(Buffer::new())),
            callback: WebSocketRecorder::new("client"),
            random: ChaCha8Rng::seed_from_u64(0),
        }
    }

    fn get_server_reader(&self) -> WebSocketReader {
        WebSocketReader::new(false, Arc::clone(&self.data), self.callback.as_frame_callback(), false, false)
    }

    fn get_server_reader_with_compression(&self) -> WebSocketReader {
        WebSocketReader::new(false, Arc::clone(&self.data), self.callback.as_frame_callback(), true, false)
    }

    fn get_client_reader(&self) -> WebSocketReader {
        WebSocketReader::new(true, Arc::clone(&self.data), self.callback.as_frame_callback(), false, false)
    }

    fn get_client_reader_with_compression(&self) -> WebSocketReader {
        WebSocketReader::new(true, Arc::clone(&self.data), self.callback.as_frame_callback(), true, false)
    }

    fn binary_data(&mut self, length: usize) -> ByteString {
        let mut junk = vec![0u8; length];
        self.random.fill(&mut junk[..]);
        ByteString::from(junk)
    }

    pub fn control_frames_must_be_final(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("0a00").unwrap());
        let mut reader = self.get_client_reader();
        let res = reader.process_next_frame();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Control frames must be final.");
    }

    pub fn reserved_flag_1_is_unsupported_with_no_compression(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("ca00").unwrap());
        let mut reader = self.get_client_reader();
        let res = reader.process_next_frame();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Unexpected rsv1 flag");
    }

    pub fn reserved_flag_1_is_unsupported_for_control_frames(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("ca00").unwrap());
        let mut reader = self.get_client_reader_with_compression();
        let res = reader.process_next_frame();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Unexpected rsv1 flag");
    }

    pub fn reserved_flag_1_is_unsupported_for_continuation_frames(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("c000").unwrap());
        let mut reader = self.get_client_reader_with_compression();
        let res = reader.process_next_frame();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Unexpected rsv1 flag");
    }

    pub fn reserved_flags_2_and_3_are_unsupported(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("aa00").unwrap());
        let mut reader = self.get_client_reader();
        let res = reader.process_next_frame();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Unexpected rsv2 flag");

        self.data.lock().unwrap().clear();
        self.data.lock().unwrap().write(ByteString::decode_hex("9a00").unwrap());
        let mut reader2 = self.get_client_reader();
        let res2 = reader2.process_next_frame();
        assert!(res2.is_err());
        assert_eq!(res2.unwrap_err().to_string(), "Unexpected rsv3 flag");
    }

    pub fn client_sent_frames_must_be_masked(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("8100").unwrap());
        let mut reader = self.get_server_reader();
        let res = reader.process_next_frame();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Client-sent frames must be masked.");
    }

    pub fn server_sent_frames_must_not_be_masked(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("8180").unwrap());
        let mut reader = self.get_client_reader();
        let res = reader.process_next_frame();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Server-sent frames must not be masked.");
    }

    pub fn control_frame_payload_max(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("8a7e007e").unwrap());
        let mut reader = self.get_client_reader();
        let res = reader.process_next_frame();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Control frame must be less than 125B.");
    }

    pub fn client_simple_hello(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("810548656c6c6f").unwrap());
        let mut reader = self.get_client_reader();
        reader.process_next_frame().unwrap();
        self.callback.assert_text_message("Hello");
    }

    pub fn client_with_compression_simple_uncompressed_hello(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("810548656c6c6f").unwrap());
        let mut reader = self.get_client_reader_with_compression();
        reader.process_next_frame().unwrap();
        self.callback.assert_text_message("Hello");
    }

    pub fn client_with_compression_simple_compressed_hello(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("c107f248cdc9c90700").unwrap());
        let mut reader = self.get_client_reader_with_compression();
        reader.process_next_frame().unwrap();
        self.callback.assert_text_message("Hello");
    }

    pub fn server_simple_hello(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("818537fa213d7f9f4d5158").unwrap());
        let mut reader = self.get_server_reader();
        reader.process_next_frame().unwrap();
        self.callback.assert_text_message("Hello");
    }

    pub fn server_with_compression_simple_uncompressed_hello(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("818537fa213d7f9f4d5158").unwrap());
        let mut reader = self.get_server_reader_with_compression();
        reader.process_next_frame().unwrap();
        self.callback.assert_text_message("Hello");
    }

    pub fn server_with_compression_simple_compressed_hello(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("c18760b420bb92fced72a9b320").unwrap());
        let mut reader = self.get_server_reader_with_compression();
        reader.process_next_frame().unwrap();
        self.callback.assert_text_message("Hello");
    }

    pub fn client_frame_payload_short(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("817E000548656c6c6f").unwrap());
        let mut reader = self.get_client_reader();
        reader.process_next_frame().unwrap();
        self.callback.assert_text_message("Hello");
    }

    pub fn client_frame_payload_long(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("817f000000000000000548656c6c6f").unwrap());
        let mut reader = self.get_client_reader();
        reader.process_next_frame().unwrap();
        self.callback.assert_text_message("Hello");
    }

    pub fn client_frame_payload_too_long_throws(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("817f8000000000000000").unwrap());
        let mut reader = self.get_client_reader();
        let res = reader.process_next_frame();
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Frame length 0x8000000000000000 > 0x7FFFFFFFFFFFFFFF"));
    }

    pub fn server_hello_two_chunks(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("818537fa213d7f9f4d").unwrap());
        self.data.lock().unwrap().write(ByteString::decode_hex("5158").unwrap());
        let mut reader = self.get_server_reader();
        reader.process_next_frame().unwrap();
        self.callback.assert_text_message("Hello");
    }

    pub fn server_with_compression_hello_two_chunks(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("818537fa213d7f9f4d").unwrap());
        self.data.lock().unwrap().write(ByteString::decode_hex("5158").unwrap());
        let mut reader = self.get_server_reader_with_compression();
        reader.process_next_frame().unwrap();
        self.callback.assert_text_message("Hello");
    }

    pub fn server_with_compression_compressed_hello_two_chunks(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("418460b420bb92fced72").unwrap());
        self.data.lock().unwrap().write(ByteString::decode_hex("80833851d9d4f156d9").unwrap());
        let mut reader = self.get_server_reader_with_compression();
        reader.process_next_frame().unwrap();
        self.callback.assert_text_message("Hello");
    }

    pub fn client_two_frame_hello(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("010348656c").unwrap());
        self.data.lock().unwrap().write(ByteString::decode_hex("80026c6f").unwrap());
        let mut reader = self.get_client_reader();
        reader.process_next_frame().unwrap();
        self.callback.assert_text_message("Hello");
    }

    pub fn client_with_compression_two_frame_hello(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("010348656c").unwrap());
        self.data.lock().unwrap().write(ByteString::decode_hex("80026c6f").unwrap());
        let mut reader = self.get_client_reader_with_compression();
        reader.process_next_frame().unwrap();
        self.callback.assert_text_message("Hello");
    }

    pub fn client_with_compression_two_frame_compressed_hello(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("4104f248cdc9").unwrap());
        self.data.lock().unwrap().write(ByteString::decode_hex("8003c90700").unwrap());
        let mut reader = self.get_client_reader_with_compression();
        reader.process_next_frame().unwrap();
        self.callback.assert_text_message("Hello");
    }

    pub fn client_two_frame_hello_with_pongs(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("010348656c").unwrap());
        for _ in 0..4 {
            self.data.lock().unwrap().write(ByteString::decode_hex("8a00").unwrap());
        }
        self.data.lock().unwrap().write(ByteString::decode_hex("80026c6f").unwrap());
        let mut reader = self.get_client_reader();
        reader.process_next_frame().unwrap();
        for _ in 0..4 {
            self.callback.assert_pong(ByteString::EMPTY);
        }
        self.callback.assert_text_message("Hello");
    }

    pub fn client_two_frame_compressed_hello_with_pongs(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("4104f248cdc9").unwrap());
        for _ in 0..4 {
            self.data.lock().unwrap().write(ByteString::decode_hex("8a00").unwrap());
        }
        self.data.lock().unwrap().write(ByteString::decode_hex("8003c90700").unwrap());
        let mut reader = self.get_client_reader_with_compression();
        reader.process_next_frame().unwrap();
        for _ in 0..4 {
            self.callback.assert_pong(ByteString::EMPTY);
        }
        self.callback.assert_text_message("Hello");
    }

    pub fn client_incomplete_message_body_throws(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("810548656c").unwrap());
        let mut reader = self.get_client_reader();
        let res = reader.process_next_frame();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().downcast_ref::<std::io::Error>().unwrap().kind(), ErrorKind::UnexpectedEof);
    }

    pub fn client_uncompressed_message_with_compressed_flag_throws(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("c10548656c6c6f").unwrap());
        let mut reader = self.get_client_reader_with_compression();
        let res = reader.process_next_frame();
        assert!(res.is_err());
    }

    pub fn client_incomplete_control_frame_body_throws(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("8a0548656c").unwrap());
        let mut reader = self.get_client_reader();
        let res = reader.process_next_frame();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().downcast_ref::<std::io::Error>().unwrap().kind(), ErrorKind::UnexpectedEof);
    }

    pub fn server_incomplete_message_body_throws(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("818537fa213d7f9f4d").unwrap());
        let mut reader = self.get_server_reader();
        let res = reader.process_next_frame();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().downcast_ref::<std::io::Error>().unwrap().kind(), ErrorKind::UnexpectedEof);
    }

    pub fn server_incomplete_control_frame_body_throws(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("8a8537fa213d7f9f4d").unwrap());
        let mut reader = self.get_server_reader();
        let res = reader.process_next_frame();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().downcast_ref::<std::io::Error>().unwrap().kind(), ErrorKind::UnexpectedEof);
    }

    pub fn client_simple_binary(&mut self) {
        let bytes = self.binary_data(256);
        self.data.lock().unwrap().write(ByteString::decode_hex("827E0100").unwrap());
        self.data.lock().unwrap().write(bytes.clone());
        let mut reader = self.get_client_reader();
        reader.process_next_frame().unwrap();
        self.callback.assert_binary_message(bytes);
    }

    pub fn client_two_frame_binary(&mut self) {
        let bytes = self.binary_data(200);
        self.data.lock().unwrap().write(ByteString::decode_hex("0264").unwrap());
        self.data.lock().unwrap().write(ByteString::from(&bytes[0..100]));
        self.data.lock().unwrap().write(ByteString::decode_hex("8064").unwrap());
        self.data.lock().unwrap().write(ByteString::from(&bytes[100..200]));
        let mut reader = self.get_client_reader();
        reader.process_next_frame().unwrap();
        self.callback.assert_binary_message(bytes);
    }

    pub fn two_frame_not_continuation(&mut self) {
        let bytes = self.binary_data(200);
        self.data.lock().unwrap().write(ByteString::decode_hex("0264").unwrap());
        self.data.lock().unwrap().write(ByteString::from(&bytes[0..100]));
        self.data.lock().unwrap().write(ByteString::decode_hex("8264").unwrap());
        self.data.lock().unwrap().write(ByteString::from(&bytes[100..200]));
        let mut reader = self.get_client_reader();
        let res = reader.process_next_frame();
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Expected continuation opcode. Got: 2"));
    }

    pub fn empty_ping_calls_callback(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("8900").unwrap());
        let mut reader = self.get_client_reader();
        reader.process_next_frame().unwrap();
        self.callback.assert_ping(ByteString::EMPTY);
    }

    pub fn ping_calls_callback(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("890548656c6c6f").unwrap());
        let mut reader = self.get_client_reader();
        reader.process_next_frame().unwrap();
        self.callback.assert_ping(ByteString::from("Hello"));
    }

    pub fn empty_close_calls_callback(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("8800").unwrap());
        let mut reader = self.get_client_reader();
        reader.process_next_frame().unwrap();
        self.callback.assert_closing(1005, "");
    }

    pub fn close_length_of_one_throws(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("880100").unwrap());
        let mut reader = self.get_client_reader();
        let res = reader.process_next_frame();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Malformed close payload length of 1.");
    }

    pub fn close_calls_callback(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("880703e848656c6c6f").unwrap());
        let mut reader = self.get_client_reader();
        reader.process_next_frame().unwrap();
        self.callback.assert_closing(1000, "Hello");
    }

    pub fn close_incomplete_calls_callback(&mut self) {
        self.data.lock().unwrap().write(ByteString::decode_hex("880703e948656c6c6f").unwrap());
        // In Kotlin, data.close() is called. In this mock, we just simulate the reader's behavior.
        let mut reader = self.get_client_reader();
        reader.process_next_frame().unwrap();
        self.callback.assert_closing(1001, "Hello");
}}
