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

use std::sync::Mutex;
use once_cell::sync::Lazy;

// Mocking the external Android/Robolectric dependencies as they are not provided in the source
// but are required for the code to be compilable.
pub struct RobolectricTestRunner;

pub struct ApplicationProvider;
impl ApplicationProvider {
    pub fn get_application_context() -> String {
        "android_application_context".to_string()
    }
}

// Mocking PlatformRegistry based on the usage in the Kotlin source
pub struct PlatformRegistry;
impl PlatformRegistry {
    // Using Lazy<Mutex<Option<T>>> to safely implement the singleton mutable state 
    // that was previously a 'var' in the companion object.
    pub fn application_context() -> &'static Mutex<Option<String>> {
        static INSTANCE: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
        &INSTANCE
    }

    pub fn set_application_context(context: String) {
        let mut lock = Self::application_context().lock().unwrap();
        *lock = Some(context);
    }
}

/// actual typealias PublicSuffixTestRunner = RobolectricTestRunner
pub type PublicSuffixTestRunner = RobolectricTestRunner;

/// actual fun beforePublicSuffixTest() {
///   PlatformRegistry.applicationContext = ApplicationProvider.getApplicationContext()
/// }
pub fn before_public_suffix_test() {
    let context = ApplicationProvider::get_application_context();
    PlatformRegistry::set_application_context(context);
}