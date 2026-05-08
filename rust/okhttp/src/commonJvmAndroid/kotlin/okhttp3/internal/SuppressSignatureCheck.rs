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

// In Kotlin, `SuppressSignatureCheck` is an internal annotation used to 
// signal to the OkHttp signature checker that a specific class, constructor, 
// or function should be ignored.
//
// Rust does not have a direct equivalent to Kotlin's runtime/compile-time 
// annotations that can be targeted at constructors or functions for 
// external tool processing in the same way. 
//
// To preserve the business behavior (which is providing a marker for a 
// static analysis tool), we represent this as a marker trait. 
// Any item that was annotated with `@SuppressSignatureCheck` in Kotlin 
// can be marked by implementing this trait.

use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

pub trait SuppressSignatureCheck {}