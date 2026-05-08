/*
 * Copyright (C) 2014 Square, Inc.
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

use std::sync::LazyLock;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

// This is a string like "5.0.0", "5.0.0-alpha.762", or "5.3.0-SNAPSHOT" indicating the version of
// OkHttp in the current runtime. Use this to include the OkHttp version in custom `User-Agent`
// headers.
//
// Official OkHttp releases follow [semantic versioning][semver]. Versions with the `-SNAPSHOT`
// qualifier are not unique and should only be used in development environments. If you create
// custom builds of OkHttp please include a qualifier your version name, like "4.7.0-mycompany.3".
// The version string is configured in the root project's `build.gradle`.
//
// Note that OkHttp's runtime version may be different from the version specified in your
// project's build file due to the dependency resolution features of your build tool.
//
// [semver]: https://semver.org
pub struct OkHttp;

impl OkHttp {
    pub fn version() -> &'static str {
        Self::VERSION.as_ref()
    }

    // In a real production environment, this would be populated by the build system
    // (e.g., via an environment variable or a generated file).
    static VERSION: LazyLock<String> = LazyLock::new(|| {
        std::env::var("OKHTTP_VERSION").unwrap_or_else(|_| "5.0.0".to_string())
    });
}
}
