use std::collections::HashSet;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrl;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrlBuilder;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::UrlComponentEncodingTester::*;
use crate::android_test::build_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::idn::Punycode::*;

pub struct HttpUrlTest;

impl HttpUrlTest {
    fn parse(&self, url: &str) -> HttpUrl {
        url.to_http_url()
    }

    fn assert_invalid(&self, string: &str, exception_message: Option<&str>) {
        // In Rust, we assume to_http_url_result() returns Result<HttpUrl, Box<dyn std::error::Error>>
        // to simulate the Kotlin try-catch block for IllegalArgumentException.
        match string.to_http_url_result() {
            Ok(result) => {
                if let Some(msg) = exception_message {
                    panic!("Expected failure with {} but got {}", msg, result);
                } else {
                    panic!("Expected failure but got {}", result);
                }
            }
            Err(iae) => {
                if let Some(msg) = exception_message {
                    assert!(
                        iae.to_string().contains(msg),
                        "Exception message did not contain expected message: {}",
                        msg
                    );
                }
            }
        }
    }

    pub fn parse_trims_ascii_whitespace(&self) {
        let expected = self.parse("http://host/");
        // Leading.
        assert_eq!(self.parse("http://host/\u{000c}\n\t \r"), expected);
        // Trailing.
        assert_eq!(self.parse("\r\n\u{000c} \thttp://host/"), expected);
        // Both.
        assert_eq!(self.parse(" http://host/ "), expected);
        // Both.
        assert_eq!(self.parse("    http://host/    "), expected);
        assert_eq!(self.parse("http://host/").resolve("   "), expected);
        assert_eq!(self.parse("http://host/").resolve("  .  "), expected);
    }

    pub fn parse_host_ascii_non_printable(&self) {
        let host = "host\u{0001}";
        self.assert_invalid(&format!("http://{}/", host), Some("Invalid URL host: \"host\u{0001}\""));
    }

    pub fn parse_does_not_trim_other_whitespace_characters(&self) {
        assert_eq!(self.parse("http://h/\u{000b}").encoded_path(), "/%0B");
        assert_eq!(self.parse("http://h/\u{001c}").encoded_path(), "/%1C");
        assert_eq!(self.parse("http://h/\u{001d}").encoded_path(), "/%1D");
        assert_eq!(self.parse("http://h/\u{001e}").encoded_path(), "/%1E");
        assert_eq!(self.parse("http://h/\u{001f}").encoded_path(), "/%1F");
        assert_eq!(self.parse("http://h/\u{0085}").encoded_path(), "/%C2%85");
        assert_eq!(self.parse("http://h/\u{00a0}").encoded_path(), "/%C2%A0");
        assert_eq!(self.parse("http://h/\u{1680}").encoded_path(), "/%E1%9A%80");
        assert_eq!(self.parse("http://h/\u{180e}").encoded_path(), "/%E1%A0%8E");
        assert_eq!(self.parse("http://h/\u{2000}").encoded_path(), "/%E2%80%80");
        assert_eq!(self.parse("http://h/\u{2001}").encoded_path(), "/%E2%80%81");
        assert_eq!(self.parse("http://h/\u{2002}").encoded_path(), "/%E2%80%82");
        assert_eq!(self.parse("http://h/\u{2003}").encoded_path(), "/%E2%80%83");
        assert_eq!(self.parse("http://h/\u{2004}").encoded_path(), "/%E2%80%84");
        assert_eq!(self.parse("http://h/\u{2005}").encoded_path(), "/%E2%80%85");
        assert_eq!(self.parse("http://h/\u{2006}").encoded_path(), "/%E2%80%86");
        assert_eq!(self.parse("http://h/\u{2007}").encoded_path(), "/%E2%80%87");
        assert_eq!(self.parse("http://h/\u{2008}").encoded_path(), "/%E2%80%88");
        assert_eq!(self.parse("http://h/\u{2009}").encoded_path(), "/%E2%80%89");
        assert_eq!(self.parse("http://h/\u{200a}").encoded_path(), "/%E2%80%8A");
        assert_eq!(self.parse("http://h/\u{200b}").encoded_path(), "/%E2%80%8B");
        assert_eq!(self.parse("http://h/\u{200c}").encoded_path(), "/%E2%80%8C");
        assert_eq!(self.parse("http://h/\u{200d}").encoded_path(), "/%E2%80%8D");
        assert_eq!(self.parse("http://h/\u{200e}").encoded_path(), "/%E2%80%8E");
        assert_eq!(self.parse("http://h/\u{200f}").encoded_path(), "/%E2%80%8F");
        assert_eq!(self.parse("http://h/\u{2028}").encoded_path(), "/%E2%80%A8");
        assert_eq!(self.parse("http://h/\u{2029}").encoded_path(), "/%E2%80%A9");
        assert_eq!(self.parse("http://h/\u{202f}").encoded_path(), "/%E2%80%AF");
        assert_eq!(self.parse("http://h/\u{205f}").encoded_path(), "/%E2%81%9F");
        assert_eq!(self.parse("http://h/\u{3000}").encoded_path(), "/%E3%80%80");
    }

