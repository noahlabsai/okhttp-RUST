/*
 * Copyright (C) 2020 Square, Inc.
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

use crate::mockwebserver::src::main::kotlin::mockwebserver3::QueueDispatcher as QueueDispatcherDelegate;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::{MockResponse, RecordedRequest};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Dispatcher;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::MockResponse::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::RecordedRequest::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::QueueDispatcher::*;

// Rust translation of okhttp3.mockwebserver.QueueDispatcher.
// This class acts as a wrapper around the mockwebserver3.QueueDispatcher.


impl QueueDispatcher {
    pub fn new() -> Self {
        Self {
            delegate: QueueDispatcherDelegate::new(),
        }
    }

    pub fn enqueue_response(&self, response: MockResponse) {
        // In Kotlin: delegate.enqueue(response.wrap())
        self.delegate.enqueue(response.wrap());
    }

    pub fn set_fail_fast(&self, fail_fast: bool) {
        self.delegate.set_fail_fast(fail_fast);
    }

    pub fn set_fail_fast_response(&self, fail_fast_response: Option<MockResponse>) {
        // In Kotlin: delegate.setFailFast(failFastResponse?.wrap())
        let wrapped = fail_fast_response.map(|r| r.wrap());
        self.delegate.set_fail_fast(wrapped);
    }

    pub fn peek(&self) -> MockResponse {
        // In Kotlin: throw UnsupportedOperationException("unexpected call")
        panic!("unexpected call");
    }
}

impl Dispatcher for QueueDispatcher {
    fn dispatch(&self, _request: RecordedRequest) -> MockResponse {
        // In Kotlin: throw UnsupportedOperationException("unexpected call")
        panic!("unexpected call");
    }

    fn shutdown(&self) {
        self.delegate.close();
    }
}
