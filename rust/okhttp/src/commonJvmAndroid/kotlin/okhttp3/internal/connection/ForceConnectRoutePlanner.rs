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

use std::sync::Arc;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

// RoutePlanner trait definition to support the delegation pattern.
pub trait RoutePlanner: Send + Sync {
    type Plan;

    fn plan(&self) -> Self::Plan;
}

// RealRoutePlanner is the concrete implementation that ForceConnectRoutePlanner delegates to.
// This is a stub based on the provided Kotlin source context.
pub trait RealRoutePlanner: RoutePlanner {
    fn plan_connect(&self) -> <Self as RoutePlanner>::Plan;
}

/*
 * A RoutePlanner that will always establish a new connection, ignoring any connection pooling
 */
pub struct ForceConnectRoutePlanner<T: RealRoutePlanner> {
    delegate: Arc<T>,
}

impl<T: RealRoutePlanner> ForceConnectRoutePlanner<T> {
    pub fn new(delegate: Arc<T>) -> Self {
        Self { delegate }
    }
}

impl<T: RealRoutePlanner> RoutePlanner for ForceConnectRoutePlanner<T> {
    type Plan = T::Plan;

    fn plan(&self) -> Self::Plan {
        // In Kotlin: override fun plan(): RoutePlanner.Plan = delegate.planConnect()
        self.delegate.plan_connect()
    }
}