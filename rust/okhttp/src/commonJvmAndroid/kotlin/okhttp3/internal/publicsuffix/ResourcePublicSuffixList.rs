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

use okio::{FileSystem, Path, Source};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::BasePublicSuffixList::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

pub struct ResourcePublicSuffixList {
    pub path: Path,
    pub file_system: FileSystem,
}

impl ResourcePublicSuffixList {
    pub fn new() -> Self {
        Self {
            path: Self::PUBLIC_SUFFIX_RESOURCE(),
            file_system: FileSystem::RESOURCES,
        }
    }

    // Companion object equivalent for PUBLIC_SUFFIX_RESOURCE
    pub fn PUBLIC_SUFFIX_RESOURCE() -> Path {
        // In Kotlin: "okhttp3/internal/publicsuffix/${PublicSuffixDatabase::class.java.simpleName}.list".toPath()
        // PublicSuffixDatabase::class.java.simpleName is "PublicSuffixDatabase"
        let path_str = format!("okhttp3/internal/publicsuffix/PublicSuffixDatabase.list");
        Path::from(path_str)
    }
}

impl BasePublicSuffixList for ResourcePublicSuffixList {
    fn list_source(&self) -> Box<dyn Source> {
        Box::new(self.file_system.source(self.path.clone()))
    }

    fn path(&self) -> Box<dyn std::any::Any> {
        Box::new(self.path.clone())
    }

    fn get_state(&self) -> &BasePublicSuffixListState {
        // Note: BasePublicSuffixList in Kotlin is a base class. 
        // In the provided BasePublicSuffixList translation, the state is managed.
        // Since ResourcePublicSuffixList inherits from BasePublicSuffixList, 
        // it must provide access to the state defined in the base.
        // This implementation assumes the state is held within the struct or managed by the trait.
        unimplemented!("State management is handled by the BasePublicSuffixList trait/struct implementation")
    }
}