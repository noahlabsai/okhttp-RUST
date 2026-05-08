use std::collections::HashMap;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::MediaType;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;

// Extension trait for charset_name() to match Kotlin's extension function
pub trait MediaTypeExt {
    fn charset_name(&self) -> Option<String>;
}

impl MediaTypeExt for MediaType {
    fn charset_name(&self) -> Option<String> {
        self.parameter("charset").map(|s| s.to_uppercase())
    }
}

pub struct MediaTypeTest;

impl MediaTypeTest {
    fn parse(&self, string: &str) -> MediaType {
        MediaType::to_media_type_or_null(string)
            .expect("Failed to parse MediaType")
    }

    fn assert_invalid(&self, string: &str, _exception_message: Option<&str>) {
        let result = MediaType::to_media_type_or_null(string);
        assert!(
            result.is_none(),
            "Expected invalid media type for {}: {:?}", 
            string, 
            _exception_message
        );
    }

    fn assert_media_type(&self, string: &str) {
        assert_eq!(string, self.parse(string).to_string());
    }

    pub fn test_parse(&self) {
        let media_type = self.parse("text/plain;boundary=foo;charset=utf-8");
        assert_eq!(media_type.type_(), "text");
        assert_eq!(media_type.subtype(), "plain");
        assert_eq!(media_type.charset_name(), Some("UTF-8".to_string()));
        assert_eq!(media_type.to_string(), "text/plain;boundary=foo;charset=utf-8");
        assert_eq!(media_type, self.parse("text/plain;boundary=foo;charset=utf-8"));
        // Note: hashCode() is typically handled by Eq/Hash in Rust
    }

    pub fn test_valid_parse(&self) {
        self.assert_media_type("text/plain");
        self.assert_media_type("application/atom+xml; charset=utf-8");
        self.assert_media_type("application/atom+xml; a=1; a=2; b=3");
        self.assert_media_type("image/gif; foo=bar");
        self.assert_media_type("text/plain; a=1");
        self.assert_media_type("text/plain; a=1; a=2; b=3");
        self.assert_media_type("text/plain; charset=utf-16");
        self.assert_media_type("text/plain; \t \n \r a=b");
        self.assert_media_type("text/plain;");
        self.assert_media_type("text/plain; ");
        self.assert_media_type("text/plain; a=1;");
        self.assert_media_type("text/plain; a=1; ");
        self.assert_media_type("text/plain; a=1;; b=2");
        self.assert_media_type("text/plain;;");
        self.assert_media_type("text/plain; ;");
    }

    pub fn test_invalid_parse(&self) {
        self.assert_invalid("", Some("No subtype found for: \"\""));
        self.assert_invalid("/", Some("No subtype found for: \"/\""));
        self.assert_invalid("text", Some("No subtype found for: \"text\""));
        self.assert_invalid("text/", Some("No subtype found for: \"text/\""));
        self.assert_invalid("te<t/plain", Some("No subtype found for: \"te<t/plain\""));
        self.assert_invalid(" text/plain", Some("No subtype found for: \" text/plain\""));
        self.assert_invalid("te xt/plain", Some("No subtype found for: \"te xt/plain\""));
        self.assert_invalid("text /plain", Some("No subtype found for: \"text /plain\""));
        self.assert_invalid("text/ plain", Some("No subtype found for: \"text/ plain\""));
        self.assert_invalid(
            "text/pl@in",
            Some("Parameter is not formatted correctly: \"@in\" for: \"text/pl@in\""),
        );
        self.assert_invalid(
            "text/plain; a",
            Some("Parameter is not formatted correctly: \"a\" for: \"text/plain; a\""),
        );
        self.assert_invalid(
            "text/plain; a=",
            Some("Parameter is not formatted correctly: \"a=\" for: \"text/plain; a=\""),
        );
        self.assert_invalid(
            "text/plain; a=@",
            Some("Parameter is not formatted correctly: \"a=@\" for: \"text/plain; a=@\""),
        );
        self.assert_invalid(
            "text/plain; a=\"@",
            Some("Parameter is not formatted correctly: \"a=\"@\" for: \"text/plain; a=\"@\""),
        );
        self.assert_invalid(
            "text/plain; a=1; b",
            Some("Parameter is not formatted correctly: \"b\" for: \"text/plain; a=1; b\""),
        );
        self.assert_invalid(
            "text/plain; a=1; b=",
            Some("Parameter is not formatted correctly: \"b=\" for: \"text/plain; a=1; b=\""),
        );
        self.assert_invalid(
            "text/plain; a=\u{2025}",
            Some("Parameter is not formatted correctly: \"a=\u{2025}\" for: \"text/plain; a=\u{2025}\""),
        );
        self.assert_invalid(
            "text/pl ain",
            Some("Parameter is not formatted correctly: \" ain\" for: \"text/pl ain\""),
        );
        self.assert_invalid(
            "text/plain ",
            Some("Parameter is not formatted correctly: \" \" for: \"text/plain \""),
        );
        self.assert_invalid(
            "text/plain ; a=1",
            Some("Parameter is not formatted correctly: \" ; a=1\" for: \"text/plain ; a=1\""),
        );
    }