    pub fn new_builder_resolve(&self) {
        let base = self.parse("http://host/a/b");
        assert_eq!(base.new_builder("https://host2").unwrap().build(), self.parse("https://host2/"));
        assert_eq!(base.new_builder("//host2").unwrap().build(), self.parse("http://host2/"));
        assert_eq!(base.new_builder("/path").unwrap().build(), self.parse("http://host/path"));
        assert_eq!(base.new_builder("path").unwrap().build(), self.parse("http://host/a/path"));
        assert_eq!(base.new_builder("?query").unwrap().build(), self.parse("http://host/a/b?query"));
        assert_eq!(base.new_builder("#fragment").unwrap().build(), self.parse("http://host/a/b#fragment"));
        assert_eq!(base.new_builder("").unwrap().build(), self.parse("http://host/a/b"));
        assert!(base.new_builder("ftp://b").is_none());
        assert!(base.new_builder("ht+tp://b").is_none());
        assert!(base.new_builder("ht-tp://b").is_none());
        assert!(base.new_builder("ht.tp://b").is_none());
    }

    pub fn redacted_url(&self) {
        let base_with_password_and_username = self.parse("http://username:password@host/a/b#fragment");
        let base_with_username_only = self.parse("http://username@host/a/b#fragment");
        let base_with_password_only = self.parse("http://password@host/a/b#fragment");
        assert_eq!(base_with_password_and_username.redact(), "http://host/...");
        assert_eq!(base_with_username_only.redact(), "http://host/...");
        assert_eq!(base_with_password_only.redact(), "http://host/...");
    }

    pub fn resolve_no_scheme(&self) {
        let base = self.parse("http://host/a/b");
        assert_eq!(base.resolve("//host2"), self.parse("http://host2/"));
        assert_eq!(base.resolve("/path"), self.parse("http://host/path"));
        assert_eq!(base.resolve("path"), self.parse("http://host/a/path"));
        assert_eq!(base.resolve("?query"), self.parse("http://host/a/b?query"));
        assert_eq!(base.resolve("#fragment"), self.parse("http://host/a/b#fragment"));
        assert_eq!(base.resolve(""), self.parse("http://host/a/b"));
        assert_eq!(base.resolve("\\path"), self.parse("http://host/path"));
    }

    pub fn resolve_unsupported_scheme(&self) {
        let base = self.parse("http://a/");
        assert!(base.resolve("ftp://b").is_none());
        assert!(base.resolve("ht+tp://b").is_none());
        assert!(base.resolve("ht-tp://b").is_none());
        assert!(base.resolve("ht.tp://b").is_none());
    }

    pub fn resolve_scheme_like_path(&self) {
        let base = self.parse("http://a/");
        assert_eq!(base.resolve("http//b/"), self.parse("http://a/http//b/"));
        assert_eq!(base.resolve("ht+tp//b/"), self.parse("http://a/ht+tp//b/"));
        assert_eq!(base.resolve("ht-tp//b/"), self.parse("http://a/ht-tp//b/"));
        assert_eq!(base.resolve("ht.tp//b/"), self.parse("http://a/ht.tp//b/"));
    }

