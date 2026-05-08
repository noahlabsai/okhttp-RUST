use std::error::Error;

/// Custom error types to mirror the Java/Kotlin exception hierarchy used in the original source.
/// In a real production Rust environment, these would likely be part of a larger error enum
/// or provided by a TLS library (like rustls or native-tls).
#[derive(Debug)]
pub enum OkHttpError {
    ProtocolException(String),
    InterruptedIOException(String),
    SslHandshakeException {
        message: String,
        cause: Option<Box<dyn Error + Send + Sync>>,
    },
    SslPeerUnverifiedException(String),
    SslException(String),
    IoException(String),
    Other(Box<dyn Error + Send + Sync>),
}

impl std::fmt::Display for OkHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OkHttpError::ProtocolException(s) => write!(f, "ProtocolException: {}", s),
            OkHttpError::InterruptedIOException(s) => write!(f, "InterruptedIOException: {}", s),
            OkHttpError::SslHandshakeException { message, .. } => write!(f, "SSLHandshakeException: {}", message),
            OkHttpError::SslPeerUnverifiedException(s) => write!(f, "SSLPeerUnverifiedException: {}", s),
            OkHttpError::SslException(s) => write!(f, "SSLException: {}", s),
            OkHttpError::IoException(s) => write!(f, "IOException: {}", s),
            OkHttpError::Other(e) => write!(f, "Error: {}", e),
        }
    }
}

impl Error for OkHttpError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            OkHttpError::SslHandshakeException { cause, .. } => cause.as_ref().map(|e| e.as_ref() as &(dyn Error + 'static)),
            OkHttpError::Other(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

/// Mirroring the CertificateException for the cause check in SSLHandshakeException.
#[derive(Debug)]
pub struct CertificateException(pub String);
impl std::fmt::Display for CertificateException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CertificateException: {}", self.0)
    }
}
impl Error for CertificateException {}

/// Returns true if a TLS connection should be retried after [e].
pub fn retry_tls_handshake(e: &OkHttpError) -> bool {
    match e {
        // If there was a protocol problem, don't recover.
        OkHttpError::ProtocolException(_) => false,

        // If there was an interruption or timeout (SocketTimeoutException), don't recover.
        // For the socket connect timeout case we do not try the same host with a different
        // ConnectionSpec: we assume it is unreachable.
        OkHttpError::InterruptedIOException(_) => false,

        // If the problem was a CertificateException from the X509TrustManager, do not retry.
        OkHttpError::SslHandshakeException { cause, .. } => {
            if let Some(cause_err) = cause {
                if cause_err.is::<CertificateException>() {
                    return false;
                }
            }
            // If it's a handshake exception but NOT caused by a CertificateException,
            // it falls through to the SSLException logic (which returns true).
            true
        }

        // e.g. a certificate pinning error.
        OkHttpError::SslPeerUnverifiedException(_) => false,

        // Retry for all other SSL failures.
        OkHttpError::SslException(_) => true,

        // Default case for other IOExceptions or generic errors.
        _ => false,
    }
}