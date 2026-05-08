use std::collections::HashSet;
use std::error::Error;
use std::fmt;

// The original Kotlin code imports HttpUrl.Companion.toHttpUrlOrNull.
// In Rust, we should import the actual HttpUrl type from the okhttp crate.
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrl;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::idn::Punycode::*;

// Mocking WebPlatformToAsciiData as it is a data provider not defined in the source file.
#[derive(Debug, Clone, PartialEq)]
pub struct WebPlatformToAsciiEntry {
    pub input: Option<String>,
    pub output: Option<String>,
    pub comment: Option<String>,
}

pub struct WebPlatformToAsciiData;

impl WebPlatformToAsciiData {
    pub fn load() -> Vec<WebPlatformToAsciiEntry> {
        // This would normally load from a file.
        Vec::new()
    }
}

// Custom Error for Assertion failures to match Kotlin's AssertionError.
#[derive(Debug)]
pub struct AssertionError(String);

impl fmt::Display for AssertionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for AssertionError {}

/* Runs the web platform ToAscii tests. */
pub struct WebPlatformToAsciiTest {
    pub known_failures: HashSet<String>,
}

impl WebPlatformToAsciiTest {
    pub fn new() -> Self {
        let known_failures = HashSet::from([
            // OkHttp rejects empty labels.
            "x..xn--zca".to_string(),
            "x..\u{00df}".to_string(),
            // OkHttp rejects labels longer than 63 code points, the web platform tests don't.
            "x01234567890123456789012345678901234567890123456789012345678901x.xn--zca".to_string(),
            "x01234567890123456789012345678901234567890123456789012345678901x.\u{00df}".to_string(),
            "x01234567890123456789012345678901234567890123456789012345678901x".to_string(),
            "x01234567890123456789012345678901234567890123456789012345678901\u{2020}".to_string(),
            // OkHttp rejects domain names longer than 253 code points, the web platform tests don't.
            "01234567890123456789012345678901234567890123456789.01234567890123456789012345678901234567890123456789.01234567890123456789012345678901234567890123456789.01234567890123456789012345678901234567890123456789.0123456789012345678901234567890123456789012345678.x".to_string(),
            "01234567890123456789012345678901234567890123456789.01234567890123456789012345678901234567890123456789.01234567890123456789012345678901234567890123456789.01234567890123456789012345678901234567890123456789.0123456789012345678901234567890123456789012345678.xn--zca".to_string(),
            "01234567890123456789012345678901234567890123456789.01234567890123456789012345678901234567890123456789.01234567890123456789012345678901234567890123456789.01234567890123456789012345678901234567890123456789.0123456789012345678901234567890123456789012345678.\u{00df}".to_string(),
            // OkHttp does not reject invalid Punycode.
            "xn--a".to_string(),
            "xn--a.\u{00df}".to_string(),
            "xn--a.xn--zca".to_string(),
            "xn--a-yoc".to_string(),
            // OkHttp doesn't reject U+FFFD encoded in Punycode.
            "xn--zn7c.com".to_string(),
            // OkHttp doesn't reject a U+200D.
            "xn--1ug.example".to_string(),
            // OkHttp doesn't implement CheckJoiners.
            "\u{200D}.example".to_string(),
            // OkHttp doesn't implement CheckBidi.
            "\u{064aa}".to_string(),
        ]);

        Self { known_failures }
    }

    pub fn test(&self) -> Result<(), Box<dyn Error>> {
        let list = WebPlatformToAsciiData::load();
        let mut failures: Vec<Box<dyn Error>> = Vec::new();

        for entry in list {
            let mut failure: Option<Box<dyn Error>> = None;
            
            // Kotlin's entry.input!! is a force unwrap
            if let Some(input) = &entry.input {
                if let Err(e) = self.test_to_ascii(input, entry.output.as_deref(), entry.comment.as_deref()) {
                    failure = Some(e);
                }
            } else {
                panic!("entry.input was null");
            }

            if let Some(input) = &entry.input {
                if self.known_failures.contains(input) {
                    if failure.is_none() {
                        failures.push(Box::new(AssertionError(format!("known failure didn't fail: {:?}", entry))));
                    }
                } else {
                    if let Some(f) = failure {
                        failures.push(f);
                    }
                }
            }
        }

        if !failures.is_empty() {
            for failure in &failures {
                println!("{}", failure);
            }
            return Err(failures.into_iter().next().unwrap());
        }

        Ok(())
    }

    fn test_to_ascii(
        &self,
        input: &str,
        output: Option<&str>,
        comment: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let url_string = format!("https://{}/", input);
        // Using the static method from HttpUrl to mimic toHttpUrlOrNull()
        let url = HttpUrl::to_http_url_or_null(&url_string);
        
        let actual_host = url.and_then(|u| u.host());
        
        if actual_host != output {
            let name = comment.unwrap_or(input);
            return Err(Box::new(AssertionError(format!(
                "Assertion failed for {}: expected {:?}, but was {:?}",
                name, output, actual_host
            ))));
        }
        
        Ok(())
    }
}
