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

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::MediaType;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::MediaTypeGetTest::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test::src::test::kotlin::okhttp::android::test::DisabledInitialiserTest::*;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Challenge::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;

// Mocking the Platform.isAndroid check as it's a JVM/Android specific check
// In a real Rust translation of the OkHttp library, this would be a cfg feature or a runtime check.
fn is_android() -> bool {
    cfg!(target_os = "android")
}

// Mocking Locale for the purpose of the test translation
// Since Rust's standard library doesn't have a global Locale.setDefault, 
// and the tests are checking case-insensitivity (Turkish I), 
// we represent the logic.
pub struct Locale {
    pub language: String,
    pub country: String,
}

impl Locale {
    pub fn new(language: &str, country: &str) -> Self {
        Self {
            language: language.to_string(),
            country: country.to_string(),
        }
    }
    pub fn get_default() -> Self {
        Locale::new("en", "US")
    }
    pub fn set_default(_locale: Locale) {
        // In a real JVM environment, this changes the global state.
        // In Rust, this would typically be handled via a thread-local or a context object.
    }
}

pub struct MediaTypeJvmTest {
    get_test: MediaTypeGetTest,
}

impl MediaTypeJvmTest {
    pub fn new() -> Self {
        Self {
            get_test: MediaTypeGetTest::new(),
        }
    }

    // Equivalent to the extension function override in Kotlin
    fn charset_name(&self, media_type: &MediaType) -> Option<String> {
        media_type.charset().map(|c| c.name())
    }

    pub fn test_charset_name_is_double_quoted_and_single_quoted_android(&self) {
        let media_type = "text/plain;charset=\"'utf-8'\"".to_media_type();
        if is_android() {
            // Charset.forName("'utf-8'") == UTF-8 on Android
            assert_eq!(self.charset_name(&media_type), Some("UTF-8".to_string()));
        } else {
            assert!(media_type.charset().is_none());
        }
    }

    pub fn test_default_charset(&self) {
        let no_charset = self.get_test.parse("text/plain");
        
        // Assuming Charsets::UTF_8 and US_ASCII are available in the MediaType implementation
        // and charset() takes a default.
        assert_eq!(
            no_charset.charset("UTF-8").expect("Should have charset").name(),
            "UTF-8"
        );
        assert_eq!(
            no_charset.charset("US-ASCII").expect("Should have charset").name(),
            "US-ASCII"
        );

        let charset = self.get_test.parse("text/plain; charset=iso-8859-1");
        assert_eq!(
            charset.charset("UTF-8").expect("Should have charset").name(),
            "ISO-8859-1"
        );
        assert_eq!(
            charset.charset("US-ASCII").expect("Should have charset").name(),
            "ISO-8859-1"
        );
    }

    pub fn test_turkish_dotless_i_with_en_us(&self) {
        self.with_locale(Locale::new("en", "US"), || {
            let media_type = self.get_test.parse("IMAGE/JPEG");
            assert_eq!(media_type.type_(), "image");
            assert_eq!(media_type.subtype(), "jpeg");
        });
    }

    pub fn test_turkish_dotless_i_with_tr_tr(&self) {
        self.with_locale(Locale::new("tr", "TR"), || {
            let media_type = self.get_test.parse("IMAGE/JPEG");
            assert_eq!(media_type.type_(), "image");
            assert_eq!(media_type.subtype(), "jpeg");
        });
    }

    fn with_locale<T, F>(&self, locale: Locale, block: F) -> T 
    where 
        F: FnOnce() -> T 
    {
        let previous = Locale::get_default();
        Locale::set_default(locale);
        let result = block();
        Locale::set_default(previous);
        result
    }

    pub fn test_illegal_charset_name(&self) {
        let media_type = self.get_test.parse("text/plain; charset=\"!@#$%^&*()\"");
        assert!(self.charset_name(&media_type).is_none());
    }

    pub fn test_unsupported_charset(&self) {
        let media_type = self.get_test.parse("text/plain; charset=utf-wtf");
        assert!(self.charset_name(&media_type).is_none());
    }

    pub fn test_charset_name_is_double_quoted_and_single_quoted(&self) {
        let media_type = self.get_test.parse("text/plain;charset=\"'utf-8'\"");
        assert!(self.charset_name(&media_type).is_none());
    }

    pub fn test_charset_name_is_double_quoted_single_quote(&self) {
        let media_type = self.get_test.parse("text/plain;charset=\"'\"");
        assert!(self.charset_name(&media_type).is_none());
    }
}