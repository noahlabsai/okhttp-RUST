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

use std::any::Any;
use std::sync::{Arc, Mutex};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::platform::Platform;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test::src::test::kotlin::okhttp::android::test::DisabledInitialiserTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;

// Mocking the JVM-specific types as they are required for the logic but not provided in the context.
// In a real production environment, these would be FFI bindings or provided by a JVM-Rust bridge.
#[derive(Debug, Clone, PartialEq)]
pub struct Method;
#[derive(Debug, Clone, PartialEq)]
pub struct Class;

#[derive(Debug, Clone, PartialEq)]
pub struct Jdk8WithJettyBootPlatform {
    put_method: Method,
    get_method: Method,
    remove_method: Method,
    client_provider_class: Class,
    server_provider_class: Class,
}

impl Jdk8WithJettyBootPlatform {
    fn alpn_protocol_names(protocols: &[Protocol]) -> Vec<String> {
        // This mimics the alpnProtocolNames helper usually found in the Platform base class
        protocols.iter().map(|p| format!("{:?}", p)).collect()
    }

    pub fn build_if_supported() -> Option<Self> {
        let jvm_version = std::env::var("java.specification.version").unwrap_or_else(|_| "unknown".to_string());
        
        if let Ok(version) = jvm_version.parse::<i32>() {
            if version >= 9 {
                return None;
            }
        }

        // The following block represents the reflection-based lookup in Kotlin.
        // Since Rust cannot perform JVM reflection, this is a structural translation.
        // In a real scenario, this would use JNI.
        
        // Mocking the successful reflection lookup for structural completeness
        // In actual implementation, these would be results of JNI calls.
        Some(Self {
            put_method: Method,
            get_method: Method,
            remove_method: Method,
            client_provider_class: Class,
            server_provider_class: Class,
        })
    }
}

impl Platform for Jdk8WithJettyBootPlatform {
    fn configure_tls_extensions(
        &self,
        ssl_socket: &SSLSocket,
        _hostname: Option<String>,
        protocols: Vec<Protocol>,
    ) {
        let names = Self::alpn_protocol_names(&protocols);

        // The Kotlin code uses Proxy.newProxyInstance and Method.invoke.
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            // Logic: 
            // 1. Create AlpnProvider proxy with names
            // 2. putMethod.invoke(null, sslSocket, alpnProvider)
            Ok(())
        })();

        if let Err(e) = result {
            panic!("failed to set ALPN: {}", e);
        }
    }

    fn after_handshake(&self, ssl_socket: &SSLSocket) {
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            // Logic: removeMethod.invoke(null, sslSocket)
            Ok(())
        })();

        if let Err(e) = result {
            panic!("failed to remove ALPN: {}", e);
        }
    }

    fn get_selected_protocol(&self, ssl_socket: &SSLSocket) -> Option<String> {
        let result = (|| -> Result<Option<String>, Box<dyn std::error::Error>> {
            // Logic: 
            // 1. provider = Proxy.getInvocationHandler(getMethod.invoke(null, sslSocket)) as AlpnProvider
            // 2. check provider.unsupported and provider.selected
            
            // Structural representation of the logic
            let provider_unsupported = false;
            let provider_selected: Option<String> = None;

            if !provider_unsupported && provider_selected.is_none() {
                println!("ALPN callback dropped: HTTP/2 is disabled. Is alpn-boot on the boot class path?");
                return Ok(None);
            }
            
            if provider_unsupported {
                Ok(None)
            } else {
                Ok(provider_selected)
            }
        })();

        match result {
            Ok(val) => val,
            Err(e) => panic!("failed to get ALPN selected protocol: {}", e),
        }
    }
}

// Internal helper to mimic the InvocationHandler behavior
struct AlpnProvider {
    protocols: Vec<String>,
    unsupported: Mutex<bool>,
    selected: Mutex<Option<String>>,
}

impl AlpnProvider {
    fn new(protocols: Vec<String>) -> Self {
        Self {
            protocols,
            unsupported: Mutex::new(false),
            selected: Mutex::new(None),
        }
    }

    // Mimics the `invoke` method of java.lang.reflect.InvocationHandler
    fn invoke(&self, method_name: &str, return_type: &str, args: Vec<Box<dyn Any>>) -> Option<Box<dyn Any>> {
        if method_name == "supports" && return_type == "boolean" {
            return Some(Box::new(true));
        } else if method_name == "unsupported" && return_type == "void" {
            let mut unsupported = self.unsupported.lock().unwrap();
            *unsupported = true;
            return None;
        } else if method_name == "protocols" && args.is_empty() {
            return Some(Box::new(self.protocols.clone()));
        } else if (method_name == "selectProtocol" || method_name == "select") 
            && return_type == "java.lang.String" 
            && args.len() == 1 
        {
            if let Some(peer_protocols) = args[0].downcast_ref::<Vec<String>>() {
                for protocol in peer_protocols {
                    if self.protocols.contains(protocol) {
                        let mut selected = self.selected.lock().unwrap();
                        *selected = Some(protocol.clone());
                        return Some(Box::new(protocol.clone()));
                    }
                }
                let mut selected = self.selected.lock().unwrap();
                if !self.protocols.is_empty() {
                    let fallback = self.protocols[0].clone();
                    *selected = Some(fallback.clone());
                    return Some(Box::new(fallback));
                }
            }
        } else if (method_name == "protocolSelected" || method_name == "selected") && args.len() == 1 {
            if let Some(protocol) = args[0].downcast_ref::<String>() {
                let mut selected = self.selected.lock().unwrap();
                *selected = Some(protocol.clone());
            }
            return None;
        }
        None
    }
}
