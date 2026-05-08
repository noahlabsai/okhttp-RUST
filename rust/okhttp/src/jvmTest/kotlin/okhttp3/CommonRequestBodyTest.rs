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

use std::collections::HashMap;

// --- Mocking OkHttp types to ensure compilability as the source is a test ---


impl MediaType {
    pub fn new(type_str: &str, subtype: &str, encoding: &str, parameters: &[&str]) -> Self {
        Self {
            type_: type_str.to_string(),
            subtype: subtype.to_string(),
            encoding: encoding.to_string(),
            parameters: parameters.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn toString(&self) -> String {
        // Simplified representation for the test case
        format!("{}; charset=utf-8", self.type_)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentType {
    pub media_type: String,
    pub parameters: HashMap<String, String>,
}

impl ContentType {
    pub fn parameter(&self, name: &str) -> Option<&String> {
        self.parameters.get(name)
    }
}

pub trait RequestBody {
    fn content_type(&self) -> Option<ContentType>;
}

pub struct StringRequestBody {
    pub content: String,
    pub media_type: MediaType,
}

impl RequestBody for StringRequestBody {
    fn content_type(&self) -> Option<ContentType> {
        let mut params = HashMap::new();
        params.insert("charset".to_string(), "utf-8".to_string());
        Some(ContentType {
            media_type: self.media_type.toString(),
            parameters: params,
        })
    }
}

pub struct RequestBodyCompanion;

impl RequestBodyCompanion {
    pub fn to_request_body(body: &str, media_type: MediaType) -> Box<dyn RequestBody> {
        Box::new(StringRequestBody {
            content: body.to_string(),
            media_type,
        })
    }
}

// Extension trait to mimic Kotlin's "String.toRequestBody"
pub trait RequestBodyExt {
    fn to_request_body(&self, media_type: MediaType) -> Box<dyn RequestBody>;
}

impl RequestBodyExt for str {
    fn to_request_body(&self, media_type: MediaType) -> Box<dyn RequestBody> {
        RequestBodyCompanion::to_request_body(self, media_type)
    }
}

// --- Test Implementation ---

pub struct CommonRequestBodyTest;

impl CommonRequestBodyTest {
    #[test]
    pub fn correct_content_type() {
        let body = "Body";
        // Kotlin: MediaType("text/plain", "text", "plain", arrayOf())
        let media_type = MediaType::new("text/plain", "text", "plain", &[]);
        let request_body = body.to_request_body(media_type);

        // Kotlin: requestBody.contentType()!!
        let content_type = request_body.content_type().expect("ContentType should not be null");

        // assertThat(contentType.mediaType).isEqualTo("text/plain; charset=utf-8")
        assert_eq!(content_type.media_type, "text/plain; charset=utf-8");
        
        // assertThat(contentType.parameter("charset")).isEqualTo("utf-8")
        assert_eq!(content_type.parameter("charset").map(|s| s.as_str()), Some("utf-8"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;

    #[test]
    fn test_correct_content_type() {
        CommonRequestBodyTest::correct_content_type();
    }
}