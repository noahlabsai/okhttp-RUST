/*
 * Copyright (C) 2013 Square, Inc.
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

use std::collections::HashSet;
use std::sync::Mutex;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Route;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

/*
 * A denylist of failed routes to avoid when creating a new connection to a target address. This is
 * used so that OkHttp can learn from its mistakes: if there was a failure attempting to connect to
 * a specific IP address or proxy server, that failure is remembered and alternate routes are
 * preferred.
 */
pub struct RouteDatabase {
    // Using Mutex to preserve the @Synchronized behavior from Kotlin
    failed_routes: Mutex<HashSet<Route>>,
}

impl Default for RouteDatabase {
    fn default() -> Self {
        Self {
            failed_routes: Mutex::new(HashSet::new()),
        }
    }
}

impl RouteDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    // Returns a snapshot of the failed routes.
    // Equivalent to the Kotlin getter: val failedRoutes: Set<Route> @Synchronized get() = _failedRoutes.toSet()
    pub fn failed_routes(&self) -> HashSet<Route> {
        let lock = self.failed_routes.lock().expect("Mutex poisoned");
        lock.clone()
    }

    /* Records a failure connecting to [failed_route]. */
    pub fn failed(&self, failed_route: Route) {
        let mut lock = self.failed_routes.lock().expect("Mutex poisoned");
        lock.insert(failed_route);
    }

    /* Records success connecting to [route]. */
    pub fn connected(&self, route: Route) {
        let mut lock = self.failed_routes.lock().expect("Mutex poisoned");
        lock.remove(&route);
    }

    /* Returns true if [route] has failed recently and should be avoided. */
    pub fn should_postpone(&self, route: &Route) -> bool {
        let lock = self.failed_routes.lock().expect("Mutex poisoned");
        lock.contains(route)
    }
}