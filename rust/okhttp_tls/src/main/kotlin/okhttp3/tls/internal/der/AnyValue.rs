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
use std::hash::{Hash, Hasher};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

/*
 * A value whose type is not specified statically. Use this with [Adapters.any] which will attempt
 * to resolve a concrete type.
 */
#[derive(Debug, Clone, PartialEq)]
pub struct AnyValue {
    pub tag_class: i32,
    pub tag: i64,
    pub constructed: bool,
    pub length: i64,
    pub bytes: ByteString,
}

impl AnyValue {
    pub fn new(
        tag_class: i32,
        tag: i64,
        constructed: bool,
        length: i64,
        bytes: ByteString,
    ) -> Self {
        Self {
            tag_class,
            tag,
            constructed,
            length,
            bytes,
        }
    }
}

impl Hash for AnyValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // The Kotlin implementation manually implements hashCode to avoid Long.hashCode(long)
        // which wasn't available on Android 5. In Rust, we replicate the specific 
        // arithmetic logic provided in the source to preserve behavioral correctness.
        let mut result: i32 = 0;
        result = 31 * result + self.tag_class;
        result = 31 * result + (self.tag as i32);
        result = 31 * result + (if self.constructed { 0 } else { 1 });
        result = 31 * result + (self.length as i32);
        
        // For the ByteString part, we use the standard hash of the ByteString
        // as Kotlin's ByteString.hashCode() is used.
        let mut bytes_hasher = std::collections::hash_map::DefaultHasher::new();
        self.bytes.hash(&mut bytes_hasher);
        let bytes_hash = bytes_hasher.finish() as i32;
        
        result = 31 * result + bytes_hash;
        
        state.write_i32(result);
    }
}

// Helper trait to allow writing i32 to hasher for the custom hash logic
trait WriteI32 {
    fn write_i32(&mut self, i: i32);
}

impl<H: Hasher> WriteI32 for H {
    fn write_i32(&mut self, i: i32) {
        self.write_u64(i as u64);
    }
}

impl Default for AnyValue {
    fn default() -> Self {
        Self {
            tag_class: 0,
            tag: 0,
            constructed: false,
            length: -1,
            bytes: ByteString::new(),
        }
    }
}