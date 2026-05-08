use std::any::{Any, TypeId};
use std::sync::Arc;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrl::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::CacheControl::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::Tags::*;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;

// Mocking the assertion framework behavior as per Kotlin's assertk
macro_rules! assert_eq {
    ($left:expr, $right:expr) => {
        if $left != $right {
            panic!("Assertion failed: left {:?} != right {:?}", $left, $right);
        }
    };
}

macro_rules! assert_null {
    ($left:expr) => {
        if $left.is_some() {
            panic!("Assertion failed: expected None, found Some({:?})", $left);
        }
    };
}

macro_rules! assert_same_instance {
    ($left:expr, $right:expr) => {
        if Arc::ptr_eq($left, $right) == false {
            panic!("Assertion failed: expected same instance");
        }
    };
}

pub struct RequestCommonTest;

impl RequestCommonTest {
    pub fn constructor_normal(&self) {
        let url = "https://example.com/".to_http_url();
        let body = "hello".to_request_body();
        let headers = Headers::headers_of(&["User-Agent", "RequestTest"]);
        let method = "PUT";
        let request = Request::builder()
            .url("https://example.com/")
            .method(method)
            .headers(headers.clone())
            .body(Some(body.clone()))
            .build();

        assert_eq!(request.url, url);
        assert_eq!(request.headers, headers);
        assert_eq!(request.method, method);
        assert_eq!(request.body, Some(body));
        assert_eq!(request.tags, Tags::Empty);
    }

    pub fn constructor_no_body_no_method(&self) {
        let url = "https://example.com/".to_http_url();
        let headers = Headers::headers_of(&["User-Agent", "RequestTest"]);
        let request = Request::builder()
            .url("https://example.com/")
            .headers(headers.clone())
            .build();

        assert_eq!(request.url, url);
        assert_eq!(request.headers, headers);
        assert_eq!(request.method, "GET");
        assert_null!(request.body);
        assert_eq!(request.tags, Tags::Empty);
    }

    pub fn constructor_no_method(&self) {
        let url = "https://example.com/".to_http_url();
        let body = "hello".to_request_body();
        let headers = Headers::headers_of(&["User-Agent", "RequestTest"]);
        let request = Request::builder()
            .url("https://example.com/")
            .headers(headers.clone())
            .body(Some(body.clone()))
            .build();

        assert_eq!(request.url, url);
        assert_eq!(request.headers, headers);
        assert_eq!(request.method, "POST");
        assert_eq!(request.body, Some(body));
        assert_eq!(request.tags, Tags::Empty);
    }

    pub fn constructor_no_body(&self) {
        let url = "https://example.com/".to_http_url();
        let headers = Headers::headers_of(&["User-Agent", "RequestTest"]);
        let method = "DELETE";
        let request = Request::builder()
            .url("https://example.com/")
            .headers(headers.clone())
            .method(method)
            .build();

        assert_eq!(request.url, url);
        assert_eq!(request.headers, headers);
        assert_eq!(request.method, method);
        assert_null!(request.body);
        assert_eq!(request.tags, Tags::Empty);
    }

    pub fn new_builder_url_resets_url(&self) {
        let request_without_cache = Request::builder().url("http://localhost/api").build();
        let built_request_without_cache = request_without_cache.new_builder().url("http://localhost/api/foo").build();
        assert_eq!(built_request_without_cache.url, "http://localhost/api/foo".to_http_url());

        let request_with_cache = Request::builder().url("http://localhost/api").build();
        // cache url object
        let _ = &request_with_cache.url;
        let built_request_with_cache = request_with_cache.new_builder().url("http://localhost/api/foo").build();
        assert_eq!(built_request_with_cache.url, "http://localhost/api/foo".to_http_url());
    }

    pub fn cache_control(&self) {
        let request = Request::builder()
            .cache_control(CacheControl::builder().no_cache().build())
            .url("https://square.com")
            .build();
        
        assert_eq!(request.headers.get_all("Cache-Control"), vec!["no-cache"]);
        assert!(request.cache_control().no_cache);
    }

    pub fn empty_cache_control_clears_all_cache_control_headers(&self) {
        let request = Request::builder()
            .header("Cache-Control", "foo")
            .cache_control(CacheControl::builder().build())
            .url("https://square.com")
            .build();
        
        assert!(request.headers.get_all("Cache-Control").is_empty());
    }

