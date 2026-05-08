/*
 * Copyright (C) 2012 The Android Open Source Project
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

use unicode_normalization::UnicodeNormalization;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::JavaModules::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

// Normalizes the given string using NFC (Normalization Form C).
// 
// In Kotlin, this uses `java.text.Normalizer.normalize(string, NFC)`.
// In Rust, the `unicode-normalization` crate provides the equivalent functionality.
pub fn normalize_nfc(string: String) -> String {
    // .nfc() returns an iterator of chars; we collect them back into a String.
    string.nfc().collect()
}