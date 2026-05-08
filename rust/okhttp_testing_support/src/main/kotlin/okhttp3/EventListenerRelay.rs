use std::sync::{Arc, Mutex};
use crate::okhttp3::{Call, EventListener, EventListenerAdapter, EventRecorder, CallEvent};
use crate::okhttp::src::jvmTest::kotlin::okhttp3::CallTagsTest::*;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::EventListenerAdapter::*;

// A special [EventListener] for testing the mechanics of event listeners.
//
// Each instance processes a single event on [call], and then adds a successor [EventListenerRelay]
// on the same [call] to process the next event.
//
// By forcing the list of listeners to change after every event, we can detect if buggy code caches
// a stale [EventListener] in a field or local variable.
pub struct EventListenerRelay {
    pub call: Arc<Call>,
    pub event_recorder: Arc<EventRecorder>,
    event_listener_adapter: EventListenerAdapter,
    pub event_count: Mutex<i32>,
}

impl EventListenerRelay {
    pub fn new(call: Arc<Call>, event_recorder: Arc<EventRecorder>) -> Arc<Self> {
        let relay = Arc::new(Self {
            call: call.clone(),
            event_recorder: event_recorder.clone(),
            event_listener_adapter: EventListenerAdapter::new(),
            event_count: Mutex::new(0),
        });

        // In Kotlin: .apply { listeners += ::onEvent }
        let relay_clone = Arc::clone(&relay);
        
        // We use a closure to bridge the EventListenerAdapter's callback to the relay's on_event method.
        relay.event_listener_adapter.add_listener(move |call_event: CallEvent| {
            relay_clone.on_event(call_event);
        });

        relay
    }

    // Returns the event listener associated with this relay.
    // Corresponds to the Kotlin 'val eventListener: EventListener get() = eventListenerAdapter'
    pub fn event_listener(&self) -> &EventListenerAdapter {
        &self.event_listener_adapter
    }

    fn on_event(&self, call_event: CallEvent) {
        let mut count = self.event_count.lock().unwrap();
        let current_count = *count;
        *count += 1;

        if current_count == 0 {
            self.event_recorder.log_event(call_event);
            
            // Create the successor relay
            let next = EventListenerRelay::new(
                Arc::clone(&self.call), 
                Arc::clone(&self.event_recorder)
            );
            
            // Add the successor's listener to the call
            self.call.add_event_listener(next.event_listener());
        }
    }
}
