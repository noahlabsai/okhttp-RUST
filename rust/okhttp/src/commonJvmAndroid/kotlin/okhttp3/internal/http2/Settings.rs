/*
 * Copyright (C) 2012 Square, Inc.
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

// Settings describe characteristics of the sending peer, which are used by the receiving peer.
// Settings are connection scoped.
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::build_logic::settings_gradle::*;


impl Settings {
    // From the HTTP/2 specs, the default initial window size for all streams is 64 KiB. (Chrome 25
    // uses 10 MiB).
    pub const DEFAULT_INITIAL_WINDOW_SIZE: i32 = 65535;

    // HTTP/2: Size in bytes of the table used to decode the sender's header blocks.
    pub const HEADER_TABLE_SIZE: usize = 1;

    // HTTP/2: The peer must not send a PUSH_PROMISE frame when this is 0.
    pub const ENABLE_PUSH: usize = 2;

    // Sender's maximum number of concurrent streams.
    pub const MAX_CONCURRENT_STREAMS: usize = 3;

    // Window size in bytes.
    pub const INITIAL_WINDOW_SIZE: usize = 4;

    // HTTP/2: Size in bytes of the largest frame payload the sender will accept.
    pub const MAX_FRAME_SIZE: usize = 5;

    // HTTP/2: Advisory only. Size in bytes of the largest header list the sender will accept.
    pub const MAX_HEADER_LIST_SIZE: usize = 6;

    // Total number of settings.
    pub const COUNT: usize = 10;

    pub fn new() -> Self {
        Self {
            set: 0,
            values: [0; Settings::(COUNT as usize)],
        }
    }

    // Returns -1 if unset.
    pub fn header_table_size(&self) -> i32 {
        let bit = 1 << Self::HEADER_TABLE_SIZE;
        if (bit & self.set) != 0 {
            self.values[Self::HEADER_TABLE_SIZE]
        } else {
            -1
        }
    }

    pub fn initial_window_size(&self) -> i32 {
        let bit = 1 << Self::INITIAL_WINDOW_SIZE;
        if (bit & self.set) != 0 {
            self.values[Self::INITIAL_WINDOW_SIZE]
        } else {
            Self::DEFAULT_INITIAL_WINDOW_SIZE
        }
    }

    pub fn clear(&mut self) {
        self.set = 0;
        self.values.fill(0);
    }

    // Sets the value for the given id. Discards unknown settings.
    pub fn set(&mut self, id: usize, value: i32) -> &mut Self {
        if id >= self.values.len() {
            return self; // Discard unknown settings.
        }

        let bit = 1 << id;
        self.set |= bit;
        self.values[id] = value;
        self
    }

    // Returns true if a value has been assigned for the setting `id`.
    pub fn is_set(&self, id: usize) -> bool {
        if id >= Self::COUNT {
            return false;
        }
        let bit = 1 << id;
        (self.set & bit) != 0
    }

    // Returns the value for the setting `id`, or 0 if unset.
    pub fn get(&self, id: usize) -> i32 {
        self.values[id]
    }

    // Returns the number of settings that have values assigned.
    pub fn size(&self) -> i32 {
        self.set.count_ones() as i32
    }

    pub fn get_enable_push(&self, default_value: bool) -> bool {
        let bit = 1 << Self::ENABLE_PUSH;
        if (bit & self.set) != 0 {
            self.values[Self::ENABLE_PUSH] == 1
        } else {
            default_value
        }
    }

    pub fn get_max_concurrent_streams(&self) -> i32 {
        let bit = 1 << Self::MAX_CONCURRENT_STREAMS;
        if (bit & self.set) != 0 {
            self.values[Self::MAX_CONCURRENT_STREAMS]
        } else {
            i32::MAX
        }
    }

    pub fn get_max_frame_size(&self, default_value: i32) -> i32 {
        let bit = 1 << Self::MAX_FRAME_SIZE;
        if (bit & self.set) != 0 {
            self.values[Self::MAX_FRAME_SIZE]
        } else {
            default_value
        }
    }

    pub fn get_max_header_list_size(&self, default_value: i32) -> i32 {
        let bit = 1 << Self::MAX_HEADER_LIST_SIZE;
        if (bit & self.set) != 0 {
            self.values[Self::MAX_HEADER_LIST_SIZE]
        } else {
            default_value
        }
    }

    // Writes `other` into this. If any setting is populated by this and `other`, the
    // value and flags from `other` will be kept.
    pub fn merge(&mut self, other: &Settings) {
        for i in 0..Self::COUNT {
            if !other.is_set(i) {
                continue;
            }
            self.set(i, other.get(i));
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}