use std::collections::HashMap;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;

// --- Mocking the okhttp3::Headers implementation to make the test compilable ---
// In a real scenario, these would be imported from the actual okhttp3 crate.


impl Headers {
    pub fn name(&self, index: usize) -> String {
        if index >= self.pairs.len() {
            panic!("IndexOutOfBoundsException");
        }
        self.pairs[index].0.clone()
    }

    pub fn value(&self, index: usize) -> String {
        if index >= self.pairs.len() {
            panic!("IndexOutOfBoundsException");
        }
        self.pairs[index].1.clone()
    }

    pub fn builder() -> Headers::Builder {
        Headers::Builder::default()
    }

    pub fn headers_of(pairs: &[&str]) -> Result<Self, String> {
        if pairs.len() % 2 != 0 {
            return Err("IllegalArgumentException: Odd number of headers".to_string());
        }
        let mut result = Vec::new();
        for i in (0..pairs.len()).step_by(2) {
            let name = pairs[i].trim();
            let value = pairs[i + 1].trim();
            if name.is_empty() {
                return Err("IllegalArgumentException: Empty name".to_string());
            }
            if name.contains('\u{0000}') {
                return Err("IllegalArgumentException: Null char in name".to_string());
            }
            if value.contains('\u{0000}') {
                return Err("IllegalArgumentException: Null char in value".to_string());
            }
            // Unicode check simulation
            for (idx, c) in name.chars().enumerate() {
                if !c.is_ascii() {
                    return Err(format!("Unexpected char {:#x} at {} in header name: {}", c as u32, idx, name));
                }
            }
            for (idx, c) in value.chars().enumerate() {
                if !c.is_ascii() {
                    return Err(format!("Unexpected char {:#x} at {} in {} value: {}", c as u32, idx, name, value));
                }
            }
            result.push((name.to_string(), value.to_string()));
        }
        Ok(Headers { pairs: result })
    }

    pub fn from_map(map: HashMap<String, String>) -> Result<Self, String> {
        let mut pairs = Vec::new();
        for (k, v) in map {
            let name = k.trim();
            let value = v.trim();
            if name.is_empty() {
                return Err("IllegalArgumentException: Empty name".to_string());
            }
            if name.contains('\u{0000}') {
                return Err("IllegalArgumentException: Null char in name".to_string());
            }
            if value.contains('\u{0000}') {
                return Err("IllegalArgumentException: Null char in value".to_string());
            }
            for (idx, c) in name.chars().enumerate() {
                if !c.is_ascii() {
                    return Err(format!("Unexpected char {:#x} at {} in header name: {}", c as u32, idx, name));
                }
            }
            for (idx, c) in value.chars().enumerate() {
                if !c.is_ascii() {
                    return Err(format!("Unexpected char {:#x} at {} in {} value: {}", c as u32, idx, name, value));
                }
            }
            pairs.push((name.to_string(), value.to_string()));
        }
        Ok(Headers { pairs })
    }

    pub fn to_string(&self) -> String {
        let mut sb = String::new();
        let sensitive = ["authorization", "proxy-authorization", "cookie", "set-cookie"];
        for (name, value) in &self.pairs {
            let display_value = if sensitive.contains(&name.to_lowercase().as_str()) {
                "██"
            } else {
                value
            };
            sb.push_str(&format!("{}: {}\n", name, display_value));
        }
        sb
    }


    impl Default for Headers::Builder {
        fn default() -> Self {
            Headers::Builder { pairs: Vec::new() }
        }
    }

    impl Headers::Builder {
        pub fn add(mut self, name: &str, value: &str) -> Result<Self, String> {
            let trimmed_name = name.trim();
            let trimmed_value = value.trim();
            
            if trimmed_name.is_empty() {
                return Err("IllegalArgumentException: Empty name".to_string());
            }

            for (idx, c) in trimmed_name.chars().enumerate() {
                if !c.is_ascii() {
                    return Err(format!("Unexpected char {:#x} at {} in header name: {}", c as u32, idx, trimmed_name));
                }
            }

            let is_sensitive = ["Authorization", "Proxy-Authorization", "Cookie", "Set-Cookie"]
                .iter().any(|&s| s.eq_ignore_ascii_case(trimmed_name));

            for (idx, c) in trimmed_value.chars().enumerate() {
                if !c.is_ascii() {
                    if is_sensitive {
                        return Err(format!("Unexpected char {:#x} at {} in {} value", c as u32, idx, trimmed_name));
                    } else {
                        return Err(format!("Unexpected char {:#x} at {} in {} value: {}", c as u32, idx, trimmed_name, trimmed_value));
                    }
                }
            }

            self.pairs.push((trimmed_name.to_string(), trimmed_value.to_string()));
            Ok(self)
        }

        pub fn add_all(mut self, headers: &Headers) -> Self {
            self.pairs.extend(headers.pairs.clone());
            self
        }

