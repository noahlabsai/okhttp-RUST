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

use std::error::Error;
use std::sync::Arc;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::MockWebServer;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{
    Call, Callback, OkHttpClient, Request, Response,
};

// Translation of DispatcherCleanupTest.
// Note: @StartStop is a JUnit 5 extension for lifecycle management.
pub struct DispatcherCleanupTest {
    server: MockWebServer,
}

impl DispatcherCleanupTest {
    pub fn new() -> Self {
        Self {
            server: MockWebServer::new(),
        }
    }

    pub async fn test_finish(&self) {
        let okhttp = OkHttpClient::new();

        // Define the callback implementation
        struct TestCallback;
        impl Callback for TestCallback {

            fn on_response(&self, _call: &Call, response: Response) {
                // response.close() is handled by Drop in Rust
                drop(response);
            }
        }

        let callback = Arc::new(TestCallback);

        // repeat(10_000)
        for _ in 0..10_000 {
            let request = Request::builder()
                .url(self.server.url("/"))
                .build();

            let call = okhttp.new_call(request);
            call.enqueue(callback.clone());
        }

        // okhttp.dispatcher.executorService.shutdown()
        okhttp.dispatcher().executor_service().shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::mockwebserver::src::test::java::mockwebserver3::CustomDispatcherTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::CallTagsTest::*;

    #[tokio::test]
    async fn test_dispatcher_cleanup_finish() {
        let test_instance = DispatcherCleanupTest::new();
        test_instance.test_finish().r#await;
    }
}
