/*
 * Copyright (C) 2020 Square, Inc.
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

// ASN.1 object identifiers used internally by this implementation.
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

pub struct ObjectIdentifiers;

impl ObjectIdentifiers {
    pub const EC_PUBLIC_KEY: &'static str = "1.2.840.10045.2.1";
    pub const SHA256_WITH_ECDSA: &'static str = "1.2.840.10045.4.3.2";
    pub const RSA_ENCRYPTION: &'static str = "1.2.840.113549.1.1.1";
    pub const SHA256_WITH_RSA_ENCRYPTION: &'static str = "1.2.840.113549.1.1.11";
    pub const SUBJECT_ALTERNATIVE_NAME: &'static str = "2.5.29.17";
    pub const BASIC_CONSTRAINTS: &'static str = "2.5.29.19";
    pub const COMMON_NAME: &'static str = "2.5.4.3";
    pub const ORGANIZATIONAL_UNIT_NAME: &'static str = "2.5.4.11";
}