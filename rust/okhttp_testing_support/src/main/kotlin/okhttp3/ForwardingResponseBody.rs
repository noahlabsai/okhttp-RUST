use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::ResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::MediaType;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::ResponseBodyTest::*;

// ForwardingResponseBody is a decorator for ResponseBody that allows overriding its methods.
// In Rust, since ResponseBody is implemented as a wrapper around a trait (ResponseBodyTrait),
// this is implemented as a struct that holds a ResponseBodyImpl.
#[derive(Debug, Clone)]
pub struct ForwardingResponseBody {
    delegate: ResponseBodyImpl,
}

impl ForwardingResponseBody {
    // Creates a new ForwardingResponseBody.
    // 
    // # Panics
    // Panics if the delegate is null (handled by Option in Rust, but preserved as per Kotlin's requireNotNull).
    pub fn new(delegate: Option<ResponseBodyImpl>) -> Self {
        let delegate = delegate.expect("delegate == null");
        Self { delegate }
    }

    // Returns the underlying ResponseBody delegate.
    pub fn delegate(&self) -> &ResponseBodyImpl {
        &self.delegate
    }

    // Returns the content type of the delegate.
    pub fn content_type(&self) -> Option<MediaType> {
        self.delegate.content_type()
    }

    // Returns the content length of the delegate.
    pub fn content_length(&self) -> i64 {
        self.delegate.content_length()
    }

    // Returns the source of the delegate.
    pub fn source(&self) -> Box<dyn BufferedSource> {
        self.delegate.source()
    }
}

impl std::fmt::Display for ForwardingResponseBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Kotlin's javaClass.simpleName is "ForwardingResponseBody"
        write!(f, "ForwardingResponseBody({})", self.delegate)
    }
}

impl std::fmt::Debug for ForwardingResponseBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ForwardingResponseBody")
            .field("delegate", &self.delegate)
            .finish()
    }
}

// To maintain the "open class" behavior and inheritance from ResponseBody, 
// we ensure ForwardingResponseBody can be used wherever a ResponseBodyImpl is expected,
// or we implement a trait if ResponseBody was a trait. 
// Given the provided context, ResponseBodyImpl is the concrete implementation.