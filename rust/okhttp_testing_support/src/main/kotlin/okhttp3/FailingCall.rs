/*
 * Copyright (C) 2024 Square, Inc.
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

use std::any::{Any, TypeId};
use std::error::Error;
use std::sync::Arc;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::CallTagsTest::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::ResponseBodyTest::*;

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{
    Call, Callback, EventListener, Request, Response, Timeout,
};

// A Call implementation that panics on every method call.
// Used for testing purposes to ensure that certain Call methods are not invoked.
pub struct FailingCall;

impl FailingCall {
    pub fn new() -> Self {
        Self
    }
}

impl Call for FailingCall {
    fn request(&self) -> Request {
        panic!("unexpected")
    }

    fn execute(&self) -> Result<Response, Box<dyn Error>> {
        panic!("unexpected")
    }

    fn enqueue(&self, _response_callback: Arc<dyn Callback>) {
        panic!("unexpected")
    }

    fn cancel(&self) {
        panic!("unexpected")
    }

    fn is_executed(&self) -> bool {
        panic!("unexpected")
    }

    fn is_canceled(&self) -> bool {
        panic!("unexpected")
    }

    fn timeout(&self) -> Timeout {
        panic!("unexpected")
    }

    fn add_event_listener(&self, _event_listener: EventListener) {
        panic!("unexpected")
    }

    fn tag<T: Any>(&self, _type_id: TypeId) -> Option<Arc<T>> {
        panic!("unexpected")
    }

    fn tag_compute_if_absent<T: Any, F>(&self, _type_id: TypeId, _compute_if_absent: F) -> Arc<T>
    where
        F: Fn() -> T,
    {
        panic!("unexpected")
    }

    fn clone_call(&self) -> Box<dyn Call> {
        panic!("unexpected")
    }
}