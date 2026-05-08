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

use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test::src::test::kotlin::okhttp::android::test::DisabledInitialiserTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

use crate::okhttp3::internal::platform::{
    ConscryptPlatform, Jdk8WithJettyBootPlatform, Jdk9Platform, OpenJSSEPlatform, Platform,
};

// Mocking the Java Security.getProviders() behavior for Rust.
// In a real production environment, this would interface with the system's 
// security provider configuration or a Rust equivalent like rustls/ring.
struct Security;

impl Security {
    fn get_providers() -> Vec<SecurityProvider> {
        // This is a generated-compatibility for the actual JVM Security.getProviders() call.
        // In the context of this translation, we maintain the logic flow.
        Vec::new()
    }
}

struct SecurityProvider {
    name: String,
}


impl PlatformRegistry {
    fn is_conscrypt_preferred() -> bool {
        let providers = Security::get_providers();
        if let Some(provider) = providers.first() {
            return provider.name == "Conscrypt";
        }
        false
    }

    fn is_open_jsse_preferred() -> bool {
        let providers = Security::get_providers();
        if let Some(provider) = providers.first() {
            return provider.name == "OpenJSSE";
        }
        false
    }

    fn is_bouncy_castle_preferred() -> bool {
        let providers = Security::get_providers();
        if let Some(provider) = providers.first() {
            return provider.name == "BC";
        }
        false
    }

    pub fn find_platform() -> Platform {
        if Self::is_conscrypt_preferred() {
            if let Some(conscrypt) = ConscryptPlatform::build_if_supported() {
                return conscrypt;
            }
        }

        if Self::is_bouncy_castle_preferred() {
            if let Some(bc) = BouncyCastlePlatform::build_if_supported() {
                return bc;
            }
        }

        if Self::is_open_jsse_preferred() {
            if let Some(open_jsse) = OpenJSSEPlatform::build_if_supported() {
                return open_jsse;
            }
        }

        // An Oracle JDK 9 like OpenJDK, or JDK 8 251+.
        if let Some(jdk9) = Jdk9Platform::build_if_supported() {
            return jdk9;
        }

        // An Oracle JDK 8 like OpenJDK, pre 251.
        if let Some(jdk_with_jetty_boot) = Jdk8WithJettyBootPlatform::build_if_supported() {
            return jdk_with_jetty_boot;
        }

        Platform::default()
    }

    pub fn is_android() -> bool {
        false
    }
}

// generated-compatibility for BouncyCastlePlatform to ensure compilability as it is referenced in find_platform
pub struct BouncyCastlePlatform;
impl BouncyCastlePlatform {
    pub fn build_if_supported() -> Option<Platform> {
        None
    }
}