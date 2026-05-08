/*
 * Copyright (C) 2021 Square, Inc.
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

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::CacheControl::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers::*;
use std::time::Duration as StdDuration;
use crate::android_test::build_gradle::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;

pub trait CacheControlCommon {
    fn common_to_string(&mut self) -> String;
}

impl CacheControlCommon for CacheControl {
    fn common_to_string(&mut self) -> String {
        if let Some(ref val) = self.header_value {
            return val.clone();
        }

        let mut result = String::new();
        if self.no_cache {
            result.push_str("no-cache, ");
        }
        if self.no_store {
            result.push_str("no-store, ");
        }
        if self.max_age_seconds != -1 {
            result.push_str(&format!("max-age={}, ", self.max_age_seconds));
        }
        if self.s_max_age_seconds != -1 {
            result.push_str(&format!("s-maxage={}, ", self.s_max_age_seconds));
        }
        if self.is_private {
            result.push_str("private, ");
        }
        if self.is_public {
            result.push_str("public, ");
        }
        if self.must_revalidate {
            result.push_str("must-revalidate, ");
        }
        if self.max_stale_seconds != -1 {
            result.push_str(&format!("max-stale={}, ", self.max_stale_seconds));
        }
        if self.min_fresh_seconds != -1 {
            result.push_str(&format!("min-fresh={}, ", self.min_fresh_seconds));
        }
        if self.only_if_cached {
            result.push_str("only-if-cached, ");
        }
        if self.no_transform {
            result.push_str("no-transform, ");
        }
        if self.immutable {
            result.push_str("immutable, ");
        }

        if result.is_empty() {
            self.header_value = Some("".to_string());
            return "".to_string();
        }

        // Remove trailing ", "
        let len = result.len();
        result.truncate(len - 2);
        
        self.header_value = Some(result.clone());
        result
    }
}

pub fn common_clamp_to_int(val: i64) -> i32 {
    if val > i32::MAX as i64 {
        i32::MAX
    } else {
        val as i32
    }
}

pub fn common_force_network() -> CacheControl {
    CacheControl::Builder::default()
        .no_cache()
        .build()
}

pub fn common_force_cache() -> CacheControl {
    CacheControl::Builder::default()
        .only_if_cached()
        .max_stale(StdDuration::from_secs(i32::MAX as u64))
        .build()
}

pub trait CacheControlBuilderCommon {
    fn common_build(self) -> CacheControl;
    fn common_no_cache(self) -> Self;
    fn common_no_store(self) -> Self;
    fn common_only_if_cached(self) -> Self;
    fn common_no_transform(self) -> Self;
    fn common_immutable(self) -> Self;
}

impl CacheControlBuilderCommon for CacheControl::Builder {
    fn common_build(self) -> CacheControl {
        CacheControl {
            no_cache: self.no_cache,
            no_store: self.no_store,
            max_age_seconds: self.max_age_seconds,
            s_max_age_seconds: -1,
            is_private: false,
            is_public: false,
            must_revalidate: false,
            max_stale_seconds: self.max_stale_seconds,
            min_fresh_seconds: self.min_fresh_seconds,
            only_if_cached: self.only_if_cached,
            no_transform: self.no_transform,
            immutable: self.immutable,
            header_value: None,
        }
    }

    fn common_no_cache(mut self) -> Self {
        self.no_cache = true;
        self
    }

    fn common_no_store(mut self) -> Self {
        self.no_store = true;
        self
    }

    fn common_only_if_cached(mut self) -> Self {
        self.only_if_cached = true;
        self
    }

    fn common_no_transform(mut self) -> Self {
        self.no_transform = true;
        self
    }

    fn common_immutable(mut self) -> Self {
        self.immutable = true;
        self
    }
}

pub fn common_parse(headers: &Headers) -> CacheControl {
    let mut no_cache = false;
    let mut no_store = false;
    let mut max_age_seconds = -1;
    let mut s_max_age_seconds = -1;
    let mut is_private = false;
    let mut is_public = false;
    let mut must_revalidate = false;
    let mut max_stale_seconds = -1;
    let mut min_fresh_seconds = -1;
    let mut only_if_cached = false;
    let mut no_transform = false;
    let mut immutable = false;

    let mut can_use_header_value = true;
    let mut header_value: Option<String> = None;

    for i in 0..headers.size() {
        let name = headers.name(i);
        let value = headers.value(i);

        if name.eq_ignore_ascii_case("Cache-Control") {
            if header_value.is_some() {
                can_use_header_value = false;
            } else {
                header_value = Some(value.clone());
            }
        } else if name.eq_ignore_ascii_case("Pragma") {
            can_use_header_value = false;
        } else {
            continue;
        }

        let mut pos = 0;
        let value_chars: Vec<char> = value.chars().collect();
        while pos < value_chars.len() {
            let token_start = pos;
            
            // Find index of '=', ',', or ';'
            let mut found_pos = value_chars.len();
            for j in pos..value_chars.len() {
                if value_chars[j] == '=' || value_chars[j] == ',' || value_chars[j] == ';' {
                    found_pos = j;
                    break;
                }
            }
            pos = found_pos;
            
            let directive: String = value_chars[token_start..pos].iter().collect::<String>().trim().to_string();
            let parameter: Option<String>;

            if pos == value_chars.len() || value_chars[pos] == ',' || value_chars[pos] == ';' {
                pos += 1;
                parameter = None;
            } else {
                pos += 1; // Consume '='
                
                // Skip whitespace
                while pos < value_chars.len() && value_chars[pos].is_whitespace() {
                    pos += 1;
                }

                if pos < value_chars.len() && value_chars[pos] == '\"' {
                    pos += 1; // Consume open quote
                    let parameter_start = pos;
                    let mut close_quote_pos = value_chars.len();
                    for j in pos..value_chars.len() {
                        if value_chars[j] == '\"' {
                            close_quote_pos = j;
                            break;
                        }
                    }
                    parameter = Some(value_chars[parameter_start..close_quote_pos].iter().collect());
                    pos = close_quote_pos + 1;
                } else {
                    let parameter_start = pos;
                    let mut end_pos = value_chars.len();
                    for j in pos..value_chars.len() {
                        if value_chars[j] == ',' || value_chars[j] == ';' {
                            end_pos = j;
                            break;
                        }
                    }
                    parameter = Some(value_chars[parameter_start..end_pos].iter().collect::<String>().trim().to_string());
                    pos = end_pos;
                }
            }

            match directive.to_lowercase().as_str() {
                "no-cache" => no_cache = true,
                "no-store" => no_store = true,
                "max-age" => max_age_seconds = to_non_negative_int(parameter.as_deref(), -1),
                "s-maxage" => s_max_age_seconds = to_non_negative_int(parameter.as_deref(), -1),
                "private" => is_private = true,
                "public" => is_public = true,
                "must-revalidate" => must_revalidate = true,
                "max-stale" => max_stale_seconds = to_non_negative_int(parameter.as_deref(), i32::MAX),
                "min-fresh" => min_fresh_seconds = to_non_negative_int(parameter.as_deref(), -1),
                "only-if-cached" => only_if_cached = true,
                "no-transform" => no_transform = true,
                "immutable" => immutable = true,
                _ => {}
            }
        }
    }

    if !can_use_header_value {
        header_value = None;
    }

    CacheControl {
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

fn to_non_negative_int(s: Option<&str>, default: i32) -> i32 {
    if let Some(val_str) = s {
        if let Ok(val) = val_str.parse::<i32>() {
            if val >= 0 {
                return val;
            }
        }
    }
    default
}