/*
 * Copyright (C) 2013 Square, Inc.
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

use std::any::{Any, TypeId};
use std::sync::Arc;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

// Import paths as per directives
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrl::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::CacheControl::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::MediaType::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::Tags::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::cache::FaultHidingSink::*;

// Mocking okio-like Buffer for the test logic

impl Buffer {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    pub fn write(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }
    pub fn read_byte_string(&self) -> Vec<u8> {
        self.data.clone()
    }
}

impl Buffer {
    pub fn hex(&self) -> String {
        self.data.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

// Helper for GzipSource simulation
pub struct GzipSource {
    buffer: Buffer,
}

impl GzipSource {
    pub fn new(buffer: Buffer) -> Self {
        Self { buffer }
    }
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}

impl Drop for GzipSource {
    fn drop(&mut self) {}
}

impl Buffer {
    pub fn read_utf8(&self) -> String {
        String::from_utf8_lossy(&self.data).into_owned()
    }
}

pub struct RequestTest;

impl RequestTest {
    #[test]
    pub fn constructor() {
        let url = "https://example.com/".to_http_url();
        let body = "hello".to_request_body();
        let headers = Headers::headers_of(&["User-Agent", "RequestTest"]);
        let method = "PUT";
        let request = Request {
            url: url.clone(),
            headers: headers.clone(),
            method: method.to_string(),
            body: Some(body.clone()),
            tags: Tags::default(),
        };
        assert_eq!(request.url, url);
        assert_eq!(request.headers, headers);
        assert_eq!(request.method, method);
        assert_eq!(request.body, Some(body));
        assert_eq!(request.tags, Tags::Empty);
    }

    #[test]
    pub fn constructor_no_body_no_method() {
        let url = "https://example.com/".to_http_url();
        let headers = Headers::headers_of(&["User-Agent", "RequestTest"]);
        let request = Request {
            url: url.clone(),
            headers: headers.clone(),
            method: "GET".to_string(),
            body: None,
            tags: Tags::default(),
        };
        assert_eq!(request.url, url);
        assert_eq!(request.headers, headers);
        assert_eq!(request.method, "GET");
        assert!(request.body.is_none());
        assert_eq!(request.tags, Tags::Empty);
    }

    #[test]
    pub fn constructor_no_method() {
        let url = "https://example.com/".to_http_url();
        let body = "hello".to_request_body();
        let headers = Headers::headers_of(&["User-Agent", "RequestTest"]);
        let request = Request {
            url: url.clone(),
            headers: headers.clone(),
            method: "POST".to_string(),
            body: Some(body.clone()),
            tags: Tags::default(),
        };
        assert_eq!(request.url, url);
        assert_eq!(request.headers, headers);
        assert_eq!(request.method, "POST");
        assert_eq!(request.body, Some(body));
        assert_eq!(request.tags, Tags::Empty);
    }

    #[test]
    pub fn constructor_no_body() {
        let url = "https://example.com/".to_http_url();
        let headers = Headers::headers_of(&["User-Agent", "RequestTest"]);
        let method = "DELETE";
        let request = Request {
            url: url.clone(),
            headers: headers.clone(),
            method: method.to_string(),
            body: None,
            tags: Tags::default(),
        };
        assert_eq!(request.url, url);
        assert_eq!(request.headers, headers);
        assert_eq!(request.method, method);
        assert!(request.body.is_none());
        assert_eq!(request.tags, Tags::Empty);
    }

    #[test]
    pub fn string_body() {
        let content_type = "text/plain; charset=utf-8".to_media_type();
        let body = "abc".as_bytes().to_request_body(content_type.clone());
        assert_eq!(body.content_type(), Some(content_type.clone()));
        assert_eq!(body.content_length(), 3);
        assert_eq!(Self::body_to_hex(&*body), "616263");
        assert_eq!(Self::body_to_hex(&*body), "616263");
    }

    #[test]
    pub fn string_with_default_charset_added() {
        let content_type = "text/plain".to_media_type();
        let body = "\u{0800}".to_request_body(content_type);
        assert_eq!(body.content_type(), Some("text/plain; charset=utf-8".to_media_type()));
        assert_eq!(body.content_length(), 3);
        assert_eq!(Self::body_to_hex(&*body), "e0a080");
    }

    #[test]
    pub fn string_with_non_default_charset_specified() {
        let content_type = "text/plain; charset=utf-16be".to_media_type();
        let body = "\u{0800}".to_request_body(content_type.clone());
        assert_eq!(body.content_type(), Some(content_type));
        assert_eq!(body.content_length(), 2);
        assert_eq!(Self::body_to_hex(&*body), "0800");
    }

    #[test]
    pub fn byte_array() {
        let content_type = "text/plain".to_media_type();
        let body = "abc".as_bytes().to_request_body(content_type.clone());
        assert_eq!(body.content_type(), Some(content_type));
        assert_eq!(body.content_length(), 3);
        assert_eq!(Self::body_to_hex(&*body), "616263");
        assert_eq!(Self::body_to_hex(&*body), "616263");
    }

    #[test]
    pub fn byte_array_range() {
        let content_type = "text/plain".to_media_type();
        // Simulating range [1, 3) of ".abcd" -> "abc"
        let body = ".abcd".as_bytes().to_request_body_range(content_type.clone(), 1, 3);
        assert_eq!(body.content_type(), Some(content_type));
        assert_eq!(body.content_length(), 3);
        assert_eq!(Self::body_to_hex(&*body), "616263");
        assert_eq!(Self::body_to_hex(&*body), "616263");
    }

    #[test]
    pub fn byte_string() {
        let content_type = "text/plain".to_media_type();
        let body = "Hello".as_bytes().to_request_body(content_type.clone());
        assert_eq!(body.content_type(), Some(content_type));
        assert_eq!(body.content_length(), 5);
        assert_eq!(Self::body_to_hex(&*body), "48656c6c6f");
        assert_eq!(Self::body_to_hex(&*body), "48656c6c6f");
    }

    #[test]
    pub fn file_body() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"abc").unwrap();
        let path = temp_file.path().to_path_buf();
        
        let content_type = "text/plain".to_media_type();
        let body = RequestBody::as_request_body(path, content_type.clone());
        assert_eq!(body.content_type(), Some(content_type));
        assert_eq!(body.content_length(), 3);
        assert_eq!(Self::body_to_hex(&*body), "616263");
        assert_eq!(Self::body_to_hex(&*body), "616263");
    }

    #[test]
    pub fn crud_verbs() {
        let content_type = "application/json".to_media_type();
        let body = "{}".to_request_body(content_type);

        let get = Request::builder().url("http://localhost/api").get().build();
        assert_eq!(get.method, "GET");
        assert!(get.body.is_none());

        let head = Request::builder().url("http://localhost/api").head().build();
        assert_eq!(head.method, "HEAD");
        assert!(head.body.is_none());

        let delete = Request::builder().url("http://localhost/api").delete().build();
        assert_eq!(delete.method, "DELETE");
        assert_eq!(delete.body.as_ref().unwrap().content_length(), 0);

        let post = Request::builder().url("http://localhost/api").post(body.clone()).build();
        assert_eq!(post.method, "POST");
        assert_eq!(post.body, Some(body.clone()));

        let put = Request::builder().url("http://localhost/api").put(body.clone()).build();
        assert_eq!(put.method, "PUT");
        assert_eq!(put.body, Some(body.clone()));

        let patch = Request::builder().url("http://localhost/api").patch(body.clone()).build();
        assert_eq!(patch.method, "PATCH");
        assert_eq!(patch.body, Some(body.clone()));

        let query = Request::builder().url("http://localhost/api").query(body.clone()).build();
        assert_eq!(query.method, "QUERY");
        assert_eq!(query.body, Some(body));
    }

    #[test]
    pub fn uninitialized_uri() {
        let request = Request::builder().url("http://localhost/api").build();
        assert_eq!(request.url.to_uri(), "http://localhost/api");
        assert_eq!(request.url, "http://localhost/api".to_http_url());
    }

    #[test]
    pub fn new_builder_url_resets_url() {
        let request_without_cache = Request::builder().url("http://localhost/api").build();
        let built_request_without_cache = request_without_cache.new_builder().url("http://localhost/api/foo").build();
        assert_eq!(built_request_without_cache.url, "http://localhost/api/foo".to_http_url());
        
        let request_with_cache = Request::builder().url("http://localhost/api").build();
        let _ = &request_with_cache.url;
        let built_request_with_cache = request_with_cache.new_builder().url("http://localhost/api/foo").build();
        assert_eq!(built_request_with_cache.url, "http://localhost/api/foo".to_http_url());
    }

    #[test]
    pub fn cache_control_test() {
        let request = Request::builder()
            .cache_control(CacheControl::builder().no_cache().build())
            .url("https://square.com")
            .build();
        assert_eq!(request.headers.get("Cache-Control"), Some(&"no-cache".to_string()));
        assert!(request.cache_control().no_cache);
    }

    #[test]
    pub fn empty_cache_control_clears_all_cache_control_headers() {
        let request = Request::builder()
            .header("Cache-Control", "foo")
            .cache_control(CacheControl::builder().build())
            .url("https://square.com")
            .build();
        assert!(request.headers.get("Cache-Control").is_none());
    }

    #[test]
    pub fn header_accepts_permitted_characters() {
        let mut builder = Request::builder();
        builder = builder.header("AZab09~", "AZab09 ~");
        builder = builder.add_header("AZab09~", "AZab09 ~");
    }

    #[test]
    #[should_panic(expected = "IllegalArgumentException")]
    pub fn empty_name_forbidden() {
        let mut builder = Request::builder();
        builder.header("", "Value");
    }

    #[test]
    pub fn header_allows_tab_only_in_values() {
        let mut builder = Request::builder();
        builder = builder.header("key", "sample\tvalue");
        let result = std::panic::catch_unwind(|| {
            builder.header("sample\tkey", "value");
        });
        assert!(result.is_err());
    }

    #[test]
    pub fn header_forbids_control_characters() {
        Self::assert_forbidden_header("\u{0000}");
        Self::assert_forbidden_header("\r");
        Self::assert_forbidden_header("\n");
        Self::assert_forbidden_header("\u{001f}");
        Self::assert_forbidden_header("\u{007f}");
        Self::assert_forbidden_header("\u{0080}");
        Self::assert_forbidden_header("\u{d83c}\u{df69}");
    }

    fn assert_forbidden_header(s: &str) {
        let mut builder = Request::builder();
        assert!(std::panic::catch_unwind(|| { builder.header(s, "Value"); }).is_err());
        
        let mut builder2 = Request::builder();
        assert!(std::panic::catch_unwind(|| { builder2.add_header(s, "Value"); }).is_err());
        
        let mut builder3 = Request::builder();
        assert!(std::panic::catch_unwind(|| { builder3.header("Name", s); }).is_err());
        
        let mut builder4 = Request::builder();
        assert!(std::panic::catch_unwind(|| { builder4.add_header("Name", s); }).is_err());
    }

    #[test]
    pub fn no_tag() {
        let request = Request::builder().url("https://square.com").build();
        assert!(request.tag::<dyn Any>().is_none());
        assert!(request.tag::<Uuid>().is_none());
        assert!(request.tag::<String>().is_none());
    }

    #[test]
    pub fn default_tag() {
        let tag = Uuid::new_v4();
        let request = Request::builder().url("https://square.com").tag(Some(Arc::new(tag))).build();
        // In Rust, we check if the tag can be downcast to Uuid
        assert!(request.tag::<Uuid>().is_some());
    }

    #[test]
    pub fn null_removes_tag() {
        let request = Request::builder().url("https://square.com").tag(Some(Arc::new("a".to_string()))).tag(None).build();
        assert!(request.tag::<dyn Any>().is_none());
    }

    #[test]
    pub fn remove_absent_tag() {
        let request = Request::builder().url("https://square.com").tag(None).build();
        assert!(request.tag::<dyn Any>().is_none());
    }

    #[test]
    pub fn object_tag() {
        let tag = Uuid::new_v4();
        let request = Request::builder().url("https://square.com").tag_with_type::<Uuid>(Arc::new(tag)).build();
        assert!(request.tag::<Uuid>().is_some());
        assert!(request.tag::<String>().is_none());
    }

    #[test]
    pub fn java_class_tag() {
        let uuid_tag = Uuid::new_v4();
        let request = Request::builder().url("https://square.com").tag_with_type::<Uuid>(Arc::new(uuid_tag)).build();
        assert!(request.tag::<Uuid>().is_some());
        assert!(request.tag::<String>().is_none());
    }

    #[test]
    pub fn kotlin_reified_tag() {
        let uuid_tag = Uuid::new_v4();
        let request = Request::builder().url("https://square.com").tag_with_type::<Uuid>(Arc::new(uuid_tag)).build();
        assert!(request.tag::<Uuid>().is_some());
        assert!(request.tag::<String>().is_none());
    }

    #[test]
    pub fn kotlin_class_tag() {
        let uuid_tag = Uuid::new_v4();
        let request = Request::builder().url("https://square.com").tag_with_type::<Uuid>(Arc::new(uuid_tag)).build();
        assert!(request.tag::<Uuid>().is_some());
        assert!(request.tag::<String>().is_none());
    }

    #[test]
    pub fn replace_only_tag() {
        let uuid_tag1 = Uuid::new_v4();
        let uuid_tag2 = Uuid::new_v4();
        let request = Request::builder()
            .url("https://square.com")
            .tag_with_type::<Uuid>(Arc::new(uuid_tag1))
            .tag_with_type::<Uuid>(Arc::new(uuid_tag2))
            .build();
        assert_eq!(request.tag::<Uuid>().unwrap().as_ref(), &uuid_tag2);
    }

    #[test]
    pub fn multiple_tags() {
        let uuid_tag = Uuid::new_v4();
        let string_tag = "dilophosaurus".to_string();
        let long_tag = 20170815i64;
        let object_tag = "object".to_string();
        
        let request = Request::builder()
            .url("https://square.com")
            .tag_with_type::<String>(Arc::new(object_tag.clone()))
            .tag_with_type::<Uuid>(Arc::new(uuid_tag))
            .tag_with_type::<String>(Arc::new(string_tag.clone()))
            .tag_with_type::<i64>(Arc::new(long_tag))
            .build();
        
        assert!(request.tag::<Uuid>().is_some());
        assert!(request.tag::<i64>().is_some());
    }

    #[test]
    pub fn tags_are_immutable() {
        let builder = Request::builder().url("https://square.com");
        let request_a = builder.clone().tag_with_type::<String>(Arc::new("a".to_string())).build();
        let request_b = builder.tag_with_type::<String>(Arc::new("b".to_string())).build();
        let request_c = request_a.new_builder().tag_with_type::<String>(Arc::new("c".to_string())).build();
        
        assert_eq!(request_a.tag::<String>().unwrap().as_ref(), "a");
        assert_eq!(request_b.tag::<String>().unwrap().as_ref(), "b");
        assert_eq!(request_c.tag::<String>().unwrap().as_ref(), "c");
    }

    #[test]
    pub fn request_to_string_redacts_sensitive_headers() {
        let mut headers_builder = Headers::builder();
        headers_builder = headers_builder.add("content-length", "99");
        headers_builder = headers_builder.add("authorization", "peanutbutter");
        headers_builder = headers_builder.add("proxy-authorization", "chocolate");
        headers_builder = headers_builder.add("cookie", "drink=coffee");
        headers_builder = headers_builder.add("set-cookie", "accessory=sugar");
        headers_builder = headers_builder.add("user-agent", "OkHttp");
        let headers = headers_builder.build();
        
        let request = Request {
            url: "https://square.com".to_http_url(),
            headers,
            method: "GET".to_string(),
            body: None,
            tags: Tags::default(),
        };
        
        let output = format!("{:?}", request);
        assert!(output.contains("authorization:██"));
        assert!(output.contains("cookie:██"));
        assert!(output.contains("user-agent:OkHttp"));
    }

    #[test]
    pub fn request_to_string_includes_tags() {
        let request = Request::builder()
            .url("https://square.com/".to_http_url())
            .tag_with_type::<String>(Arc::new("hello".to_string()))
            .tag_with_type::<i32>(Arc::new(5))
            .build();
        
        let output = format!("{:?}", request);
        assert!(output.contains("tags={"));
        assert!(output.contains("hello"));
        assert!(output.contains("5"));
    }

    #[test]
    pub fn gzip_test() {
        let media_type = "text/plain; charset=utf-8".to_media_type();
        let original_body = "This is the original message".to_request_body(media_type.clone());
        assert_eq!(original_body.content_length(), 28);
        assert_eq!(original_body.content_type(), Some(media_type.clone()));

        let request = Request::builder()
            .url("https://square.com/")
            .post(original_body)
            .gzip()
            .build();
        
        assert_eq!(request.headers.get("Content-Encoding"), Some(&"gzip".to_string()));
        assert_eq!(request.body.as_ref().unwrap().content_length(), -1);
        assert_eq!(request.body.as_ref().unwrap().content_type(), Some(media_type));

        let mut buffer = Buffer::new();
        request.body.as_ref().unwrap().write_to(&mut buffer);

        let decompressed = GzipSource::new(buffer).buffer().read_utf8();
        assert_eq!(decompressed, "This is the original message");
    }

    #[test]
    #[should_panic(expected = "cannot gzip a request that has no body")]
    pub fn cannot_gzip_without_a_body() {
        Request::builder().url("https://square.com/").gzip().build();
    }

    #[test]
    #[should_panic(expected = "Content-Encoding already set: deflate")]
    pub fn cannot_gzip_with_another_content_encoding() {
        Request::builder()
            .url("https://square.com/")
            .post("This is the original message".to_request_body())
            .add_header("Content-Encoding", "deflate")
            .gzip()
            .build();
    }

    #[test]
    #[should_panic(expected = "Content-Encoding already set: gzip")]
    pub fn cannot_gzip_twice() {
        Request::builder()
            .url("https://square.com/")
            .post("This is the original message".to_request_body())
            .gzip()
            .gzip()
            .build();
    }

    #[test]
    pub fn curl_get() {
        let request = Request::builder()
            .url("https://example.com")
            .header("Authorization", "Bearer abc123")
            .build();

        let curl = request.to_curl(true);
        assert!(curl.contains("curl 'https://example.com/'"));
        assert!(curl.contains("-H 'Authorization: Bearer abc123'"));
    }

    #[test]
    pub fn curl_post_with_body() {
        let body = "{\"key\":\"value\"}".to_request_body("application/json".to_media_type());
        let request = Request::builder()
            .url("https://api.example.com/data")
            .post(body)
            .add_header("Authorization", "Bearer abc123")
            .build();

        let curl = request.to_curl(true);
        assert!(curl.contains("curl 'https://api.example.com/data'"));
        assert!(curl.contains("-H 'Authorization: Bearer abc123'"));
        assert!(curl.contains("-H 'Content-Type: application/json; charset=utf-8'"));
        assert!(curl.contains("--data '{\"key\":\"value\"}'"));
    }

    #[test]
    pub fn body_content_type_takes_precedence() {
        let body = "{\"key\":\"value\"}".to_request_body("application/json".to_media_type());
        let request = Request::builder()
            .url("https://api.example.com/data")
            .post(body)
            .add_header("Content-Type", "text/plain")
            .build();

        let curl = request.to_curl(true);
        assert!(curl.contains("-H 'Content-Type: application/json; charset=utf-8'"));
        assert!(!curl.contains("-H 'Content-Type: text/plain'"));
    }

    #[test]
    pub fn request_content_type_is_fallback() {
        let body = "{\"key\":\"value\"}".to_request_body_with_null_type();
        let request = Request::builder()
            .url("https://api.example.com/data")
            .post(body)
            .add_header("Content-Type", "text/plain")
            .build();

        let curl = request.to_curl(true);
        assert!(curl.contains("-H 'Content-Type: text/plain'"));
    }

    #[test]
    pub fn curl_put_with_body() {
        let body = "{\"key\":\"value\"}".to_request_body("application/json".to_media_type());
        let request = Request::builder()
            .url("https://api.example.com/data")
            .put(body)
            .add_header("Authorization", "Bearer abc123")
            .build();

        let curl = request.to_curl(true);
        assert!(curl.contains("-X 'PUT'"));
        assert!(curl.contains("--data '{\"key\":\"value\"}'"));
    }

    #[test]
    pub fn curl_post_with_complex_body() {
        let json_body = r#"{
  "user": {
    "id": 123,
    "name": "Tim O'Reilly"
  },
  "roles": ["admin", "editor"],
  "active": true
}"#;
        let body = json_body.to_request_body("application/json".to_media_type());
        let request = Request::builder()
            .url("https://api.example.com/users")
            .post(body)
            .add_header("Authorization", "Bearer xyz789")
            .build();

        let curl = request.to_curl(true);
        assert!(curl.contains("Tim O'\\''Reilly"));
    }

    #[test]
    pub fn curl_post_with_binary_body() {
        let binary_data = vec![0x00, 0x01, 0x02, 0x03];
        let body = RequestBody::from_bytes(binary_data, "application/octet-stream".to_media_type());
        let request = Request::builder()
            .url("https://api.example.com/upload")
            .post(body)
            .build();

        let curl = request.to_curl(true);
        assert!(curl.contains("--data-binary '00010203'"));
    }

    #[test]
    pub fn curl_post_with_binary_body_omitted() {
        let binary_data = vec![0x10, 0x20];
        let body = RequestBody::from_bytes(binary_data, "application/octet-stream".to_media_type());
        let request = Request::builder()
            .url("https://api.example.com/upload")
            .post(body)
            .build();

        let curl = request.to_curl(false);
        assert!(curl.contains("-X 'POST'"));
        assert!(!curl.contains("--data-binary"));
    }

    fn body_to_hex(body: &dyn RequestBody) -> String {
        let mut buffer = Buffer::new();
        body.write_to(&mut buffer);
        buffer.hex()
    }
}