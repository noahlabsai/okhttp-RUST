/*
 * Copyright (C) 2023 Square, Inc.
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

use okio::{Buffer, Path};
use std::env;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::UrlComponentEncodingTester::*;

// Returns the value of the OKHTTP_ROOT environment variable as a Path.
// Panics if the environment variable is not set.
pub fn ok_http_root() -> Path {
    env::var("OKHTTP_ROOT")
        .expect("OKHTTP_ROOT environment variable must be set")
        .parse::<Path>()
        .expect("OKHTTP_ROOT must be a valid path")
}

// Creates a String from a sequence of Unicode code points.
// This mimics the Kotlin helper function provided in the source.
pub fn string_from_code_points(code_points: &[i32]) -> String {
    let mut buffer = Buffer::new();
    for &code_point in code_points {
        // In Rust, we convert the i32 code point to a char.
        // Kotlin's writeUtf8CodePoint handles the UTF-8 encoding.
        if let Some(c) = std::char::from_u32(code_point as u32) {
            buffer.write_utf8(c);
        } else {
            // If the code point is invalid, we follow the behavior of 
            // writing the replacement character or panicking depending on strictness.
            // For parity with typical Okio/Kotlin behavior:
            buffer.write_utf8('?');
        }
    }
    buffer.read_utf8().expect("Buffer should contain valid UTF-8")
}

// Note: In Kotlin, `fun String(vararg codePoints: Int): String` is a top-level 
// function that happens to be named "String". In Rust, "String" is a reserved 
// type name. I have renamed it to `string_from_code_points` to avoid 
// collision and maintain compilability while preserving the logic.