        pub fn build(self) -> Headers {
            Headers { pairs: self.pairs }
        }
    }
}

// Helper trait to mimic Kotlin's .toHeaders() extension on Map
pub trait MapHeadersExt {
    fn to_headers(&self) -> Result<Headers, String>;
}

impl MapHeadersExt for HashMap<String, String> {
    fn to_headers(&self) -> Result<Headers, String> {
        Headers::from_map(self.clone())
    }
}

// --- Test Suite ---

pub struct HeadersTest;

impl HeadersTest {
    pub fn of_trims() {
        let headers = Headers::headers_of(&["\t User-Agent \n", " \r OkHttp "]).unwrap();
        assert_eq!(headers.name(0), "User-Agent");
        assert_eq!(headers.value(0), "OkHttp");
    }

    pub fn of_throws_odd_number_of_headers() {
        let result = Headers::headers_of(&["User-Agent", "OkHttp", "Content-Length"]);
        assert!(result.is_err());
    }

    pub fn of_throws_on_empty_name() {
        let result = Headers::headers_of(&["", "OkHttp"]);
        assert!(result.is_err());
    }

    pub fn of_accepts_empty_value() {
        let headers = Headers::headers_of(&["User-Agent", ""]).unwrap();
        assert_eq!(headers.value(0), "");
    }

    pub fn of_makes_defensive_copy() {
        let mut names_and_values = vec!["User-Agent", "OkHttp"];
        let headers = Headers::headers_of(&names_and_values).unwrap();
        names_and_values[1] = "Chrome";
        assert_eq!(headers.value(0), "OkHttp");
    }

    pub fn of_rejects_null_char() {
        let result = Headers::headers_of(&["User-Agent", "Square\u{0000}OkHttp"]);
        assert!(result.is_err());
    }

    pub fn of_map_throws_on_empty_name() {
        let mut map = HashMap::new();
        map.insert("".to_string(), "OkHttp".to_string());
        let result = map.to_headers();
        assert!(result.is_err());
    }

    pub fn of_map_throws_on_blank_name() {
        let mut map = HashMap::new();
        map.insert(" ".to_string(), "OkHttp".to_string());
        let result = map.to_headers();
        assert!(result.is_err());
    }

    pub fn of_map_accepts_empty_value() {
        let mut map = HashMap::new();
        map.insert("User-Agent".to_string(), "".to_string());
        let headers = map.to_headers().unwrap();
        assert_eq!(headers.value(0), "");
    }

    pub fn of_map_trims_key() {
        let mut map = HashMap::new();
        map.insert(" User-Agent ".to_string(), "OkHttp".to_string());
        let headers = map.to_headers().unwrap();
        assert_eq!(headers.name(0), "User-Agent");
    }

    pub fn of_map_trims_value() {
        let mut map = HashMap::new();
        map.insert("User-Agent".to_string(), " OkHttp ".to_string());
        let headers = map.to_headers().unwrap();
        assert_eq!(headers.value(0), "OkHttp");
    }

    pub fn of_map_makes_defensive_copy() {
        let mut names_and_values = HashMap::new();
        names_and_values.insert("User-Agent".to_string(), "OkHttp".to_string());
        let headers = names_and_values.to_headers().unwrap();
        names_and_values.insert("User-Agent".to_string(), "Chrome".to_string());
        assert_eq!(headers.value(0), "OkHttp");
    }

    pub fn of_map_rejects_null_char_in_name() {
        let mut map = HashMap::new();
        map.insert("User-\u{0000}Agent".to_string(), "OkHttp".to_string());
        let result = map.to_headers();
        assert!(result.is_err());
    }

    pub fn of_map_rejects_null_char_in_value() {
        let mut map = HashMap::new();
        map.insert("User-Agent".to_string(), "Square\u{0000}OkHttp".to_string());
        let result = map.to_headers();
        assert!(result.is_err());
    }

    pub fn builder_rejects_unicode_in_header_name() {
        let result = Headers::builder().add("héader1", "value1");
        let err = result.err().expect("Should fail");
        assert_eq!(err, "Unexpected char 0xe9 at 1 in header name: héader1");
    }

    pub fn builder_rejects_unicode_in_header_value() {
        let result = Headers::builder().add("header1", "valué1");
        let err = result.err().expect("Should fail");
        assert_eq!(err, "Unexpected char 0xe9 at 4 in header1 value: valué1");
    }

    pub fn vararg_factory_rejects_unicode_in_header_name() {
        let result = Headers::headers_of(&["héader1", "value1"]);
        let err = result.err().expect("Should fail");
        assert_eq!(err, "Unexpected char 0xe9 at 1 in header name: héader1");
    }

    pub fn vararg_factory_rejects_unicode_in_header_value() {
        let result = Headers::headers_of(&["header1", "valué1"]);
        let err = result.err().expect("Should fail");
        assert_eq!(err, "Unexpected char 0xe9 at 4 in header1 value: valué1");
    }

