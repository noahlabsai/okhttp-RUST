use std::fmt;
use std::io::{Error, ErrorKind};

/// Translation of okhttp3.Protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    HTTP_1_0,
    HTTP_1_1,
    SPDY_3,
    HTTP_2,
    H2_PRIOR_KNOWLEDGE,
    QUIC,
    HTTP_3,
}

impl Protocol {
    pub const HTTP_1_0: Protocol = Protocol::HTTP_1_0;
    pub const HTTP_1_1: Protocol = Protocol::HTTP_1_1;
    pub const SPDY_3: Protocol = Protocol::SPDY_3;
    pub const HTTP_2: Protocol = Protocol::HTTP_2;
    pub const H2_PRIOR_KNOWLEDGE: Protocol = Protocol::H2_PRIOR_KNOWLEDGE;
    pub const QUIC: Protocol = Protocol::QUIC;
    pub const HTTP_3: Protocol = Protocol::HTTP_3;

    /// Companion object method: Protocol.get(string)
    pub fn get(protocol_string: &str) -> Result<Protocol, Error> {
        match protocol_string {
            "http/1.0" => Ok(Protocol::HTTP_1_0),
            "http/1.1" => Ok(Protocol::HTTP_1_1),
            "spdy/3.1" => Ok(Protocol::SPDY_3),
            "h2" => Ok(Protocol::HTTP_2),
            "h2_prior_knowledge" => Ok(Protocol::H2_PRIOR_KNOWLEDGE),
            "quic" => Ok(Protocol::QUIC),
            "h3" | "h3-29" => Ok(Protocol::HTTP_3),
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Unknown protocol: {}", protocol_string),
            )),
        }
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Protocol::HTTP_1_0 => "http/1.0",
            Protocol::HTTP_1_1 => "http/1.1",
            Protocol::SPDY_3 => "spdy/3.1",
            Protocol::HTTP_2 => "h2",
            Protocol::H2_PRIOR_KNOWLEDGE => "h2_prior_knowledge",
            Protocol::QUIC => "quic",
            Protocol::HTTP_3 => "h3",
        };
        write!(f, "{}", s)
    }
}

/// Translation of okhttp3.ProtocolTest
pub struct ProtocolTest;

impl ProtocolTest {
    #[cfg(test)]
    pub fn test_get_known() {
        assert_eq!(Protocol::get("http/1.0").unwrap(), Protocol::HTTP_1_0);
        assert_eq!(Protocol::get("http/1.1").unwrap(), Protocol::HTTP_1_1);
        assert_eq!(Protocol::get("spdy/3.1").unwrap(), Protocol::SPDY_3);
        assert_eq!(Protocol::get("h2").unwrap(), Protocol::HTTP_2);
        assert_eq!(Protocol::get("h2_prior_knowledge").unwrap(), Protocol::H2_PRIOR_KNOWLEDGE);
        assert_eq!(Protocol::get("quic").unwrap(), Protocol::QUIC);
        assert_eq!(Protocol::get("h3").unwrap(), Protocol::HTTP_3);
        assert_eq!(Protocol::get("h3-29").unwrap(), Protocol::HTTP_3);
    }

    #[cfg(test)]
    pub fn test_get_unknown() {
        let result = Protocol::get("tcp");
        assert!(result.is_err());
    }

    #[cfg(test)]
    pub fn test_to_string() {
        assert_eq!(Protocol::HTTP_1_0.to_string(), "http/1.0");
        assert_eq!(Protocol::HTTP_1_1.to_string(), "http/1.1");
        assert_eq!(Protocol::SPDY_3.to_string(), "spdy/3.1");
        assert_eq!(Protocol::HTTP_2.to_string(), "h2");
        assert_eq!(Protocol::H2_PRIOR_KNOWLEDGE.to_string(), "h2_prior_knowledge");
        assert_eq!(Protocol::QUIC.to_string(), "quic");
        assert_eq!(Protocol::HTTP_3.to_string(), "h3");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_known() {
        ProtocolTest::test_get_known();
    }

    #[test]
    fn test_get_unknown() {
        ProtocolTest::test_get_unknown();
    }

    #[test]
    fn test_to_string() {
        ProtocolTest::test_to_string();
    }
}