    pub fn test_double_quotes_are_special(&self) {
        let media_type = self.parse("text/plain;a=\";charset=utf-8;b=\"");
        assert!(media_type.charset_name().is_none());
    }

    pub fn test_single_quotes_are_not_special(&self) {
        let media_type = self.parse("text/plain;a=';charset=utf-8;b='");
        assert_eq!(media_type.charset_name(), Some("UTF-8".to_string()));
    }

    pub fn test_parse_with_special_characters(&self) {
        let media_type = self.parse("!#$%&'*+-.{|}~/!#$%&'*+-.{|}~; !#$%&'*+-.{|}~=!#$%&'*+-.{|}~");
        assert_eq!(media_type.type_(), "!#$%&'*+-.{|}~");
        assert_eq!(media_type.subtype(), "!#$%&'*+-.{|}~");
    }

    pub fn test_charset_is_one_of_many_parameters(&self) {
        let media_type = self.parse("text/plain;a=1;b=2;charset=utf-8;c=3");
        assert_eq!(media_type.type_(), "text");
        assert_eq!(media_type.subtype(), "plain");
        assert_eq!(media_type.charset_name(), Some("UTF-8".to_string()));
        assert_eq!(media_type.parameter("charset"), Some("utf-8".to_string()));
        assert_eq!(media_type.parameter("a"), Some("1".to_string()));
        assert_eq!(media_type.parameter("b"), Some("2".to_string()));
        assert_eq!(media_type.parameter("c"), Some("3".to_string()));
        assert_eq!(media_type.parameter("CHARSET"), Some("utf-8".to_string()));
        assert_eq!(media_type.parameter("A"), Some("1".to_string()));
        assert_eq!(media_type.parameter("B"), Some("2".to_string()));
        assert_eq!(media_type.parameter("C"), Some("3".to_string()));
    }

    pub fn test_charset_and_quoting(&self) {
        let media_type = self.parse("text/plain;a=\";charset=us-ascii\";charset=\"utf-8\";b=\"iso-8859-1\"");
        assert_eq!(media_type.charset_name(), Some("UTF-8".to_string()));
    }

    pub fn test_duplicated_charsets(&self) {
        let media_type = self.parse("text/plain; charset=utf-8; charset=UTF-8");
        assert_eq!(media_type.charset_name(), Some("UTF-8".to_string()));
    }

    pub fn test_multiple_charsets_returns_first_match(&self) {
        let media_type = self.parse("text/plain; charset=utf-8; charset=utf-16");
        assert_eq!(media_type.charset_name(), Some("UTF-8".to_string()));
    }

    pub fn test_charset_name_is_single_quoted(&self) {
        let media_type = self.parse("text/plain;charset='utf-8'");
        assert_eq!(media_type.charset_name(), Some("UTF-8".to_string()));
    }

    pub fn test_parse_dangling_semicolon(&self) {
        let media_type = self.parse("text/plain;");
        assert_eq!(media_type.type_(), "text");
        assert_eq!(media_type.subtype(), "plain");
        assert!(media_type.charset_name().is_none());
        assert_eq!(media_type.to_string(), "text/plain;");
    }

    pub fn test_parameter(&self) {
        let media_type = self.parse("multipart/mixed; boundary=\"abcd\"");
        assert_eq!(media_type.parameter("boundary"), Some("abcd".to_string()));
        assert_eq!(media_type.parameter("BOUNDARY"), Some("abcd".to_string()));
    }

    pub fn test_multiple_parameters(&self) {
        let media_type = self.parse("Message/Partial; number=2; total=3; id=\"oc=abc@example.com\"");
        assert_eq!(media_type.parameter("number"), Some("2".to_string()));
        assert_eq!(media_type.parameter("total"), Some("3".to_string()));
        assert_eq!(media_type.parameter("id"), Some("oc=abc@example.com".to_string()));
        assert!(media_type.parameter("foo").is_none());
    }

    pub fn test_repeated_parameter(&self) {
        let media_type = self.parse("multipart/mixed; number=2; number=3");
        assert_eq!(media_type.parameter("number"), Some("2".to_string()));
    }
}