    pub fn rfc3886_normal_examples(&self) {
        let url = self.parse("http://a/b/c/d;p?q");
        assert!(url.resolve("g:h").is_none());
        assert_eq!(url.resolve("g"), self.parse("http://a/b/c/g"));
        assert_eq!(url.resolve("./g"), self.parse("http://a/b/c/g"));
        assert_eq!(url.resolve("g/"), self.parse("http://a/b/c/g/"));
        assert_eq!(url.resolve("/g"), self.parse("http://a/g"));
        assert_eq!(url.resolve("//g"), self.parse("http://g"));
        assert_eq!(url.resolve("?y"), self.parse("http://a/b/c/d;p?y"));
        assert_eq!(url.resolve("g?y"), self.parse("http://a/b/c/g?y"));
        assert_eq!(url.resolve("#s"), self.parse("http://a/b/c/d;p?q#s"));
        assert_eq!(url.resolve("g#s"), self.parse("http://a/b/c/g#s"));
        assert_eq!(url.resolve("g?y#s"), self.parse("http://a/b/c/g?y#s"));
        assert_eq!(url.resolve(";x"), self.parse("http://a/b/c/;x"));
        assert_eq!(url.resolve("g;x"), self.parse("http://a/b/c/g;x"));
        assert_eq!(url.resolve("g;x?y#s"), self.parse("http://a/b/c/g;x?y#s"));
        assert_eq!(url.resolve(""), self.parse("http://a/b/c/d;p?q"));
        assert_eq!(url.resolve("."), self.parse("http://a/b/c/"));
        assert_eq!(url.resolve("./"), self.parse("http://a/b/c/"));
        assert_eq!(url.resolve(".."), self.parse("http://a/b/"));
        assert_eq!(url.resolve("../"), self.parse("http://a/b/"));
        assert_eq!(url.resolve("../g"), self.parse("http://a/b/g"));
        assert_eq!(url.resolve("../.."), self.parse("http://a/"));
        assert_eq!(url.resolve("../../"), self.parse("http://a/"));
        assert_eq!(url.resolve("../../g"), self.parse("http://a/g"));
    }

    pub fn rfc3886_abnormal_examples(&self) {
        let url = self.parse("http://a/b/c/d;p?q");
        assert_eq!(url.resolve("../../../g"), self.parse("http://a/g"));
        assert_eq!(url.resolve("../../../../g"), self.parse("http://a/g"));
        assert_eq!(url.resolve("/./g"), self.parse("http://a/g"));
        assert_eq!(url.resolve("/../g"), self.parse("http://a/g"));
        assert_eq!(url.resolve("g."), self.parse("http://a/b/c/g."));
        assert_eq!(url.resolve(".g"), self.parse("http://a/b/c/.g"));
        assert_eq!(url.resolve("g.."), self.parse("http://a/b/c/g.."));
        assert_eq!(url.resolve("..g"), self.parse("http://a/b/c/..g"));
        assert_eq!(url.resolve("./../g"), self.parse("http://a/b/g"));
        assert_eq!(url.resolve("./g/."), self.parse("http://a/b/c/g/"));
        assert_eq!(url.resolve("g/./h"), self.parse("http://a/b/c/g/h"));
        assert_eq!(url.resolve("g/../h"), self.parse("http://a/b/c/h"));
        assert_eq!(url.resolve("g;x=1/./y"), self.parse("http://a/b/c/g;x=1/y"));
        assert_eq!(url.resolve("g;x=1/../y"), self.parse("http://a/b/c/y"));
        assert_eq!(url.resolve("g?y/./x"), self.parse("http://a/b/c/g?y/./x"));
        assert_eq!(url.resolve("g?y/../x"), self.parse("http://a/b/c/g?y/../x"));
        assert_eq!(url.resolve("g#s/./x"), self.parse("http://a/b/c/g#s/./x"));
        assert_eq!(url.resolve("g#s/../x"), self.parse("http://a/b/c/g#s/../x"));
        assert_eq!(url.resolve("http:g"), self.parse("http://a/b/c/g"));
    }

