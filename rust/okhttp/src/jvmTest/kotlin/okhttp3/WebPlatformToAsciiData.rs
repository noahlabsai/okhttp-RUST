use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::PathBuf;

/// A test from the [Web Platform To ASCII](https://github.com/web-platform-tests/wpt/blob/master/url/resources/toascii.json).
///
/// Each test is a line of the file `toascii.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebPlatformToAsciiData {
    pub input: Option<String>,
    pub output: Option<String>,
    pub comment: Option<String>,
}

impl fmt::Display for WebPlatformToAsciiData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "input={:?} output={:?}", self.input, self.output)
    }
}

impl WebPlatformToAsciiData {
    /// Companion object method to load the test data from the JSON file.
    pub fn load() -> Result<Vec<WebPlatformToAsciiData>, Box<dyn std::error::Error>> {
        // In the Kotlin source, okHttpRoot and SYSTEM_FILE_SYSTEM are environment-specific globals.
        // We implement this using standard Rust path and file I/O.
        let ok_http_root = std::env::var("OKHTTP_ROOT").unwrap_or_else(|_| ".".to_string());
        let mut path = PathBuf::from(ok_http_root);
        path.push("okhttp/src/jvmTest/resources/web-platform-test-toascii.json");

        let content = fs::read_to_string(path)?;
        let data: Vec<WebPlatformToAsciiData> = serde_json::from_str(&content)?;
        Ok(data)
    }
}