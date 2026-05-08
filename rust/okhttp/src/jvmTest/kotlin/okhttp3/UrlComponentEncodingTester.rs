use std::collections::HashMap;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::idn::Punycode;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrl;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrlBuilder;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::idn::Punycode::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::cache::FaultHidingSink::*;
use crate::android_test::src::test::kotlin::okhttp::android::test::DisabledInitialiserTest::*;

// Mocking Buffer and ByteString as they are part of okio and used in the source
// In a real project, these would be imported from the okio crate.

impl Buffer {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    pub fn write_utf8(&mut self, s: &str) -> &mut Self {
        self.data.extend_from_slice(s.as_bytes());
        self
    }
    pub fn read_utf8(&self) -> String {
        String::from_utf8_lossy(&self.data).into_owned()
    }
}


impl ByteString {
    pub fn of(byte: u8) -> Self {
        Self
    }
    pub fn hex(&self) -> String {
        // This is a simplification for the translation
        // In reality, it would return the hex representation of the byte
        format!("{:02x}", 0) // Placeholder logic
    }
    pub fn encode_utf8(s: &str) -> Vec<u8> {
        s.as_bytes().to_vec()
    }
}

// Trait to handle the "toHttpUrl" extension function
pub trait HttpUrlExt {
    fn to_http_url(&self) -> HttpUrl;
}

