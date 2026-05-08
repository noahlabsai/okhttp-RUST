/*
 * Copyright (C) 2020 Square, Inc.
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

use std::io::{self, Read};
use okio::{Buffer, InflaterSource};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;

use flate2::decompress::inflate; // Note: In a real okio-rust port, Inflater would be the flate2 equivalent
use flate2::zlib::ZlibDecoder; // Used for the 'nowrap' logic in Inflater(true)

const OCTETS_TO_ADD_BEFORE_INFLATION: i32 = 0x0000ffff;

// Mocking the JVM Inflater behavior as used in okhttp.
// In a production Rust environment, this would wrap `flate2` or `miniz_oxide`.
pub struct Inflater {
    pub bytes_read: i64,
    pub finished: bool,
}

impl Inflater {
    pub fn new(nowrap: bool) -> Self {
        // In Kotlin, Inflater(true) means nowrap = true
        Self {
            bytes_read: 0,
            finished: false,
        }
    }

    pub fn reset(&mut self) {
        self.bytes_read = 0;
        self.finished = false;
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }
}

pub struct MessageInflater {
    no_context_takeover: bool,
    deflated_bytes: Buffer,
    inflater: Option<Inflater>,
    inflater_source: Option<InflaterSource>,
}

impl MessageInflater {
    pub fn new(no_context_takeover: bool) -> Self {
        Self {
            no_context_takeover,
            deflated_bytes: Buffer::new(),
            inflater: None,
            inflater_source: None,
        }
    }

    // Inflates `buffer` in place as described in RFC 7692 section 7.2.2.
    pub fn inflate(&mut self, buffer: &mut Buffer) -> io::Result<()> {
        if self.deflated_bytes.size() != 0 {
            panic!("deflatedBytes.size == 0L required");
        }

        // Lazily initialize inflater
        if self.inflater.is_none() {
            self.inflater = Some(Inflater::new(true));
        }
        
        // Lazily initialize inflater_source
        if self.inflater_source.is_none() {
            // In Rust's okio port, InflaterSource takes the source and the inflater
            // We use a reference to the inflater here.
            let mut inflater = self.inflater.as_ref().unwrap();
            // Note: In actual okio-rust, InflaterSource would be constructed here.
            // Since we are translating the logic:
            self.inflater_source = Some(InflaterSource::new(self.deflated_bytes.clone(), inflater));
        }

        let mut inflater = self.inflater.as_mut().unwrap();
        let mut inflater_source = self.inflater_source.as_mut().unwrap();

        if self.no_context_takeover {
            inflater.reset();
        }

        // deflatedBytes.writeAll(buffer)
        self.deflated_bytes.write_all(buffer)?;
        
        // deflatedBytes.writeInt(OCTETS_TO_ADD_BEFORE_INFLATION)
        self.deflated_bytes.write_int(OCTETS_TO_ADD_BEFORE_INFLATION as i32);

        let total_bytes_to_read = inflater.bytes_read + self.deflated_bytes.size();

        // We cannot read all, as the source does not close.
        // Instead, we ensure that all bytes from source have been processed by inflater.
        loop {
            // readOrInflate(buffer, Long.MAX_VALUE)
            inflater_source.read_or_inflate(buffer, i64::MAX)?;
            
            if inflater.bytes_read >= total_bytes_to_read || inflater.is_finished() {
                break;
            }
        }

        // The inflater data was self-terminated and there's unexpected trailing data. 
        // Tear it all down so we don't leak that data into the input of the next message.
        if inflater.bytes_read < total_bytes_to_read {
            self.deflated_bytes.clear();
            if let Some(mut source) = self.inflater_source.take() {
                let _ = source.close();
            }
            self.inflater = None;
        }

        Ok(())
    }
}

impl Drop for MessageInflater {
    fn drop(&mut self) {
        // Equivalent to close()
        if let Some(mut source) = self.inflater_source.take() {
            let _ = source.close();
        }
        self.inflater = None;
    }
}

// Implementing a close method to match the Closeable interface
impl MessageInflater {
    pub fn close(&mut self) -> io::Result<()> {
        if let Some(mut source) = self.inflater_source.take() {
            source.close()?;
        }
        self.inflater = None;
        Ok(())
    }
}