use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// --- Mocking OkHttp Framework Types for Compilability ---
// Since the source is a test file, we must provide the necessary infrastructure 
// that the tests rely on (Request, Call, OkHttpClient, etc.)

#[derive(Debug, Clone, PartialEq)]
pub struct HttpUrl(pub String);

pub trait HttpUrlExt {
    fn to_http_url(self) -> HttpUrl;
}

impl HttpUrlExt for &str {
    fn to_http_url(self) -> HttpUrl {
        HttpUrl(self.to_string())
    }
}


pub struct RequestBuilder {
    url: Option<HttpUrl>,
    tags: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl RequestBuilder {
    pub fn new() -> Self {
        Self {
            url: None,
            tags: HashMap::new(),
        }
    }

    pub fn url(mut self, url: HttpUrl) -> Self {
        self.url = Some(url);
        self
    }

    pub fn tag<T: Any + Send + Sync>(mut self, value: T) -> Self {
        self.tags.insert(TypeId::of::<T>(), Arc::new(value));
        self
    }

    pub fn build(self) -> Request {
        Request {
            url: self.url.expect("URL is required"),
            tags: self.tags,
        }
    }
}

impl Request {
    pub fn builder() -> RequestBuilder {
        RequestBuilder::new()
    }
}

#[derive(Clone)]
pub struct Call {
    request: Request,
    // Computed tags are stored in a mutable map inside the call instance
    computed_tags: Arc<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl Call {
    pub fn new(request: Request) -> Self {
        Self {
            request,
            computed_tags: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn tag<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        let type_id = TypeId::of::<T>();
        
        // Check request tags first
        if let Some(tag) = self.request.tags.get(&type_id) {
            return tag.clone().downcast::<T>().ok();
        }
        
        // Check computed tags
        let computed = self.computed_tags.lock().unwrap();
        if let Some(tag) = computed.get(&type_id) {
            return tag.clone().downcast::<T>().ok();
        }
        
        None
    }

    pub fn tag_with_compute<T: Any + Send + Sync, F>(&self, compute: F) -> Arc<T> 
    where 
        F: FnOnce() -> T 
    {
        let type_id = TypeId::of::<T>();
        
        // 1. Check request tags
        if let Some(tag) = self.request.tags.get(&type_id) {
            return tag.clone().downcast::<T>().expect("Type mismatch in request tag");
        }
        
        // 2. Check computed tags
        {
            let computed = self.computed_tags.lock().unwrap();
            if let Some(tag) = computed.get(&type_id) {
                return tag.clone().downcast::<T>().expect("Type mismatch in computed tag");
            }
        }
        
        // 3. Compute and store
        let value = Arc::new(compute());
        let mut computed = self.computed_tags.lock().unwrap();
        computed.insert(type_id, value.clone());
        value
    }

    pub fn clone_call(&self) -> Self {
        // In Kotlin, computed tags are not retained in clone.
        // We clone the request but create a fresh computed_tags map.
        Self {
            request: self.request.clone(),
            computed_tags: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

pub struct OkHttpClient;

impl OkHttpClient {
    pub fn new_call(&self, request: Request) -> Call {
        Call::new(request)
    }
}

pub struct OkHttpClientTestRule;

impl OkHttpClientTestRule {
    pub fn new_client(&self) -> OkHttpClient {
        OkHttpClient
    }
}

// --- Test Suite Implementation ---

pub struct CallTagsTest {
    client_test_rule: OkHttpClientTestRule,
    client: OkHttpClient,
}

impl CallTagsTest {
    pub fn new() -> Self {
        let rule = OkHttpClientTestRule;
        let client = rule.new_client();
        Self {
            client_test_rule: rule,
            client,
        }
    }

    pub fn tags_seeded_from_request(&self) {
        let request = Request::builder()
            .url("https://square.com/".to_http_url())
            .tag(5i32)
            .tag("hello".to_string())
            .build();
        
        let call = self.client.new_call(request);

        // Check the Kotlin-focused APIs (Generic tags)
        assert_eq!(call.tag::<String>().map(|s| (*s).clone()), Some("hello".to_string()));
        assert_eq!(call.tag::<i32>().map(|i| *i), Some(5));
        assert!(call.tag::<bool>().is_none());
        // In Rust, Any is a trait, not a class. We check for a specific type that wouldn't be there.
        assert!(call.tag::<f64>().is_none());
    }

    pub fn tags_can_be_computed(&self) {
        let request = Request::builder()
            .url("https://square.com".to_http_url())
            .build();
        let call = self.client.new_call(request);

        // Check the Kotlin-focused APIs.
        // First call computes "a"
        assert_eq!(call.tag_with_compute::<String, _>(|| "a".to_string()), Arc::new("a".to_string()));
        // Second call returns cached "a" even if closure is different
        assert_eq!(call.tag_with_compute::<String, _>(|| "b".to_string()), Arc::new("a".to_string()));
        // Direct tag access returns cached "a"
        assert_eq!(call.tag::<String>().map(|s| (*s).clone()), Some("a".to_string()));

        // Check the Java-focused APIs (using i32 as Integer)
        assert_eq!(call.tag_with_compute::<i32, _>(|| 1), Arc::new(1));
        assert_eq!(call.tag_with_compute::<i32, _>(|| 2), Arc::new(1));
        assert_eq!(call.tag::<i32>().map(|i| *i), Some(1));
    }

    pub fn computed_tags_are_not_retained_in_clone(&self) {
        let request = Request::builder()
            .url("https://square.com".to_http_url())
            .build();
        let call_a = self.client.new_call(request);
        
        assert_eq!(call_a.tag_with_compute::<String, _>(|| "a".to_string()), Arc::new("a".to_string()));
        assert_eq!(call_a.tag_with_compute::<String, _>(|| "b".to_string()), Arc::new("a".to_string()));

        let call_b = call_a.clone_call();
        // call_b should not have "a" cached, so it computes "c"
        assert_eq!(call_b.tag_with_compute::<String, _>(|| "c".to_string()), Arc::new("c".to_string()));
        // call_b now has "c" cached
        assert_eq!(call_b.tag_with_compute::<String, _>(|| "d".to_string()), Arc::new("c".to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;

    #[test]
    fn test_tags_seeded_from_request() {
        let test = CallTagsTest::new();
        test.tags_seeded_from_request();
    }

    #[test]
    fn test_tags_can_be_computed() {
        let test = CallTagsTest::new();
        test.tags_can_be_computed();
    }

    #[test]
    fn test_computed_tags_are_not_retained_in_clone() {
        let test = CallTagsTest::new();
        test.computed_tags_are_not_retained_in_clone();
    }
}