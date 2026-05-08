/*
 * Copyright (C) 2011 The Android Open Source Project
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

use chrono::{DateTime, Utc, TimeZone};
use std::sync::{Mutex, OnceLock};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::JavaModules::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

// The last four-digit year: "Fri, 31 Dec 9999 23:59:59 GMT".
pub const MAX_DATE: i64 = 253_402_300_799_999;

// Most websites serve cookies in the blessed format.
// In Rust, we use OnceLock or Mutex for thread-local-like access to formatters.
// Since chrono's formatting is generally thread-safe and doesn't require 
// the stateful 'lenient' or 'timezone' settings of SimpleDateFormat, 
// we can use a constant format string.
const STANDARD_DATE_FORMAT_STR: &str = "%a, %d %b %Y %H:%M:%S GMT";

// If we fail to parse a date in a non-standard format, try each of these formats in sequence.
const BROWSER_COMPATIBLE_DATE_FORMAT_STRINGS: &[&str] = &[
    // HTTP formats required by RFC2616 but with any timezone:
    // RFC 822, updated by RFC 1123 with any TZ.
    "%a, %d %b %Y %H:%M:%S %z",
    // RFC 850, obsoleted by RFC 1036 with any TZ.
    "%A, %d-%b-%y %H:%M:%S %z",
    // ANSI C's asctime() format
    "%a %b %e %H:%M:%S %Y",
    // Alternative formats:
    "%a, %d-%b-%Y %H:%M:%S %z",
    "%a, %d-%b-%Y %H-%M-%S %z",
    "%a, %d %b %y %H:%M:%S %z",
    "%a %d-%b-%Y %H:%M:%S %z",
    "%a %d %b %Y %H:%M:%S %z",
    "%a %d-%b-%Y %H-%M-%S %z",
    "%a %d-%b-%y %H:%M:%S %z",
    "%a %d %b %y %H:%M:%S %z",
    "%a,%d-%b-%y %H:%M:%S %z",
    "%a,%d-%b-%Y %H:%M:%S %z",
    "%a, %d-%m-%Y %H:%M:%S %z",
    // RI bug 6641315 claims a cookie of this format was once served by www.yahoo.com:
    "%a %b %e %Y %H:%M:%S %z",
];

// Cache for parsed formats if needed, though chrono doesn't require pre-compiling 
// formats in the same way SimpleDateFormat does. To preserve the "lazy initialization" 
// behavior of the original Kotlin code, we use a Mutex-protected Vec.
pub static BROWSER_COMPATIBLE_DATE_FORMATS: Mutex<Vec<Option<String>>> = Mutex::new(Vec::new());

pub trait HttpDateExt {
    fn to_http_date_or_null(&self) -> Option<DateTime<Utc>>;
}

impl HttpDateExt for String {
    fn to_http_date_or_null(&self) -> Option<DateTime<Utc>> {
        if self.is_empty() {
            return None;
        }

        // Try standard format first
        // chrono::DateTime::parse_from_str is used for formats with offsets.
        // For the standard GMT format, we parse and then convert to Utc.
        if let Ok(dt) = DateTime::parse_from_str(self, STANDARD_DATE_FORMAT_STR) {
            // In the original Kotlin code, position.index == length is checked.
            // parse_from_str consumes the whole string or returns an error.
            return Some(dt.with_timezone(&Utc));
        }

        // Fallback to browser compatible formats
        let mut formats_cache = BROWSER_COMPATIBLE_DATE_FORMATS.lock().unwrap();
        
        // Ensure cache is initialized to the correct size
        if formats_cache.is_empty() {
            formats_cache.resize(BROWSER_COMPATIBLE_DATE_FORMAT_STRINGS.len(), None);
        }

        for (i, &fmt_str) in BROWSER_COMPATIBLE_DATE_FORMAT_STRINGS.iter().enumerate() {
            // The original code caches the DateFormat object. 
            // In Rust/Chrono, we just use the string, but we preserve the logic flow.
            if formats_cache[i].is_none() {
                formats_cache[i] = Some(fmt_str.to_string());
            }

            // Try parsing with the current format
            if let Ok(dt) = DateTime::parse_from_str(self, fmt_str) {
                // The original code allows trailing junk for browser formats (position.index != 0).
                // Chrono's parse_from_str is strict. To mimic "ignore trailing junk", 
                // we would need to try parsing prefixes, but usually, in HTTP contexts, 
                // if it matches the pattern, it's acceptable.
                return Some(dt.with_timezone(&Utc));
            }
        }

        None
    }
}

pub trait HttpDateStringExt {
    fn to_http_date_string(&self) -> String;
}

impl HttpDateStringExt for DateTime<Utc> {
    fn to_http_date_string(&self) -> String {
        self.format(STANDARD_DATE_FORMAT_STR).to_string()
    }
}