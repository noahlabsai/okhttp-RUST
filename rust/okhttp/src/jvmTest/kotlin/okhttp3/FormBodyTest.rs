use std::collections::HashMap;
use std::io::{Read, Write};
use crate::android_test::build_gradle::*;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::cache::FaultHidingSink::*;

// Mocking the necessary OkHttp components as they are dependencies of the test
// In a real project, these would be imported from the crate.

impl MediaType {
    pub fn new(type_str: &str, subtype: &str, encoding: Option<&str>) -> Self {
        Self {
            type_str: type_str.to_string(),
            subtype: subtype.to_string(),
            encoding: encoding.map(|s| s.to_string()),
        }
    }

    pub fn to_string(&self) -> String {
        let mut s = format!("{}/{}", self.type_str, self.subtype);
        if let Some(ref enc) = self.encoding {
            s.push_str("; charset=");
            s.push_str(enc);
        }
        s
    }
}

pub trait RequestBody {
    fn content_length(&self) -> i64;
    fn content_type(&self) -> Option<MediaType>;
    fn write_to(&self, sink: &mut Buffer) -> std::io::Result<()>;
}


impl FormBody {
    pub fn builder() -> FormBody::Builder {
        FormBody::Builder::new()
    }

    pub fn size(&self) -> usize {
        self.pairs.len()
    }

    pub fn name(&self, index: usize) -> &str {
        &self.pairs[index].0
    }

    pub fn value(&self, index: usize) -> &str {
        &self.pairs[index].1
    }

    pub fn encoded_name(&self, index: usize) -> String {
        // In actual OkHttp, this uses a specific form encoding logic
        self.form_encode(&self.pairs[index].0)
    }

    pub fn encoded_value(&self, index: usize) -> String {
        self.form_encode(&self.pairs[index].1)
    }

    fn form_encode(&self, value: &str) -> String {
        // Simplified mock of OkHttp's form encoding for the test's sake
        // In production, this would be a complex utility function
        value.replace(' ', "+")
            .replace('+', "%2B")
            .replace('=', "%3D")
            .replace('&', "%26")
            .replace(',', "%2C")
            .replace('%', "%25")
    }
}

impl RequestBody for FormBody {
    fn content_length(&self) -> i64 {
        let mut length = 0;
        for (i, (name, value)) in self.pairs.iter().enumerate() {
            if i > 0 { length += 1; } // '&'
            length += self.encoded_name(self.pairs.iter().position(|p| p == (name, value)).unwrap()).len() as i64;
            length += 1; // '='
            length += self.encoded_value(self.pairs.iter().position(|p| p == (name, value)).unwrap()).len() as i64;
        }
        // This is a mock; actual implementation calculates based on encoded bytes
        // For the test to pass, we'd need the real encoding logic.
        // Since we are translating the TEST, we assume FormBody is implemented correctly.
        length
    }

    fn content_type(&self) -> Option<MediaType> {
        Some(MediaType::new("application", "x-www-form-urlencoded", Some("UTF-8")))
    }

    fn write_to(&self, sink: &mut Buffer) -> std::io::Result<()> {
        let mut first = true;
        for (i, _) in self.pairs.iter().enumerate() {
            if !first {
                sink.write_all(b"&")?;
            }
            first = false;
            sink.write_all(self.encoded_name(i).as_bytes())?;
            sink.write_all(b"=")?;
            sink.write_all(self.encoded_value(i).as_bytes())?;
        }
        Ok(())
    }
}

impl FormBody {

    impl FormBody::Builder {
        pub fn new() -> Self {
            Self {
                pairs: Vec::new(),
                charset: "UTF-8".to_string(),
            }
        }

        pub fn with_charset(mut self, charset: &str) -> Self {
            self.charset = charset.to_string();
            self
        }

        pub fn add(mut self, name: &str, value: &str) -> Self {
            self.pairs.push((name.to_string(), value.to_string()));
            self
        }

        pub fn add_encoded(mut self, name: &str, value: &str) -> Self {
            // In real OkHttp, this adds without further encoding
            self.pairs.push((name.to_string(), value.to_string()));
            self
        }

        pub fn build(self) -> FormBody {
            FormBody {
                pairs: self.pairs,
                charset: self.charset,
            }
        }
    }
}

// Mocking okio.Buffer

impl Buffer {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn read_utf8(&mut self) -> String {
        let s = String::from_utf8_lossy(&self.data).to_string();
        self.data.clear();
        s
    }

    pub fn read_utf8_len(&mut self, len: usize) -> String {
        let s = String::from_utf8_lossy(&self.data[..len]).to_string();
        self.data.drain(0..len);
        s
    }

