use std::collections::HashMap;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Challenge::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;

// Mocking the necessary OkHttp structures to make the test compilable.
// In a real project, these would be imported from the okhttp3 crate.


impl Challenge {
    pub fn new(scheme: &str, auth_params: HashMap<Option<String>, String>) -> Self {
        Self {
            scheme: scheme.to_string(),
            auth_params,
        }
    }

    pub fn realm(&self) -> Option<&String> {
        self.auth_params.get(&Some("realm".to_string())).map(|s| s)
    }
}


impl Headers {

    impl Builder {
        pub fn new() -> Self {
            Self { headers: Vec::new() }
        }

        pub fn add(mut self, name: &str, value: &str) -> Self {
            // Handle the case where the user passes "Name: Value" instead of separate args
            if name.contains(':') {
                let parts: Vec<&str> = name.splitn(2, ':').collect();
                self.headers.push((parts[0].trim().to_string(), parts[1].trim().to_string()));
            } else {
                self.headers.push((name.to_string(), value.to_string()));
            }
            self
        }

        pub fn build(self) -> Headers {
            Headers { data: self.headers }
        }
    }

    pub fn builder() -> Self::Builder {
        Self::Builder::new()
    }

    pub fn parse_challenges(&self, name: &str) -> Vec<Challenge> {
        // This is a mock implementation of the internal parseChallenges logic
        // to ensure the test code compiles and reflects the expected behavior.
        let mut all_challenges = Vec::new();
        for (h_name, h_value) in &self.data {
            if h_name == name {
                // Simplified parsing logic for the sake of the test translation
                let parts: Vec<&str> = h_value.split(',').collect();
                for part in parts {
                    let part = part.trim();
                    if part.is_empty() { continue; }
                    
                    let space_idx = part.find(' ').unwrap_or(part.len());
                    let scheme = &part[..space_idx];
                    let params_str = &part[space_idx..].trim();
                    
                    let mut auth_params = HashMap::new();
                    if !params_str.is_empty() {
                        // This is a very naive parser; the real one handles quotes and escapes
                        let kv_pairs: Vec<&str> = params_str.split(',').collect();
                        for kv in kv_pairs {
                            let kv = kv.trim();
                            if let Some(eq_idx) = kv.find('=') {
                                let key = kv[..eq_idx].trim();
                                let val = kv[eq_idx + 1..].trim().trim_matches('"');
                                auth_params.insert(Some(key.to_string()), val.to_string());
                            } else if !kv.is_empty() {
                                auth_params.insert(None, kv.to_string());
                            }
                        }
                    }
                    all_challenges.push(Challenge::new(scheme, auth_params));
                }
            }
        }
        all_challenges
    }
}

// Since the original code uses assertk, we use standard Rust assertions here.
macro_rules! assert_eq_map {
    ($left:expr, $right:expr) => {
        assert_eq!($left, $right);
    };
}

pub struct HeadersChallengesTest;

