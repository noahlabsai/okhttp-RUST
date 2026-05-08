/*
 * Copyright (C) 2023 Square, Inc.
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

// Recipe to build an `IdnaMappingTable`.
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::idn::IdnaMappingTable::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::internal::idn::IdnaMappingTableTest::*;


impl IdnaMappingTableData {
    pub fn new(sections: String, ranges: String, mappings: String) -> Self {
        Self {
            sections,
            ranges,
            mappings,
        }
    }
}