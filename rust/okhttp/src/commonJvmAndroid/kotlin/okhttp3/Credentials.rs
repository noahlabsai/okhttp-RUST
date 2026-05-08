/*
 * Copyright (C) 2014 Square, Inc.
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

use base64::{engine::general_purpose, Engine as _};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Challenge::*;

// Factory for HTTP authorization credentials.
pub struct Credentials;

impl Credentials {
    // Returns an auth credential for the Basic scheme.
    // 
    // In Kotlin, the default charset is ISO_8859_1. 
    // In Rust, we represent the charset as a function or a type that handles the encoding.
    // Since ISO_8859_1 is a 1-to-1 mapping for the first 256 Unicode codepoints, 
    // we implement the encoding logic to match the behavior of `okio.ByteString.encode(charset)`.
    pub fn basic(
        username: &str,
        password: &str,
        charset: Option<&str>,
    ) -> String {
        let username_and_password = format!("{}:{}", username, password);
        
        // Default to ISO_8859_1 if no charset is provided
        let encoding = charset.unwrap_or("ISO-8859-1");
        
        let bytes = match encoding {
            "ISO-8859-1" => {
                // ISO-8859-1 encoding: each char is cast to u8. 
                // This matches the JVM's ISO_8859_1 behavior for strings.
                username_and_password.chars().map(|c| c as u8).collect::<Vec<u8>>()
            }
            "UTF-8" => {
                username_and_password.as_bytes().to_vec()
            }
            _ => {
                // For other charsets, we fall back to UTF-8 as a safe default in Rust,
                // though in a full production system, a crate like `encoding_rs` would be used.
                username_and_password.as_bytes().to_vec()
            }
        };

        let encoded = general_purpose::STANDARD.encode(bytes);
        format!("Basic {}", encoded)
    }

    // Overload to support the default parameter behavior of Kotlin's @JvmOverloads
    pub fn basic_default(username: &str, password: &str) -> String {
        Self::basic(username, password, None)
    }
}