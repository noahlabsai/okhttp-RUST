/*
 * Copyright (C) 2019 Square, Inc.
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

// Annotation marking a test as flaky, and requires extra logging and linking against
// a known github issue. This does not ignore the failure.
//
// In Rust, annotations are typically represented as attributes.
// Since this is a marker annotation used for test metadata, we define it as a
// marker struct.
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flaky;

// Marker trait to identify types or tests that are considered flaky.
pub trait IsFlaky {}

impl IsFlaky for Flaky {}