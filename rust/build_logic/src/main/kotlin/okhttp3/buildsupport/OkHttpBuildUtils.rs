/*
 * Copyright (c) 2026 OkHttp Authors
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

// Assuming Project is defined in a shared module as per the provided translation memory
// If this were a standalone crate, Project would be defined here or imported.
use crate::build_logic::src::main::kotlin::JavaModules::Project;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::build_logic::src::main::kotlin::JavaModules::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

pub trait OkHttpBuildUtils {
    fn platform(&self) -> String;
    fn test_java_version(&self) -> i32;
    fn android_build(&self) -> bool;
    fn alpn_boot_version(&self) -> Option<String>;
}

impl OkHttpBuildUtils for Project {
    fn platform(&self) -> String {
        self.find_property("okhttp.platform")
            .map(|p| p.to_string())
            .unwrap_or_else(|| "jdk9".to_string())
    }

    fn test_java_version(&self) -> i32 {
        self.find_property("test.java.version")
            .and_then(|p| p.parse::<i32>().ok())
            .unwrap_or(21)
    }

    fn android_build(&self) -> bool {
        self.find_property("androidBuild")
            .and_then(|p| p.parse::<bool>().ok())
            .unwrap_or(false)
    }

    fn alpn_boot_version(&self) -> Option<String> {
        self.find_property("alpn.boot.version")
            .map(|p| p.to_string())
    }
}

// Note: The Project struct must implement find_property to support the above.
// Based on the Kotlin source, findProperty returns an Any? which is then converted to String.
// In Rust, this is represented as returning an Option<String> or similar.
impl Project {
    pub fn find_property(&self, name: &str) -> Option<String> {
        // This is a mock implementation of the Gradle Project.findProperty method
        // as the actual implementation depends on the Gradle Rust binding/wrapper.
        None 
    }
}