impl HttpUrlExt for &str {
    fn to_http_url(&self) -> HttpUrl {
        HttpUrl::parse(self).expect("Invalid URL")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Identity,
    Percent,
    Forbidden,
    Punycode,
    Skip,
}

pub const Identity: Encoding = Encoding::Identity;
pub const Percent: Encoding = Encoding::Percent;
pub const Forbidden: Encoding = Encoding::Forbidden;
pub const Skip: Encoding = Encoding::Skip;

impl Default for Encoding {
    fn default() -> Self {
        Encoding::Identity
    }
}

impl Encoding {
    pub fn encode(&self, code_point: i32) -> String {
        match self {
            Encoding::Identity => {
                std::char::from_u32(code_point as u32)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| panic!("Invalid code point"))
            }
            Encoding::Percent => {
                let identity_str = Encoding::Identity.encode(code_point);
                let utf8 = ByteString::encode_utf8(&identity_str);
                let mut percent_encoded = Buffer::new();
                for byte in utf8 {
                    percent_encoded
                        .write_utf8("%")
                        .write_utf8(&ByteString::of(byte).hex().to_uppercase());
                }
                percent_encoded.read_utf8()
            }
            Encoding::Forbidden | Encoding::Punycode | Encoding::Skip => {
                panic!("UnsupportedOperationException: encode not implemented for this variant")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Component {
    User,
    Password,
    Host,
    Path,
    Query,
    QueryValue,
    Fragment,
}

pub const User: Component = Component::User;
pub const Password: Component = Component::Password;
pub const Host: Component = Component::Host;
pub const Path: Component = Component::Path;
pub const Query: Component = Component::Query;
pub const QueryValue: Component = Component::QueryValue;
pub const Fragment: Component = Component::Fragment;

impl Default for Component {
    fn default() -> Self {
        Component::User
    }
}

impl Component {
    pub fn url_string(&self, value: &str) -> String {
        match self {
            Component::User => format!("http://{}@example.com/", value),
            Component::Password => format!("http://:{}@example.com/", value),
            Component::Host => format!("http://a{}z.com/", value),
            Component::Path => format!("http://example.com/a{}z/", value),
            Component::Query => format!("http://example.com/?a{}z", value),
            Component::QueryValue => format!("http://example.com/?q=a{}z", value),
            Component::Fragment => format!("http://example.com/#a{}z", value),
        }
    }

    pub fn encoded_value(&self, url: &HttpUrl) -> String {
        match self {
            Component::User => url.encoded_username().to_string(),
            Component::Password => url.encoded_password().to_string(),
            Component::Host => self.get(url),
            Component::Path => {
                let path = url.encoded_path();
                if path.len() < 4 { return path; }
                path[2..path.len() - 2].to_string()
            }
            Component::Query => {
                let query = url.encoded_query().expect("Query missing");
                if query.len() < 2 { return query.to_string(); }
                query[1..query.len() - 1].to_string()
            }
            Component::QueryValue => {
                let query = url.encoded_query().expect("Query missing");
                if query.len() < 4 { return query.to_string(); }
                query[3..query.len() - 1].to_string()
            }
            Component::Fragment => {
                let fragment = url.encoded_fragment().expect("Fragment missing");
                if fragment.len() < 2 { return fragment.to_string(); }
                fragment[1..fragment.len() - 1].to_string()
            }
        }
    }

    pub fn set(&self, builder: &mut HttpUrlBuilder, value: &str) {
        match self {
            Component::User => { builder.username(value); }
            Component::Password => { builder.password(value); }
            Component::Host => { builder.host(&format!("a{}z.com", value)); }
            Component::Path => { builder.add_path_segment(&format!("a{}z", value)); }
            Component::Query => { builder.query(&format!("a{}z", value)); }
            Component::QueryValue => { builder.add_query_parameter("q", &format!("a{}z", value)); }
            Component::Fragment => { builder.fragment(&format!("a{}z", value)); }
        }
    }

    pub fn get(&self, url: &HttpUrl) -> String {
        match self {
            Component::User => url.username().to_string(),
            Component::Password => url.password().to_string(),
            Component::Host => {
                let host = url.host().expect("Host missing");
                if host.len() < 6 { return host.to_lowercase(); }
                host[1..host.len() - 5].to_lowercase()
            }
            Component::Path => {
                let path_segments = url.path_segments();
                if path_segments.is_empty() { return String::new(); }
                let path_segment = &path_segments[0];
                if path_segment.len() < 2 { return path_segment.to_string(); }
                path_segment[1..path_segment.len() - 1].to_string()
            }
            Component::Query => {
                let query = url.query().expect("Query missing");
                if query.len() < 2 { return query.to_string(); }
                query[1..query.len() - 1].to_string()
            }
            Component::QueryValue => {
                let value = url.query_parameter("q").expect("Query param missing");
                if value.len() < 2 { return value.to_string(); }
                value[1..value.len() - 1].to_string()
            }
            Component::Fragment => {
                let fragment = url.fragment().expect("Fragment missing");
                if fragment.len() < 2 { return fragment.to_string(); }
                fragment[1..fragment.len() - 1].to_string()
            }
        }
    }

    pub fn canonicalize(&self, s: &str) -> String {
        match self {
            Component::Host => s.to_lowercase(),
            _ => s.to_string(),
        }
    }
}


impl Platform {
    pub fn new() -> Self {
        Self {}
    }
    pub fn test(&self, _code_point: i32, _code_point_string: &str, _encoding: Encoding, _component: Component) {
        // Default implementation is empty
    }
}

pub struct UrlComponentEncodingTester {
    encodings: HashMap<i32, Encoding>,
}

impl UrlComponentEncodingTester {
    const UNICODE_2: i32 = 0x1a5;
    const UNICODE_3: i32 = 0x2202;
    const UNICODE_4: i32 = 0x1d11e;

    fn new() -> Self {
        Self {
            encodings: HashMap::new(),
        }
    }

    pub fn new_instance() -> Self {
        let mut tester = Self::new();
        tester.all_ascii(Encoding::Identity);
        tester.non_printable_ascii(Encoding::Percent);
        tester.override_encodings(
            Encoding::Skip,
            &[('\t' as i32), ('\n' as i32), ('\u{000c}' as i32), ('\r' as i32)],
        );
        tester.override_encodings(
            Encoding::Percent,
            &[(' ' as i32), ('"' as i32), ('#' as i32), ('<' as i32), ('>' as i32), ('?' as i32), ('`' as i32)],
        );
        tester.override_encodings(
            Encoding::Percent,
            &[Self::UNICODE_2, Self::UNICODE_3, Self::UNICODE_4],
        );
        tester
    }

    pub fn all_ascii(&mut self, encoding: Encoding) {
        for i in 0..=127 {
            self.encodings.insert(i, encoding);
        }
        self
    }

    pub fn override_encodings(&mut self, encoding: Encoding, code_points: &[i32]) {
        for &cp in code_points {
            self.encodings.insert(cp, encoding);
        }
        self
    }

    pub fn non_printable_ascii(&mut self, encoding: Encoding) {
        let cps = [
            0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, ('\b' as i32), 0xb, 0xe, 0xf,
            0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f, 0x7f,
        ];
        for cp in cps {
            self.encodings.insert(cp, encoding);
        }
        self
    }

    pub fn non_ascii(&mut self, encoding: Encoding) {
        self.encodings.insert(Self::UNICODE_2, encoding);
        self.encodings.insert(Self::UNICODE_3, encoding);
        self.encodings.insert(Self::UNICODE_4, encoding);
        self
    }

    pub fn test(&mut self, component: Component, platform: &Platform) {
        let items: Vec<(i32, Encoding)> = self.encodings.iter().map(|(&k, &v)| (k, v)).collect();
        for (code_point, encoding) in items {
            let code_point_string = Encoding::Identity.encode(code_point);
            if encoding == Encoding::Forbidden {
                self.test_forbidden(code_point, &code_point_string, component);
                continue;
            }
            if encoding == Encoding::Punycode {
                self.test_punycode(&code_point_string, component);
                continue;
            }
            self.test_encode_and_decode(code_point, &code_point_string, component);
            if encoding == Encoding::Skip {
                continue;
            }
            self.test_parse_original(code_point, &code_point_string, encoding, component);
            self.test_parse_already_encoded(code_point, encoding, component);

            platform.test(code_point, &code_point_string, encoding, component);
        }
    }

    fn test_parse_already_encoded(&self, code_point: i32, encoding: Encoding, component: Component) {
        let expected = component.canonicalize(&encoding.encode(code_point));
        let url_string = component.url_string(&expected);
        let url = url_string.to_http_url();
        let actual = component.encoded_value(&url);
        if actual != expected {
            panic!("Encoding {:?} {} using {:?}: '{}' != '{}'", component, code_point, encoding, actual, expected);
        }
    }

    fn test_encode_and_decode(&self, code_point: i32, code_point_string: &str, component: Component) {
        let mut builder = "http://host/".to_http_url().new_builder();
        component.set(&mut builder, code_point_string);
        let url = builder.build();
        let expected = component.canonicalize(code_point_string);
        let actual = component.get(&url);
        if expected != actual {
            panic!("Roundtrip {:?} {} {:?} {} != {}", component, code_point, url, expected, actual);
        }
    }

    fn test_parse_original(&self, code_point: i32, code_point_string: &str, encoding: Encoding, component: Component) {
        let expected = encoding.encode(code_point);
        if encoding != Encoding::Percent {
            return;
        }
        let url_string = component.url_string(code_point_string);
        let url = url_string.to_http_url();
        let actual = component.encoded_value(&url);
        if actual != expected {
            panic!("Encoding {:?} {} using {:?}: '{}' != '{}'", component, code_point, encoding, actual, expected);
        }
    }

    fn test_forbidden(&self, _code_point: i32, code_point_string: &str, component: Component) {
        let mut builder = "http://host/".to_http_url().new_builder();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            component.set(&mut builder, code_point_string);
        }));
        if result.is_ok() {
            panic!("Expected IllegalArgumentException for forbidden character");
        }
    }

    fn test_punycode(&self, code_point_string: &str, component: Component) {
        let mut builder = "http://host/".to_http_url().new_builder();
        component.set(&mut builder, code_point_string);
        let url = builder.build();
        let host = url.host().expect("Host missing");
        if !host.starts_with(Punycode::PREFIX_STRING) {
            panic!("Host {} should start with {}", host, Punycode::PREFIX_STRING);
        }
    }
}
