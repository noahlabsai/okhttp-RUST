use std::sync::{Arc, Mutex};
use okio::Buffer;

// Import the required types from the specified path
// Based on the provided recovery instructions:
// PublicSuffixDatabase: import from okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::{
    ConfiguredPublicSuffixList, PublicSuffixDatabase,
};

#[derive(Debug, Clone)]
pub struct ConfiguredPublicSuffixDatabaseTest {
    list: Arc<Mutex<ConfiguredPublicSuffixList>>,
    public_suffix_database: PublicSuffixDatabase,
}

impl ConfiguredPublicSuffixDatabaseTest {
    pub fn new() -> Self {
        let list = Arc::new(Mutex::new(ConfiguredPublicSuffixList::default()));
        // PublicSuffixDatabase takes the list as a dependency
        let public_suffix_database = PublicSuffixDatabase::new(list.clone());
        
        Self {
            list,
            public_suffix_database,
        }
    }

    pub fn longest_match_wins(&self) {
        {
            let mut list = self.list.lock().unwrap();
            let mut buffer = Buffer::new();
            buffer.write_utf8("com\n");
            buffer.write_utf8("my.square.com\n");
            buffer.write_utf8("square.com\n");
            list.bytes = buffer.read_byte_string();
        }

        assert_eq!(
            self.public_suffix_database.get_effective_tld_plus_one("example.com"),
            Some("example.com".to_string())
        );
        assert_eq!(
            self.public_suffix_database.get_effective_tld_plus_one("foo.example.com"),
            Some("example.com".to_string())
        );
        assert_eq!(
            self.public_suffix_database.get_effective_tld_plus_one("foo.bar.square.com"),
            Some("bar.square.com".to_string())
        );
        assert_eq!(
            self.public_suffix_database.get_effective_tld_plus_one("foo.my.square.com"),
            Some("foo.my.square.com".to_string())
        );
    }

    pub fn wildcard_match(&self) {
        {
            let mut list = self.list.lock().unwrap();
            let mut buffer = Buffer::new();
            buffer.write_utf8("*.square.com\n");
            buffer.write_utf8("com\n");
            buffer.write_utf8("example.com\n");
            list.bytes = buffer.read_byte_string();
        }

        assert_eq!(
            self.public_suffix_database.get_effective_tld_plus_one("my.square.com"),
            None
        );
        assert_eq!(
            self.public_suffix_database.get_effective_tld_plus_one("foo.my.square.com"),
            Some("foo.my.square.com".to_string())
        );
        assert_eq!(
            self.public_suffix_database.get_effective_tld_plus_one("bar.foo.my.square.com"),
            Some("foo.my.square.com".to_string())
        );
    }

    pub fn boundary_searches(&self) {
        {
            let mut list = self.list.lock().unwrap();
            let mut buffer = Buffer::new();
            buffer.write_utf8("bbb\n");
            buffer.write_utf8("ddd\n");
            buffer.write_utf8("fff\n");
            list.bytes = buffer.read_byte_string();
        }

        assert_eq!(
            self.public_suffix_database.get_effective_tld_plus_one("aaa"),
            None
        );
        assert_eq!(
            self.public_suffix_database.get_effective_tld_plus_one("ggg"),
            None
        );
        assert_eq!(
            self.public_suffix_database.get_effective_tld_plus_one("ccc"),
            None
        );
        assert_eq!(
            self.public_suffix_database.get_effective_tld_plus_one("eee"),
            None
        );
    }

    pub fn exception_rule(&self) {
        {
            let mut list = self.list.lock().unwrap();
            let mut buffer = Buffer::new();
            buffer.write_utf8("*.jp\n");
            buffer.write_utf8("*.square.jp\n");
            buffer.write_utf8("example.com\n");
            buffer.write_utf8("square.com\n");
            list.bytes = buffer.read_byte_string();

            let mut ex_buffer = Buffer::new();
            ex_buffer.write_utf8("my.square.jp\n");
            list.exception_bytes = ex_buffer.read_byte_string();
        }

        assert_eq!(
            self.public_suffix_database.get_effective_tld_plus_one("my.square.jp"),
            Some("my.square.jp".to_string())
        );
        assert_eq!(
            self.public_suffix_database.get_effective_tld_plus_one("foo.my.square.jp"),
            Some("my.square.jp".to_string())
        );
        assert_eq!(
            self.public_suffix_database.get_effective_tld_plus_one("my1.square.jp"),
            None
        );
    }

    pub fn no_effective_tld_plus_one(&self) {
        {
            let mut list = self.list.lock().unwrap();
            let mut buffer = Buffer::new();
            buffer.write_utf8("*.jp\n");
            buffer.write_utf8("*.square.jp\n");
            buffer.write_utf8("example.com\n");
            buffer.write_utf8("square.com\n");
            list.bytes = buffer.read_byte_string();

            let mut ex_buffer = Buffer::new();
            ex_buffer.write_utf8("my.square.jp\n");
            list.exception_bytes = ex_buffer.read_byte_string();
        }

        assert_eq!(
            self.public_suffix_database.get_effective_tld_plus_one("example.com"),
            None
        );
        assert_eq!(
            self.public_suffix_database.get_effective_tld_plus_one("foo.square.jp"),
            None
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

    #[test]
    fn test_longest_match_wins() {
        let test = ConfiguredPublicSuffixDatabaseTest::new();
        test.longest_match_wins();
    }

    #[test]
    fn test_wildcard_match() {
        let test = ConfiguredPublicSuffixDatabaseTest::new();
        test.wildcard_match();
    }

    #[test]
    fn test_boundary_searches() {
        let test = ConfiguredPublicSuffixDatabaseTest::new();
        test.boundary_searches();
    }

    #[test]
    fn test_exception_rule() {
        let test = ConfiguredPublicSuffixDatabaseTest::new();
        test.exception_rule();
    }

    #[test]
    fn test_no_effective_tld_plus_one() {
        let test = ConfiguredPublicSuffixDatabaseTest::new();
        test.no_effective_tld_plus_one();
    }
}