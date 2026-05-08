use std::fmt;
use std::sync::LazyLock;

// Mocking okio::ByteString as it is a dependency. 
// In a real production environment, this would be imported from the okio crate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ByteString(pub Vec<u8>);

impl ByteString {
    pub fn encode_utf8(s: &str) -> Self {
        ByteString(s.as_bytes().to_vec())
    }

    pub fn utf8(&self) -> String {
        String::from_utf8_lossy(&self.0).into_owned()
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }
}

// HTTP header: the name is an ASCII string, but the value can be UTF-8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    // Name in case-insensitive ASCII encoding.
    pub name: ByteString,
    // Value in UTF-8 encoding.
    pub value: ByteString,
    pub hpack_size: usize,
}

impl Header {
    // Primary constructor equivalent to the data class primary constructor.
    pub fn new(name: ByteString, value: ByteString) -> Self {
        let hpack_size = 32 + name.size() + value.size();
        Header {
            name,
            value,
            hpack_size,
        }
    }

    // Constructor for (String, String)
    pub fn from_strings(name: &str, value: &str) -> Self {
        Self::new(ByteString::encode_utf8(name), ByteString::encode_utf8(value))
    }

    // Constructor for (ByteString, String)
    pub fn from_byte_string_and_string(name: ByteString, value: &str) -> Self {
        Self::new(name, ByteString::encode_utf8(value))
    }

    // Special header names defined in HTTP/2 spec.
    pub const RESPONSE_STATUS_UTF8: &'static str = ":status";
    pub const TARGET_METHOD_UTF8: &'static str = ":method";
    pub const TARGET_PATH_UTF8: &'static str = ":path";
    pub const TARGET_SCHEME_UTF8: &'static str = ":scheme";
    pub const TARGET_AUTHORITY_UTF8: &'static str = ":authority";

    pub fn pseudo_prefix() -> &'static ByteString {
        static PSEUDO_PREFIX: LazyLock<ByteString> = LazyLock::new(|| ByteString::encode_utf8(":"));
        &*PSEUDO_PREFIX
    }

    pub fn response_status() -> &'static ByteString {
        static RESPONSE_STATUS: LazyLock<ByteString> = LazyLock::new(|| ByteString::encode_utf8(Self::RESPONSE_STATUS_UTF8));
        &*RESPONSE_STATUS
    }

    pub fn target_method() -> &'static ByteString {
        static TARGET_METHOD: LazyLock<ByteString> = LazyLock::new(|| ByteString::encode_utf8(Self::TARGET_METHOD_UTF8));
        &*TARGET_METHOD
    }

    pub fn target_path() -> &'static ByteString {
        static TARGET_PATH: LazyLock<ByteString> = LazyLock::new(|| ByteString::encode_utf8(Self::TARGET_PATH_UTF8));
        &*TARGET_PATH
    }

    pub fn target_scheme() -> &'static ByteString {
        static TARGET_SCHEME: LazyLock<ByteString> = LazyLock::new(|| ByteString::encode_utf8(Self::TARGET_SCHEME_UTF8));
        &*TARGET_SCHEME
    }

    pub fn target_authority() -> &'static ByteString {
        static TARGET_AUTHORITY: LazyLock<ByteString> = LazyLock::new(|| ByteString::encode_utf8(Self::TARGET_AUTHORITY_UTF8));
        &*TARGET_AUTHORITY
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name.utf8(), self.value.utf8())
    }
}