    pub fn skip(&mut self, len: usize) {
        self.data.drain(0..len);
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}

impl Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.data.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Read for Buffer {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.data.is_empty() { return Ok(0); }
        let len = std::cmp::min(buf.len(), self.data.len());
        buf[..len].copy_from_slice(&self.data[..len]);
        self.data.drain(0..len);
        Ok(len)
    }
}

// The Test Class
pub struct FormBodyTest;

impl FormBodyTest {
    #[test]
    pub fn url_encoding() {
        let body = FormBody::builder()
            .add("a+=& b", "c+=& d")
            .add("space, the", "final frontier")
            .add("%25", "%25")
            .build();

        assert_eq!(body.size(), 3);
        assert_eq!(body.encoded_name(0), "a%2B%3D%26+b");
        assert_eq!(body.encoded_name(1), "space%2C+the");
        assert_eq!(body.encoded_name(2), "%2525");
        assert_eq!(body.name(0), "a+=& b");
        assert_eq!(body.name(1), "space, the");
        assert_eq!(body.name(2), "%25");
        assert_eq!(body.encoded_value(0), "c%2B%3D%26+d");
        assert_eq!(body.encoded_value(1), "final+frontier");
        assert_eq!(body.encoded_value(2), "%2525");
        assert_eq!(body.value(0), "c+=& d");
        assert_eq!(body.value(1), "final frontier");
        assert_eq!(body.value(2), "%25");
        
        let ct = body.content_type().expect("Content type should exist");
        assert_eq!(ct.to_string(), "application/x-www-form-urlencoded");

        let expected = "a%2B%3D%26+b=c%2B%3D%26+d&space%2C+the=final+frontier&%2525=%2525";
        assert_eq!(body.content_length(), expected.len() as i64);
        
        let mut out = Buffer::new();
        body.write_to(&mut out).unwrap();
        assert_eq!(out.read_utf8(), expected);
    }

    #[test]
    pub fn add_encoded() {
        let body = FormBody::builder()
            .add_encoded("a+=& b", "c+=& d")
            .add_encoded("e+=& f", "g+=& h")
            .add_encoded("%25", "%25")
            .build();
        
        let expected = "a+%3D%26+b=c+%3D%26+d&e+%3D%26+f=g+%3D%26+h&%25=%25";
        let mut out = Buffer::new();
        body.write_to(&mut out).unwrap();
        assert_eq!(out.read_utf8(), expected);
    }

    #[test]
    pub fn encoded_pair() {
        let body = FormBody::builder()
            .add("sim", "ple")
            .build();
        let expected = "sim=ple";
        assert_eq!(body.content_length(), expected.len() as i64);
        let mut buffer = Buffer::new();
        body.write_to(&mut buffer).unwrap();
        assert_eq!(buffer.read_utf8(), expected);
    }

    #[test]
    pub fn encode_multiple_pairs() {
        let body = FormBody::builder()
            .add("sim", "ple")
            .add("hey", "there")
            .add("help", "me")
            .build();
        let expected = "sim=ple&hey=there&help=me";
        assert_eq!(body.content_length(), expected.len() as i64);
        let mut buffer = Buffer::new();
        body.write_to(&mut buffer).unwrap();
        assert_eq!(buffer.read_utf8(), expected);
    }

    #[test]
    pub fn build_empty_form() {
        let body = FormBody::builder().build();
        let expected = "";
        assert_eq!(body.content_length(), expected.len() as i64);
        let mut buffer = Buffer::new();
        body.write_to(&mut buffer).unwrap();
        assert_eq!(buffer.read_utf8(), expected);
    }

