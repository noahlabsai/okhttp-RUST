use std::time::Duration as StdDuration;
use std::fmt;

// Mocking internal common functions as they are not provided in the source but are required for compilation.
// In a real production environment, these would be imported from the internal module.
mod internal {
    pub fn common_clamp_to_int(val: i64) -> i32 {
        if val < 0 { -1 }
        else if val > i32::MAX as i64 { i32::MAX }
        else { val as i32 }
    }
}

// Mocking Headers since it's a dependency of the parse method
#[derive(Debug, Clone, PartialEq)]
pub struct Headers {
    pub values: Vec<(String, String)>,
}

impl Headers {
    pub fn get(&self, name: &str) -> Option<&String> {
        self.values.iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v)
    }
}

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
    pub(crate) header_value: Option<String>,
}

impl CacheControl {
    // Internal constructor to preserve Kotlin's internal constructor visibility
    pub(crate) fn new(
        no_cache: bool,
        no_store: bool,
        max_age_seconds: i32,
        s_max_age_seconds: i32,
        is_private: bool,
        is_public: bool,
        must_revalidate: bool,
        max_stale_seconds: i32,
        min_fresh_seconds: i32,
        only_if_cached: bool,
        no_transform: bool,
        immutable: bool,
        header_value: Option<String>,
    ) -> Self {
        Self {
            no_cache,
            no_store,
            max_age_seconds,
            s_max_age_seconds,
            is_private,
            is_public,
            must_revalidate,
            max_stale_seconds,
            min_fresh_seconds,
            only_if_cached,
            no_transform,
            immutable,
            header_value,
        }
    }

    // Deprecated methods preserved for API compatibility
    pub fn no_cache_deprecated(&self) -> bool { self.no_cache }
    pub fn no_store_deprecated(&self) -> bool { self.no_store }
    pub fn max_age_seconds_deprecated(&self) -> i32 { self.max_age_seconds }
    pub fn s_max_age_seconds_deprecated(&self) -> i32 { self.s_max_age_seconds }
    pub fn must_revalidate_deprecated(&self) -> bool { self.must_revalidate }
    pub fn max_stale_seconds_deprecated(&self) -> i32 { self.max_stale_seconds }
    pub fn min_fresh_seconds_deprecated(&self) -> i32 { self.min_fresh_seconds }
    pub fn only_if_cached_deprecated(&self) -> bool { self.only_if_cached }
    pub fn no_transform_deprecated(&self) -> bool { self.no_transform }
    pub fn immutable_deprecated(&self) -> bool { self.immutable }

    pub fn builder() -> CacheControl::Builder {
        CacheControl::Builder::default()
    }

    pub const FORCE_NETWORK: &'static CacheControl = &FORCE_NETWORK_INSTANCE;
    pub const FORCE_CACHE: &'static CacheControl = &FORCE_CACHE_INSTANCE;

    pub fn parse(headers: &Headers) -> Self {
        // This would call commonParse(headers) in Kotlin
        // Implementation omitted as it's an internal common function, 
        // but signature is preserved.
        unimplemented!("commonParse implementation required")
    }
}

