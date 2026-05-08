/*
 * Copyright (C) 2014 Square, Inc.
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
use std::sync::Mutex;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;

// Mocking the extension traits based on the provided translation memory
pub trait HttpDateExt {
    fn to_http_date_or_null(&self) -> Option<DateTime<Utc>>;
}

impl HttpDateExt for String {
    fn to_http_date_or_null(&self) -> Option<DateTime<Utc>> {
        // In a real implementation, this would contain the parsing logic for RFC 822, 850, and asctime
        // For the purpose of this test translation, we assume the implementation exists.
        None 
    }
}

impl HttpDateExt for &str {
    fn to_http_date_or_null(&self) -> Option<DateTime<Utc>> {
        self.to_string().to_http_date_or_null()
    }
}

pub trait HttpDateStringExt {
    fn to_http_date_string(&self) -> String;
}

impl HttpDateStringExt for DateTime<Utc> {
    fn to_http_date_string(&self) -> String {
        // In a real implementation, this would format the date as "Thu, 01 Jan 1970 00:00:00 GMT"
        "".to_string()
    }
}

// Helper to mimic java.util.Date(long millis)
struct Date {
    time: i64,
}

impl Date {
    fn new(millis: i64) -> Self {
        Self { time: millis }
    }

    fn to_http_date_string(&self) -> String {
        let dt = Utc.timestamp_millis_opt(self.time).unwrap();
        dt.to_http_date_string()
    }
}

// Mocking TimeZone for the test setup/teardown logic
struct TimeZone;
impl TimeZone {
    fn get_default() -> String {
        "UTC".to_string()
    }

    fn get_time_zone(_id: &str) -> String {
        id.to_string()
    }
}

pub struct HttpDateTest {
    original_default: Option<String>,
}

impl HttpDateTest {
    pub fn new() -> Self {
        Self {
            original_default: None,
        }
    }

    pub fn set_up(&mut self) {
        self.original_default = Some(TimeZone::get_default());
        // The default timezone should affect none of these tests: HTTP specified GMT, so we set it to
        // something else.
        TimeZone::set_default(TimeZone::get_time_zone("America/Los_Angeles"));
    }

    pub fn tear_down(&mut self) {
        if let Some(ref original) = self.original_default {
            TimeZone::set_default(original.clone());
        }
    }

    #[test]
    pub fn parse_standard_formats(&self) {
        // RFC 822, updated by RFC 1123 with GMT.
        assert_eq!("Thu, 01 Jan 1970 00:00:00 GMT".to_http_date_or_null().expect("!!").timestamp_millis(), 0);
        assert_eq!("Fri, 06 Jun 2014 12:30:30 GMT".to_http_date_or_null().expect("!!").timestamp_millis(), 1402057830000);

        // RFC 850, obsoleted by RFC 1036 with GMT.
        assert_eq!("Thursday, 01-Jan-70 00:00:00 GMT".to_http_date_or_null().expect("!!").timestamp_millis(), 0);
        assert_eq!("Friday, 06-Jun-14 12:30:30 GMT".to_http_date_or_null().expect("!!").timestamp_millis(), 1402057830000);

        // ANSI C's asctime(): should use GMT, not platform default.
        assert_eq!("Thu Jan 1 00:00:00 1970".to_http_date_or_null().expect("!!").timestamp_millis(), 0);
        assert_eq!("Fri Jun 6 12:30:30 2014".to_http_date_or_null().expect("!!").timestamp_millis(), 1402057830000);
    }

    #[test]
    pub fn format(&self) {
        assert_eq!(Date::new(0).to_http_date_string(), "Thu, 01 Jan 1970 00:00:00 GMT");
        assert_eq!(Date::new(1402057830000).to_http_date_string(), "Fri, 06 Jun 2014 12:30:30 GMT");
    }

    #[test]
    pub fn parse_non_standard_strings(&self) {
        // RFC 822, updated by RFC 1123 with any TZ
        assert_eq!("Thu, 01 Jan 1970 00:00:00 GMT-01:00".to_http_date_or_null().expect("!!").timestamp_millis(), 3600000);
        assert_eq!("Thu, 01 Jan 1970 00:00:00 PST".to_http_date_or_null().expect("!!").timestamp_millis(), 28800000);
        
        // Ignore trailing junk
        assert_eq!("Thu, 01 Jan 1970 00:00:00 GMT JUNK".to_http_date_or_null().expect("!!").timestamp_millis(), 0);
        
        // Missing timezones treated as bad.
        assert!( "Thu, 01 Jan 1970 00:00:00".to_http_date_or_null().is_none());
        
        // Missing seconds treated as bad.
        assert!( "Thu, 01 Jan 1970 00:00 GMT".to_http_date_or_null().is_none());
        
        // Extra spaces treated as bad.
        assert!( "Thu,  01 Jan 1970 00:00 GMT".to_http_date_or_null().is_none());
        
        // Missing leading zero treated as bad.
        assert!( "Thu, 1 Jan 1970 00:00 GMT".to_http_date_or_null().is_none());

        // RFC 850, obsoleted by RFC 1036 with any TZ.
        assert_eq!("Thursday, 01-Jan-1970 00:00:00 GMT-01:00".to_http_date_or_null().expect("!!").timestamp_millis(), 3600000);
        assert_eq!("Thursday, 01-Jan-1970 00:00:00 PST".to_http_date_or_null().expect("!!").timestamp_millis(), 28800000);
        
        // Ignore trailing junk
        assert_eq!("Thursday, 01-Jan-1970 00:00:00 PST JUNK".to_http_date_or_null().expect("!!").timestamp_millis(), 28800000);

        // ANSI C's asctime() format
        // This format ignores the timezone entirely even if it is present and uses GMT.
        assert_eq!("Fri Jun 6 12:30:30 2014 PST".to_http_date_or_null().expect("!!").timestamp_millis(), 1402057830000);
        
        // Ignore trailing junk.
        assert_eq!("Fri Jun 6 12:30:30 2014 JUNK".to_http_date_or_null().expect("!!").timestamp_millis(), 1402057830000);
    }
}