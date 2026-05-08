/*
 * Copyright (C) 2024 Block, Inc.
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
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixList;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

// An implementation of I/O for `PublicSuffixDatabase` by directly passing in ByteStrings.
pub struct ConfiguredPublicSuffixList {
    pub bytes: ByteString,
    pub exception_bytes: ByteString,
}

impl Default for ConfiguredPublicSuffixList {
    fn default() -> Self {
        Self {
            bytes: ByteString::EMPTY,
            exception_bytes: ByteString::EMPTY,
        }
    }
}

impl PublicSuffixList for ConfiguredPublicSuffixList {

    fn bytes(&self) -> ByteString {
        self.bytes.clone()
    }

    fn exception_bytes(&self) -> ByteString {
        self.exception_bytes.clone()
    }
}