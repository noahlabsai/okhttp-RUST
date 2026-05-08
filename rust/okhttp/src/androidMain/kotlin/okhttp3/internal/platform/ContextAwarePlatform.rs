/*
 * Copyright (C) 2024 Block, Inc.
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

// Mock or FFI representation of android.content.Context
// In a real Android Rust environment, this would be a pointer or a wrapper around a JNI object.
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Context;

pub trait ContextAwarePlatform {
    // Gets the application context.
    fn application_context(&self) -> Option<Context>;

    // Sets the application context.
    fn set_application_context(&mut self, context: Option<Context>);
}