    pub fn parse_authority_slash_count_doesnt_matter(&self) {
        let expected = self.parse("http://host/path");
        assert_eq!(self.parse("http:host/path"), expected);
        assert_eq!(self.parse("http:/host/path"), expected);
        assert_eq!(self.parse("http:\\host/path"), expected);
        assert_eq!(self.parse("http://host/path"), expected);
        assert_eq!(self.parse("http:\\/host/path"), expected);
        assert_eq!(self.parse("http:/\\host/path"), expected);
        assert_eq!(self.parse("http:\\\\host/path"), expected);
        assert_eq!(self.parse("http:///host/path"), expected);
        assert_eq!(self.parse("http:\\//host/path"), expected);
        assert_eq!(self.parse("http:/\\/host/path"), expected);
        assert_eq!(self.parse("http://\\host/path"), expected);
        assert_eq!(self.parse("http:\\\\/host/path"), expected);
        assert_eq!(self.parse("http:/\\\\host/path"), expected);
        assert_eq!(self.parse("http:\\\\\\host/path"), expected);
        assert_eq!(self.parse("http:////host/path"), expected);
    }

    pub fn resolve_authority_slash_count_doesnt_matter_with_different_scheme(&self) {
        let base = self.parse("https://a/b/c");
        let expected = self.parse("http://host/path");
        assert_eq!(base.resolve("http:host/path"), expected);
        assert_eq!(base.resolve("http:/host/path"), expected);
        assert_eq!(base.resolve("http:\\host/path"), expected);
        assert_eq!(base.resolve("http://host/path"), expected);
        assert_eq!(base.resolve("http:\\/host/path"), expected);
        assert_eq!(base.resolve("http:/\\host/path"), expected);
        assert_eq!(base.resolve("http:\\\\host/path"), expected);
        assert_eq!(base.resolve("http:///host/path"), expected);
        assert_eq!(base.resolve("http:\\//host/path"), expected);
        assert_eq!(base.resolve("http:/\\/host/path"), expected);
        assert_eq!(base.resolve("http://\\host/path"), expected);
        assert_eq!(base.resolve("http:\\\\/host/path"), expected);
        assert_eq!(base.resolve("http:/\\\\host/path"), expected);
        assert_eq!(base.resolve("http:\\\\\\host/path"), expected);
        assert_eq!(base.resolve("http:////host/path"), expected);
    }

    pub fn resolve_authority_slash_count_matters_with_same_scheme(&self) {
        let base = self.parse("http://a/b/c");
        assert_eq!(base.resolve("http:host/path"), self.parse("http://a/b/host/path"));
        assert_eq!(base.resolve("http:/host/path"), self.parse("http://a/host/path"));
        assert_eq!(base.resolve("http:\\host/path"), self.parse("http://a/host/path"));
        assert_eq!(base.resolve("http://host/path"), self.parse("http://host/path"));
        assert_eq!(base.resolve("http:\\/host/path"), self.parse("http://host/path"));
        assert_eq!(base.resolve("http:/\\host/path"), self.parse("http://host/path"));
        assert_eq!(base.resolve("http:\\\\host/path"), self.parse("http://host/path"));
        assert_eq!(base.resolve("http:///host/path"), self.parse("http://host/path"));
        assert_eq!(base.resolve("http:\\//host/path"), self.parse("http://host/path"));
        assert_eq!(base.resolve("http:/\\/host/path"), self.parse("http://host/path"));
        assert_eq!(base.resolve("http://\\host/path"), self.parse("http://host/path"));
        assert_eq!(base.resolve("http:\\\\/host/path"), self.parse("http://host/path"));
        assert_eq!(base.resolve("http:/\\\\host/path"), self.parse("http://host/path"));
        assert_eq!(base.resolve("http:\\\\\\host/path"), self.parse("http://host/path"));
        assert_eq!(base.resolve("http:////host/path"), self.parse("http://host/path"));
    }

    pub fn username(&self) {
        assert_eq!(self.parse("http://@host/path"), self.parse("http://host/path"));
        assert_eq!(self.parse("http://user@host/path"), self.parse("http://user@host/path"));
    }

    pub fn authority_with_multiple_at_signs(&self) {
        let http_url = self.parse("http://foo@bar@baz/path");
        assert_eq!(http_url.username(), "foo@bar");
        assert_eq!(http_url.password(), "");
        assert_eq!(http_url, self.parse("http://foo%40bar@baz/path"));
    }

    pub fn authority_with_multiple_colons(&self) {
        let http_url = self.parse("http://foo:pass1@bar:pass2@baz/path");
        assert_eq!(http_url.username(), "foo");
        assert_eq!(http_url.password(), "pass1@bar:pass2");
        assert_eq!(http_url, self.parse("http://foo:pass1%40bar%3Apass2@baz/path"));
    }

