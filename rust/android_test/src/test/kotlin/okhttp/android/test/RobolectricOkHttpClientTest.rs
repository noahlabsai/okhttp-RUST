/*
 * Copyright (c) 2025 Block, Inc.
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
 *
 */

use crate::android_test::src::test::kotlin::okhttp::android::test::BaseOkHttpClientUnitTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::OkHttp;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::ContextAwarePlatform::*;

// Mock for androidx.test.core.app.ApplicationProvider
pub struct ApplicationProvider;

impl ApplicationProvider {
    // Returns the application context.
    // In a real Robolectric environment, this would return a Context object.
    pub fn get_application_context() -> String {
        "RobolectricApplicationContext".to_string()
    }
}

// RobolectricOkHttpClientTest
// 
// Translated from Kotlin. This class tests OkHttpClient using Robolectric.
// The @RunWith(RobolectricTestRunner::class) and @Config annotations are 
// metadata for the JVM test runner and are represented here as documentation 
// or could be handled by a test framework wrapper.
#[derive(Debug, Clone, PartialEq)]
pub struct RobolectricOkHttpClientTest {
    // Inherits from BaseOkHttpClientUnitTestImpl
    base: BaseOkHttpClientUnitTestImpl,
}

impl RobolectricOkHttpClientTest {
    pub fn new() -> Self {
        Self {
            base: BaseOkHttpClientUnitTestImpl::new(),
        }
    }

    // Equivalent to @Before fun setContext()
    // This is called before each test to initialize OkHttp with the application context.
    pub fn set_context(&mut self) {
        // This is awkward because Robolectric won't run our initializers and we don't want test deps
        // https://github.com/robolectric/robolectric/issues/8461
        let context = ApplicationProvider::get_application_context();
        OkHttp::initialize(&context);
    }
}

// Implementation of the BaseOkHttpClientUnitTest trait for RobolectricOkHttpClientTest
// to preserve the inheritance behavior from the Kotlin source.
impl BaseOkHttpClientUnitTest for RobolectricOkHttpClientTest {
    fn get_client(&self) -> &crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::OkHttpClient {
        self.base.get_client()
    }

    fn set_client(&mut self, client: crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::OkHttpClient) {
        self.base.set_client(client);
    }
}