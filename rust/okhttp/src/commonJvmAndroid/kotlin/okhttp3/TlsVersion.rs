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

// Versions of TLS that can be offered when negotiating a secure socket. See
// [javax.net.ssl.SSLSocket.setEnabledProtocols].
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TlsVersion {
    TLS_1_3, // 2016.
    TLS_1_2, // 2008.
    TLS_1_1, // 2006.
    TLS_1_0, // 1999.
    SSL_3_0, // 1996.
}

pub const TLS_1_3: TlsVersion = TlsVersion::TLS_1_3;
pub const TLS_1_2: TlsVersion = TlsVersion::TLS_1_2;
pub const TLS_1_1: TlsVersion = TlsVersion::TLS_1_1;
pub const TLS_1_0: TlsVersion = TlsVersion::TLS_1_0;
pub const SSL_3_0: TlsVersion = TlsVersion::SSL_3_0;

impl Default for TlsVersion {
    fn default() -> Self {
        TlsVersion::TLS_1_3
    }
}

impl TlsVersion {
    // Returns the Java name of the TLS version.
    pub fn java_name(&self) -> &'static str {
        match self {
            TlsVersion::TLS_1_3 => "TLSv1.3",
            TlsVersion::TLS_1_2 => "TLSv1.2",
            TlsVersion::TLS_1_1 => "TLSv1.1",
            TlsVersion::TLS_1_0 => "TLSv1",
            TlsVersion::SSL_3_0 => "SSLv3",
        }
    }

    // Returns the TlsVersion corresponding to the given Java name.
    //
    // # Panics
    //
    // Panics if the `java_name` is not recognized.
    pub fn for_java_name(java_name: &str) -> Self {
        match java_name {
            "TLSv1.3" => TlsVersion::TLS_1_3,
            "TLSv1.2" => TlsVersion::TLS_1_2,
            "TLSv1.1" => TlsVersion::TLS_1_1,
            "TLSv1" => TlsVersion::TLS_1_0,
            "SSLv3" => TlsVersion::SSL_3_0,
            _ => panic!("Unexpected TLS version: {}", java_name),
        }
    }
}

impl std::fmt::Display for TlsVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.java_name())
    }
}