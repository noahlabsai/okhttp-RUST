/*
 * Copyright (c) 2025 Block, Inc.
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

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::CompressionInterceptor::DecompressionAlgorithm;
use okio::{BufferedSource, GzipSource, Source};
use std::sync::OnceLock;

/// Gzip implementation of the DecompressionAlgorithm.
/// In Kotlin, this is an 'object' (singleton). In Rust, we use a static instance
/// accessed via a function or a OnceLock to preserve the singleton behavior.
pub struct Gzip;

impl Gzip {
    /// Returns the singleton instance of Gzip.
    pub fn instance() -> &'static Self {
        pub static INSTANCE: OnceLock<Gzip> = OnceLock::new();
        INSTANCE.get_or_init(|| Gzip)
    }
}

impl DecompressionAlgorithm for Gzip {
    fn encoding(&self) -> &str {
        "gzip"
    }

    fn decompress(&self, compressed_source: BufferedSource) -> Box<dyn Source> {
        // GzipSource in okio wraps a BufferedSource to provide decompression.
        Box::new(GzipSource::new(compressed_source))
    }
}