impl HeadersChallengesTest {
    #[test]
    pub fn test_digest_challenge_with_strict_rfc2617_header() {
        let headers = Headers::builder()
            .add(
                "WWW-Authenticate",
                "Digest realm=\"myrealm\", nonce=\"fjalskdflwejrlaskdfjlaskdjflaksjdflkasdf\", qop=\"auth\", stale=\"FALSE\"",
            )
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0].scheme, "Digest");
        assert_eq!(challenges[0].realm(), Some(&"myrealm".to_string()));
        
        let mut expected_auth_params = HashMap::new();
        expected_auth_params.insert(Some("realm".to_string()), "myrealm".to_string());
        expected_auth_params.insert(Some("nonce".to_string()), "fjalskdflwejrlaskdfjlaskdjflaksjdflkasdf".to_string());
        expected_auth_params.insert(Some("qop".to_string()), "auth".to_string());
        expected_auth_params.insert(Some("stale".to_string()), "FALSE".to_string());
        
        assert_eq!(challenges[0].auth_params, expected_auth_params);
    }

    #[test]
    pub fn test_digest_challenge_with_differently_ordered_auth_params() {
        let headers = Headers::builder()
            .add(
                "WWW-Authenticate",
                "Digest qop=\"auth\", realm=\"myrealm\", nonce=\"fjalskdflwejrlaskdfjlaskdjflaksjdflkasdf\", stale=\"FALSE\"",
            )
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0].scheme, "Digest");
        assert_eq!(challenges[0].realm(), Some(&"myrealm".to_string()));
        
        let mut expected_auth_params = HashMap::new();
        expected_auth_params.insert(Some("realm".to_string()), "myrealm".to_string());
        expected_auth_params.insert(Some("nonce".to_string()), "fjalskdflwejrlaskdfjlaskdjflaksjdflkasdf".to_string());
        expected_auth_params.insert(Some("qop".to_string()), "auth".to_string());
        expected_auth_params.insert(Some("stale".to_string()), "FALSE".to_string());
        
        assert_eq!(challenges[0].auth_params, expected_auth_params);
    }

    #[test]
    pub fn test_digest_challenge_with_differently_ordered_auth_params2() {
        let headers = Headers::builder()
            .add(
                "WWW-Authenticate",
                "Digest qop=\"auth\", nonce=\"fjalskdflwejrlaskdfjlaskdjflaksjdflkasdf\", realm=\"myrealm\", stale=\"FALSE\"",
            )
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0].scheme, "Digest");
        assert_eq!(challenges[0].realm(), Some(&"myrealm".to_string()));
        
        let mut expected_auth_params = HashMap::new();
        expected_auth_params.insert(Some("realm".to_string()), "myrealm".to_string());
        expected_auth_params.insert(Some("nonce".to_string()), "fjalskdflwejrlaskdfjlaskdjflaksjdflkasdf".to_string());
        expected_auth_params.insert(Some("qop".to_string()), "auth".to_string());
        expected_auth_params.insert(Some("stale".to_string()), "FALSE".to_string());
        
        assert_eq!(challenges[0].auth_params, expected_auth_params);
    }

    #[test]
    pub fn test_digest_challenge_with_missing_realm() {
        let headers = Headers::builder()
            .add(
                "WWW-Authenticate",
                "Digest qop=\"auth\", underrealm=\"myrealm\", nonce=\"fjalskdflwejrlaskdfjlaskdjflaksjdflkasdf\", stale=\"FALSE\"",
            )
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0].scheme, "Digest");
        assert!(challenges[0].realm().is_none());
        
        let mut expected_auth_params = HashMap::new();
        expected_auth_params.insert(Some("underrealm".to_string()), "myrealm".to_string());
        expected_auth_params.insert(Some("nonce".to_string()), "fjalskdflwejrlaskdfjlaskdjflaksjdflkasdf".to_string());
        expected_auth_params.insert(Some("qop".to_string()), "auth".to_string());
        expected_auth_params.insert(Some("stale".to_string()), "FALSE".to_string());
        
        assert_eq!(challenges[0].auth_params, expected_auth_params);
    }

    #[test]
    pub fn test_digest_challenge_with_additional_spaces() {
        let headers = Headers::builder()
            .add(
                "WWW-Authenticate",
                "Digest qop=\"auth\",    realm=\"myrealm\", nonce=\"fjalskdflwejrlaskdfjlaskdjflaksjdflkasdf\", stale=\"FALSE\"",
            )
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0].scheme, "Digest");
        assert_eq!(challenges[0].realm(), Some(&"myrealm".to_string()));
        
        let mut expected_auth_params = HashMap::new();
        expected_auth_params.insert(Some("realm".to_string()), "myrealm".to_string());
        expected_auth_params.insert(Some("nonce".to_string()), "fjalskdflwejrlaskdfjlaskdjflaksjdflkasdf".to_string());
        expected_auth_params.insert(Some("qop".to_string()), "auth".to_string());
        expected_auth_params.insert(Some("stale".to_string()), "FALSE".to_string());
        
        assert_eq!(challenges[0].auth_params, expected_auth_params);
    }

    #[test]
    pub fn test_digest_challenge_with_additional_spaces_before_first_auth_param() {
        let headers = Headers::builder()
            .add(
                "WWW-Authenticate",
                "Digest    realm=\"myrealm\", nonce=\"fjalskdflwejrlaskdfjlaskdjflaksjdflkasdf\", qop=\"auth\", stale=\"FALSE\"",
            )
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0].scheme, "Digest");
        assert_eq!(challenges[0].realm(), Some(&"myrealm".to_string()));
        
        let mut expected_auth_params = HashMap::new();
        expected_auth_params.insert(Some("realm".to_string()), "myrealm".to_string());
        expected_auth_params.insert(Some("nonce".to_string()), "fjalskdflwejrlaskdfjlaskdjflaksjdflkasdf".to_string());
        expected_auth_params.insert(Some("qop".to_string()), "auth".to_string());
        expected_auth_params.insert(Some("stale".to_string()), "FALSE".to_string());
        
        assert_eq!(challenges[0].auth_params, expected_auth_params);
    }

    #[test]
    pub fn test_digest_challenge_with_camel_cased_names() {
        let headers = Headers::builder()
            .add(
                "WWW-Authenticate",
                "DiGeSt qop=\"auth\", rEaLm=\"myrealm\", nonce=\"fjalskdflwejrlaskdfjlaskdjflaksjdflkasdf\", stale=\"FALSE\"",
            )
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0].scheme, "DiGeSt");
        assert_eq!(challenges[0].realm(), Some(&"myrealm".to_string()));
        
        let mut expected_auth_params = HashMap::new();
        expected_auth_params.insert(Some("realm".to_string()), "myrealm".to_string());
        expected_auth_params.insert(Some("nonce".to_string()), "fjalskdflwejrlaskdfjlaskdjflaksjdflkasdf".to_string());
        expected_auth_params.insert(Some("qop".to_string()), "auth".to_string());
        expected_auth_params.insert(Some("stale".to_string()), "FALSE".to_string());
        
        assert_eq!(challenges[0].auth_params, expected_auth_params);
    }

    #[test]
    pub fn test_digest_challenge_with_camel_cased_names2() {
        let headers = Headers::builder()
            .add(
                "WWW-Authenticate",
                "DIgEsT rEaLm=\"myrealm\", nonce=\"fjalskdflwejrlaskdfjlaskdjflaksjdflkasdf\", qop=\"auth\", stale=\"FALSE\"",
            )
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0].scheme, "DIgEsT");
        assert_eq!(challenges[0].realm(), Some(&"myrealm".to_string()));
        
        let mut expected_auth_params = HashMap::new();
        expected_auth_params.insert(Some("realm".to_string()), "myrealm".to_string());
        expected_auth_params.insert(Some("nonce".to_string()), "fjalskdflwejrlaskdfjlaskdjflaksjdflkasdf".to_string());
        expected_auth_params.insert(Some("qop".to_string()), "auth".to_string());
        expected_auth_params.insert(Some("stale".to_string()), "FALSE".to_string());
        
        assert_eq!(challenges[0].auth_params, expected_auth_params);
    }

    #[test]
    pub fn test_digest_challenge_with_token_form_of_auth_param() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Digest realm=myrealm")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0].scheme, "Digest");
        assert_eq!(challenges[0].realm(), Some(&"myrealm".to_string()));
        
        let mut expected = HashMap::new();
        expected.insert(Some("realm".to_string()), "myrealm".to_string());
        assert_eq!(challenges[0].auth_params, expected);
    }

    #[test]
    pub fn test_digest_challenge_without_auth_params() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Digest")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0].scheme, "Digest");
        assert!(challenges[0].realm().is_none());
        assert!(challenges[0].auth_params.is_empty());
    }

    #[test]
    pub fn basic_challenge() {
        let headers = Headers::builder()
            .add("WWW-Authenticate: Basic realm=\"protected area\"")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        let mut params = HashMap::new();
        params.insert(Some("realm".to_string()), "protected area".to_string());
        
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0], Challenge::new("Basic", params));
    }

    #[test]
    pub fn basic_challenge_with_charset() {
        let headers = Headers::builder()
            .add("WWW-Authenticate: Basic realm=\"protected area\", charset=\"UTF-8\"")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        let mut expected_auth_params = HashMap::new();
        expected_auth_params.insert(Some("realm".to_string()), "protected area".to_string());
        expected_auth_params.insert(Some("charset".to_string()), "UTF-8".to_string());
        
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0], Challenge::new("Basic", expected_auth_params));
    }

    #[test]
    pub fn basic_challenge_with_unexpected_charset() {
        let headers = Headers::builder()
            .add("WWW-Authenticate: Basic realm=\"protected area\", charset=\"US-ASCII\"")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        let mut expected_auth_params = HashMap::new();
        expected_auth_params.insert(Some("realm".to_string()), "protected area".to_string());
        expected_auth_params.insert(Some("charset".to_string()), "US-ASCII".to_string());
        
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0], Challenge::new("Basic", expected_auth_params));
    }

    #[test]
    pub fn separators_before_first_challenge() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", " ,  , Basic realm=myrealm")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        let mut params = HashMap::new();
        params.insert(Some("realm".to_string()), "myrealm".to_string());
        
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0], Challenge::new("Basic", params));
    }

    #[test]
    pub fn spaces_around_key_value_separator() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Basic realm = \"myrealm\"")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        let mut params = HashMap::new();
        params.insert(Some("realm".to_string()), "myrealm".to_string());
        
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0], Challenge::new("Basic", params));
    }

    #[test]
    pub fn multiple_challenges_in_one_header() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Basic realm = \"myrealm\",Digest")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        let mut params1 = HashMap::new();
        params1.insert(Some("realm".to_string()), "myrealm".to_string());
        
        assert_eq!(challenges.len(), 2);
        assert_eq!(challenges[0], Challenge::new("Basic", params1));
        assert_eq!(challenges[1], Challenge::new("Digest", HashMap::new()));
    }

    #[test]
    pub fn multiple_challenges_with_same_scheme_but_different_realm_in_one_header() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Basic realm = \"myrealm\",Basic realm=myotherrealm")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        let mut params1 = HashMap::new();
        params1.insert(Some("realm".to_string()), "myrealm".to_string());
        let mut params2 = HashMap::new();
        params2.insert(Some("realm".to_string()), "myotherrealm".to_string());
        
        assert_eq!(challenges.len(), 2);
        assert_eq!(challenges[0], Challenge::new("Basic", params1));
        assert_eq!(challenges[1], Challenge::new("Basic", params2));
    }

    #[test]
    pub fn separators_before_first_auth_param() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Digest, Basic ,,realm=\"myrealm\"")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        let mut params = HashMap::new();
        params.insert(Some("realm".to_string()), "myrealm".to_string());
        
        assert_eq!(challenges.len(), 2);
        assert_eq!(challenges[0], Challenge::new("Digest", HashMap::new()));
        assert_eq!(challenges[1], Challenge::new("Basic", params));
    }

    #[test]
    pub fn only_comma_between_challenges() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Digest,Basic realm=\"myrealm\"")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        let mut params = HashMap::new();
        params.insert(Some("realm".to_string()), "myrealm".to_string());
        
        assert_eq!(challenges.len(), 2);
        assert_eq!(challenges[0], Challenge::new("Digest", HashMap::new()));
        assert_eq!(challenges[1], Challenge::new("Basic", params));
    }

    #[test]
    pub fn multiple_separators_between_challenges() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Digest,,,, Basic ,,realm=\"myrealm\"")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        let mut params = HashMap::new();
        params.insert(Some("realm".to_string()), "myrealm".to_string());
        
        assert_eq!(challenges.len(), 2);
        assert_eq!(challenges[0], Challenge::new("Digest", HashMap::new()));
        assert_eq!(challenges[1], Challenge::new("Basic", params));
    }

    #[test]
    pub fn unknown_auth_params() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Digest,,,, Basic ,,foo=bar,realm=\"myrealm\"")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        let mut expected_auth_params = HashMap::new();
        expected_auth_params.insert(Some("realm".to_string()), "myrealm".to_string());
        expected_auth_params.insert(Some("foo".to_string()), "bar".to_string());
        
        assert_eq!(challenges.len(), 2);
        assert_eq!(challenges[0], Challenge::new("Digest", HashMap::new()));
        assert_eq!(challenges[1], Challenge::new("Basic", expected_auth_params));
    }

    #[test]
    pub fn escaped_characters_in_quoted_string() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Digest,,,, Basic ,,,realm=\"my\\\\\\\"r\\ealm\"")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        let mut params = HashMap::new();
        params.insert(Some("realm".to_string()), "my\\\"realm".to_string());
        
        assert_eq!(challenges.len(), 2);
        assert_eq!(challenges[0], Challenge::new("Digest", HashMap::new()));
        assert_eq!(challenges[1], Challenge::new("Basic", params));
    }

    #[test]
    pub fn comma_in_quoted_string_and_before_first_challenge() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", ",Digest,,,, Basic ,,,realm=\"my, realm,\"")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        let mut params = HashMap::new();
        params.insert(Some("realm".to_string()), "my, realm,".to_string());
        
        assert_eq!(challenges.len(), 2);
        assert_eq!(challenges[0], Challenge::new("Digest", HashMap::new()));
        assert_eq!(challenges[1], Challenge::new("Basic", params));
    }

    #[test]
    pub fn unescaped_double_quote_in_quoted_string_with_even_number_of_backslashes_in_front() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Digest,,,, Basic ,,,realm=\"my\\\\\\\\\"r\\ealm\"")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        // Based on Kotlin test, this should only return the first valid challenge
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0].scheme, "Digest");
    }

    #[test]
    pub fn unescaped_double_quote_in_quoted_string() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Digest,,,, Basic ,,,realm=\"my\"realm\"")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0].scheme, "Digest");
    }

    #[test]
    #[ignore = "TODO(jwilson): reject parameters that use invalid characters"]
    pub fn double_quote_in_token() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Digest,,,, Basic ,,,realm=my\"realm")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0].scheme, "Digest");
    }

    #[test]
    pub fn token68_instead_of_auth_params() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Other abc==")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        let mut params = HashMap::new();
        params.insert(None, "abc==".to_string());
        
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0], Challenge::new("Other", params));
    }

    #[test]
    pub fn token68_and_auth_params() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Other abc==, realm=myrealm")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        let mut params = HashMap::new();
        params.insert(None, "abc==".to_string());
        
        assert_eq!(challenges.len(), 1);
        assert_eq!(challenges[0], Challenge::new("Other", params));
    }

    #[test]
    pub fn repeated_auth_param_key() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Other realm=myotherrealm, realm=myrealm")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        // Based on Kotlin test, this returns an empty list
        assert!(challenges.is_empty());
    }

    #[test]
    pub fn multiple_authenticate_headers() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Digest")
            .add("WWW-Authenticate", "Basic realm=myrealm")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        let mut params = HashMap::new();
        params.insert(Some("realm".to_string()), "myrealm".to_string());
        
        assert_eq!(challenges.len(), 2);
        assert_eq!(challenges[0], Challenge::new("Digest", HashMap::new()));
        assert_eq!(challenges[1], Challenge::new("Basic", params));
    }

    #[test]
    pub fn multiple_authenticate_headers_in_different_order() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Basic realm=myrealm")
            .add("WWW-Authenticate", "Digest")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        let mut params = HashMap::new();
        params.insert(Some("realm".to_string()), "myrealm".to_string());
        
        assert_eq!(challenges.len(), 2);
        assert_eq!(challenges[0], Challenge::new("Basic", params));
        assert_eq!(challenges[1], Challenge::new("Digest", HashMap::new()));
    }

    #[test]
    pub fn multiple_basic_authenticate_headers() {
        let headers = Headers::builder()
            .add("WWW-Authenticate", "Basic realm=myrealm")
            .add("WWW-Authenticate", "Basic realm=myotherrealm")
            .build();
        let challenges = headers.parse_challenges("WWW-Authenticate");
        
        let mut params1 = HashMap::new();
        params1.insert(Some("realm".to_string()), "myrealm".to_string());
        let mut params2 = HashMap::new();
        params2.insert(Some("realm".to_string()), "myotherrealm".to_string());
        
        assert_eq!(challenges.len(), 2);
        assert_eq!(challenges[0], Challenge::new("Basic", params1));
        assert_eq!(challenges[1], Challenge::new("Basic", params2));
    }
}