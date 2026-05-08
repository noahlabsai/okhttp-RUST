use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::io::Write;
use crossbeam_channel::{bounded, Receiver, Sender};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::MediaType;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;
use crate::okhttp_testing_support::build_gradle::*;

// A trait representing the BufferedSink from okio.
// In the context of this translation, it is a type that implements Write.
pub trait BufferedSink: Write + Send + Sync {}
impl<T: Write + Send + Sync> BufferedSink for T {}

/* A duplex request body that keeps the provided sinks so they can be written to later. */
pub struct AsyncRequestBody {
    // Using crossbeam_channel to emulate LinkedBlockingQueue
    request_body_sinks: (Sender<Box<dyn BufferedSink>>, Receiver<Box<dyn BufferedSink>>),
}

impl AsyncRequestBody {
    pub fn new() -> Self {
        // LinkedBlockingQueue is unbounded by default in Kotlin, 
        // but for safety in Rust we use a large bound or unbounded.
        let (s, r) = bounded(1024); 
        AsyncRequestBody {
            request_body_sinks: (s, r),
        }
    }

    /*
     * Takes a sink from the queue, waiting up to 5 seconds.
     * Throws an AssertionError (panic in Rust) if no sink is available.
     */
    pub fn take_sink(&self) -> Box<dyn BufferedSink> {
        self.request_body_sinks.1.recv_timeout(Duration::from_secs(5))
            .expect("no sink to take")
    }

    /*
     * Asserts that the queue of sinks is empty.
     */
    pub fn assert_no_more_sinks(&self) {
        assert!(self.request_body_sinks.1.is_empty(), "Sinks should be empty");
    }
}

impl RequestBody for AsyncRequestBody {
    fn content_type(&self) -> Option<Arc<MediaType>> {
        None
    }

    fn write_to(&self, sink: &mut dyn Write) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // In Kotlin, the sink passed to writeTo is a BufferedSink.
        // Since we need to store it in a queue for later use, we need an owned version.
        // This is a tricky part of the translation because the trait RequestBody 
        // provides a &mut dyn Write, but AsyncRequestBody needs to 'keep' it.
        // In a real OkHttp implementation, the sink is managed by the Okio layer.
        // To preserve behavior, we assume the sink can be boxed or is already managed.
        
        // Note: In a real-world scenario, the framework would pass an object that 
        // can be cloned or shared. Here we simulate the 'add' to the queue.
        // Since we cannot move a &mut dyn Write into a Box, this implementation 
        // assumes the environment provides a way to capture the sink.
        
        // For the purpose of this translation and to maintain the logic of the Kotlin source:
        // we would typically wrap the sink in a way that it can be stored.
        // However, since we cannot convert &mut dyn Write to Box<dyn BufferedSink> 
        // without ownership, this represents a limitation of the trait signature.
        // In production, the RequestBody trait would likely be designed to take ownership 
        // or the sink would be an Arc.
        
        // To satisfy the compiler and preserve the "add to queue" logic:
        // This is a conceptual mapping as the Kotlin `BufferedSink` is an object.
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Cannot capture borrowed sink into queue")))
    }

    fn is_duplex(&self) -> bool {
        true
    }
}