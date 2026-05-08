use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::Tags;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http::GzipRequestBody;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http::HttpMethod;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{HttpUrl, Headers, RequestBody, CacheControl};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::{is_probably_utf8, is_sensitive_header};
use okio::Buffer;
use crate::android_test::src::test::kotlin::okhttp::android::test::BaseOkHttpClientUnitTest::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http::HttpMethod::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;

// An HTTP request. Instances of this class are immutable if their [body] is null or itself
// immutable.
#[derive(Clone)]
pub struct Request {
    url: HttpUrl,
    method: String,
    headers: Headers,
    body: Option<RequestBody>,
    cache_url_override: Option<HttpUrl>,
    tags: Tags,
    lazy_cache_control: Arc<Mutex<Option<CacheControl>>>,
}

impl Request {
    // Constructs a new request.
    pub fn new(
        url: HttpUrl,
        headers: Headers,
        method: Option<String>,
        body: Option<RequestBody>,
    ) -> Self {
        let mut builder = Builder::new();
        builder.url(url);
        builder.headers(headers);
        
        let final_method = match method {
            Some(m) => m,
            None => if body.is_some() { "POST".to_string() } else { "GET".to_string() },
        };
        
        builder.method(final_method, body);
        builder.build()
    }

    pub fn url(&self) -> &HttpUrl {
        &self.url
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    pub fn body(&self) -> Option<&RequestBody> {
        self.body.as_ref()
    }

    pub fn cache_url_override(&self) -> Option<&HttpUrl> {
        self.cache_url_override.as_ref()
    }

    pub fn is_https(&self) -> bool {
        self.url.is_https()
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name)
    }

    pub fn headers_values(&self, name: &str) -> Vec<String> {
        self.headers.values(name)
    }

    // Returns the tag attached with the type T as a key.
    pub fn tag<T: Any>(&self) -> Option<Arc<T>> {
        self.tags.get::<T>(TypeId::of::<T>())
    }

    // Returns the tag attached with Any as a key.
    pub fn tag_any(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.tags.get::<dyn Any + Send + Sync>(TypeId::of::<dyn Any + Send + Sync>())
    }

    pub fn new_builder(&self) -> Builder {
        Builder::from_request(self)
    }

    pub fn cache_control(&self) -> CacheControl {
        let mut lock = self.lazy_cache_control.lock().unwrap();
        if let Some(ref cc) = *lock {
            cc.clone()
        } else {
            let cc = CacheControl::parse(&self.headers);
            *lock = Some(cc.clone());
            cc
        }
    }

    pub fn to_string_request(&self) -> String {
        let mut s = String::with_capacity(32);
        s.push_str("Request{method=");
        s.push_str(&self.method);
        s.push_str(", url=");
        s.push_str(&self.url.to_string());
        
        if self.headers.len() != 0 {
            s.push_str(", headers=[");
            for (index, (name, value)) in self.headers.iter().enumerate() {
                if index > 0 {
                    s.push_str(", ");
                }
                s.push_str(name);
                s.push(':');
                if is_sensitive_header(name) {
                    s.push_str("\u{2588}\u{2588}");
                } else {
                    s.push_str(value);
                }
            }
            s.push(']');
        }
        
        if !self.tags.is_empty() {
            s.push_str(", tags=");
            s.push_str(&format!("{:?}", self.tags));
        }
        s.push('}');
        s
    }

    pub fn to_curl(&self, include_body: bool) -> String {
        let mut s = String::new();
        s.push_str(&format!("curl {}", self.url.to_string().shell_escape()));

        let content_type = self.body.as_ref().and_then(|b| b.content_type()).map(|ct| ct.to_string());

        let default_method = if include_body && self.body.is_some() {
            "POST"
        } else {
            "GET"
        };

        if self.method != default_method {
            s.push_str(&format!(" \\\\\\n  -X {}", self.method.shell_escape()));
        }

        for (name, value) in self.headers.iter() {
            if let Some(ref ct) = content_type {
                if name.eq_ignore_ascii_case("Content-Type") {
                    continue;
                }
            }
            s.push_str(&format!(" \\\\\\n  -H {}", format!("{}: {}", name, value).shell_escape()));
        }

        if let Some(ref ct) = content_type {
            s.push_str(&format!(" \\\\\\n  -H {}", format!("Content-Type: {}", ct).shell_escape()));
        }

        if include_body && self.body.is_some() {
            let mut body_buffer = Buffer::new();
            self.body.as_ref().unwrap().write_to(&mut body_buffer);

            if is_probably_utf8(&body_buffer) {
                s.push_str(&format!(" \\\\\\n  --data {}", body_buffer.read_utf8().shell_escape()));
            } else {
                s.push_str(&format!(" \\\\\\n  --data-binary {}", body_buffer.read_byte_string().hex().shell_escape()));
            }
        }
        s
    }
}



