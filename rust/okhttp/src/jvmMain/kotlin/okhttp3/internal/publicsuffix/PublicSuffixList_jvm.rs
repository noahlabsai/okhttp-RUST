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

use crate::okhttp3::internal::publicsuffix::{PublicSuffixList, ResourcePublicSuffixList};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

// Companion object implementation for PublicSuffixList.
// In Rust, companion object methods are implemented as associated functions on the type.
impl PublicSuffixList {
    // Returns the default PublicSuffixList implementation.
    // This corresponds to the `Companion.Default` property in Kotlin.
    pub fn default_list() -> Box<dyn PublicSuffixList> {
        Box::new(ResourcePublicSuffixList::new())
    }
}

// To maintain the exact API surface of `PublicSuffixList.Companion.Default`, 
// we can provide a constant or a function. Since the Kotlin code uses a getter 
// that instantiates `ResourcePublicSuffixList()`, a function is the correct translation.
pub fn get_default_public_suffix_list() -> Box<dyn PublicSuffixList> {
    PublicSuffixList::default_list()
}