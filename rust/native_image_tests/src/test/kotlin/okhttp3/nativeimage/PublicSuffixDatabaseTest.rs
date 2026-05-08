/*
 * Copyright (C) 2022 Square, Inc.
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

// Assuming the existence of the okhttp3 crate and its HttpUrl implementation
// as per the provided Kotlin source and dependency mapping.
use okhttp3::HttpUrl;

pub struct PublicSuffixDatabaseTest;

impl PublicSuffixDatabaseTest {
    #[test]
    pub fn test_resources_loaded() {
        // In Kotlin: "https://api.twitter.com".toHttpUrl()
        // In Rust, we use the associated function from the HttpUrl implementation.
        let url = HttpUrl::parse("https://api.twitter.com")
            .expect("Invalid URL");

        // In Kotlin: assertThat(url.topPrivateDomain()).isEqualTo("twitter.com")
        // We use assert_eq! as the idiomatic Rust equivalent to assertk's isEqualTo.
        assert_eq!(url.top_private_domain(), "twitter.com");
    }
}

// To ensure the test is actually runnable by the Rust test runner, 
// we provide a free function wrapper since #[test] cannot be inside an impl block 
// for a struct in some Rust versions/configurations, or we simply define the test 
// as a free function.

#[cfg(test)]
mod tests {
    use super::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

    #[test]
    fn test_resources_loaded() {
        let url = HttpUrl::parse("https://api.twitter.com")
            .expect("Invalid URL");
        assert_eq!(url.top_private_domain(), "twitter.com");
    }
}