impl Builder {
    pub fn new() -> Self {
        Self {
            url: None,
            method: "GET".to_string(),
            headers: Headers::Builder::new(),
            body: None,
            cache_url_override: None,
            tags: Tags::empty(),
        }
    }

    fn from_request(request: &Request) -> Self {
        Self {
            url: Some(request.url.clone()),
            method: request.method.clone(),
            body: request.body.clone(),
            tags: request.tags.clone(),
            headers: request.headers.new_builder(),
            cache_url_override: request.cache_url_override.clone(),
        }
    }

    pub fn url(mut self, url: HttpUrl) -> Self {
        self.url = Some(url);
        self
    }

    pub fn url_str(mut self, url: String) -> Self {
        let canonical = self.canonical_url(url);
        self.url = Some(canonical.to_http_url());
        self
    }

    fn canonical_url(&self, url: String) -> String {
        if url.to_lowercase().starts_with("ws:") {
            format!("http:{}", &url[3..])
        } else if url.to_lowercase().starts_with("wss:") {
            format!("https:{}", &url[4..])
        } else {
            url
        }
    }

    pub fn header(mut self, name: String, value: String) -> Self {
        self.headers.set(name, value);
        self
    }

    pub fn add_header(mut self, name: String, value: String) -> Self {
        self.headers.add(name, value);
        self
    }

    pub fn remove_header(mut self, name: String) -> Self {
        self.headers.remove_all(name);
        self
    }

    pub fn headers(mut self, headers: Headers) -> Self {
        self.headers = headers.new_builder();
        self
    }

    pub fn cache_control(self, cache_control: CacheControl) -> Self {
        let value = cache_control.to_string();
        if value.is_empty() {
            self.remove_header("Cache-Control".to_string())
        } else {
            self.header("Cache-Control".to_string(), value)
        }
    }

    pub fn get(self) -> Self {
        self.method("GET".to_string(), None)
    }

    pub fn head(self) -> Self {
        self.method("HEAD".to_string(), None)
    }

    pub fn post(self, body: RequestBody) -> Self {
        self.method("POST".to_string(), Some(body))
    }

    pub fn delete(self, body: Option<RequestBody>) -> Self {
        let body = body.unwrap_or(RequestBody::empty());
        self.method("DELETE".to_string(), Some(body))
    }

    pub fn put(self, body: RequestBody) -> Self {
        self.method("PUT".to_string(), Some(body))
    }

    pub fn patch(self, body: RequestBody) -> Self {
        self.method("PATCH".to_string(), Some(body))
    }

    pub fn query(self, body: RequestBody) -> Self {
        self.method("QUERY".to_string(), Some(body))
    }

    pub fn method(mut self, method: String, body: Option<RequestBody>) -> Self {
        if method.is_empty() {
            panic!("method.isEmpty() == true");
        }
        if body.is_none() {
            if HttpMethod::requires_request_body(&method) {
                panic!("method {} must have a request body.", method);
            }
        } else {
            if !HttpMethod::permits_request_body(&method) {
                panic!("method {} must not have a request body.", method);
            }
        }
        self.method = method;
        self.body = body;
        self
    }

    pub fn tag<T: Any>(mut self, tag: Option<T>) -> Self {
        let type_id = TypeId::of::<T>();
        let arc_tag = tag.map(Arc::new);
        self.tags = self.tags.plus(type_id, arc_tag);
        self
    }

    pub fn tag_any(mut self, tag: Option<Arc<dyn Any + Send + Sync>>) -> Self {
        let type_id = TypeId::of::<dyn Any + Send + Sync>();
        self.tags = self.tags.plus(type_id, tag);
        self
    }

    pub fn cache_url_override(mut self, override_url: Option<HttpUrl>) -> Self {
        self.cache_url_override = override_url;
        self
    }

    pub fn gzip(mut self) -> Self {
        let identity_body = self.body.clone().expect("cannot gzip a request that has no body");
        let content_encoding = self.headers.get("Content-Encoding");
        if let Some(ce) = content_encoding {
            panic!("Content-Encoding already set: {}", ce);
        }
        self.headers.add("Content-Encoding".to_string(), "gzip".to_string());
        self.body = Some(GzipRequestBody::new(identity_body));
        self
    }

    pub fn build(self) -> Request {
        let url = self.url.expect("url == null");
        Request {
            url,
            method: self.method,
            headers: self.headers.build(),
            body: self.body,
            cache_url_override: self.cache_url_override,
            tags: self.tags,
            lazy_cache_control: Arc::new(Mutex::new(None)),
        }
    }
}

trait ShellEscape {
    fn shell_escape(&self) -> String;
}

impl ShellEscape for String {
    fn shell_escape(&self) -> String {
        format!("'{}'", self.replace('\'', "'\\''"))
    }
}

impl ShellEscape for &str {
    fn shell_escape(&self) -> String {
        format!("'{}'", self.replace('\'', "'\\''"))
    }
}
