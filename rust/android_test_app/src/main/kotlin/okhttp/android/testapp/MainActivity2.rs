/*
 * Copyright (C) 2025 Block, Inc.
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

use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::MainActivity::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

// MainActivity2 inherits from MainActivity.
// In Rust, inheritance is represented by composition or trait implementation.
// Since MainActivity is a struct, we use composition to preserve the behavior.
#[derive(Debug, Clone, PartialEq)]
pub struct MainActivity2 {
    pub base: MainActivity,
}

impl MainActivity2 {
    pub fn new() -> Self {
        Self {
            base: MainActivity::new(),
        }
    }
}

// To preserve the "inheritance" of MainActivity's methods, 
// we can implement a trait or provide delegation methods.
impl MainActivity2 {
    pub fn on_create(&self, saved_instance_state: Option<Box<dyn std::any::Any>>) {
        self.base.on_create(saved_instance_state);
    }
}