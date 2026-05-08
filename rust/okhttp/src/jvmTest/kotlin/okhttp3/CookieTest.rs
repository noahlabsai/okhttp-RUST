use std::collections::HashMap;
use chrono::{DateTime, Utc, TimeZone, NaiveDateTime};
use crate::okhttp3::{Cookie, HttpUrl, Headers};
use crate::okhttp3::internal::http::MAX_DATE;
use crate::okhttp3::internal::parse_cookie;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http::DateFormatting::*;

// Mocking the "to_http_url" extension function for the test environment
trait HttpUrlExt {
    fn to_http_url(self) -> HttpUrl;
}

impl HttpUrlExt for &str {
    fn to_http_url(self) -> HttpUrl {
        HttpUrl::parse(self).expect("Invalid URL in test")
    }
}

pub struct CookieTest {
    url: HttpUrl,
}

impl CookieTest {
    pub fn new() -> Self {
        Self {
            url: "https://example.com/".to_http_url(),
        }
    }

    pub fn simple_cookie(&self) {
        let cookie = Cookie::parse(&self.url, "SID=31d4d96e407aad42");
        assert_eq!(cookie.unwrap().to_string(), "SID=31d4d96e407aad42; path=/");
    }

    pub fn no_equals_sign(&self) {
        assert!(Cookie::parse(&self.url, "foo").is_none());
        assert!(Cookie::parse(&self.url, "foo; Path=/").is_none());
    }

    pub fn empty_name(&self) {
        assert!(Cookie::parse(&self.url, "=b").is_none());
        assert!(Cookie::parse(&self.url, " =b").is_none());
        assert!(Cookie::parse(&self.url, "\r\t \n=b").is_none());
    }

    pub fn space_in_name(&self) {
        let cookie = Cookie::parse(&self.url, "a b=cd").expect("Cookie should be parsed");
        assert_eq!(cookie.name, "a b");
    }

    pub fn space_in_value(&self) {
        let cookie = Cookie::parse(&self.url, "ab=c d").expect("Cookie should be parsed");
        assert_eq!(cookie.value, "c d");
    }

    pub fn trim_leading_and_trailing_whitespace_from_name(&self) {
        assert_eq!(Cookie::parse(&self.url, " a=b").unwrap().name, "a");
        assert_eq!(Cookie::parse(&self.url, "a =b").unwrap().name, "a");
        assert_eq!(Cookie::parse(&self.url, "\r\t \na\n\t \n=b").unwrap().name, "a");
    }

    pub fn empty_value(&self) {
        assert_eq!(Cookie::parse(&self.url, "a=").unwrap().value, "");
        assert_eq!(Cookie::parse(&self.url, "a= ").unwrap().value, "");
        assert_eq!(Cookie::parse(&self.url, "a=\r\t \n").unwrap().value, "");
    }

    pub fn trim_leading_and_trailing_whitespace_from_value(&self) {
        assert_eq!(Cookie::parse(&self.url, "a= ").unwrap().value, "");
        assert_eq!(Cookie::parse(&self.url, "a= b").unwrap().value, "b");
        assert_eq!(Cookie::parse(&self.url, "a=b ").unwrap().value, "b");
        assert_eq!(Cookie::parse(&self.url, "a=\r\t \nb\n\t \n").unwrap().value, "b");
    }

    pub fn invalid_characters(&self) {
        assert!(Cookie::parse(&self.url, "a\u{0000}b=cd").is_none());
        assert!(Cookie::parse(&self.url, "ab=c\u{0000}d").is_none());
        assert!(Cookie::parse(&self.url, "a\u{0001}b=cd").is_none());
        assert!(Cookie::parse(&self.url, "ab=c\u{0001}d").is_none());
        assert!(Cookie::parse(&self.url, "a\u{0009}b=cd").is_none());
        assert!(Cookie::parse(&self.url, "ab=c\u{0009}d").is_none());
        assert!(Cookie::parse(&self.url, "a\u{001f}b=cd").is_none());
        assert!(Cookie::parse(&self.url, "ab=c\u{001f}d").is_none());
        assert!(Cookie::parse(&self.url, "a\u{007f}b=cd").is_none());
        assert!(Cookie::parse(&self.url, "ab=c\u{007f}d").is_none());
        assert!(Cookie::parse(&self.url, "a\u{0080}b=cd").is_none());
        assert!(Cookie::parse(&self.url, "ab=c\u{0080}d").is_none());
        assert!(Cookie::parse(&self.url, "a\u{00ff}b=cd").is_none());
        assert!(Cookie::parse(&self.url, "ab=c\u{00ff}d").is_none());
    }

