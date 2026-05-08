/*
 * Copyright 2013 Twitter, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use okio::{Buffer, ByteString};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;

// Assuming these are provided by the internal http2 module as per the Kotlin imports
use crate::okhttp3::internal::http2::Huffman::{decode, encode, encoded_length};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;

pub struct HuffmanTest;

impl HuffmanTest {
    #[test]
    pub fn round_trip_for_request_and_response() {
        let s = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        
        for i in 0..s.len() {
            let substring = &s[0..i];
            let data = ByteString::from(substring);
            Self::assert_round_trip(data);
        }

        let mut random = ChaCha8Rng::seed_from_u64(123456789);
        
        let mut buf = vec![0u8; 4096];
        random.fill_bytes(&mut buf);
        
        let data = ByteString::from(buf);
        Self::assert_round_trip(data);
    }

    fn assert_round_trip(data: ByteString) {
        let mut encode_buffer = Buffer::new();
        
        encode(&data, &mut encode_buffer);
        
        let expected_length = encoded_length(&data) as i64;
        assert_eq!(
            expected_length, 
            encode_buffer.size(), 
            "Encoded length mismatch"
        );

        let mut decode_buffer = Buffer::new();
        
        let size = encode_buffer.size();
        decode(&mut encode_buffer, size, &mut decode_buffer);
        
        let decoded_data = decode_buffer.read_byte_string();
        assert_eq!(
            data, 
            decoded_data, 
            "Round trip data mismatch"
        );
    }
}
