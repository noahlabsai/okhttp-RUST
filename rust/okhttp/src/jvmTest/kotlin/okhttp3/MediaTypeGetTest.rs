/*
 * Copyright (C) 2022 Square, Inc.
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

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::MediaType;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::MediaTypeTest;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::MediaTypeTest::*;

// Extension trait to provide the `to_media_type` functionality 
// equivalent to MediaType.Companion.toMediaType in Kotlin.
pub trait MediaTypeCompanionExt {
    fn to_media_type(&self) -> MediaType;
}

impl MediaTypeCompanionExt for str {
    fn to_media_type(&self) -> MediaType {
        // In a real production environment, this would call the actual 
        // MediaType::parse implementation.
        MediaType::parse(self).expect("Failed to parse media type")
    }
}

pub struct MediaTypeGetTest;

impl MediaTypeGetTest {
    pub fn new() -> Self {
        Self
    }

    // Overrides the parse method from MediaTypeTest to use the extension function.
    pub fn parse(&self, string: &str) -> MediaType {
        string.to_media_type()
    }

    // Overrides the assert_invalid method from MediaTypeTest.
    // 
    // In Rust, since we don't have a direct equivalent to `assertFailsWith`,
    // we use a closure to capture the result and verify it is an Err.
    pub fn assert_invalid(&self, string: &str, exception_message: Option<&str>) {
        // We simulate the behavior of assertFailsWith<IllegalArgumentException>
        // by attempting to parse and checking if it returns an error.
        // Note: This assumes MediaType::parse returns a Result in the actual implementation.
        let result = std::panic::catch_unwind(|| {
            self.parse(string)
        });

        match result {
            Ok(_) => {
                panic!("Expected IllegalArgumentException but no exception was thrown for string: {}", string);
            }
            Err(e) => {
                // In Kotlin, e.message is checked. In Rust, we'd need to downcast the panic payload.
                if let Some(msg) = e.downcast_ref::<String>() {
                    assert_eq!(Some(msg.as_str()), exception_message);
                } else if let Some(msg) = e.downcast_ref::<&str>() {
                    assert_eq!(Some(*msg), exception_message);
                } else {
                    // If the panic payload isn't a string, we can't verify the message, 
                    // but the fact that it failed is what assertFailsWith primarily checks.
                    if exception_message.is_some() {
                        panic!("Exception thrown but message could not be extracted for verification");
                    }
                }
            }
        }
    }
}

// To maintain the inheritance relationship from MediaTypeTest, 
// we implement the methods of MediaTypeTest for MediaTypeGetTest 
// or use composition. Since Rust doesn't have class inheritance, 
// we provide the specific overrides here.