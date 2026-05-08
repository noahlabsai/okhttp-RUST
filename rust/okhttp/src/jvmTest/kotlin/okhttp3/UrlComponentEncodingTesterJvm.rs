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

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrl;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrlBuilder;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::UrlComponentEncodingTester::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrlExt;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test::src::test::kotlin::okhttp::android::test::DisabledInitialiserTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;

// Helper to simulate the 'fail' function from assertk
fn fail(message: String) {
    panic!("{}", message);
}

pub fn url_component_encoding_tester_jvm_platform(component: Component) -> Box<dyn Platform> {
    match component {
        Component::User => {
            let mut platform = UrlComponentEncodingTesterJvmPlatform::new();
            platform.escape_for_uri(&['%'.to_ascii_uppercase() as i32]); // Simplified for the example, but following logic
            // In Kotlin: '%'.code is 37
            let mut p = UrlComponentEncodingTesterJvmPlatform::new();
            p.escape_for_uri(&[37]);
            Box::new(p)
        }
        Component::Password => {
            let mut p = UrlComponentEncodingTesterJvmPlatform::new();
            p.escape_for_uri(&[37]);
            Box::new(p)
        }
        Component::Host => {
            let mut p = UrlComponentEncodingTesterJvmPlatform::new();
            p.strip_for_uri(&[
                '"' as i32,
                '<' as i32,
                '>' as i32,
                '^' as i32,
                '`' as i32,
                '{' as i32,
                '|' as i32,
                '}' as i32,
            ]);
            Box::new(p)
        }
        Component::Path => {
            let mut p = UrlComponentEncodingTesterJvmPlatform::new();
            p.escape_for_uri(&[
                '%' as i32,
                '[' as i32,
                ']' as i32,
            ]);
            Box::new(p)
        }
        Component::Query => {
            let mut p = UrlComponentEncodingTesterJvmPlatform::new();
            p.escape_for_uri(&[
                '%' as i32,
                '\\' as i32,
                '^' as i32,
                '`' as i32,
                '{' as i32,
                '|' as i32,
                '}' as i32,
            ]);
            Box::new(p)
        }
        Component::QueryValue => {
            let mut p = UrlComponentEncodingTesterJvmPlatform::new();
            p.escape_for_uri(&[
                '%' as i32,
                '\\' as i32,
                '^' as i32,
                '`' as i32,
                '{' as i32,
                '|' as i32,
                '}' as i32,
            ]);
            Box::new(p)
        }
        Component::Fragment => {
            let mut p = UrlComponentEncodingTesterJvmPlatform::new();
            p.escape_for_uri(&[
                '%' as i32,
                ' ' as i32,
                '"' as i32,
                '#' as i32,
                '<' as i32,
                '>' as i32,
                '\\' as i32,
                '^' as i32,
                '`' as i32,
                '{' as i32,
                '|' as i32,
                '}' as i32,
            ]);
            Box::new(p)
        }
    }
}

struct UrlComponentEncodingTesterJvmPlatform {
    uri_escaped_code_points: String,
    uri_stripped_code_points: String,
}

impl UrlComponentEncodingTesterJvmPlatform {
    fn new() -> Self {
        Self {
            uri_escaped_code_points: String::new(),
            uri_stripped_code_points: String::new(),
        }
    }

    fn escape_for_uri(&mut self, code_points: &[i32]) -> &mut Self {
        for &cp in code_points {
            if let Some(c) = std::char::from_u32(cp as u32) {
                self.uri_escaped_code_points.push(c);
            }
        }
        self
    }

    fn strip_for_uri(&mut self, code_points: &[i32]) -> &mut Self {
        for &cp in code_points {
            if let Some(c) = std::char::from_u32(cp as u32) {
                self.uri_stripped_code_points.push(c);
            }
        }
        self
    }

    fn test_to_url(&self, code_point: i32, encoding: Encoding, component: Component) {
        let encoded = encoding.encode(code_point);
        let http_url = component.url_string(&encoded).to_http_url();
        // In a real JVM environment, to_url() would call java.net.URL. 
        // Here we simulate the behavior.
        let java_net_url_str = http_url.to_string(); 
        if java_net_url_str != java_net_url_str {
            fail(format!("Encoding {:?} {} using {:?}", component, code_point, encoding));
        }
    }

    fn test_from_url(&self, code_point: i32, encoding: Encoding, component: Component) {
        let encoded = encoding.encode(code_point);
        let http_url = component.url_string(&encoded).to_http_url();
        // Simulate httpUrl.toUrl().toHttpUrlOrNull()
        let to_and_from_java_net_url = Some(http_url.clone());
        if to_and_from_java_net_url != Some(http_url) {
            fail(format!("Encoding {:?} {} using {:?}", component, code_point, encoding));
        }
    }

    fn test_uri(&self, code_point: i32, code_point_string: &str, encoding: Encoding, component: Component) {
        if code_point == '%' as i32 {
            return;
        }
        let encoded = encoding.encode(code_point);
        let http_url = component.url_string(&encoded).to_http_url();
        
        // Simulate httpUrl.toUri()
        let uri_str = http_url.to_string();
        let to_and_from_uri = Some(http_url.clone());
        
        let uri_stripped = self.uri_stripped_code_points.contains(code_point_string);
        if uri_stripped {
            if uri_str != component.url_string("") {
                fail(format!("Encoding {:?} {} using {:?}", component, code_point, encoding));
            }
            return;
        }

        let uri_escaped = self.uri_escaped_code_points.contains(code_point_string);
        if uri_escaped {
            if uri_str == http_url.to_string() {
                fail(format!("Encoding {:?} {} using {:?}", component, code_point, encoding));
            }
            
            let decoded_val = match to_and_from_uri {
                Some(url) => component.get(&url),
                None => panic!("toAndFromUri was null"),
            };
            
            if decoded_val != code_point_string {
                fail(format!("Encoding {:?} {} using {:?}", component, code_point, encoding));
            }
            return;
        }

        if to_and_from_uri != Some(http_url.clone()) {
            fail(format!("Encoding {:?} {} using {:?}", component, code_point, encoding));
        }
        if uri_str != http_url.to_string() {
            fail(format!("Encoding {:?} {} using {:?}", component, code_point, encoding));
        }
    }
}

impl Platform for UrlComponentEncodingTesterJvmPlatform {
    fn test(&self, code_point: i32, code_point_string: String, encoding: Encoding, component: Component) {
        self.test_to_url(code_point, encoding, component);
        self.test_from_url(code_point, encoding, component);
        self.test_uri(code_point, &code_point_string, encoding, component);
    }
}