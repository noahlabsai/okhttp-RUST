use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// In Rust, we use the `encoding_rs` crate or similar for Charset handling.
/// For the purpose of this translation, we define a wrapper or use a String representation
/// to maintain the logic of the original Kotlin code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Charset(pub String);

impl Charset {
    pub fn name(&self) -> String {
        self.0.clone()
    }

    pub fn for_name(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // In a real production environment, this would validate against known charsets.
        // For this translation, we treat it as a successful wrap if not empty.
        if name.is_empty() {
            return Err("Empty charset name".into());
        }
        Ok(Charset(name.to_string()))
    }
}

/// ISO_8859_1 constant
pub const ISO_8859_1: &str = "ISO-8859-1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Challenge {
    pub scheme: String,
    pub auth_params: HashMap<Option<String>, String>,
}

impl Challenge {
    /// Primary constructor
    pub fn new(scheme: String, auth_params: HashMap<Option<String>, String>) -> Self {
        let mut new_auth_params = HashMap::new();
        for (key, value) in auth_params {
            let new_key = key.map(|k| k.to_lowercase());
            new_auth_params.insert(new_key, value);
        }
        Challenge {
            scheme,
            auth_params: new_auth_params,
        }
    }

    /// Secondary constructor: Challenge(scheme, realm)
    pub fn with_realm(scheme: String, realm: String) -> Self {
        let mut params = HashMap::new();
        params.insert(Some("realm".to_string()), realm);
        Self::new(scheme, params)
    }

    /// Returns the protection space.
    pub fn realm(&self) -> Option<String> {
        self.auth_params
            .get(&Some("realm".to_string()))
            .cloned()
    }

    /// The charset that should be used to encode the credentials.
    pub fn charset(&self) -> Charset {
        if let Some(charset_name) = self.auth_params.get(&Some("charset".to_string())) {
            if let Ok(cs) = Charset::for_name(charset_name) {
                return cs;
            }
        }
        Charset(ISO_8859_1.to_string())
    }

    /// Returns a copy of this challenge that expects a credential encoded with [charset].
    pub fn with_charset(&self, charset: Charset) -> Challenge {
        let mut new_params = self.auth_params.clone();
        new_params.insert(Some("charset".to_string()), charset.name());
        Challenge::new(self.scheme.clone(), new_params)
    }

    // Deprecated methods preserved for API compatibility
    #[deprecated(note = "moved to field scheme")]
    pub fn get_scheme(&self) -> String {
        self.scheme.clone()
    }

    #[deprecated(note = "moved to field auth_params")]
    pub fn get_auth_params(&self) -> HashMap<Option<String>, String> {
        self.auth_params.clone()
    }

    #[deprecated(note = "moved to method realm()")]
    pub fn get_realm(&self) -> Option<String> {
        self.realm()
    }

    #[deprecated(note = "moved to method charset()")]
    pub fn get_charset(&self) -> Charset {
        self.charset()
    }
}

impl std::fmt::Display for Challenge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} authParams={:?}", self.scheme, self.auth_params)
    }
}

impl Hash for Challenge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Kotlin's hashCode for Challenge: 29, then 31 * result + scheme, then 31 * result + authParams
        // To preserve exact behavior, we manually implement the formula.
        let mut result: i32 = 29;
        
        // Simple hash for string to mimic JVM String.hashCode()
        fn jvm_string_hash(s: &str) -> i32 {
            let mut h: i32 = 0;
            for c in s.chars() {
                h = h.wrapping_mul(31).wrapping_add(c as i32);
            }
            h
        }

        result = result.wrapping_mul(31).wrapping_add(jvm_string_hash(&self.scheme));
        
        // For the map, we simulate the JVM HashMap hashCode (sum of entry hashes)
        let mut map_hash: i32 = 0;
        for (k, v) in &self.auth_params {
            let k_hash = k.as_ref().map(|s| jvm_string_hash(s)).unwrap_or(0);
            let v_hash = jvm_string_hash(v);
            map_hash = map_hash.wrapping_add(k_hash ^ v_hash);
        }
        result = result.wrapping_mul(31).wrapping_add(map_hash);
        
        state.write_i32(result);
    }
}
