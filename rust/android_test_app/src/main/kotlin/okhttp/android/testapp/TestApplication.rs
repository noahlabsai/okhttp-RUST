/*
 * Copyright (C) 2023 Block, Inc.
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

// Mocking Android framework types as they are platform-specific JVM types.
// In a real Rust Android environment (like using ndk-glue or similar), 
// these would be provided by the platform bindings.
pub struct Application {
    pub package_name: String,
    pub application_context: String, // Simplified representation of Context
}

impl Application {
    pub fn new(package_name: &str, context: &str) -> Self {
        Self {
            package_name: package_name.to_string(),
            application_context: context.to_string(),
        }
    }

    pub fn on_create(&mut self) {
        // Base Application.onCreate() logic would go here
    }

    pub fn get_process_name(&self) -> Option<String> {
        // This would call the actual Android JNI getProcessName()
        Some("com.example.process".to_string())
    }
}

// Mocking OkHttp initialization logic
pub struct OkHttp;
impl OkHttp {
    pub fn initialize(context: &str) {
        println!("OkHttp initialized with context: {}", context);
    }
}

// Mocking Build.VERSION
pub struct BuildVersion;
impl BuildVersion {
    pub const SDK_INT: i32 = 28; // Example: Android P
    pub const VERSION_CODES_P: i32 = 28;
}

pub struct TestApplication {
    pub base: Application,
}

impl TestApplication {
    pub fn new(package_name: &str, context: &str) -> Self {
        Self {
            base: Application::new(package_name, context),
        }
    }

    pub fn on_create(&mut self) {
        // super.onCreate()
        self.base.on_create();

        if self.is_secondary_process() {
            OkHttp::initialize(&self.base.application_context);
        }
    }

    fn is_secondary_process(&self) -> bool {
        self.get_process() != Some(self.base.package_name.clone())
    }

    fn get_process(&self) -> Option<String> {
        if BuildVersion::SDK_INT >= BuildVersion::VERSION_CODES_P {
            self.base.get_process_name()
        } else {
            // In Kotlin, this uses reflection to call ActivityThread.currentProcessName().
            // In Rust, this would require JNI calls to the JVM.
            self.invoke_current_process_name()
        }
    }

    fn invoke_current_process_name(&self) -> Option<String> {
        // This simulates the reflection:
        // Class.forName("android.app.ActivityThread").getDeclaredMethod("currentProcessName").invoke(null)
        // Since we cannot execute JVM reflection in pure Rust without JNI, 
        // we represent the behavior of returning the process name.
        Some("com.example.process".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secondary_process_initialization() {
        let mut app = TestApplication::new("com.example.app", "context_handle");
        // If get_process returns something different from package_name, it's a secondary process
        app.on_create();
    }
}