    pub fn max_age(&self) {
        assert_eq!(parse_cookie(50000, &self.url, "a=b; Max-Age=1").unwrap().expires_at, 51000);
        assert_eq!(parse_cookie(50000, &self.url, "a=b; Max-Age=9223372036854724").unwrap().expires_at, MAX_DATE);
        assert_eq!(parse_cookie(50000, &self.url, "a=b; Max-Age=9223372036854725").unwrap().expires_at, MAX_DATE);
        assert_eq!(parse_cookie(50000, &self.url, "a=b; Max-Age=9223372036854726").unwrap().expires_at, MAX_DATE);
        assert_eq!(parse_cookie(9223372036854773807, &self.url, "a=b; Max-Age=1").unwrap().expires_at, MAX_DATE);
        assert_eq!(parse_cookie(9223372036854773807, &self.url, "a=b; Max-Age=2").unwrap().expires_at, MAX_DATE);
        assert_eq!(parse_cookie(9223372036854773807, &self.url, "a=b; Max-Age=3").unwrap().expires_at, MAX_DATE);
        assert_eq!(parse_cookie(50000, &self.url, "a=b; Max-Age=10000000000000000000").unwrap().expires_at, MAX_DATE);
    }

    pub fn max_age_non_positive(&self) {
        assert_eq!(parse_cookie(50000, &self.url, "a=b; Max-Age=-1").unwrap().expires_at, i64::MIN);
        assert_eq!(parse_cookie(50000, &self.url, "a=b; Max-Age=0").unwrap().expires_at, i64::MIN);
        assert_eq!(parse_cookie(50000, &self.url, "a=b; Max-Age=-9223372036854775808").unwrap().expires_at, i64::MIN);
        assert_eq!(parse_cookie(50000, &self.url, "a=b; Max-Age=-9223372036854775809").unwrap().expires_at, i64::MIN);
        assert_eq!(parse_cookie(50000, &self.url, "a=b; Max-Age=-10000000000000000000").unwrap().expires_at, i64::MIN);
    }

    pub fn domain_and_path(&self) {
        let cookie = Cookie::parse(&self.url, "SID=31d4d96e407aad42; Path=/; Domain=example.com").expect("Parsed");
        assert_eq!(cookie.domain, Some("example.com".to_string()));
        assert_eq!(cookie.path, "/");
        assert!(!cookie.host_only);
        assert_eq!(cookie.to_string(), "SID=31d4d96e407aad42; domain=example.com; path=/");
    }

    pub fn secure_and_http_only(&self) {
        let cookie = Cookie::parse(&self.url, "SID=31d4d96e407aad42; Path=/; Secure; HttpOnly").expect("Parsed");
        assert!(cookie.secure);
        assert!(cookie.http_only);
        assert_eq!(cookie.to_string(), "SID=31d4d96e407aad42; path=/; secure; httponly");
    }

    pub fn expires_date(&self) {
        assert_eq!(Cookie::parse(&self.url, "a=b; Expires=Thu, 01 Jan 1970 00:00:00 GMT").unwrap().expires_at, self.date("1970-01-01T00:00:00.000+0000"));
        assert_eq!(Cookie::parse(&self.url, "a=b; Expires=Wed, 09 Jun 2021 10:18:14 GMT").unwrap().expires_at, self.date("2021-06-09T10:18:14.000+0000"));
        assert_eq!(Cookie::parse(&self.url, "a=b; Expires=Sun, 06 Nov 1994 08:49:37 GMT").unwrap().expires_at, self.date("1994-11-06T08:49:37.000+0000"));
    }

    pub fn awkward_dates(&self) {
        let dates = [
            "a=b; Expires=Thu, 01 Jan 70 00:00:00 GMT",
            "a=b; Expires=Thu, 01 January 1970 00:00:00 GMT",
            "a=b; Expires=Thu, 01 Janucember 1970 00:00:00 GMT",
            "a=b; Expires=Thu, 1 Jan 1970 00:00:00 GMT",
            "a=b; Expires=Thu, 01 Jan 1970 0:00:00 GMT",
            "a=b; Expires=Thu, 01 Jan 1970 00:0:00 GMT",
            "a=b; Expires=Thu, 01 Jan 1970 00:00:0 GMT",
            "a=b; Expires=00:00:00 Thu, 01 Jan 1970 GMT",
            "a=b; Expires=00:00:00 1970 Jan 01",
            "a=b; Expires=00:00:00 1970 Jan 1",
        ];
        for d in dates {
            assert_eq!(Cookie::parse(&self.url, d).unwrap().expires_at, 0);
        }
    }

