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

use std::any::Any;
use std::sync::Arc;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

// Mocking Android Context as it is a platform-specific JVM type
#[derive(Debug, Clone, PartialEq)]
pub struct Context;

// Mocking the Platform trait/class
pub trait Platform: Send + Sync + 'static {}

// Mocking PlatformRegistry to handle the static application context
pub struct PlatformRegistry;

impl PlatformRegistry {
    pub fn set_application_context(_context: Context) {
        // Implementation would store the context globally
    }
}

// Mocking the Platform.get() functionality
pub struct PlatformImpl;
impl Platform for PlatformImpl {}

impl PlatformImpl {
    pub fn get() -> Arc<dyn Platform> {
        Arc::new(PlatformImpl)
    }
}

// Mocking the androidx.startup.Initializer interface
pub trait Initializer<T> {
    fn create(&self, context: Context) -> T;
    fn dependencies(&self) -> Vec<Box<dyn Any>>;
}

/*
 * Androidx Startup initializer to ensure that the AndroidPlatform has access to the application context.
 */
pub struct PlatformInitializer;

impl Initializer<Arc<dyn Platform>> for PlatformInitializer {
    fn create(&self, context: Context) -> Arc<dyn Platform> {
        PlatformRegistry::set_application_context(context);

        PlatformImpl::get()
    }

    fn dependencies(&self) -> Vec<Box<dyn Any>> {
        // Returns an empty list as per the Kotlin implementation: listOf()
        Vec::new()
    }
}
