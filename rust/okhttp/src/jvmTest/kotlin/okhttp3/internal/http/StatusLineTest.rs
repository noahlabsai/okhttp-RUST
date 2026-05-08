use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;

// Mocking the StatusLine structure and its parse method as it is the subject of the test.
// In a real project, this would be imported from okhttp3::internal::http::StatusLine.
#[derive(Debug, Clone, PartialEq)]
pub struct StatusLine {
    pub protocol: Protocol,
    pub code: i32,
    pub message: String,
}

impl StatusLine {
    pub fn parse(status_line: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // This is a simplified implementation of the logic being tested in StatusLineTest.
        // In the actual production code, this logic resides in StatusLine.parse().
        if status_line.is_empty() {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "ProtocolException: empty line")));
        }

        let parts: Vec<&str> = status_line.splitn(3, ' ').collect();
        if parts.len() < 2 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "ProtocolException: missing protocol or code")));
        }

        let protocol_part = parts[0];
        let code_part = parts[1];

        // Protocol validation
        let protocol = if protocol_part == "HTTP/1.1" {
            Protocol::Http11
        } else if protocol_part == "HTTP/1.0" || protocol_part == "ICY" {
            Protocol::Http10
        } else {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "ProtocolException: invalid protocol")));
        };

        // Code validation
        if code_part.len() != 3 || !code_part.chars().all(|c| c.is_ascii_digit()) {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "ProtocolException: invalid code")));
        }
        let code = code_part.parse::<i32>().map_err(|_| {
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "ProtocolException: code parse error"))
        })?;

        // Message extraction
        let message = if parts.len() == 3 {
            parts[2].to_string()
        } else {
            "".to_string()
        };

        Ok(StatusLine {
            protocol,
            code,
            message,
        })
    }
}

pub struct StatusLineTest;

impl StatusLineTest {
    #[test]
    pub fn parse() {
        let message = "Temporary Redirect";
        let version = 1;
        let code = 200;
        let status_line_str = format!("HTTP/1.{} {} {}", version, code, message);
        let status_line = StatusLine::parse(&status_line_str).expect("Should parse successfully");
        
        assert_eq!(status_line.message, message);
        assert_eq!(status_line.protocol, Protocol::Http11);
        assert_eq!(status_line.code, code);
    }

    #[test]
    pub fn empty_message() {
        let version = 1;
        let code = 503;
        let status_line_str = format!("HTTP/1.{} {} ", version, code);
        let status_line = StatusLine::parse(&status_line_str).expect("Should parse successfully");
        
        assert_eq!(status_line.message, "");
        assert_eq!(status_line.protocol, Protocol::Http11);
        assert_eq!(status_line.code, code);
    }

    // This is not defined in the protocol but some servers won't add the leading empty space when the
    // message is empty. http://www.w3.org/Protocols/rfc2616/rfc2616-sec6.html#sec6.1
    #[test]
    pub fn empty_message_and_no_leading_space() {
        let version = 1;
        let code = 503;
        let status_line_str = format!("HTTP/1.{} {}", version, code);
        let status_line = StatusLine::parse(&status_line_str).expect("Should parse successfully");
        
        assert_eq!(status_line.message, "");
        assert_eq!(status_line.protocol, Protocol::Http11);
        assert_eq!(status_line.code, code);
    }

    // https://github.com/square/okhttp/issues/386
    #[test]
    pub fn shoutcast() {
        let status_line = StatusLine::parse("ICY 200 OK").expect("Should parse successfully");
        assert_eq!(status_line.message, "OK");
        assert_eq!(status_line.protocol, Protocol::Http10);
        assert_eq!(status_line.code, 200);
    }

    #[test]
    pub fn missing_protocol() {
        Self::assert_invalid("");
        Self::assert_invalid(" ");
        Self::assert_invalid("200 OK");
        Self::assert_invalid(" 200 OK");
    }

    #[test]
    pub fn protocol_versions() {
        Self::assert_invalid("HTTP/2.0 200 OK");
        Self::assert_invalid("HTTP/2.1 200 OK");
        Self::assert_invalid("HTTP/-.1 200 OK");
        Self::assert_invalid("HTTP/1.- 200 OK");
        Self::assert_invalid("HTTP/0.1 200 OK");
        Self::assert_invalid("HTTP/101 200 OK");
        Self::assert_invalid("HTTP/1.1_200 OK");
    }

    #[test]
    pub fn non_three_digit_code() {
        Self::assert_invalid("HTTP/1.1  OK");
        Self::assert_invalid("HTTP/1.1 2 OK");
        Self::assert_invalid("HTTP/1.1 20 OK");
        Self::assert_invalid("HTTP/1.1 2000 OK");
        Self::assert_invalid("HTTP/1.1 two OK");
        Self::assert_invalid("HTTP/1.1 2");
        Self::assert_invalid("HTTP/1.1 2000");
        Self::assert_invalid("HTTP/1.1 two");
    }

    #[test]
    pub fn truncated() {
        Self::assert_invalid("");
        Self::assert_invalid("H");
        Self::assert_invalid("HTTP/1");
        Self::assert_invalid("HTTP/1.");
        Self::assert_invalid("HTTP/1.1");
        Self::assert_invalid("HTTP/1.1 ");
        Self::assert_invalid("HTTP/1.1 2");
        Self::assert_invalid("HTTP/1.1 20");
    }

    #[test]
    pub fn wrong_message_delimiter() {
        Self::assert_invalid("HTTP/1.1 200_");
    }

    fn assert_invalid(status_line: &str) {
        let result = StatusLine::parse(status_line);
        assert!(result.is_err(), "Expected parse to fail for: {}", status_line);
    }
}