    pub fn username_and_password(&self) {
        assert_eq!(self.parse("http://username:password@host/path"), self.parse("http://username:password@host/path"));
        assert_eq!(self.parse("http://username:@host/path"), self.parse("http://username@host/path"));
    }

    pub fn password_with_empty_username(&self) {
        assert_eq!(self.parse("http://:@host/path"), self.parse("http://host/path"));
        assert_eq!(self.parse("http://:password@@host/path").encoded_password(), "password%40");
    }

    pub fn unprintable_characters_are_percent_encoded(&self) {
        assert_eq!(self.parse("http://host/\u{0000}").encoded_path(), "/%00");
        assert_eq!(self.parse("http://host/\u{0008}").encoded_path(), "/%08");
        assert_eq!(self.parse("http://host/\u{fffd}").encoded_path(), "/%EF%BF%BD");
    }

    pub fn username_characters(&self) {
        UrlComponentEncodingTester::new_instance()
            .override_encodings(Encoding::Percent, &['[' as i32, ']' as i32, '{' as i32, '}' as i32, '|' as i32, '^' as i32, '\'' as i32, ';' as i32, '=' as i32, '@' as i32])
            .override_encodings(Encoding::Skip, &[':' as i32, '/' as i32, '\\' as i32, '?' as i32, '#' as i32])
            .test(Component::User);
    }

    pub fn password_characters(&self) {
        UrlComponentEncodingTester::new_instance()
            .override_encodings(Encoding::Percent, &['[' as i32, ']' as i32, '{' as i32, '}' as i32, '|' as i32, '^' as i32, '\'' as i32, ':' as i32, ';' as i32, '=' as i32, '@' as i32])
            .override_encodings(Encoding::Skip, &['/' as i32, '\\' as i32, '?' as i32, '#' as i32])
            .test(Component::Password);
    }

    pub fn host_contains_illegal_character(&self) {
        self.assert_invalid("http://\n/", Some("Invalid URL host: \"\n\""));
        self.assert_invalid("http:// /", Some("Invalid URL host: \" \""));
        self.assert_invalid("http://%20/", Some("Invalid URL host: \"%20\""));
    }

    pub fn hostname_lowercase_characters_mapped_directly(&self) {
        assert_eq!(self.parse("http://abcd").host(), "abcd");
        assert_eq!(self.parse("http://σ").host(), "xn--4xa");
    }

    pub fn hostname_uppercase_characters_converted_to_lowercase(&self) {
        assert_eq!(self.parse("http://ABCD").host(), "abcd");
        assert_eq!(self.parse("http://Σ").host(), "xn--4xa");
    }

    pub fn hostname_ignored_characters(&self) {
        assert_eq!(self.parse("http://AB\u{00ad}CD").host(), "abcd");
    }

    pub fn hostname_multiple_character_mapping(&self) {
        assert_eq!(self.parse("http://\u{2121}").host(), "tel");
    }

    pub fn hostname_mapping_last_mapped_code_point(&self) {
        assert_eq!(self.parse("http://\u{D87E}\u{DE1D}").host(), "xn--pu5l");
    }

    pub fn hostname_mapping_last_disallowed_code_point(&self) {
        self.assert_invalid("http://\u{DBFF}\u{DFFF}", Some("Invalid URL host: \"\u{DBFF}\u{DFFF}\""));
    }

    pub fn hostname_uri(&self) {
        UrlComponentEncodingTester::new_instance()
            .non_printable_ascii(Encoding::Forbidden)
            .non_ascii(Encoding::Punycode)
            .override_encodings(Encoding::Forbidden, &['\t' as i32, '\n' as i32, '\u{000c}' as i32, '\r' as i32, ' ' as i32])
            .override_encodings(Encoding::Forbidden, &['#' as i32, '%' as i32, '/' as i32, ':' as i32, '?' as i32, '@' as i32, '[' as i32, '\\' as i32, ']' as i32])
            .override_encodings(Encoding::Skip, &['\"' as i32, '<' as i32, '>' as i32, '^' as i32, '`' as i32, '{' as i32, '|' as i32, '}' as i32])
            .test(Component::Host);
    }

