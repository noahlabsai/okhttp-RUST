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

use crate::okhttp3::internal::platform::Platform;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test::src::test::kotlin::okhttp::android::test::DisabledInitialiserTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

// PlatformRegistry is a singleton object that provides access to the current platform implementation.
// In Kotlin, this was an `expect object`, which in Rust is represented as a struct with associated functions.

impl PlatformRegistry {
    // Finds the platform implementation for the current environment.
    pub fn find_platform() -> Platform {
        // In a multi-platform Rust project, this would be implemented via cfg attributes
        // or a trait-based provider. Since this is a translation of an 'expect' object,
        // the implementation is deferred to the platform-specific target files.
        #[cfg(target_os = "android")]
        {
            // This is a generated-compatibility for the actual Android implementation
            // which would typically call a platform-specific provider.
            panic!("PlatformRegistry::find_platform: Android implementation required")
        }
        #[cfg(not(target_os = "android"))]
        {
            // This is a generated-compatibility for the actual JVM/Other implementation.
            panic!("PlatformRegistry::find_platform: Non-Android implementation required")
        }
    }

    // Returns true if the current platform is Android.
    pub fn is_android() -> bool {
        #[cfg(target_os = "android")]
        {
            true
        }
        #[cfg(not(target_os = "android"))]
        {
            false
        }
    }
}
