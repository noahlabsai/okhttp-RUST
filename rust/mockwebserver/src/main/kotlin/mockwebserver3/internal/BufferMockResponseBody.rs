/*
 * Copyright (c) 2022 Block, Inc.
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
 *
 */

use crate::mockwebserver::src::main::kotlin::mockwebserver3::MockResponseBody;
use std::io::{Result, Write};
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

// In the original Kotlin code, `okio.Buffer` is used. 
// In Rust, the equivalent for a byte buffer that can be cloned and written to a sink 
// is typically a `Vec<u8>`.
pub type Buffer = Vec<u8>;

// Extension trait to provide `to_mock_response_body` functionality to `Buffer` (Vec<u8>).
pub trait BufferMockResponseBodyExt {
    fn to_mock_response_body(&self) -> Box<dyn MockResponseBody>;
}

impl BufferMockResponseBodyExt for Buffer {
    fn to_mock_response_body(&self) -> Box<dyn MockResponseBody> {
        // val defensiveCopy = clone()
        let defensive_copy = self.clone();
        Box::new(BufferMockResponseBody {
            buffer: defensive_copy,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BufferMockResponseBody {
    pub buffer: Buffer,
}

impl BufferMockResponseBody {
    pub fn new(buffer: Buffer) -> Self {
        Self { buffer }
    }
}

impl MockResponseBody for BufferMockResponseBody {
    fn content_length(&self) -> i64 {
        // override val contentLength = buffer.size
        self.buffer.len() as i64
    }

    fn write_to(&self, sink: &mut dyn Write) -> Result<()> {
        // buffer.copyTo(sink.buffer)
        // In Rust, writing the buffer to the sink is a direct write call.
        sink.write_all(&self.buffer)?;
        
        // sink.emitCompleteSegments() 
        // In the context of std::io::Write, flushing ensures all data is emitted.
        sink.flush()?;
        
        Ok(())
    }
}