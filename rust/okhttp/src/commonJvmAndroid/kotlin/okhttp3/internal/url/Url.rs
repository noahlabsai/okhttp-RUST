/*
 * Copyright (C) 2022 Square, Inc.
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

use std::collections::HashMap;
use crate::android_test::build_gradle::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::cache::FaultHidingSink::*;

// Mocking okio::Buffer as it's a dependency in the original code.
// In a real production environment, this would be the actual okio crate.

impl Buffer {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn write_utf8(&mut self, s: &str, _pos: usize, _limit: usize) {
        // Note: The Kotlin code uses writeUtf8(this, pos, i) which is a substring write.
        // For the sake of this translation, we assume the logic is handled by the caller 
        // or implemented as a slice.
        self.data.extend_from_slice(s.as_bytes());
    }

    pub fn write_utf8_str(&mut self, s: &str) {
        self.data.extend_from_slice(s.as_bytes());
    }

    pub fn write_utf8_code_point(&mut self, cp: u32) {
        let mut buf = [0u8; 4];
        let len = cp.encode_utf8(&mut buf);
        self.data.extend_from_slice(&buf[..len]);
    }

    pub fn write_byte(&mut self, b: u8) {
        self.data.push(b);
    }

    pub fn read_byte(&mut self) -> u8 {
        self.data.remove(0)
    }

    pub fn exhausted(&self) -> bool {
        self.data.is_empty()
    }

    pub fn read_utf8(&self) -> String {
        String::from_utf8_lossy(&self.data).into_owned()
    }

    pub fn write_string(&mut self, _input: &str, _pos: usize, _limit: usize, _charset: &str) {
        // Simplified for translation; in reality, would use the provided charset.
        let slice = &_input[(_pos).._limit];
        self.data.extend_from_slice(slice.as_bytes());
    }
}

pub fn parse_hex_digit(c: char) -> i32 {
    c.to_digit(16).map(|d| d as i32).unwrap_or(-1)
}

pub const HEX_DIGITS: &[u8] = b"0123456789ABCDEF";
pub const USERNAME_ENCODE_SET: &str = " \"':;<=>@[]^`{}|/\\?#";
pub const PASSWORD_ENCODE_SET: &str = " \"':;<=>@[]^`{}|/\\?#";
pub const PATH_SEGMENT_ENCODE_SET: &str = " \"<>^`{}|/\\?#";
pub const PATH_SEGMENT_ENCODE_SET_URI: &str = "[]";
pub const QUERY_ENCODE_SET: &str = " \"'<>#";
pub const QUERY_COMPONENT_REENCODE_SET: &str = " \"'<>#&=";
pub const QUERY_COMPONENT_ENCODE_SET: &str = " !\"#$&'(),/:;<=>?@[]\\^`{|}~";
pub const QUERY_COMPONENT_ENCODE_SET_URI: &str = "\\^`{|}";
pub const FORM_ENCODE_SET: &str = " !\"#$&'()+,/:;<=>?@[\\]^`{|}~";
pub const FRAGMENT_ENCODE_SET: &str = "";
pub const FRAGMENT_ENCODE_SET_URI: &str = " \"#<>\\^`{|}";

pub trait BufferExt {
    fn write_canonicalized(
        &mut self,
        input: &str,
        pos: usize,
        limit: usize,
        encode_set: &str,
        already_encoded: bool,
        strict: bool,
        plus_is_space: bool,
        unicode_allowed: bool,
        charset: Option<&str>,
    );
    fn write_percent_decoded(&mut self, encoded: &str, pos: usize, limit: usize, plus_is_space: bool);
}

impl BufferExt for Buffer {
    fn write_canonicalized(
        &mut self,
        input: &str,
        pos: usize,
        limit: usize,
        encode_set: &str,
        already_encoded: bool,
        strict: bool,
        plus_is_space: bool,
        unicode_allowed: bool,
        charset: Option<&str>,
    ) {
        let mut encoded_char_buffer: Option<Buffer> = None;
        let input_chars: Vec<char> = input.chars().collect();
        let mut i = pos;

        while i < limit && i < input_chars.len() {
            let cp = input_chars[i] as u32;
            let c = input_chars[i];

            if already_encoded && (c == '\t' || c == '\n' || c == '\u{000c}' || c == '\r') {
                // Skip this character.
            } else if c == ' ' && encode_set == FORM_ENCODE_SET {
                self.write_utf8_str("+");
            } else if c == '+' && plus_is_space {
                self.write_utf8_str(if already_encoded { "+" } else { "%2B" });
            } else if cp < 0x20
                || cp == 0x7f
                || (cp >= 0x80 && !unicode_allowed)
                || encode_set.contains(c)
                || (c == '%' && (!already_encoded || (strict && !is_percent_encoded(input, i, limit))))
            {
                let mut buf = encoded_char_buffer.get_or_insert_with(Buffer::new);

                if charset.is_none() || charset == Some("UTF-8") {
                    buf.write_utf8_code_point(cp);
                } else {
                    // In a real impl, this would use the specific charset
                    buf.write_string(input, i, i + 1, charset.unwrap_or("UTF-8"));
                }

                while !buf.exhausted() {
                    let b = buf.read_byte();
                    self.write_byte(b'%');
                    self.write_byte(HEX_DIGITS[(b >> 4) as usize & 0xf]);
                    self.write_byte(HEX_DIGITS[b as usize & 0xf]);
                }
            } else {
                self.write_utf8_code_point(cp);
            }
            i += 1;
        }
    }

    fn write_percent_decoded(&mut self, encoded: &str, pos: usize, limit: usize, plus_is_space: bool) {
        let encoded_chars: Vec<char> = encoded.chars().collect();
        let mut i = pos;
        while i < limit && i < encoded_chars.len() {
            let c = encoded_chars[i];
            if c == '%' && i + 2 < limit && i + 2 < encoded_chars.len() {
                let d1 = parse_hex_digit(encoded_chars[i + 1]);
                let d2 = parse_hex_digit(encoded_chars[i + 2]);
                if d1 != -1 && d2 != -1 {
                    self.write_byte(((d1 << 4) + d2) as u8);
                    i += 3;
                    continue;
                }
            } else if c == '+' && plus_is_space {
                self.write_byte(b' ');
                i += 1;
                continue;
            }
            self.write_utf8_code_point(c as u32);
            i += 1;
        }
    }
}

pub fn is_percent_encoded(input: &str, pos: usize, limit: usize) -> bool {
    let chars: Vec<char> = input.chars().collect();
    if pos + 2 >= limit || pos + 2 >= chars.len() {
        return false;
    }
    chars[pos] == '%' && parse_hex_digit(chars[pos + 1]) != -1 && parse_hex_digit(chars[pos + 2]) != -1
}

pub trait StringCanonicalizeExt {
    fn canonicalize_with_charset(
        &self,
        pos: usize,
        limit: usize,
        encode_set: &str,
        already_encoded: bool,
        strict: bool,
        plus_is_space: bool,
        unicode_allowed: bool,
        charset: Option<&str>,
    ) -> String;

    fn canonicalize(
        &self,
        pos: usize,
        limit: usize,
        encode_set: &str,
        already_encoded: bool,
        strict: bool,
        plus_is_space: bool,
        unicode_allowed: bool,
    ) -> String;

    fn percent_decode(&self, pos: usize, limit: usize, plus_is_space: bool) -> String;
}

impl StringCanonicalizeExt for String {
    fn canonicalize_with_charset(
        &self,
        pos: usize,
        limit: usize,
        encode_set: &str,
        already_encoded: bool,
        strict: bool,
        plus_is_space: bool,
        unicode_allowed: bool,
        charset: Option<&str>,
    ) -> String {
        let chars: Vec<char> = self.chars().collect();
        let mut i = pos;
        while i < limit && i < chars.len() {
            let cp = chars[i] as u32;
            let c = chars[i];
            if cp < 0x20
                || cp == 0x7f
                || (cp >= 0x80 && !unicode_allowed)
                || encode_set.contains(c)
                || (c == '%' && (!already_encoded || (strict && !is_percent_encoded(self, i, limit))))
                || (c == '+' && plus_is_space)
            {
                let mut out = Buffer::new();
                // writeUtf8(this, pos, i)
                let prefix: String = chars[pos..i].iter().collect();
                out.write_utf8_str(&prefix);
                out.write_canonicalized(
                    self,
                    i,
                    limit,
                    encode_set,
                    already_encoded,
                    strict,
                    plus_is_space,
                    unicode_allowed,
                    charset,
                );
                return out.read_utf8();
            }
            i += 1;
        }
        // Fast path
        let result: String = chars[pos..limit.min(chars.len())].iter().collect();
        result
    }

    fn canonicalize(
        &self,
        pos: usize,
        limit: usize,
        encode_set: &str,
        already_encoded: bool,
        strict: bool,
        plus_is_space: bool,
        unicode_allowed: bool,
    ) -> String {
        self.canonicalize_with_charset(
            pos,
            limit,
            encode_set,
            already_encoded,
            strict,
            plus_is_space,
            unicode_allowed,
            None,
        )
    }

    fn percent_decode(&self, pos: usize, limit: usize, plus_is_space: bool) -> String {
        let chars: Vec<char> = self.chars().collect();
        for i in pos..limit {
            if i >= chars.len() { break; }
            let c = chars[i];
            if c == '%' || (c == '+' && plus_is_space) {
                let mut out = Buffer::new();
                let prefix: String = chars[pos..i].iter().collect();
                out.write_utf8_str(&prefix);
                out.write_percent_decoded(self, i, limit, plus_is_space);
                return out.read_utf8();
            }
        }
        let result: String = chars[pos..limit.min(chars.len())].iter().collect();
        result
    }
}