    pub fn host_ipv6(&self) {
        assert_eq!(self.parse("http://[::1]/").host(), "::1");
        assert_eq!(self.parse("http://[::1]/").to_string(), "http://[::1]/");
        assert_eq!(self.parse("http://[::1]:8080/").port(), 8080);
        assert_eq!(self.parse("http://user:password@[::1]/").password(), "password");
        assert_eq!(self.parse("http://user:password@[::1]:8080/").host(), "::1");
        assert_eq!(self.parse("http://[%3A%3A%31]/").host(), "::1");
        assert_eq!(self.parse("http://%5B%3A%3A1%5D/").host(), "::1");
    }

    pub fn host_ipv6_address_different_formats(&self) {
        let a3 = "2001:db8::1:0:0:1";
        assert_eq!(self.parse("http://[2001:db8:0:0:1:0:0:1]").host(), a3);
        assert_eq!(self.parse("http://[2001:0db8:0:0:1:0:0:1]").host(), a3);
        assert_eq!(self.parse("http://[2001:db8::1:0:0:1]").host(), a3);
        assert_eq!(self.parse("http://[2001:db8::0:1:0:0:1]").host(), a3);
        assert_eq!(self.parse("http://[2001:0db8::1:0:0:1]").host(), a3);
        assert_eq!(self.parse("http://[2001:db8:0:0:1::1]").host(), a3);
        assert_eq!(self.parse("http://[2001:db8:0000:0:1::1]").host(), a3);
        assert_eq!(self.parse("http://[2001:DB8:0:0:1::1]").host(), a3);
    }

    pub fn host_ipv6_address_leading_compression(&self) {
        assert_eq!(self.parse("http://[::0001]").host(), "::1");
        assert_eq!(self.parse("http://[0000::0001]").host(), "::1");
        assert_eq!(self.parse("http://[0000:0000:0000:0000:0000:0000:0000:0001]").host(), "::1");
        assert_eq!(self.parse("http://[0000:0000:0000:0000:0000:0000::0001]").host(), "::1");
    }

    pub fn host_ipv6_address_trailing_compression(&self) {
        assert_eq!(self.parse("http://[0001:0000::]").host(), "1::");
        assert_eq!(self.parse("http://[0001::0000]").host(), "1::");
        assert_eq!(self.parse("http://[0001::]").host(), "1::");
        assert_eq!(self.parse("http://[1::]").host(), "1::");
    }

    pub fn host_ipv6_address_too_many_digits_in_group(&self) {
        self.assert_invalid("http://[00000:0000:0000:0000:0000:0000:0000:0001]", Some("Invalid URL host: \"[00000:0000:0000:0000:0000:0000:0000:0001]\""));
        self.assert_invalid("http://[::00001]", Some("Invalid URL host: \"[::00001]\""));
    }

    pub fn host_ipv6_address_misplaced_colons(&self) {
        self.assert_invalid("http://[:0000:0000:0000:0000:0000:0000:0000:0001]", Some("Invalid URL host: \"[:0000:0000:0000:0000:0000:0000:0000:0001]\""));
        self.assert_invalid("http://[:::0000:0000:0000:0000:0000:0000:0000:0001]", Some("Invalid URL host: \"[:::0000:0000:0000:0000:0000:0000:0000:0001]\""));
        self.assert_invalid("http://[:1]", Some("Invalid URL host: \"[:1]\""));
        self.assert_invalid("http://[:::1]", Some("Invalid URL host: \"[:::1]\""));
        self.assert_invalid("http://[0000:0000:0000:0000:0000:0000:0001:]", Some("Invalid URL host: \"[0000:0000:0000:0000:0000:0000:0001:]\""));
        self.assert_invalid("http://[0000:0000:0000:0000:0000:0000:0000:0001:]", Some("Invalid URL host: \"[0000:0000:0000:0000:0000:0000:0000:0001:]\""));
        self.assert_invalid("http://[0000:0000:0000:0000:0000:0000:0000:0001::]", Some("Invalid URL host: \"[0000:0000:0000:0000:0000:0000:0000:0001::]\""));
        self.assert_invalid("http://[0000:0000:0000:0000:0000:0000:0000:00
)}}
