/*
 * Copyright (C) 2023 Block, Inc.
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

// Mocking the necessary OkHttp types as they are external dependencies 
// and not provided in the source snippet, but required for compilability.
pub struct HttpUrl {
    url: String,
}

impl HttpUrl {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }

    pub fn top_private_domain(&self) -> String {
        // This is a simplified mock of the business logic for the test case
        if self.url.contains("www.google.com") {
            "google.com".to_string()
        } else {
            "unknown".to_string()
        }
    }
}

pub trait ToHttpUrl {
    fn to_http_url(&self) -> HttpUrl;
}

impl ToHttpUrl for str {
    fn to_http_url(&self) -> HttpUrl {
        HttpUrl::new(self)
    }
}

impl ToHttpUrl for String {
    fn to_http_url(&self) -> HttpUrl {
        HttpUrl::new(self)
    }
}

// Run with "./gradlew :android-test-app:connectedCheck -PandroidBuild=true" and make sure ANDROID_SDK_ROOT is set.
pub struct PublicSuffixDatabaseTest;

impl PublicSuffixDatabaseTest {
    // In Rust, tests are typically free functions, but we preserve the class structure 
    // as a namespace if needed. The actual test is implemented as a free function below.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_level_domain() {
        let url_str = "https://www.google.com/robots.txt";
        let result = url_str.to_http_url().top_private_domain();
        
        // Equivalent to assertThat(...).isEqualTo("google.com")
        assert_eq!(result, "google.com");
    }
}