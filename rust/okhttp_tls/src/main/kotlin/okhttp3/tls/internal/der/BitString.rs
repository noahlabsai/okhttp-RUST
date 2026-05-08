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

use okio::ByteString;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;

/*
 * Like a [ByteString], but whose bits are not necessarily a strict multiple of 8.
 */
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BitString {
    pub byte_string: ByteString,
    /* 0-7 unused bits in the last byte. */
    pub unused_bits_count: i32,
}

impl BitString {
    pub fn new(byte_string: ByteString, unused_bits_count: i32) -> Self {
        Self {
            byte_string,
            unused_bits_count,
        }
    }
}
