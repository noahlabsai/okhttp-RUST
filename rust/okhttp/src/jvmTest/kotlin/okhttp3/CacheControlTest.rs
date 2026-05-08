use std::fmt;
use std::time::Duration;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;

#[derive(Debug, Clone, PartialEq)]
pub struct CacheControl {
    pub no_cache: bool,
    pub no_store: bool,
    pub max_age_seconds: i32,
    pub s_max_age_seconds: i32,
    pub is_private: bool,
    pub is_public: bool,
    pub must_revalidate: bool,
    pub max_stale_seconds: i32,
    pub min_fresh_seconds: i32,
    pub only_if_cached: bool,
    pub no_transform: bool,
    pub immutable: bool,
}

impl fmt::Display for CacheControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.no_cache { parts.push("no-cache".to_string()); }
        if self.no_store { parts.push("no-store".to_string()); }
        if self.max_age_seconds >= 0 { parts.push(format!("max-age={}", self.max_age_seconds)); }
        if self.s_max_age_seconds >= 0 { parts.push(format!("s-maxage={}", self.s_max_age_seconds)); }
        if self.is_private { parts.push("private".to_string()); }
        if self.is_public { parts.push("public".to_string()); }
        if self.must_revalidate { parts.push("must-revalidate".to_string()); }
        if self.max_stale_seconds >= 0 { parts.push(format!("max-stale={}", self.max_stale_seconds)); }
        if self.min_fresh_seconds >= 0 { parts.push(format!("min-fresh={}", self.min_fresh_seconds)); }
        if self.only_if_cached { parts.push("only-if-cached".to_string()); }
        if self.no_transform { parts.push("no-transform".to_string()); }
        if self.immutable { parts.push("immutable".to_string()); }
        write!(f, "{}", parts.join(", "))
    }
}

impl CacheControl {

    pub fn builder() -> Self::Builder {
        Self::Builder::default()
    }
}

impl Default for CacheControl::Builder {
    fn default() -> Self {
        Self {
            no_cache: false,
            no_store: false,
            max_age_seconds: -1,
            s_max_age_seconds: -1,
            is_private: false,
            is_public: false,
            must_revalidate: false,
            max_stale_seconds: -1,
            min_fresh_seconds: -1,
            only_if_cached: false,
            no_transform: false,
            immutable: false,
        }
    }
}

impl CacheControl::Builder {
    pub fn no_cache(mut self) -> Self {
        self.no_cache = true;
        self
    }

    pub fn no_store(mut self) -> Self {
        self.no_store = true;
        self
    }

    pub fn max_age(mut self, duration: Duration) -> Self {
        self.max_age_seconds = duration.as_secs() as i32;
        self
    }

    pub fn max_stale(mut self, duration: Duration) -> Self {
        self.max_stale_seconds = duration.as_secs() as i32;
        self
    }

    pub fn min_fresh(mut self, duration: Duration) -> Self {
        self.min_fresh_seconds = duration.as_secs() as i32;
        self
    }

    pub fn only_if_cached(mut self) -> Self {
        self.only_if_cached = true;
        self
    }

    pub fn no_transform(mut self) -> Self {
        self.no_transform = true;
        self
    }

    pub fn immutable(mut self) -> Self {
        self.immutable = true;
        self
    }

    pub fn build(self) -> CacheControl {
        CacheControl {
            no_cache: self.no_cache,
            no_store: self.no_store,
            max_age_seconds: self.max_age_seconds,
            s_max_age_seconds: self.s_max_age_seconds,
            is_private: self.is_private,
            is_public: self.is_public,
            must_revalidate: self.must_revalidate,
            max_stale_seconds: self.max_stale_seconds,
            min_fresh_seconds: self.min_fresh_seconds,
            only_if_cached: self.only_if_cached,
            no_transform: self.no_transform,
            immutable: self.immutable,
        }
    }
}

pub struct CacheControlTest;

impl CacheControlTest {
    #[test]
    pub fn empty_builder_is_empty() {
        let cache_control = CacheControl::builder().build();
        assert_eq!(cache_control.to_string(), "");
        assert!(!cache_control.no_cache);
        assert!(!cache_control.no_store);
        assert_eq!(cache_control.max_age_seconds, -1);
        assert_eq!(cache_control.s_max_age_seconds, -1);
        assert!(!cache_control.is_private);
        assert!(!cache_control.is_public);
        assert!(!cache_control.must_revalidate);
        assert_eq!(cache_control.max_stale_seconds, -1);
        assert_eq!(cache_control.min_fresh_seconds, -1);
        assert!(!cache_control.only_if_cached);
        assert!(!cache_control.must_revalidate);
    }

    #[test]
    pub fn complete_builder() {
        let cache_control = CacheControl::builder()
            .no_cache()
            .no_store()
            .max_age(Duration::from_secs(1))
            .max_stale(Duration::from_secs(2))
            .min_fresh(Duration::from_secs(3))
            .only_if_cached()
            .no_transform()
            .immutable()
            .build();

        assert_eq!(
            cache_control.to_string(),
            "no-cache, no-store, max-age=1, max-stale=2, min-fresh=3, only-if-cached, no-transform, immutable"
        );
        assert!(cache_control.no_cache);
        assert!(cache_control.no_store);
        assert_eq!(cache_control.max_age_seconds, 1);
        assert_eq!(cache_control.max_stale_seconds, 2);
        assert_eq!(cache_control.min_fresh_seconds, 3);
        assert!(cache_control.only_if_cached);
        assert!(cache_control.no_transform);
        assert!(cache_control.immutable);

        // These members are accessible to response headers only.
        assert_eq!(cache_control.s_max_age_seconds, -1);
        assert!(!cache_control.is_private);
        assert!(!cache_control.is_public);
        assert!(!cache_control.must_revalidate);
    }
}
)}
