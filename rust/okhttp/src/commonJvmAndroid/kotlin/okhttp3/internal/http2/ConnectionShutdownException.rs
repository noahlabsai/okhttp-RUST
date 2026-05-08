use std::error::Error;
use std::fmt;

/// Thrown when an HTTP/2 connection is shutdown (either explicitly or if the peer has sent a GOAWAY
/// frame) and an attempt is made to use the connection.
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionShutdownException;

impl fmt::Display for ConnectionShutdownException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Connection shutdown")
    }
}

impl Error for ConnectionShutdownException {}