    pub fn invalid_year(&self) {
        assert_eq!(Cookie::parse(&self.url, "a=b; Expires=Thu, 01 Jan 1600 00:00:00 GMT").unwrap().expires_at, MAX_DATE);
        assert_eq!(Cookie::parse(&self.url, "a=b; Expires=Thu, 01 Jan 19999 00:00:00 GMT").unwrap().expires_at, MAX_DATE);
        assert_eq!(Cookie::parse(&self.url, "a=b; Expires=Thu, 01 Jan 00:00:00 GMT").unwrap().expires_at, MAX_DATE);
    }

    pub fn invalid_month(&self) {
        assert_eq!(Cookie::parse(&self.url, "a=b; Expires=Thu, 01 Foo 1970 00:00:00 GMT").unwrap().expires_at, MAX_DATE);
        assert_eq!(Cookie::parse(&self.url, "a=b; Expires=Thu, 01 Foocember 1970 00:00:00 GMT").unwrap().expires_at, MAX_DATE);
        assert_eq!(Cookie::parse(&self.url, "a=b; Expires=Thu, 01 1970 00:00:00 GMT").unwrap().expires_at, MAX_DATE);
    }

    pub fn invalid_day_of_month(&self) {
        assert_eq!(Cookie::parse(&self.url, "a=b; Expires=Thu, 32 Jan 1970 00:00:00 GMT").unwrap().expires_at, MAX_DATE);
        assert_eq!(Cookie::parse(&self.url, "a=b; Expires=Thu, Jan 1970 00:00:00 GMT").unwrap().expires_at, MAX_DATE);
    }

    pub fn invalid_hour(&self) {
        assert_eq!(Cookie::parse(&self.url, "a=b; Expires=Thu, 01 Jan 1970 24:00:00 GMT").unwrap().expires_at, MAX_DATE);
    }

    pub fn invalid_minute(&self) {
        assert_eq!(Cookie::parse(&self.url, "a=b; Expires=Thu, 01 Jan 1970 00:60:00 GMT").unwrap().expires_at, MAX_DATE);
    }

    pub fn invalid_second(&self) {
        assert_eq!(Cookie::parse(&self.url, "a=b; Expires=Thu, 01 Jan 1970 00:00:60 GMT").unwrap().expires_at, MAX_DATE);
    }

    pub fn domain_matches(&self) {
        let cookie = Cookie::parse(&self.url, "a=b; domain=example.com").expect("Parsed");
        assert!(cookie.matches(&"http://example.com".to_http_url()));
        assert!(cookie.matches(&"http://www.example.com".to_http_url()));
        assert!(!cookie.matches(&"http://square.com".to_http_url()));
    }

    pub fn domain_matches_no_domain(&self) {
        let cookie = Cookie::parse(&self.url, "a=b").expect("Parsed");
        assert!(cookie.matches(&"http://example.com".to_http_url()));
        assert!(!cookie.matches(&"http://www.example.com".to_http_url()));
        assert!(!cookie.matches(&"http://square.com".to_http_url()));
    }

    pub fn domain_matches_ignores_leading_dot(&self) {
        let cookie = Cookie::parse(&self.url, "a=b; domain=.example.com").expect("Parsed");
        assert!(cookie.matches(&"http://example.com".to_http_url()));
        assert!(cookie.matches(&"http://www.example.com".to_http_url()));
        assert!(!cookie.matches(&"http://square.com".to_http_url()));
    }

    pub fn domain_ignored_with_trailing_dot(&self) {
        let cookie = Cookie::parse(&self.url, "a=b; domain=example.com.").expect("Parsed");
        assert!(cookie.matches(&"http://example.com".to_http_url()));
        assert!(!cookie.matches(&"http://www.example.com".to_http_url()));
        assert!(!cookie.matches(&"http://square.com".to_http_url()));
    }

