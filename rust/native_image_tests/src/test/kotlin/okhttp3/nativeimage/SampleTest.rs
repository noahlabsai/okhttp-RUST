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

use crate::mockwebserver::src::main::kotlin::mockwebserver3::{MockResponse, MockWebServer};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{HttpUrl, OkHttpClient, Request};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::MockResponse::*;
use crate::mockwebserver::src::test::java::mockwebserver3::CustomDispatcherTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::CallTagsTest::*;

pub struct SampleTest {
    server: MockWebServer,
    client: OkHttpClient,
}

impl SampleTest {
    pub fn new() -> Self {
        Self {
            server: MockWebServer::new(),
            client: OkHttpClient::new(),
        }
    }

    pub fn passing_test(&self) {
        assert_eq!("hello", "hello");
    }

    pub fn test_mock_web_server(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.server.enqueue(MockResponse {
            body: Some("abc".to_string()),
            ..Default::default()
        });
        
        self.server.start(0)?;

        let request = Request::new(self.server.url("/"));
        let response = self.client.new_call(request).execute()?;
        
        let body_content = response.body().and_then(|b| b.string());
        assert_eq!(body_content.as_deref(), Some("abc"));

        Ok(())
    }

    pub fn test_external_site(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = HttpUrl::parse("https://google.com/robots.txt")
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        
        let request = Request::new(url);
        let response = self.client.new_call(request).execute()?;
        
        assert_eq!(response.code(), 200);

        Ok(())
    }
}

pub trait HttpUrlExt {
    fn to_http_url(&self) -> Result<HttpUrl, Box<dyn std::error::Error>>;
}

impl HttpUrlExt for String {
    fn to_http_url(&self) -> Result<HttpUrl, Box<dyn std::error::Error>> {
        HttpUrl::parse(self).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

impl HttpUrlExt for &str {
    fn to_http_url(&self) -> Result<HttpUrl, Box<dyn std::error::Error>> {
        HttpUrl::parse(self).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}
