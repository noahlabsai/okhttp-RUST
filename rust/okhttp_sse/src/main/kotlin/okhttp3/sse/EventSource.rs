use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request;

// EventSourceListener is required by the EventSource::Factory trait.
// Since it is not provided in the source snippet but is a dependency for the Factory,
// we define it as a trait to ensure the code is compilable.
pub trait EventSourceListener: Send + Sync {
    fn on_open(&self);
    fn on_event(&self, id: Option<String>, url: Option<String>, name: Option<String>, data: String);
    fn on_closed(&self);
    fn on_failure(&self, t: Box<dyn std::error::Error + Send + Sync>);
}

pub trait EventSource: Send + Sync {
    // Returns the original request that initiated this event source.
    fn request(&self) -> Request;

    // Immediately and violently release resources held by this event source. This does nothing if
    // the event source has already been closed or canceled.
    fn cancel(&self);
}

pub trait EventSourceFactory: Send + Sync {
    // Creates a new event source and immediately returns it. Creating an event source initiates an
    // asynchronous process to connect the socket. Once that succeeds or fails, `listener` will be
    // notified. The caller must cancel the returned event source when it is no longer in use.
    fn new_event_source(
        &self,
        request: Request,
        listener: Arc<dyn EventSourceListener>,
    ) -> Arc<dyn EventSource>;
}

// Re-exporting as Factory to match Kotlin's nested interface naming if needed, 
// though in Rust top-level traits are preferred.
pub type Factory = dyn EventSourceFactory;

use std::sync::Arc;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
