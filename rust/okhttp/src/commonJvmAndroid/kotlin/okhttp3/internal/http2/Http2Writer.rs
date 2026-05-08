/*
 * Copyright (C) 2011 The Android Open Source Project
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

use std::io::{self, Write};
use std::sync::Mutex;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::concurrent::with_lock;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::settings_gradle::*;
use crate::build_logic::src::main::kotlin::JavaModules::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::ErrorCode::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Reader::*;

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::{
    CONNECTION_PREFACE, FLAG_ACK, FLAG_END_HEADERS, FLAG_END_STREAM, FLAG_NONE,
    INITIAL_MAX_FRAME_SIZE, TYPE_CONTINUATION, TYPE_DATA, TYPE_GOAWAY, TYPE_HEADERS,
    TYPE_PING, TYPE_PUSH_PROMISE, TYPE_RST_STREAM, TYPE_SETTINGS, TYPE_WINDOW_UPDATE,
    Header, Settings, ErrorCode, Hpack,
};

// Mocking Okio-like Buffer and BufferedSink for compilability based on the provided context
pub trait BufferedSink: Write {
    fn write_byte(&mut self, b: i8) -> io::Result<()>;
    fn write_short(&mut self, v: i16) -> io::Result<()>;
    fn write_int(&mut self, v: i32) -> io::Result<()>;
    fn write_medium(&mut self, v: i32) -> io::Result<()>;
    fn write_buffer(&mut self, buffer: &mut Buffer, byte_count: i64) -> io::Result<()>;
}


// Writes HTTP/2 transport frames.
pub struct Http2Writer<S: BufferedSink> {
    sink: S,
    client: bool,
    hpack_buffer: Mutex<Buffer>,
    max_frame_size: Mutex<i32>,
    closed: Mutex<bool>,
    hpack_writer: Mutex<Hpack::Writer>,
}

impl<S: BufferedSink> Http2Writer<S> {
    pub fn new(sink: S, client: bool) -> Self {
        let hpack_buffer = Mutex::new(Vec::new());
        // In a real implementation, Hpack::Writer would hold a reference or a handle to the buffer.
        // For this translation, we maintain the structure.
        Self {
            sink,
            client,
            hpack_buffer,
            max_frame_size: Mutex::new(INITIAL_MAX_FRAME_SIZE),
            closed: Mutex::new(false),
            hpack_writer: Mutex::new(Hpack::Writer::new()), 
        }
    }

    pub fn connection_preface(&mut self) -> io::Result<()> {
        with_lock(&self.closed, |closed| {
            if *closed {
                return Err(io::Error::new(io::ErrorKind::Other, "closed"));
            }
            if !self.client {
                return Ok(());
            }
            // Logging omitted as per Rust translation patterns for java.util.logging
            self.sink.write_all(CONNECTION_PREFACE)?;
            self.sink.flush()?;
            Ok(())
        })
    }

    // Applies `peer_settings` and then sends a settings ACK.
    pub fn apply_and_ack_settings(&mut self, peer_settings: Settings) -> io::Result<()> {
        with_lock(&self.closed, |closed| {
            if *closed {
                return Err(io::Error::new(io::ErrorKind::Other, "closed"));
            }
            
            let mut max_frame_size_lock = self.max_frame_size.lock().unwrap();
            *max_frame_size_lock = peer_settings.get_max_frame_size(*max_frame_size_lock);
            
            if peer_settings.header_table_size != -1 {
                self.hpack_writer.lock().unwrap().resize_header_table(peer_settings.header_table_size);
            }
            
            self.frame_header(
                0,
                0,
                TYPE_SETTINGS,
                FLAG_ACK,
            )?;
            self.sink.flush()?;
            Ok(())
        })
    }

    pub fn push_promise(
        &mut self,
        stream_id: i32,
        promised_stream_id: i32,
        request_headers: Vec<Header>,
    ) -> io::Result<()> {
        with_lock(&self.closed, |closed| {
            if *closed {
                return Err(io::Error::new(io::ErrorKind::Other, "closed"));
            }
            
            self.hpack_writer.lock().unwrap().write_headers(request_headers)?;

            let mut hpack_buffer = self.hpack_buffer.lock().unwrap();
            let byte_count = hpack_buffer.len() as i64;
            let max_frame_size = *self.max_frame_size.lock().unwrap();
            let length = std::cmp::min(max_frame_size as i64 - 4, byte_count) as i32;
            
            self.frame_header(
                stream_id,
                length + 4,
                TYPE_PUSH_PROMISE,
                if byte_count == length as i64 { FLAG_END_HEADERS } else { 0 },
            )?;
            
            self.sink.write_int(promised_stream_id & 0x7fffffff)?;
            self.sink.write_buffer(&mut hpack_buffer, length as i64)?;

            if byte_count > length as i64 {
                self.write_continuation_frames(stream_id, byte_count - length as i64)?;
            }
            Ok(())
        })
    }

    pub fn flush(&mut self) -> io::Result<()> {
        with_lock(&self.closed, |closed| {
            if *closed {
                return Err(io::Error::new(io::ErrorKind::Other, "closed"));
            }
            self.sink.flush()?;
            Ok(())
        })
    }

    pub fn rst_stream(&mut self, stream_id: i32, error_code: ErrorCode) -> io::Result<()> {
        with_lock(&self.closed, |closed| {
            if *closed {
                return Err(io::Error::new(io::ErrorKind::Other, "closed"));
            }
            if error_code.http_code == -1 {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "errorCode.httpCode == -1"));
            }

            self.frame_header(
                stream_id,
                4,
                TYPE_RST_STREAM,
                FLAG_NONE,
            )?;
            self.sink.write_int(error_code.http_code)?;
            self.sink.flush()?;
            Ok(())
        })
    }

    pub fn max_data_length(&self) -> i32 {
        *self.max_frame_size.lock().unwrap()
    }

    pub fn data(
        &mut self,
        out_finished: bool,
        stream_id: i32,
        source: Option<Buffer>,
        byte_count: i32,
    ) -> io::Result<()> {
        with_lock(&self.closed, |closed| {
            if *closed {
                return Err(io::Error::new(io::ErrorKind::Other, "closed"));
            }
            let mut flags = FLAG_NONE;
            if out_finished {
                flags |= FLAG_END_STREAM;
            }
            self.data_frame(stream_id, flags, source, byte_count)
        })
    }

    pub fn data_frame(
        &mut self,
        stream_id: i32,
        flags: i32,
        buffer: Option<Buffer>,
        byte_count: i32,
    ) -> io::Result<()> {
        self.frame_header(
            stream_id,
            byte_count,
            TYPE_DATA,
            flags,
        )?;
        if byte_count > 0 {
            let mut buf = buffer.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "buffer is null"))?;
            self.sink.write_buffer(&mut buf, byte_count as i64)?;
        }
        Ok(())
    }

    pub fn settings(&mut self, settings: Settings) -> io::Result<()> {
        with_lock(&self.closed, |closed| {
            if *closed {
                return Err(io::Error::new(io::ErrorKind::Other, "closed"));
            }
            self.frame_header(
                0,
                settings.size() * 6,
                TYPE_SETTINGS,
                FLAG_NONE,
            )?;
            for i in 0..Settings::COUNT {
                if !settings.is_set(i) {
                    continue;
                }
                self.sink.write_short(i as i16)?;
                self.sink.write_int(settings.get(i))?;
            }
            self.sink.flush()?;
            Ok(())
        })
    }

    pub fn ping(&mut self, ack: bool, payload1: i32, payload2: i32) -> io::Result<()> {
        with_lock(&self.closed, |closed| {
            if *closed {
                return Err(io::Error::new(io::ErrorKind::Other, "closed"));
            }
            self.frame_header(
                0,
                8,
                TYPE_PING,
                if ack { FLAG_ACK } else { FLAG_NONE },
            )?;
            self.sink.write_int(payload1)?;
            self.sink.write_int(payload2)?;
            self.sink.flush()?;
            Ok(())
        })
    }

    pub fn go_away(&mut self, last_good_stream_id: i32, error_code: ErrorCode, debug_data: Vec<u8>) -> io::Result<()> {
        with_lock(&self.closed, |closed| {
            if *closed {
                return Err(io::Error::new(io::ErrorKind::Other, "closed"));
            }
            if error_code.http_code == -1 {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "errorCode.httpCode == -1"));
            }
            self.frame_header(
                0,
                8 + debug_data.len() as i32,
                TYPE_GOAWAY,
                FLAG_NONE,
            )?;
            self.sink.write_int(last_good_stream_id)?;
            self.sink.write_int(error_code.http_code)?;
            if !debug_data.is_empty() {
                self.sink.write_all(&debug_data)?;
            }
            self.sink.flush()?;
            Ok(())
        })
    }

    pub fn window_update(&mut self, stream_id: i32, window_size_increment: i64) -> io::Result<()> {
        with_lock(&self.closed, |closed| {
            if *closed {
                return Err(io::Error::new(io::ErrorKind::Other, "closed"));
            }
            if window_size_increment == 0 || window_size_increment > 0x7fffffff {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("windowSizeIncrement == 0 || windowSizeIncrement > 0x7fffffffL: {}", window_size_increment)));
            }
            
            self.frame_header(
                stream_id,
                4,
                TYPE_WINDOW_UPDATE,
                FLAG_NONE,
            )?;
            self.sink.write_int(window_size_increment as i32)?;
            self.sink.flush()?;
            Ok(())
        })
    }

    pub fn frame_header(&mut self, stream_id: i32, length: i32, frame_type: i32, flags: i32) -> io::Result<()> {
        let max_frame_size = *self.max_frame_size.lock().unwrap();
        if length > max_frame_size {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("FRAME_SIZE_ERROR length > {}: {}", max_frame_size, length)));
        }
        if (stream_id & 0x80000000) != 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("reserved bit set: {}", stream_id)));
        }
        
        self.sink.write_medium(length)?;
        self.sink.write_byte((frame_type & 0xff) as i8)?;
        self.sink.write_byte((flags & 0xff) as i8)?;
        self.sink.write_int(stream_id & 0x7fffffff)?;
        Ok(())
    }

    pub fn close(&mut self) -> io::Result<()> {
        with_lock(&self.closed, |closed| {
            *closed = true;
            self.sink.flush()?; 
            Ok(())
        })
    }

    fn write_continuation_frames(&mut self, stream_id: i32, mut byte_count: i64) -> io::Result<()> {
        while byte_count > 0 {
            let max_frame_size = *self.max_frame_size.lock().unwrap();
            let length = std::cmp::min(max_frame_size as i64, byte_count);
            byte_count -= length;
            
            self.frame_header(
                stream_id,
                length as i32,
                TYPE_CONTINUATION,
                if byte_count == 0 { FLAG_END_HEADERS } else { 0 },
            )?;
            
            let mut hpack_buffer = self.hpack_buffer.lock().unwrap();
            self.sink.write_buffer(&mut hpack_buffer, length)?;
        }
        Ok(())
    }

    pub fn headers(&mut self, out_finished: bool, stream_id: i32, header_block: Vec<Header>) -> io::Result<()> {
        with_lock(&self.closed, |closed| {
            if *closed {
                return Err(io::Error::new(io::ErrorKind::Other, "closed"));
            }
            
            self.hpack_writer.lock().unwrap().write_headers(header_block)?;

            let mut hpack_buffer = self.hpack_buffer.lock().unwrap();
            let byte_count = hpack_buffer.len() as i64;
            let max_frame_size = *self.max_frame_size.lock().unwrap();
            let length = std::cmp::min(max_frame_size as i64, byte_count);
            
            let mut flags = if byte_count == length { FLAG_END_HEADERS } else { 0 };
            if out_finished {
                flags |= FLAG_END_STREAM;
            }
            
            self.frame_header(
                stream_id,
                length as i32,
                TYPE_HEADERS,
                flags,
            )?;
            
            self.sink.write_buffer(&mut hpack_buffer, length)?;

            if byte_count > length {
                self.write_continuation_frames(stream_id, byte_count - length)?;
            }
            Ok(())
        })
    }
}

impl<S: BufferedSink> Drop for Http2Writer<S> {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
