/*
 * Copyright (C) 2016 Square, Inc.
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

use std::sync::OnceLock;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::platform::Platform;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test::src::test::kotlin::okhttp::android::test::DisabledInitialiserTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::android::SocketAdapter::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;

// These types are expected to be defined in the platform or common modules
// In a real translation, these would be imported from the appropriate Rust modules
// representing the Java/Android SDK.
pub struct SSLContext;
pub trait X509TrustManager {}

impl SSLContext {
    pub fn get_instance(algo: &str) -> Self {
        SSLContext
    }
}

impl SSLSocket {
    pub fn ssl_parameters(&self) -> SSLParameters {
        SSLParameters
    }
    pub fn set_ssl_parameters(&mut self, _params: SSLParameters) {}
    pub fn application_protocol(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(None)
    }
}

impl SSLParameters {
    pub fn set_application_protocols(&mut self, _protocols: Vec<String>) {}
}

// OpenJDK 9+ and JDK8 build 252+.
//
// This may also be used for Android tests with Robolectric.
#[derive(Debug, Clone, PartialEq)]
pub struct Jdk9Platform;

impl Jdk9Platform {
    pub static MAJOR_VERSION: OnceLock<Option<i32>> = OnceLock::new();
    pub static IS_AVAILABLE: OnceLock<bool> = OnceLock::new();

    pub fn is_available() -> bool {
        Self::IS_AVAILABLE.get_or_init(|| {
            let version = Self::major_version();
            if let Some(major) = version {
                major >= 9
            } else {
                // In a real JVM environment, this would use reflection to check for getApplicationProtocol.
                // Since we are translating to Rust, we simulate the check.
                false
            }
        })
        .copied()
    }

    pub fn major_version() -> Option<i32> {
        Self::MAJOR_VERSION.get_or_init(|| {
            // Equivalent to System.getProperty("java.specification.version")?.toIntOrNull()
            std::env::var("java.specification.version")
                .ok()
                .and_then(|v| v.parse::<i32>().ok())
        })
        .copied()
    }

    pub fn build_if_supported() -> Option<Self> {
        if Self::is_available() {
            Some(Jdk9Platform)
        } else {
            None
        }
    }

    fn alpn_protocol_names(&self, protocols: Vec<Protocol>) -> Vec<String> {
        // This is a helper to convert Protocol enum/objects to their string representation
        protocols.into_iter().map(|p| format!("{:?}", p)).collect()
    }
}

impl Platform for Jdk9Platform {
    fn configure_tls_extensions(
        &self,
        ssl_socket: &mut SSLSocket,
        _hostname: Option<String>,
        protocols: Vec<Protocol>,
    ) {
        let mut ssl_parameters = ssl_socket.ssl_parameters();

        let names = self.alpn_protocol_names(protocols);

        ssl_parameters.set_application_protocols(names);

        ssl_socket.set_ssl_parameters(ssl_parameters);
    }

    fn get_selected_protocol(&self, ssl_socket: &SSLSocket) -> Option<String> {
        match ssl_socket.application_protocol() {
            Ok(protocol) => {
                match protocol {
                    Some(p) if !p.is_empty() => Some(p),
                    _ => None,
                }
            }
            Err(_) => {
                // Equivalent to catching UnsupportedOperationException
                None
            }
        }
    }

    fn trust_manager(&self, _ssl_socket_factory: &SSLSocketFactory) -> Option<Box<dyn X509TrustManager>> {
        // Not supported due to access checks on JDK 9+
        panic!("clientBuilder.sslSocketFactory(SSLSocketFactory) not supported on JDK 8 (>= 252) or JDK 9+");
    }

    fn new_ssl_context(&self) -> SSLContext {
        let major = Self::major_version();

        if let Some(v) = major {
            if v >= 9 {
                return SSLContext::get_instance("TLS");
            }
        }

        // Try TLSv1.3, fallback to TLS
        // In Rust, we simulate the try-catch for NoSuchAlgorithmException
        if self.try_get_instance("TLSv1.3") {
            SSLContext::get_instance("TLSv1.3")
        } else {
            SSLContext::get_instance("TLS")
        }
    }
}

impl Jdk9Platform {
    fn try_get_instance(&self, _algo: &str) -> bool {
        // Simulation of NoSuchAlgorithmException check
        true
    }
}
