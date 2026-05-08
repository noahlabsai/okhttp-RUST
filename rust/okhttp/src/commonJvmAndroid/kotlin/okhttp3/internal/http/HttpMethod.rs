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

// Translation of okhttp3.internal.http.HttpMethod
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

pub struct HttpMethod;

impl HttpMethod {
    // Despite being 'internal', this method is called by popular 3rd party SDKs.
    pub fn invalidates_cache(method: &str) -> bool {
        method == "POST"
            || method == "PATCH"
            || method == "PUT"
            || method == "DELETE"
            || method == "MOVE"
    }

    // Despite being 'internal', this method is called by popular 3rd party SDKs.
    pub fn requires_request_body(method: &str) -> bool {
        method == "POST"
            || method == "PUT"
            || method == "PATCH"
            || method == "PROPPATCH"
            || method == "QUERY"
            || method == "REPORT" // WebDAV
    }

    // Despite being 'internal', this method is called by popular 3rd party SDKs.
    pub fn permits_request_body(method: &str) -> bool {
        !(method == "GET" || method == "HEAD")
    }

    pub fn redirects_with_body(method: &str) -> bool {
        method == "PROPFIND"
    }

    pub fn redirects_to_get(method: &str) -> bool {
        method != "PROPFIND"
    }

    pub fn is_cacheable(request_method: &str) -> bool {
        request_method == "GET" || request_method == "QUERY"
    }
}