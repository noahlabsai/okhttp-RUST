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
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

// Basic I/O for `PublicSuffixDatabase.list`
pub trait PublicSuffixList {
    fn ensure_loaded(&self);

    fn bytes(&self) -> ByteString;

    fn exception_bytes(&self) -> ByteString;
}

// Companion object equivalent for PublicSuffixList
pub struct PublicSuffixListCompanion;

impl PublicSuffixListCompanion {
    // This is the Rust equivalent of the `expect val PublicSuffixList.Companion.Default`
    // Since the actual implementation is platform-specific (expect), 
    // this function provides the access point to that implementation.
    pub fn default() -> Box<dyn PublicSuffixList> {
        // The actual implementation is provided by the platform-specific module
        // in the original Kotlin multiplatform project.
        crate::okhttp3::internal::publicsuffix::platform::get_default_public_suffix_list()
    }
}

// To maintain the Kotlin-like access `PublicSuffixList::Default`, 
// we can provide a constant or a static reference if the platform implementation allows.