use std::fmt;
use std::io::{Error, ErrorKind};
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;

// Protocols that OkHttp implements for ALPN selection.
//
// ## Protocol vs Scheme
//
// Despite its name, java.net.URL.getProtocol returns the scheme (http,
// https, etc.) of the URL, not the protocol (http/1.1, spdy/3.1, etc.). OkHttp uses the word
// *protocol* to identify how HTTP messages are framed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    // An obsolete plaintext framing that does not use persistent sockets by default.
    Http10,

    // A plaintext framing that includes persistent connections.
    //
    // This version of OkHttp implements RFC 7230, and tracks revisions to that spec.
    Http11,

    // Chromium's binary-framed protocol that includes header compression, multiplexing multiple
    // requests on the same socket, and server-push. HTTP/1.1 semantics are layered on SPDY/3.
    //
    // Current versions of OkHttp do not support this protocol.
    #[deprecated(note = "OkHttp has dropped support for SPDY. Prefer Protocol::Http2.")]
    Spdy3,

    // The IETF's binary-framed protocol that includes header compression, multiplexing multiple
    // requests on the same socket, and server-push. HTTP/1.1 semantics are layered on HTTP/2.
    //
    // HTTP/2 requires deployments of HTTP/2 that use TLS 1.2 support
    // [CipherSuite.TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256], present in Java 8+ and Android 5+.
    // Servers that enforce this may send an exception message including the string
    // `INADEQUATE_SECURITY`.
    Http2,

    // Cleartext HTTP/2 with no "upgrade" round trip. This option requires the client to have prior
    // knowledge that the server supports cleartext HTTP/2.
    H2PriorKnowledge,

    // QUIC (Quick UDP Internet Connection) is a new multiplexed and secure transport atop UDP,
    // designed from the ground up and optimized for HTTP/2 semantics. HTTP/1.1 semantics are layered
    // on HTTP/2.
    //
    // QUIC is not natively supported by OkHttp, but provided to allow a theoretical interceptor that
    // provides support.
    Quic,

    // HTTP/3 is the third and upcoming major version of the Hypertext Transfer Protocol used to
    // exchange information. HTTP/3 runs over QUIC, which is published as RFC 9000.
    //
    // HTTP/3 is not natively supported by OkHttp, but provided to allow a theoretical interceptor
    // that provides support.
    Http3,
}

pub const Http10: Protocol = Protocol::Http10;
pub const Http11: Protocol = Protocol::Http11;
pub const Spdy3: Protocol = Protocol::Spdy3;
pub const Http2: Protocol = Protocol::Http2;
pub const H2PriorKnowledge: Protocol = Protocol::H2PriorKnowledge;
pub const Quic: Protocol = Protocol::Quic;
pub const Http3: Protocol = Protocol::Http3;

impl Default for Protocol {
    fn default() -> Self {
        Protocol::Http10
    }
}

pub const HTTP_1_0: Protocol = Protocol::Http10;
pub const HTTP_1_1: Protocol = Protocol::Http11;
pub const SPDY_3: Protocol = Protocol::Spdy3;
pub const HTTP_2: Protocol = Protocol::Http2;
pub const H2_PRIOR_KNOWLEDGE: Protocol = Protocol::H2PriorKnowledge;
pub const QUIC: Protocol = Protocol::Quic;
pub const HTTP_3: Protocol = Protocol::Http3;

impl Protocol {
    // Returns the string used to identify this protocol for ALPN, like "http/1.1", "spdy/3.1" or
    // "h2".
    pub fn as_str(&self) -> &'static str {
        match self {
            Protocol::Http10 => "http/1.0",
            Protocol::Http11 => "http/1.1",
            Protocol::Spdy3 => "spdy/3.1",
            Protocol::Http2 => "h2",
            Protocol::H2PriorKnowledge => "h2_prior_knowledge",
            Protocol::Quic => "quic",
            Protocol::Http3 => "h3",
        }
    }

    // Returns the protocol identified by `protocol`.
    //
    // # Errors
    // Returns `std::io::Error` if `protocol` is unknown.
    pub fn get(protocol: &str) -> Result<Self, Error> {
        // Unroll the loop over values() to save an allocation.
        if protocol == Protocol::Http10.as_str() {
            Ok(Protocol::Http10)
        } else if protocol == Protocol::Http11.as_str() {
            Ok(Protocol::Http11)
        } else if protocol == Protocol::H2PriorKnowledge.as_str() {
            Ok(Protocol::H2PriorKnowledge)
        } else if protocol == Protocol::Http2.as_str() {
            Ok(Protocol::Http2)
        } else if protocol == Protocol::Spdy3.as_str() {
            Ok(Protocol::Spdy3)
        } else if protocol == Protocol::Quic.as_str() {
            Ok(Protocol::Quic)
        } else {
            // Support HTTP3 draft like h3-29
            if protocol.starts_with(Protocol::Http3.as_str()) {
                Ok(Protocol::Http3)
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    format!("Unexpected protocol: {}", protocol),
                ))
            }
        }
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
