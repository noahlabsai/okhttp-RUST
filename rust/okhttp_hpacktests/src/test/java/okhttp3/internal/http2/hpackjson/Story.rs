use serde::{Deserialize, Serialize};
use std::fmt;

/// Representation of one story, a set of request headers to encode or decode. This class is used
/// reflectively with Moshi to parse stories from files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Story {
    pub description: Option<String>,
    pub cases: Vec<Case>,
    pub file_name: Option<String>,
}

impl Story {
    /// Companion object equivalent for the MISSING constant.
    pub fn missing() -> Self {
        Story {
            description: Some("Missing".to_string()),
            cases: Vec::new(),
            file_name: Some("missing".to_string()),
        }
    }
}

/// Implementation of toString() to be used as the test name.
impl fmt::Display for Story {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.file_name {
            Some(name) => write!(f, "{}", name),
            None => write!(f, "?"),
        }
    }
}

/// Case definition required by Story. 
/// Note: The original Kotlin snippet references `Case` but does not define it.
/// Based on the context of HPACK tests, Case typically contains input/output headers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Case {
    pub description: Option<String>,
    pub input: Vec<Header>,
    pub output: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

/// Global constant equivalent to Story.Companion.MISSING
pub lazy_static::lazy_static! {
    pub static ref STORY_MISSING: Story = Story::missing();
}