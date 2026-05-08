use std::fmt;
use std::error::Error;
use crate::okhttp3::internal::http2::ErrorCode;

/// Thrown when an HTTP/2 stream is canceled without damage to the socket that carries it.
#[derive(Debug, Clone, PartialEq)]
pub struct StreamResetException {
    pub error_code: ErrorCode,
}

impl StreamResetException {
    pub fn new(error_code: ErrorCode) -> Self {
        Self { error_code }
    }
}

impl fmt::Display for StreamResetException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "stream was reset: {}", self.error_code)
    }
}

impl Error for StreamResetException {}