use std::ops::{Index, IndexMut};
use chrono::{DateTime, Utc, TimeZone};

// The translation warnings indicated that Headers was replaced with an import.
// However, the target.rs file contains a local implementation of Headers and HeadersBuilder
// which are actually the subjects of the test. Since this is a test file, 
// we should ensure the types are correctly defined or imported.
// Given the context of the translation, we will keep the logic but clean up the 
// incorrect imports and the redundant/broken Index implementations.


impl Headers {
    pub fn new(pairs: Vec<(String, String)>) -> Self {
        Headers { pairs }
    }

    pub fn get(&self, name: &str) -> Option<&String> {
        self.pairs
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
    }

    pub fn headers_of(args: &[&str]) -> Self {
        let mut pairs = Vec::new();
        for i in (0..args.len()).step_by(2) {
            if i + 1 < args.len() {
                pairs.push((args[i].to_string(), args[i + 1].to_string()));
            }
        }
        Headers::new(pairs)
    }
}

impl Index<&str> for Headers {
    type Output = String;

    fn index(&self, index: &str) -> &Self::Output {
        self.get(index).expect("Index out of bounds or key not found")
    }
}

impl IntoIterator for Headers {
    type Item = (String, String);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.pairs.into_iter()
    }
}

pub struct HeadersBuilder {
    pairs: Vec<(String, String)>,
}

impl HeadersBuilder {
    pub fn new() -> Self {
        HeadersBuilder { pairs: Vec::new() }
    }

    pub fn add(&mut self, name: &str, value: &str) {
        self.pairs.push((name.to_string(), value.to_string()));
    }

    pub fn get(&self, name: &str) -> Option<&String> {
        self.pairs
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
    }

    pub fn set<T: ToString>(&mut self, name: &str, value: T) {
        let val_str = value.to_string();
        if let Some(pos) = self.pairs.iter().position(|(n, _)| n == name) {
            self.pairs[pos].1 = val_str;
        } else {
            self.pairs.push((name.to_string(), val_str));
        }
    }

    pub fn build(self) -> Headers {
        Headers::new(self.pairs)
    }
}

impl Index<&str> for HeadersBuilder {
    type Output = String;
    fn index(&self, index: &str) -> &Self::Output {
        self.get(index).expect("Key not found")
    }
}

impl IndexMut<&str> for HeadersBuilder {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        if let Some(pos) = self.pairs.iter().position(|(n, _)| n == index) {
            &mut self.pairs[pos].1
        } else {
            panic!("Key not found for IndexMut; use set() or add()");
        }
    }
}

pub trait ToHttpDate {
    fn to_http_date(&self) -> String;
}

impl ToHttpDate for i64 {
    fn to_http_date(&self) -> String {
        let dt = Utc.timestamp_opt(*self, 0).unwrap();
        dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
    }
}

impl ToHttpDate for DateTime<Utc> {
    fn to_http_date(&self) -> String {
        self.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
    }
}

pub struct Date(pub i64);
impl ToString for Date {
    fn to_string(&self) -> String {
        self.0.to_http_date()
    }
}

pub struct Instant(pub DateTime<Utc>);
impl ToString for Instant {
    fn to_string(&self) -> String {
        self.0.to_http_date()
    }
}

pub struct HeadersKotlinTest;

impl HeadersKotlinTest {
    pub fn get_operator() {
        let headers = Headers::headers_of(&["a", "b", "c", "d"]);
        assert_eq!(headers.get("a"), Some(&"b".to_string()));
        assert_eq!(headers.get("c"), Some(&"d".to_string()));
        assert_eq!(headers.get("e"), None);
    }

    pub fn iterator_operator() {
        let headers = Headers::headers_of(&["a", "b", "c", "d"]);

        let mut pairs = Vec::new();
        for (name, value) in headers {
            pairs.push((name, value));
        }

        assert_eq!(pairs, vec![("a".to_string(), "b".to_string()), ("c".to_string(), "d".to_string())]);
    }

    pub fn builder_get_operator() {
        let mut builder = HeadersBuilder::new();
        builder.add("a", "b");
        builder.add("c", "d");
        assert_eq!(builder.get("a"), Some(&"b".to_string()));
        assert_eq!(builder.get("c"), Some(&"d".to_string()));
        assert_eq!(builder.get("e"), None);
    }

    pub fn builder_set_operator() {
        let mut builder = HeadersBuilder::new();
        builder.set("a", "b");
        builder.set("c", "d");
        
        builder.set("e", Date(0));
        builder.set("g", Instant(Utc.timestamp_opt(0, 0).unwrap()));

        assert_eq!(builder.get("a"), Some(&"b".to_string()));
        assert_eq!(builder.get("c"), Some(&"d".to_string()));
        assert_eq!(builder.get("e"), Some(&"Thu, 01 Jan 1970 00:00:00 GMT".to_string()));
        assert_eq!(builder.get("g"), Some(&"Thu, 01 Jan 1970 00:00:00 GMT".to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;

    #[test]
    fn test_get_operator() {
        HeadersKotlinTest::get_operator();
    }

    #[test]
    fn test_iterator_operator() {
        HeadersKotlinTest::iterator_operator();
    }

    #[test]
    fn test_builder_get_operator() {
        HeadersKotlinTest::builder_get_operator();
    }

    #[test]
    fn test_builder_set_operator() {
        HeadersKotlinTest::builder_set_operator();
    }
}