    pub fn idn_domain_matches(&self) {
        let url = "http://☃.net/".to_http_url();
        let cookie = Cookie::parse(&url, "a=b; domain=☃.net").expect("Parsed");
        assert!(cookie.matches(&"http://☃.net/".to_http_url()));
        assert!(cookie.matches(&"http://xn--n3h.net/".to_http_url()));
        assert!(cookie.matches(&"http://www.☃.net/".to_http_url()));
        assert!(cookie.matches(&"http://www.xn--n3h.net/".to_http_url()));
    }

    pub fn punycode_domain_matches(&self) {
        let url = "http://xn--n3h.net/".to_http_url();
        let cookie = Cookie::parse(&url, "a=b; domain=xn--n3h.net").expect("Parsed");
        assert!(cookie.matches(&"http://☃.net/".to_http_url()));
        assert!(cookie.matches(&"http://xn--n3h.net/".to_http_url()));
        assert!(cookie.matches(&"http://www.☃.net/".to_http_url()));
        assert!(cookie.matches(&"http://www.xn--n3h.net/".to_http_url()));
    }

    pub fn domain_matches_ip_address(&self) {
        let url_with_ip = "http://123.45.234.56/".to_http_url();
        assert!(Cookie::parse(&url_with_ip, "a=b; domain=234.56").is_none());
        assert_eq!(Cookie::parse(&url_with_ip, "a=b; domain=123.45.234.56").unwrap().domain, Some("123.45.234.56".to_string()));
    }

    pub fn domain_matches_ipv6_address(&self) {
        let url = "http://[::1]/".to_http_url();
        let cookie = Cookie::parse(&url, "a=b; domain=::1").expect("Parsed");
        assert_eq!(cookie.domain, Some("::1".to_string()));
        assert!(cookie.matches(&"http://[::1]/".to_http_url()));
    }

    pub fn domain_matches_ipv6_address_with_compression(&self) {
        let url = "http://[0001:0000::]/".to_http_url();
        let cookie = Cookie::parse(&url, "a=b; domain=0001:0000::").expect("Parsed");
        assert_eq!(cookie.domain, Some("1::".to_string()));
        assert!(cookie.matches(&"http://[1::]/".to_http_url()));
    }

    pub fn domain_matches_ipv6_address_with_ipv4_suffix(&self) {
        let url = "http://[::1:ffff:ffff]/".to_http_url();
        let cookie = Cookie::parse(&url, "a=b; domain=::1:255.255.255.255").expect("Parsed");
        assert_eq!(cookie.domain, Some("::1:ffff:ffff".to_string()));
        assert!(cookie.matches(&"http://[::1:ffff:ffff]/".to_http_url()));
    }

    pub fn ipv6_address_doesnt_match(&self) {
        let url = "http://[::1]/".to_http_url();
        let cookie = Cookie::parse(&url, "a=b; domain=::2");
        assert!(cookie.is_none());
    }

    pub fn ipv6_address_malformed(&self) {
        let url = "http://[::1]/".to_http_url();
        let cookie = Cookie::parse(&url, "a=b; domain=::2::2").expect("Parsed");
        assert_eq!(cookie.domain, Some("::1".to_string()));
    }

    pub fn domain_is_public_suffix(&self) {
        let ascii = "https://foo1.foo.bar.elb.amazonaws.com".to_http_url();
        assert!(Cookie::parse(&ascii, "a=b; domain=foo.bar.elb.amazonaws.com").is_some());
        assert!(Cookie::parse(&ascii, "a=b; domain=bar.elb.amazonaws.com").is_none());
        assert!(Cookie::parse(&ascii, "a=b; domain=com").is_none());
        
        let unicode = "https://長.長.長崎.jp".to_http_url();
        assert!(Cookie::parse(&unicode, "a=b; domain=長.長崎.jp").is_some());
        assert!(Cookie::parse(&unicode, "a=b; domain=長崎.jp").is_none());
        
        let punycode = "https://xn--ue5a.xn--ue5a.xn--8ltr62k.jp".to_http_url();
        assert!(Cookie::parse(&punycode, "a=b; domain=xn--ue5a.xn--8ltr62k.jp").is_some());
        assert!(Cookie::parse(&punycode, "a=b; domain=xn--8ltr62k.jp").is_none());
    }

    pub fn host_only(&self) {
        assert!(Cookie::parse(&self.url, "a=b").unwrap().host_only);
        assert!(!Cookie::parse(&self.url, "a=b; domain=example.com").unwrap().host_only);
    }

