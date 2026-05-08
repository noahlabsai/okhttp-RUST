use std::sync::{Arc, OnceLock};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::CompressionInterceptor::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Gzip::*;
use crate::okhttp3::brotli::Brotli;

/// Transparent Brotli response support.
///
/// Adds Accept-Encoding: br to request and checks (and strips) for Content-Encoding: br in
/// responses. n.b. this replaces the transparent gzip compression in BridgeInterceptor.
pub struct BrotliInterceptor;

impl BrotliInterceptor {
    /// Returns the singleton instance of BrotliInterceptor.
    /// In Kotlin, this was an `object` extending `CompressionInterceptor`.
    /// In Rust, we use a OnceLock to provide a singleton instance of the parent class.
    pub fn instance() -> &'static CompressionInterceptor {
        pub static INSTANCE: OnceLock<CompressionInterceptor> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            // The Kotlin code: object BrotliInterceptor : CompressionInterceptor(Brotli, Gzip)
            // This passes the Brotli and Gzip decompression algorithms to the base class.
            CompressionInterceptor::new(vec![
                Arc::new(Brotli),
                Arc::new(Gzip::instance().clone()), // Assuming Gzip::instance() returns a reference or similar
            ])
        })
    }
}

// Note: Since BrotliInterceptor in Kotlin is a singleton object that inherits from 
// CompressionInterceptor, the Rust equivalent is to provide a way to access a 
// CompressionInterceptor configured with Brotli and Gzip.