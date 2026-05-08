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

use std::error::Error;

// Assuming Call and Response are defined in the same crate/module
// as per the okhttp3 package structure.
use crate::okhttp3::{Call, Response};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

// Called when the request could not be executed due to cancellation, a connectivity problem or
// timeout. Because networks can fail during an exchange, it is possible that the remote server
// accepted the request before the failure.
pub trait Callback: Send + Sync {
    fn on_failure(&self, call: &Call, e: Box<dyn Error + Send + Sync>);

    // Called when the HTTP response was successfully returned by the remote server. The callback may
    // proceed to read the response body with [Response.body]. The response is still live until its
    // response body is [closed][ResponseBody]. The recipient of the callback may consume the response
    // body on another thread.
    //
    // Note that transport-layer success (receiving a HTTP response code, headers and body) does not
    // necessarily indicate application-layer success: `response` may still indicate an unhappy HTTP
    // response code like 404 or 500.
    fn on_response(&self, call: &Call, response: Response) -> Result<(), Box<dyn Error + Send + Sync>>;
}