    pub fn default_path(&self) {
        assert_eq!(Cookie::parse(&"http://example.com/foo/bar".to_http_url(), "a=b").unwrap().path, "/foo");
        assert_eq!(Cookie::parse(&"http://example.com/foo/".to_http_url(), "a=b").unwrap().path, "/foo");
        assert_eq!(Cookie::parse(&"http://example.com/foo".to_http_url(), "a=b").unwrap().path, "/");
        assert_eq!(Cookie::parse(&"http://example.com/".to_http_url(), "a=b").unwrap().path, "/");
    }

    pub fn default_path_is_used_if_path_doesnt_have_leading_slash(&self) {
        assert_eq!(Cookie::parse(&"http://example.com/foo/bar".to_http_url(), "a=b; path=quux").unwrap().path, "/foo");
        assert_eq!(Cookie::parse(&"http://example.com/foo/bar".to_http_url(), "a=b; path=").unwrap().path, "/foo");
    }

    pub fn path_attribute_doesnt_need_to_match(&self) {
        assert_eq!(Cookie::parse(&"http://example.com/".to_http_url(), "a=b; path=/quux").unwrap().path, "/quux");
        assert_eq!(Cookie::parse(&"http://example.com/foo/bar".to_http_url(), "a=b; path=/quux").unwrap().path, "/quux");
    }

    pub fn http_only(&self) {
        assert!(!Cookie::parse(&self.url, "a=b").unwrap().http_only);
        assert!(Cookie::parse(&self.url, "a=b; HttpOnly").unwrap().http_only);
    }

    pub fn secure(&self) {
        assert!(!Cookie::parse(&self.url, "a=b").unwrap().secure);
        assert!(Cookie::parse(&self.url, "a=b; Secure").unwrap().secure);
    }

    pub fn max_age_takes_precedence_over_expires(&self) {
        assert_eq!(parse_cookie(0, &self.url, "a=b; Max-Age=1; Expires=Thu, 01 Jan 1970 00:00:02 GMT").unwrap().expires_at, 1000);
        assert_eq!(parse_cookie(0, &self.url, "a=b; Expires=Thu, 01 Jan 1970 00:00:02 GMT; Max-Age=1").unwrap().expires_at, 1000);
        assert_eq!(parse_cookie(0, &self.url, "a=b; Max-Age=2; Expires=Thu, 01 Jan 1970 00:00:01 GMT").unwrap().expires_at, 2000);
        assert_eq!(parse_cookie(0, &self.url, "a=b; Expires=Thu, 01 Jan 1970 00:00:01 GMT; Max-Age=2").unwrap().expires_at, 2000);
    }

    pub fn last_max_age_wins(&self) {
        assert_eq!(parse_cookie(0, &self.url, "a=b; Max-Age=2; Max-Age=4; Max-Age=1; Max-Age=3").unwrap().expires_at, 3000);
    }

    pub fn last_expires_at_wins(&self) {
        let cookie_str = "a=b; Expires=Thu, 01 Jan 1970 00:00:02 GMT; Expires=Thu, 01 Jan 1970 00:00:04 GMT; Expires=Thu, 01 Jan 1970 00:00:01 GMT; Expires=Thu, 01 Jan 1970 00:00:03 GMT";
        assert_eq!(parse_cookie(0, &self.url, cookie_str).unwrap().expires_at, 3000);
    }

    pub fn max_age_or_expires_makes_cookie_persistent(&self) {
        assert!(!parse_cookie(0, &self.url, "a=b").unwrap().persistent);
        assert!(parse_cookie(0, &self.url, "a=b; Max-Age=1").unwrap().persistent);
        assert!(parse_cookie(0, &self.url, "a=b; Expires=Thu, 01 Jan 1970 00:00:01 GMT").unwrap().persistent);
    }

    pub fn parse_all(&self) {
        let mut builder = Headers::builder();
        builder.add("Set-Cookie: a=b");
        builder.add("Set-Cookie: c=d");
        let headers = builder.build();
        let cookies = Cookie::parse_all(&self.url, &headers);
        assert_eq!(cookies.len(), 2);
        assert_eq!(cookies[0].to_string(), "a=b; path=/");
        assert_eq!(cookies[1].to_string(), "c=d; path=/");
    }

    pub fn builder(&self) {
        let cookie = Cookie::builder()
            .name("a")
            .value("b")
            .domain("example.com")
            .build();
        assert_eq!(cookie.name, "a");
        assert_eq!(cookie.value, "b");
        assert_eq!(cookie.expires_at, MAX_DATE);
        assert_eq!(cookie.domain, Some("example.com".to_string()));
        assert_eq!(cookie.path, "/");
        assert!(!cookie.secure);
        assert!(!cookie.http_only);
        assert!(!cookie.persistent);
        assert!(!cookie.host_only);
        assert!(cookie.same_site.is_none());
    }

