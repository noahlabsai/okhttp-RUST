/*
 * Copyright (C) 2019 Square, Inc.
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
use okio::{BufferedSource, Source};
use std::io::Read;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::CompressionInterceptor::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::ResponseBodyTest::*;

// In Rust, the Kotlin `object` (singleton) is typically represented as a unit struct
// or a module. Since it implements a trait, a unit struct is used here.
pub struct Brotli;

impl DecompressionAlgorithm for Brotli {
    fn encoding(&self) -> &str {
        "br"
    }

    fn decompress(&self, compressed_source: BufferedSource) -> Box<dyn Source> {
        // In Kotlin: BrotliInputStream(compressedSource.inputStream()).source()
        // 1. compressed_source.inputStream() provides a Read implementation.
        // 2. BrotliInputStream wraps that Read implementation.
        // 3. .source() converts the Read implementation back into an okio::Source.
        
        // Note: In a real production environment, you would use a crate like `brotli` 
        // or `brotli-decompress`. Here we preserve the logic flow.
        let input_stream = compressed_source.into_read();
        let brotli_stream = brotli::Decompressor::new(input_stream, 4096);
        
        // Convert the std::io::Read (Brotli stream) back to okio::Source
        Box::new(okio::source::from_read(brotli_stream))
    }
}

// To maintain the singleton access pattern of the Kotlin `object Brotli`, 
// we can provide a constant instance.
pub static BROTLI: Brotli = Brotli;