    pub fn header_accepts_permitted_characters(&self) {
        let mut builder = Request::builder();
        builder = builder.header("AZab09~", "AZab09 ~");
        builder = builder.add_header("AZab09~", "AZab09 ~");
    }

    pub fn empty_name_forbidden(&self) {
        let builder = Request::builder();
        let res1 = std::panic::catch_unwind(|| {
            builder.clone().header("", "Value");
        });
        assert!(res1.is_err(), "Expected IllegalArgumentException for empty header name");

        let res2 = std::panic::catch_unwind(|| {
            builder.clone().add_header("", "Value");
        });
        assert!(res2.is_err(), "Expected IllegalArgumentException for empty header name");
    }

    pub fn header_allows_tab_only_in_values(&self) {
        let mut builder = Request::builder();
        builder = builder.header("key", "sample\tvalue");
        
        let res = std::panic::catch_unwind(|| {
            builder.clone().header("sample\tkey", "value");
        });
        assert!(res.is_err(), "Expected IllegalArgumentException for tab in header name");
    }

    pub fn header_forbids_control_characters(&self) {
        self.assert_forbidden_header("\u{0000}");
        self.assert_forbidden_header("\r");
        self.assert_forbidden_header("\n");
        self.assert_forbidden_header("\u{001f}");
        self.assert_forbidden_header("\u{007f}");
        self.assert_forbidden_header("\u{0080}");
        self.assert_forbidden_header("\u{1F369}"); // ?
    }

    fn assert_forbidden_header(&self, s: &str) {
        let builder = Request::builder();
        
        assert!(std::panic::catch_unwind(|| { builder.clone().header(s, "Value"); }).is_err());
        assert!(std::panic::catch_unwind(|| { builder.clone().add_header(s, "Value"); }).is_err());
        assert!(std::panic::catch_unwind(|| { builder.clone().header("Name", s); }).is_err());
        assert!(std::panic::catch_unwind(|| { builder.clone().add_header("Name", s); }).is_err());
    }

    pub fn no_tag(&self) {
        let request = Request::builder()
            .url("https://square.com")
            .build();
        
        assert_null!(request.tag::<dyn Any>());
        assert_null!(request.tag_with_type(TypeId::of::<dyn Any>()));
        assert_null!(request.tag_with_type(TypeId::of::<String>()));
        assert_null!(request.tag::<String>());
    }

    pub fn default_tag(&self) {
        let tag = Arc::new("1234".to_string());
        let request = Request::builder()
            .url("https://square.com")
            .tag(Some(tag.clone() as Arc<dyn Any>))
            .build();
        
        assert_same_instance!(request.tag::<dyn Any>(), tag.clone());
        assert_same_instance!(request.tag_with_type(TypeId::of::<dyn Any>()), tag.clone());
        assert_null!(request.tag_with_type(TypeId::of::<String>()));
    }

    pub fn null_removes_tag(&self) {
        let request = Request::builder()
            .url("https://square.com")
            .tag(Some(Arc::new("a".to_string()) as Arc<dyn Any>))
            .tag(None)
            .build();
        
        assert_null!(request.tag::<dyn Any>());
    }

    pub fn remove_absent_tag(&self) {
        let request = Request::builder()
            .url("https://square.com")
            .tag(None)
            .build();
        
        assert_null!(request.tag::<String>());
    }

    pub fn object_tag(&self) {
        let tag = Arc::new("1234".to_string());
        let request = Request::builder()
            .url("https://square.com")
            .tag_with_type_value(TypeId::of::<dyn Any>(), Some(tag.clone() as Arc<dyn Any>))
            .build();
        
        assert_same_instance!(request.tag::<dyn Any>(), tag.clone());
        assert_same_instance!(request.tag_with_type(TypeId::of::<dyn Any>()), tag.clone());
        assert_null!(request.tag_with_type(TypeId::of::<String>()));
    }

