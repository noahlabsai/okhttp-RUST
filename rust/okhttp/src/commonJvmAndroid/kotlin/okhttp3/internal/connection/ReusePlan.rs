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

use std::sync::Arc;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

// Assuming these types are defined in the corresponding modules as per the provided context
// RealConnection is likely wrapped in Arc for shared ownership in a connection pool
pub type RealConnection = Arc<crate::okhttp3::internal::connection::RealConnection>;

pub trait RoutePlannerPlan {
    fn is_ready(&self) -> bool;
    fn connect_tcp(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn connect_tls_etc(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn handle_success(&self) -> RealConnection;
    fn cancel(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn retry(&self) -> Result<(), Box<dyn std::error::Error>>;
}

// Reuse a connection from the pool.
pub struct ReusePlan {
    pub connection: RealConnection,
}

impl ReusePlan {
    pub fn new(connection: RealConnection) -> Self {
        Self { connection }
    }
}

impl RoutePlannerPlan for ReusePlan {
    fn is_ready(&self) -> bool {
        true
    }

    fn connect_tcp(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Kotlin's error() throws an IllegalStateException
        Err("already connected".into())
    }

    fn connect_tls_etc(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Kotlin's error() throws an IllegalStateException
        Err("already connected".into())
    }

    fn handle_success(&self) -> RealConnection {
        Arc::clone(&self.connection)
    }

    fn cancel(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Kotlin's error() throws an IllegalStateException
        Err("unexpected cancel".into())
    }

    fn retry(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Kotlin's error() throws an IllegalStateException
        Err("unexpected retry".into())
    }
}