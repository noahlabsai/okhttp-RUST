use std::error::Error;
use std::sync::Arc;

// Assuming Headers is defined elsewhere in the okhttp3 crate.
// If not, this would be a trait or struct.
use crate::okhttp3::Headers;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;

// Returns the trailers that follow an HTTP response, blocking if they aren't ready yet.
// Implementations of this interface should respond to [Call.cancel] by immediately throwing an
// [IOException].
//
// Most callers won't need this interface, and should use [Response.trailers] instead.
//
// This interface is for test and production code that creates [Response] instances without making
// an HTTP call to a remote server.
pub trait TrailersSource: Send + Sync {
    // Returns the trailers that follow an HTTP response, blocking if they aren't ready yet.
    // Returns None if no trailers are available.
    fn peek(&self) -> Result<Option<Headers>, Box<dyn Error + Send + Sync>> {
        Ok(None)
    }

    // Returns the trailers that follow an HTTP response, blocking if they aren't ready yet.
    fn get(&self) -> Result<Headers, Box<dyn Error + Send + Sync>>;
}

// Implementation of TrailersSource for the EMPTY singleton.
struct EmptyTrailersSource;

impl TrailersSource for EmptyTrailersSource {
    fn peek(&self) -> Result<Option<Headers>, Box<dyn Error + Send + Sync>> {
        Ok(Some(Headers::EMPTY))
    }

    fn get(&self) -> Result<Headers, Box<dyn Error + Send + Sync>> {
        Ok(Headers::EMPTY)
    }
}

// Companion object equivalent for TrailersSource.
pub struct TrailersSourceCompanion;

impl TrailersSourceCompanion {
    // The EMPTY singleton instance of TrailersSource.
    pub fn empty() -> Arc<dyn TrailersSource> {
        Arc::new(EmptyTrailersSource)
    }
}

// To maintain the @JvmField val EMPTY behavior, we can provide a lazy static or a constant.
// Since Rust doesn't have a direct equivalent to a JVM singleton object in a trait,
// we use a function or a lazy static.
pub static EMPTY: once_cell::sync::Lazy<Arc<dyn TrailersSource>> = 
    once_cell::sync::Lazy::new(|| TrailersSourceCompanion::empty());