    pub fn kotlin_reified_tag(&self) {
        let uuid_tag = Arc::new("1234".to_string());
        let request = Request::builder()
            .url("https://square.com")
            .tag_with_type_value(TypeId::of::<String>(), Some(uuid_tag.clone() as Arc<dyn Any>))
            .build();
        
        assert_same_instance!(request.tag::<String>(), uuid_tag.clone());
        assert_null!(request.tag::<dyn Any>());
        assert_same_instance!(request.tag_with_type(TypeId::of::<String>()), uuid_tag);
    }

    pub fn kotlin_class_tag(&self) {
        let uuid_tag = Arc::new("1234".to_string());
        let request = Request::builder()
            .url("https://square.com")
            .tag_with_type_value(TypeId::of::<String>(), Some(uuid_tag.clone() as Arc<dyn Any>))
            .build();
        
        assert_null!(request.tag_with_type(TypeId::of::<dyn Any>()));
        assert_same_instance!(request.tag_with_type(TypeId::of::<String>()), uuid_tag.clone());
        assert_same_instance!(request.tag::<String>(), uuid_tag);
    }

    pub fn replace_only_tag(&self) {
        let uuid_tag1 = Arc::new("1234".to_string());
        let uuid_tag2 = Arc::new("4321".to_string());
        let request = Request::builder()
            .url("https://square.com")
            .tag_with_type_value(TypeId::of::<String>(), Some(uuid_tag1 as Arc<dyn Any>))
            .tag_with_type_value(TypeId::of::<String>(), Some(uuid_tag2.clone() as Arc<dyn Any>))
            .build();
        
        assert_same_instance!(request.tag_with_type(TypeId::of::<String>()), uuid_tag2);
    }

    pub fn multiple_tags(&self) {
        let string_tag = Arc::new("dilophosaurus".to_string());
        let long_tag = Arc::new(20170815i64);
        let object_tag = Arc::new(());
        
        let request = Request::builder()
            .url("https://square.com")
            .tag_with_type_value(TypeId::of::<dyn Any>(), Some(object_tag.clone() as Arc<dyn Any>))
            .tag_with_type_value(TypeId::of::<String>(), Some(string_tag.clone() as Arc<dyn Any>))
            .tag_with_type_value(TypeId::of::<i64>(), Some(long_tag.clone() as Arc<dyn Any>))
            .build();
        
        assert_same_instance!(request.tag::<dyn Any>(), object_tag.clone());
        assert_same_instance!(request.tag_with_type(TypeId::of::<dyn Any>()), object_tag);
        assert_same_instance!(request.tag_with_type(TypeId::of::<String>()), string_tag);
        // Long/i64 check omitted as per Kotlin TODO
    }

    pub fn tags_are_immutable(&self) {
        let builder = Request::builder().url("https://square.com");
        let request_a = builder.clone().tag_with_type_value(TypeId::of::<String>(), Some(Arc::new("a".to_string()) as Arc<dyn Any>)).build();
        let request_b = builder.tag_with_type_value(TypeId::of::<String>(), Some(Arc::new("b".to_string()) as Arc<dyn Any>)).build();
        let request_c = request_a.new_builder().tag_with_type_value(TypeId::of::<String>(), Some(Arc::new("c".to_string()) as Arc<dyn Any>)).build();
        
        assert_eq!(request_a.tag::<String>().unwrap(), "a");
        assert_eq!(request_b.tag::<String>().unwrap(), "b");
        assert_eq!(request_c.tag::<String>().unwrap(), "c");
    }

    pub fn request_to_string_redacts_sensitive_headers(&self) {
        let headers = Headers::builder()
            .add("content-length", "99")
            .add("authorization", "peanutbutter")
            .add("proxy-authorization", "chocolate")
            .add("cookie", "drink=coffee")
            .add("set-cookie", "accessory=sugar")
            .add("user-agent", "OkHttp")
            .build();
        
        let request = Request {
            url: "https://square.com".to_http_url(),
            headers,
            method: "GET".to_string(),
            body: None,
            tags: Tags::Empty,
        };
        
        let expected = format!(
            "Request{{method=GET, url=https://square.com/, headers=[\
            content-length:99,\
             authorization:██,\
             proxy-authorization:██,\
             cookie:██,\
             set-cookie:██,\
             user-agent:OkHttp\
            ]}}"
        );
        assert_eq!(request.to_string(), expected);
    }
}