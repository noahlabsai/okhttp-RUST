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

// A policy for how the pool should treat a specific address.
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

#[derive(Debug, Clone, PartialEq)]
pub struct AddressPolicy {
    // How many concurrent calls should be possible to make at any time.
    // The pool will routinely try to pre-emptively open connections to satisfy this minimum.
    // Connections will still be closed if they idle beyond the keep-alive but will be replaced.
    pub minimum_concurrent_calls: i32,
    // How long to wait to retry pre-emptive connection attempts that fail.
    pub backoff_delay_millis: i64,
    // How much jitter to introduce in connection retry backoff delays
    pub backoff_jitter_millis: i32,
}

impl Default for AddressPolicy {
    fn default() -> Self {
        Self {
            minimum_concurrent_calls: 0,
            backoff_delay_millis: 60 * 1000,
            backoff_jitter_millis: 100,
        }
    }
}

impl AddressPolicy {
    // Constructor to mimic Kotlin's default parameters
    pub fn new(
        minimum_concurrent_calls: i32,
        backoff_delay_millis: i64,
        backoff_jitter_millis: i32,
    ) -> Self {
        Self {
            minimum_concurrent_calls,
            backoff_delay_millis,
            backoff_jitter_millis,
        }
    }
}