    pub fn new_builder(&self) {
        let cookie = parse_cookie(0, &self.url, "c=d; Max-Age=1").unwrap()
            .new_builder()
            .name("a")
            .value("b")
            .domain("example.com")
            .expires_at(MAX_DATE)
            .build();
        assert_eq!(cookie.name, "a");
        assert_eq!(cookie.value, "b");
        assert_eq!(cookie.expires_at, MAX_DATE);
        assert_eq!(cookie.domain, Some("example.com".to_string()));
        assert_eq!(cookie.path, "/");
        assert!(!cookie.secure);
        assert!(!cookie.http_only);
        assert!(cookie.persistent);
        assert!(!cookie.host_only);
    }

    pub fn builder_name_validation(&self) {
        let result = std::panic::catch_unwind(|| {
            Cookie::builder().name(" a ");
        });
        assert!(result.is_err());
    }

    pub fn builder_value_validation(&self) {
        let result = std::panic::catch_unwind(|| {
            Cookie::builder().value(" b ");
        });
        assert!(result.is_err());
    }

    pub fn builder_clamps_max_date(&self) {
        let cookie = Cookie::builder()
            .name("a")
            .value("b")
            .host_only_domain("example.com")
            .expires_at(i64::MAX)
            .build();
        assert_eq!(cookie.to_string(), "a=b; expires=Fri, 31 Dec 9999 23:59:59 GMT; path=/");
    }

    pub fn builder_expires_at(&self) {
        let cookie = Cookie::builder()
            .name("a")
            .value("b")
            .host_only_domain("example.com")
            .expires_at(self.date("1970-01-01T00:00:01.000+0000"))
            .build();
        assert_eq!(cookie.to_string(), "a=b; expires=Thu, 01 Jan 1970 00:00:01 GMT; path=/");
    }

    pub fn builder_clamps_min_date(&self) {
        let cookie = Cookie::builder()
            .name("a")
            .value("b")
            .host_only_domain("example.com")
            .expires_at(self.date("1970-01-01T00:00:00.000+0000"))
            .build();
        assert_eq!(cookie.to_string(), "a=b; max-age=0; path=/");
    }

    pub fn builder_domain_validation(&self) {
        let result = std::panic::catch_unwind(|| {
            Cookie::builder().host_only_domain("a/b");
        });
        assert!(result.is_err());
    }

    pub fn builder_domain(&self) {
        let cookie = Cookie::builder()
            .name("a")
            .value("b")
            .host_only_domain("squareup.com")
            .build();
        assert_eq!(cookie.domain, Some("squareup.com".to_string()));
        assert!(cookie.host_only);
    }

    pub fn builder_path(&self) {
        let cookie = Cookie::builder()
            .name("a")
            .value("b")
            .host_only_domain("example.com")
            .path("/foo")
            .build();
        assert_eq!(cookie.path, "/foo");
    }

    pub fn builder_path_validation(&self) {
        let result = std::panic::catch_unwind(|| {
            Cookie::builder().path("foo");
        });
        assert!(result.is_err());
    }

    pub fn builder_secure(&self) {
        let cookie = Cookie::builder()
            .name("a")
            .value("b")
            .host_only_domain("example.com")
            .secure()
            .build();
        assert!(cookie.secure);
    }

    pub fn builder_http_only(&self) {
        let cookie = Cookie::builder()
            .name("a")
            .value("b")
            .host_only_domain("example.com")
            .http_only()
            .build();
        assert!(cookie.http_only);
    }

    pub fn builder_ipv6(&self) {
        let cookie = Cookie::builder()
            .name("a")
            .value("b")
            .domain("0:0:0:0:0:0:0:1")
            .build();
        assert_eq!(cookie.domain, Some("::1".to_string()));
    }

    pub fn empty_same_site(&self) {
        assert_eq!(Cookie::parse(&self.url, "a=b; SameSite=").unwrap().same_site, Some("".to_string()));
        assert_eq!(Cookie::parse(&self.url, "a=b; SameSite= ").unwrap().same_site, Some("".to_string()));
        assert_eq!(Cookie::parse(&self.url, "a=b; SameSite=\r\t \n").unwrap
)}}
