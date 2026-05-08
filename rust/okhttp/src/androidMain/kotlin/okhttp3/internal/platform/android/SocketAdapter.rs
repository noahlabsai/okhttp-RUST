/*
 * Copyright (C) 2019 Square, Inc.
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

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

// These types represent the JVM/Android platform types. 
// In a real production environment, these would be FFI bindings to the Android OpenSSL/BoringSSL 
// or Java wrappers. Here they are defined as opaque structs to maintain the API signature.
pub struct X509TrustManager;

pub trait SocketAdapter {
    fn is_supported(&self) -> bool;

    fn trust_manager(&self, _ssl_socket_factory: &SSLSocketFactory) -> Option<&X509TrustManager> {
        None
    }

    fn matches_socket(&self, ssl_socket: &SSLSocket) -> bool;

    fn matches_socket_factory(&self, _ssl_socket_factory: &SSLSocketFactory) -> bool {
        false
    }

    fn configure_tls_extensions(
        &self,
        ssl_socket: &mut SSLSocket,
        hostname: Option<String>,
        protocols: Vec<Protocol>,
    );

    fn get_selected_protocol(&self, ssl_socket: &SSLSocket) -> Option<String>;
}