    pub fn map_factory_rejects_unicode_in_header_name() {
        let mut map = HashMap::new();
        map.insert("héader1".to_string(), "value1".to_string());
        let result = map.to_headers();
        let err = result.err().expect("Should fail");
        assert_eq!(err, "Unexpected char 0xe9 at 1 in header name: héader1");
    }

    pub fn map_factory_rejects_unicode_in_header_value() {
        let mut map = HashMap::new();
        map.insert("header1".to_string(), "valué1".to_string());
        let result = map.to_headers();
        let err = result.err().expect("Should fail");
        assert_eq!(err, "Unexpected char 0xe9 at 4 in header1 value: valué1");
    }

    pub fn sensitive_headers_not_included_in_exceptions() {
        let res1 = Headers::builder().add("Authorization", "valué1");
        assert_eq!(res1.err().unwrap(), "Unexpected char 0xe9 at 4 in Authorization value");

        let res2 = Headers::builder().add("Cookie", "valué1");
        assert_eq!(res2.err().unwrap(), "Unexpected char 0xe9 at 4 in Cookie value");

        let res3 = Headers::builder().add("Proxy-Authorization", "valué1");
        assert_eq!(res3.err().unwrap(), "Unexpected char 0xe9 at 4 in Proxy-Authorization value");

        let res4 = Headers::builder().add("Set-Cookie", "valué1");
        assert_eq!(res4.err().unwrap(), "Unexpected char 0xe9 at 4 in Set-Cookie value");
    }

    pub fn headers_equals() {
        let headers1 = Headers::builder()
            .add("Connection", "close").unwrap()
            .add("Transfer-Encoding", "chunked").unwrap()
            .build();
        let headers2 = Headers::builder()
            .add("Connection", "close").unwrap()
            .add("Transfer-Encoding", "chunked").unwrap()
            .build();
        assert_eq!(headers1, headers2);
    }

    pub fn headers_not_equals() {
        let headers1 = Headers::builder()
            .add("Connection", "close").unwrap()
            .add("Transfer-Encoding", "chunked").unwrap()
            .build();
        let headers2 = Headers::builder()
            .add("Connection", "keep-alive").unwrap()
            .add("Transfer-Encoding", "chunked").unwrap()
            .build();
        assert_ne!(headers1, headers2);
    }

    pub fn headers_to_string() {
        let headers = Headers::builder()
            .add("A", "a").unwrap()
            .add("B", "bb").unwrap()
            .build();
        assert_eq!(headers.to_string(), "A: a\nB: bb\n");
    }

    pub fn headers_to_string_redacts_sensitive_headers() {
        let headers = Headers::builder()
            .add("content-length", "99").unwrap()
            .add("authorization", "peanutbutter").unwrap()
            .add("proxy-authorization", "chocolate").unwrap()
            .add("cookie", "drink=coffee").unwrap()
            .add("set-cookie", "accessory=sugar").unwrap()
            .add("user-agent", "OkHttp").unwrap()
            .build();
        
        let expected = "content-length: 99\n\
                        authorization: ██\n\
                        proxy-authorization: ██\n\
                        cookie: ██\n\
                        set-cookie: ██\n\
                        user-agent: OkHttp\n";
        assert_eq!(headers.to_string(), expected);
    }

    pub fn headers_add_all() {
        let source_headers = Headers::builder()
            .add("A", "aa").unwrap()
            .add("a", "aa").unwrap()
            .add("B", "bb").unwrap()
            .build();
        let headers = Headers::builder()
            .add("A", "a").unwrap()
            .add_all(&source_headers)
            .add("C", "c").unwrap()
            .build();
        assert_eq!(headers.to_string(), "A: a\nA: aa\na: aa\nB: bb\nC: c\n");
    }

    pub fn name_indexes_are_strict() {
        let headers = Headers::headers_of(&["a", "b", "c", "d"]).unwrap();
        
        let result = std::panic::catch_unwind(|| {
            headers.name(usize::MAX); // Simulate -1
        });
        assert!(result.is_err());

        assert_eq!(headers.name(0), "a");
        assert_eq!(headers.name(1), "c");

        let result2 = std::panic::catch_unwind(|| {
            headers.name(2);
        });
        assert!(result2.is_err());
    }

    pub fn value_indexes_are_strict() {
        let headers = Headers::headers_of(&["a", "b", "c", "d"]).unwrap();
        
        let result = std::panic::catch_unwind(|| {
            headers.value(usize::MAX); // Simulate -1
        });
        assert!(result.is_err());

        assert_eq!(headers.value(0), "b");
        assert_eq!(headers.value(1), "d");

        let result2 = std::panic::catch_unwind(|| {
            headers.value(2);
        });
        assert!(result2.is_err());
    }
}