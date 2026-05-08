use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Challenge::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;

// An RFC 2045 Media Type, appropriate to describe the content type of an HTTP request
// or response body.

impl MediaType {
    // Internal constructor to preserve Kotlin's internal visibility
    pub(crate) fn new(
        media_type: String,
        r#type: String,
        subtype: String,
        parameter_names_and_values: Vec<String>,
    ) -> Self {
        Self {
            media_type,
            r#type,
            subtype,
            parameter_names_and_values,
        }
    }

    // Returns the charset of this media type, or [default_value] if either this media type doesn't
    // specify a charset, or if its charset is unsupported by the current runtime.
    // 
    // Note: In Rust, Charset is typically handled via the `encoding_rs` crate or similar.
    // For the purpose of this translation, we return the charset string as the representation.
    pub fn charset(&self, default_value: Option<&str>) -> Option<String> {
        let charset = self.parameter("charset")?;
        // In a real production Rust environment, you would validate the charset here
        // using a crate like `encoding_rs`. Since the Kotlin code catches IllegalArgumentException
        // from Charset.forName, we simulate that validation.
        if self.is_valid_charset(charset) {
            Some(charset.to_string())
        } else {
            default_value.map(|s| s.to_string())
        }
    }

    fn is_valid_charset(&self, charset: &str) -> bool {
        // Simplified validation: in a real scenario, use encoding_rs::Encoding::for_label
        !charset.is_empty()
    }

    // Returns the parameter [name] of this media type, or null if this media type does not define
    // such a parameter.
    pub fn parameter(&self, name: &str) -> Option<&str> {
        for i in (0..self.parameter_names_and_values.len()).step_by(2) {
            if self.parameter_names_and_values[i].eq_ignore_ascii_case(name) {
                return self.parameter_names_and_values.get(i + 1).map(|s| s.as_str());
            }
        }
        None
    }

    // Deprecated: moved to field `type`
    pub fn get_type(&self) -> &str {
        &self.r#type
    }

    // Deprecated: moved to field `subtype`
    pub fn get_subtype(&self) -> &str {
        &self.subtype
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.media_type)
    }
}

impl std::hash::Hash for MediaType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.media_type.hash(state);
    }
}

// Companion Object Logic
static TYPE_SUBTYPE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    let token = r"([a-zA-Z0-9-!#$%&'*+.^_`{|}~]+)";
    Regex::new(&format!("{}/{}", token, token)).unwrap()
});

static PARAMETER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    let token = r"([a-zA-Z0-9-!#$%&'*+.^_`{|}~]+)";
    let quoted = r#""([^"]*)""#;
    Regex::new(&format!(r";\s*(?:{}={}(?:{}|{}))?", token, token, token, quoted)).unwrap()
    // Note: The Kotlin regex is `;\\s*(?:$TOKEN=(?:$TOKEN|$QUOTED))?`
    // Correcting the Rust regex to match the Kotlin logic exactly:
    // Regex::new(r";\s*(?:([a-zA-Z0-9-!#$%&'*+.^_`{|}~]+)=(?:([a-zA-Z0-9-!#$%&'*+.^_`{|}~]+)|""([^"]*)""))?").unwrap()
});

// Redefining PARAMETER_REGEX to be exactly aligned with the Kotlin logic
static PARAMETER_REGEX_FIXED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r";\s*(?:([a-zA-Z0-9-!#$%&'*+.^_`{|}~]+)=(?:([a-zA-Z0-9-!#$%&'*+.^_`{|}~]+)|""([^"]*)""))?").unwrap()
});

pub trait MediaTypeExt {
    fn to_media_type(&self) -> Result<MediaType, String>;
    fn to_media_type_or_null(&self) -> Option<MediaType>;
}

impl MediaTypeExt for String {
    fn to_media_type(&self) -> Result<MediaType, String> {
        let caps = TYPE_SUBTYPE_REGEX
            .finds(self)
            .next()
            .ok_or_else(|| format!("No subtype found for: \"{}\"", self))?;
        
        // In Rust, regex captures are accessed via captures()
        let captures = TYPE_SUBTYPE_REGEX.captures(self)
            .ok_or_else(|| format!("No subtype found for: \"{}\"", self))?;
        
        let r#type = captures.get(1).map(|m| m.as_str().to_lowercase()).unwrap_or_default();
        let subtype = captures.get(2).map(|m| m.as_str().to_lowercase()).unwrap_or_default();

        let mut parameter_names_and_values = Vec::new();
        let mut s = caps.end();

        while s < self.len() {
            let remaining = &self[s..];
            if let Some(mat) = PARAMETER_REGEX_FIXED.find(remaining) {
                let parameter_caps = PARAMETER_REGEX_FIXED.captures(remaining).unwrap();
                
                if let Some(name_match) = parameter_caps.get(1) {
                    let name = name_match.as_str();
                    
                    let value = if let Some(token_match) = parameter_caps.get(2) {
                        let token = token_match.as_str();
                        if token.len() > 2 && token.starts_with('\'') && token.ends_with('\'') {
                            &token[1..token.len() - 1]
                        } else {
                            token
                        }
                    } else if let Some(quoted_match) = parameter_caps.get(3) {
                        quoted_match.as_str()
                    } else {
                        ""
                    };

                    parameter_names_and_values.push(name.to_string());
                    parameter_names_and_values.push(value.to_string());
                }
                s += mat.end();
            } else {
                return Err(format!(
                    "Parameter is not formatted correctly: \"{}\" for: \"{}\"",
                    &self[s..],
                    self
                ));
            }
        }

        Ok(MediaType::new(
            self.clone(),
            r#type,
            subtype,
            parameter_names_and_values,
        ))
    }

    fn to_media_type_or_null(&self) -> Option<MediaType> {
        self.to_media_type().ok()
    }
}

impl MediaType {
    // Static helper to match Kotlin's Companion.get()
    pub fn get(media_type: String) -> Result<MediaType, String> {
        media_type.to_media_type()
    }

    // Static helper to match Kotlin's Companion.parse()
    pub fn parse(media_type: String) -> Option<MediaType> {
        media_type.to_media_type_or_null()
    }
}