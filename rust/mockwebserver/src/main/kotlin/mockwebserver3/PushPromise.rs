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

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers;
use crate::mockwebserver3::MockResponse;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::mockwebserver_deprecated::src::main::kotlin::okhttp3::mockwebserver::Dispatcher::*;

/* An HTTP request initiated by the server. */
#[derive(Debug, Clone, PartialEq)]
pub struct PushPromise {
    pub method: String,
    pub path: String,
    pub headers: Headers,
    pub response: MockResponse,
}

impl PushPromise {
    pub fn new(method: String, path: String, headers: Headers, response: MockResponse) -> Self {
        Self {
            method,
            path,
            headers,
            response,
        }
    }
}