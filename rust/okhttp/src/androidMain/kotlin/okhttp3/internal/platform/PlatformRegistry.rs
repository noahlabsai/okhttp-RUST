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

use std::sync::Arc;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::android::AndroidLog;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test::src::test::kotlin::okhttp::android::test::DisabledInitialiserTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::ContextAwarePlatform::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::platform::Jdk9Platform::*;

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::platform::{
    Platform, ContextAwarePlatform, Android10Platform, AndroidPlatform, Jdk9Platform,
};

// Mocking Android Build.VERSION.SDK_INT as it is a platform-specific constant
pub struct BuildVersion;
impl BuildVersion {
    pub const SDK_INT: i32 = 0; // This would be provided by the Android NDK/SDK bindings
}


impl PlatformRegistry {
    // Finds the appropriate platform implementation for the current environment.
    pub fn find_platform() -> Arc<dyn Platform> {
        AndroidLog::enable();

        // Try Android 10+ then general Android
        let android_platform = Android10Platform::build_if_supported()
            .or_else(|| AndroidPlatform::build_if_supported());

        if let Some(platform) = android_platform {
            return Arc::new(platform);
        }

        // If the API version is 0, assume this is the Android artifact, but running on the JVM without Robolectric.
        if BuildVersion::SDK_INT == 0 {
            if let Some(jdk_platform) = Jdk9Platform::build_if_supported() {
                return Arc::new(jdk_platform);
            }
            return Arc::new(Platform::default());
        }

        panic!("Expected Android API level 21+ but was {}", BuildVersion::SDK_INT);
    }

    // Returns true if the current platform is Android.
    pub fn is_android() -> bool {
        true
    }

    // Gets the application context from the current platform if it is context-aware.
    pub fn get_application_context() -> Option<crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::Context> {
        Platform::get()
            .and_then(|p| p.as_any().downcast_ref::<Arc<dyn ContextAwarePlatform>>())
            .and_then(|cap| cap.application_context())
    }

    // Sets the application context for the current platform if it is context-aware.
    pub fn set_application_context(context: Option<crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::Context>) {
        if let Some(p) = Platform::get() {
            if let Some(cap) = p.as_any().downcast_ref::<Arc<dyn ContextAwarePlatform>>() {
                // Note: In Rust, updating a shared Arc requires interior mutability (like Mutex/RwLock)
                // which would be implemented inside the ContextAwarePlatform implementation.
                cap.set_application_context(context);
            }
        }
    }
}