/*
 * Copyright (C) 2019 Square, Inc.
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

use std::sync::OnceLock;
use std::env;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

// PlatformVersion provides information about the JVM specification version.
pub struct PlatformVersion;

impl PlatformVersion {
    // Returns the major version of the JVM.
    // This is lazily initialized and cached.
    pub fn major_version() -> i32 {
        pub static MAJOR_VERSION: OnceLock<i32> = OnceLock::new();
        *MAJOR_VERSION.get_or_init(|| {
            let jvm_spec_version = Self::get_jvm_spec_version();
            match jvm_spec_version.as_str() {
                "1.8" => 8,
                _ => jvm_spec_version.parse::<i32>().unwrap_or(0),
            }
        })
    }

    // Returns the JVM specification version as a string.
    // In Rust, this mimics System.getProperty("java.specification.version", "unknown")
    // by checking the environment or returning a default.
    pub fn get_jvm_spec_version() -> String {
        // Note: In a real JVM-to-Rust bridge (like JNI), this would call the JVM.
        // For a standalone Rust translation, we check the environment variable 
        // or return the default "unknown".
        env::var("JAVA_SPECIFICATION_VERSION").unwrap_or_else(|_| "unknown".to_string())
    }
}