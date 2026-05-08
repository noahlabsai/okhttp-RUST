use regex::Regex;

/// Mocking the OkHttp structure to ensure the test is compilable.
/// In a real project, this would be imported from the okhttp3 crate.
pub struct OkHttp;

impl OkHttp {
    pub const VERSION: &'static str = "4.12.0"; // Example version string
}

/// Translation of okhttp3.OkHttpTest
pub struct OkHttpTest;

impl OkHttpTest {
    /// Translates the `testVersion` function.
    /// In Rust, tests are typically marked with #[test] and reside in a test module.
    pub fn test_version(&self) {
        let version = OkHttp::VERSION;
        
        // Kotlin: assertThat(OkHttp.VERSION).matches(Regex("[0-9]+\\.[0-9]+\\.[0-9]+(-.+)?"))
        // Rust equivalent using the regex crate.
        let pattern = r"[0-9]+\.[0-9]+\.[0-9]+(-.+)?";
        let re = Regex::new(pattern).expect("Invalid regex pattern");
        
        assert!(
            re.is_match(version),
            "OkHttp.VERSION '{}' does not match the expected version pattern",
            version
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_execution() {
        let test_suite = OkHttpTest;
        test_suite.test_version();
    }
}