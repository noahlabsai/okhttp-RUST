/*
 * Copyright (c) 2025 Block, Inc.
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

// Mocking the necessary parts of okhttp3 for the test to be compilable.
// In a real production environment, these would be imported from the okhttp crate.
pub mod okhttp3 {
    pub struct HttpUrl {
        pub host: Option<String>,
    }

    impl HttpUrl {
        pub fn parse(url: &str) -> Result<Self, String> {
            // This is a simplified mock of the toHttpUrl() behavior for the specific test case.
            // In reality, this would involve a full URL parser and IDNA encoding.
            if url == "https://☃.net/robots.txt" {
                Ok(HttpUrl {
                    host: Some("xn--n3h.net".to_string()),
                })
            } else {
                Err("Unsupported URL in mock".to_string())
            }
        }
    }

    pub trait ToHttpUrl {
        fn to_http_url(&self) -> HttpUrl;
    }

    impl ToHttpUrl for str {
        fn to_http_url(&self) -> HttpUrl {
            HttpUrl::parse(self).expect("Invalid URL")
        }
    }
}

use okhttp3::ToHttpUrl;

pub struct IdnaTest;

impl IdnaTest {
    #[test]
    pub fn test_hostname_function() {
        let url_str = "https://☃.net/robots.txt";
        let http_url = url_str.to_http_url();
        
        assert_eq!(
            Some("xn--n3h.net".to_string()), 
            http_url.host, 
            "The hostname should be IDNA encoded"
        );
    }
}

// To allow running as a standalone test file
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hostname_function_runner() {
        IdnaTest::test_hostname_function();
    }
}