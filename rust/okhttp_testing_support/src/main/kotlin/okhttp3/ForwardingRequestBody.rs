use std::error::Error;
use std::io::Write;
use std::sync::Arc;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;

// ForwardingRequestBody is a decorator for RequestBody that allows overriding its methods.
// In Rust, this is implemented as a struct that holds an Arc of the delegate RequestBody.
#[derive(Debug, Clone)]
pub struct ForwardingRequestBody {
    delegate: Arc<dyn RequestBody>,
}

impl ForwardingRequestBody {
    // Creates a new ForwardingRequestBody.
    // 
    // # Panics
    // Panics if the delegate is null (handled by Option in Rust, but the Kotlin 
    // source uses requireNotNull).
    pub fn new(delegate: Option<Arc<dyn RequestBody>>) -> Self {
        let delegate = delegate.expect("delegate == null");
        Self { delegate }
    }

    // Returns the delegate RequestBody.
    pub fn delegate(&self) -> Arc<dyn RequestBody> {
        Arc::clone(&self.delegate)
    }
}

impl RequestBody for ForwardingRequestBody {
    // Returns the Content-Type header for this body.
    fn content_type(&self) -> Option<Arc<crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::MediaType>> {
        self.delegate.content_type()
    }

    // Returns the number of bytes that will be written to sink in a call to write_to,
    // or -1 if that count is unknown.
    fn content_length(&self) -> i64 {
        self.delegate.content_length()
    }

    // Writes the content of this request to sink.
    fn write_to(&self, sink: &mut dyn Write) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.delegate.write_to(sink)
    }

    // A duplex request body is special in how it is transmitted on the network.
    fn is_duplex(&self) -> bool {
        self.delegate.is_duplex()
    }

    // Returns true if this body expects at most one call to write_to.
    fn is_one_shot(&self) -> bool {
        self.delegate.is_one_shot()
    }
}

impl std::fmt::Display for ForwardingRequestBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Kotlin: javaClass.simpleName + "(" + delegate.toString() + ")"
        // In Rust, we use the struct name explicitly.
        write!(f, "ForwardingRequestBody({})", self.delegate)
    }
}
)}