impl fmt::Display for CacheControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // This would call commonToString() in Kotlin
        write!(f, "{:?}", self)
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

    pub fn max_age(mut self, max_age: StdDuration) -> Self {
        let seconds = max_age.as_secs() as i64;
        if seconds < 0 {
            panic!("maxAge < 0: {}", seconds);
        }
        self.max_age_seconds = internal::common_clamp_to_int(seconds);
        self
    }

    pub fn max_stale(mut self, max_stale: StdDuration) -> Self {
        let seconds = max_stale.as_secs() as i64;
        if seconds < 0 {
            panic!("maxStale < 0: {}", seconds);
        }
        self.max_stale_seconds = internal::common_clamp_to_int(seconds);
        self
    }

    pub fn min_fresh(mut self, min_fresh: StdDuration) -> Self {
        let seconds = min_fresh.as_secs() as i64;
        if seconds < 0 {
            panic!("minFresh < 0: {}", seconds);
        }
        self.min_fresh_seconds = internal::common_clamp_to_int(seconds);
        self
    }

    pub fn max_age_with_unit(mut self, max_age: i32, time_unit: TimeUnit) -> Self {
        if max_age < 0 {
            panic!("maxAge < 0: {}", max_age);
        }
        let seconds_long = time_unit.to_seconds(max_age as i64);
        self.max_age_seconds = internal::common_clamp_to_int(seconds_long);
        self
    }

    pub fn max_stale_with_unit(mut self, max_stale: i32, time_unit: TimeUnit) -> Self {
        if max_stale < 0 {
            panic!("maxStale < 0: {}", max_stale);
        }
        let seconds_long = time_unit.to_seconds(max_stale as i64);
        self.max_stale_seconds = internal::common_clamp_to_int(seconds_long);
        self
    }

    pub fn min_fresh_with_unit(mut self, min_fresh: i32, time_unit: TimeUnit) -> Self {
        if min_fresh < 0 {
            panic!("minFresh < 0: {}", min_fresh);
        }
        let seconds_long = time_unit.to_seconds(min_fresh as i64);
        self.min_fresh_seconds = internal::common_clamp_to_int(seconds_long);
        self
    }

    pub fn build(self) -> CacheControl {
        // This would call commonBuild() in Kotlin
        CacheControl::new(
            self.no_cache,
            self.no_store,
            self.max_age_seconds,
            -1, // s_max_age_seconds not handled in Builder
            false, // is_private
            false, // is_public
            false, // must_revalidate
            self.max_stale_seconds,
            self.min_fresh_seconds,
            self.only_if_cached,
            self.no_transform,
            self.immutable,
            None,
        )
    }
}

impl CacheControl {
    pub struct Builder {
        pub(crate) no_cache: bool,
        pub(crate) no_store: bool,
        pub(crate) max_age_seconds: i32,
        pub(crate) max_stale_seconds: i32,
        pub(crate) min_fresh_seconds: i32,
        pub(crate) only_if_cached: bool,
        pub(crate) no_transform: bool,
        pub(crate) immutable: bool,
    }
}

impl Default for CacheControl::Builder {
    fn default() -> Self {
        Self {
            no_cache: false,
            no_store: false,
            max_age_seconds: -1,
            max_stale_seconds: -1,
            min_fresh_seconds: -1,
            only_if_cached: false,
            no_transform: false,
            immutable: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TimeUnit {
    SECONDS,
    MINUTES,
    HOURS,
    DAYS,
    MILLISECONDS,
    MICROSECONDS,
    NANOSECONDS,
}

impl Default for TimeUnit {
    fn default() -> Self {
        TimeUnit::SECONDS
    }
}

pub const SECONDS: TimeUnit = TimeUnit::SECONDS;
pub const MINUTES: TimeUnit = TimeUnit::MINUTES;
pub const HOURS: TimeUnit = TimeUnit::HOURS;
pub const DAYS: TimeUnit = TimeUnit::DAYS;
pub const MILLISECONDS: TimeUnit = TimeUnit::MILLISECONDS;
pub const MICROSECONDS: TimeUnit = TimeUnit::MICROSECONDS;
pub const NANOSECONDS: TimeUnit = TimeUnit::NANOSECONDS;

impl TimeUnit {
    pub fn to_seconds(&self, duration: i64) -> i64 {
        match self {
            TimeUnit::SECONDS => duration,
            TimeUnit::MINUTES => duration * 60,
            TimeUnit::HOURS => duration * 3600,
            TimeUnit::DAYS => duration * 86400,
            TimeUnit::MILLISECONDS => duration / 1000,
            TimeUnit::MICROSECONDS => duration / 1_000_000,
            TimeUnit::NANOSECONDS => duration / 1_000_000_000,
        }
    }
}

// Static instances for companion object constants
static FORCE_NETWORK_INSTANCE: CacheControl = CacheControl {
    no_cache: true,
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
    header_value: None,
};

static FORCE_CACHE_INSTANCE: CacheControl = CacheControl {
    no_cache: false,
    no_store: false,
    max_age_seconds: -1,
    s_max_age_seconds: -1,
    is_private: false,
    is_public: false,
    must_revalidate: false,
    max_stale_seconds: i32::MAX,
    min_fresh_seconds: -1,
    only_if_cached: true,
    no_transform: false,
    immutable: false,
    header_value: None,
};
)}