    #[test]
    pub fn character_encoding() {
        // This test verifies the specific encoding of characters.
        // We call the helper function `form_encode` which uses FormBody.
        assert_eq!(Self::form_encode(0), "%00");
        assert_eq!(Self::form_encode(1), "%01");
        assert_eq!(Self::form_encode(2), "%02");
        assert_eq!(Self::form_encode(3), "%03");
        assert_eq!(Self::form_encode(4), "%04");
        assert_eq!(Self::form_encode(5), "%05");
        assert_eq!(Self::form_encode(6), "%06");
        assert_eq!(Self::form_encode(7), "%07");
        assert_eq!(Self::form_encode(8), "%08");
        assert_eq!(Self::form_encode(9), "%09");
        assert_eq!(Self::form_encode(10), "%0A");
        assert_eq!(Self::form_encode(11), "%0B");
        assert_eq!(Self::form_encode(12), "%0C");
        assert_eq!(Self::form_encode(13), "%0D");
        assert_eq!(Self::form_encode(14), "%0E");
        assert_eq!(Self::form_encode(15), "%0F");
        assert_eq!(Self::form_encode(16), "%10");
        assert_eq!(Self::form_encode(17), "%11");
        assert_eq!(Self::form_encode(18), "%12");
        assert_eq!(Self::form_encode(19), "%13");
        assert_eq!(Self::form_encode(20), "%14");
        assert_eq!(Self::form_encode(21), "%15");
        assert_eq!(Self::form_encode(22), "%16");
        assert_eq!(Self::form_encode(23), "%17");
        assert_eq!(Self::form_encode(24), "%18");
        assert_eq!(Self::form_encode(25), "%19");
        assert_eq!(Self::form_encode(26), "%1A");
        assert_eq!(Self::form_encode(27), "%1B");
        assert_eq!(Self::form_encode(28), "%1C");
        assert_eq!(Self::form_encode(29), "%1D");
        assert_eq!(Self::form_encode(30), "%1E");
        assert_eq!(Self::form_encode(31), "%1F");
        assert_eq!(Self::form_encode(32), "+");
        assert_eq!(Self::form_encode(33), "%21");
        assert_eq!(Self::form_encode(34), "%22");
        assert_eq!(Self::form_encode(35), "%23");
        assert_eq!(Self::form_encode(36), "%24");
        assert_eq!(Self::form_encode(37), "%25");
        assert_eq!(Self::form_encode(38), "%26");
        assert_eq!(Self::form_encode(39), "%27");
        assert_eq!(Self::form_encode(40), "%28");
        assert_eq!(Self::form_encode(41), "%29");
        assert_eq!(Self::form_encode(42), "*");
        assert_eq!(Self::form_encode(43), "%2B");
        assert_eq!(Self::form_encode(44), "%2C");
        assert_eq!(Self::form_encode(45), "-");
        assert_eq!(Self::form_encode(46), ".");
        assert_eq!(Self::form_encode(47), "%2F");
        assert_eq!(Self::form_encode(48), "0");
        assert_eq!(Self::form_encode(57), "9");
        assert_eq!(Self::form_encode(58), "%3A");
        assert_eq!(Self::form_encode(59), "%3B");
        assert_eq!(Self::form_encode(60), "%3C");
        assert_eq!(Self::form_encode(61), "%3D");
        assert_eq!(Self::form_encode(62), "%3E");
        assert_eq!(Self::form_encode(63), "%3F");
        assert_eq!(Self::form_encode(64), "%40");
        assert_eq!(Self::form_encode(65), "A");
        assert_eq!(Self::form_encode(90), "Z");
        assert_eq!(Self::form_encode(91), "%5B");
        assert_eq!(Self::form_encode(92), "%5C");
        assert_eq!(Self::form_encode(93), "%5D");
        assert_eq!(Self::form_encode(94), "%5E");
        assert_eq!(Self::form_encode(95), "_");
        assert_eq!(Self::form_encode(96), "%60");
        assert_eq!(Self::form_encode(97), "a");
        assert_eq!(Self::form_encode(122), "z");
        assert_eq!(Self::form_encode(123), "%7B");
        assert_eq!(Self::form_encode(124), "%7C");
        assert_eq!(Self::form_encode(125), "%7D");
        assert_eq!(Self::form_encode(126), "%7E");
        assert_eq!(Self::form_encode(127), "%7F");
        assert_eq!(Self::form_encode(128), "%C2%80");
        assert_eq!(Self::form_encode(255), "%C3%BF");
    }

    fn form_encode(code_point: i32) -> String {
        // Wrap the codepoint with regular printable characters to prevent trimming.
        let input_str = format!("{}{}{}", 'b', std::char::from_u32(code_point as u32).unwrap_or(' '), 'c');
        let body = FormBody::builder()
            .add("a", &input_str)
            .build();
        
        let mut buffer = Buffer::new();
        body.write_to(&mut buffer).unwrap();
        buffer.skip(3); // Skip "a=b" prefix.
        buffer.read_utf8_len(buffer.size() - 1) // Skip the "c" suffix.
    }

    #[test]
    pub fn manual_charset() {
        let body = FormBody::builder()
            .with_charset("ISO-8859-1")
            .add("name", "Nicolás")
            .build();
        
        let expected = "name=Nicol%E1s";
        assert_eq!(body.content_length(), expected.len() as i64);
        let mut out = Buffer::new();
        body.write_to(&mut out).unwrap();
        assert_eq!(out.read_utf8(), expected);
    }
}