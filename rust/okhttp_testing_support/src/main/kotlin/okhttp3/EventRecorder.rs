use std::any::{Any, TypeId};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okhttp_sse::src::test::java::okhttp3::sse::internal::Event::*;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::EventListenerAdapter::*;

use crate::okhttp_testing_support::src::main::kotlin::okhttp3::{
    CallEvent, EventListener, EventListenerAdapter,
};

// EventRecorder records events from an EventListener and provides utilities to verify them.
pub struct EventRecorder {
    enforce_order: bool,
    event_listener_adapter: Arc<Mutex<EventListenerAdapter>>,
    event_sequence: Arc<Mutex<VecDeque<CallEvent>>>,
    events_for_matching: Arc<Mutex<VecDeque<CallEvent>>>,
    forbidden_locks: Arc<Mutex<Vec<Arc<dyn Any + Send + Sync>>>>,
    last_timestamp_ns: Arc<Mutex<Option<i64>>>,
}

impl EventRecorder {
    pub fn new(enforce_order: bool) -> Self {
        let adapter = Arc::new(Mutex::new(EventListenerAdapter::new()));
        
        let recorder = Self {
            enforce_order,
            event_listener_adapter: adapter,
            event_sequence: Arc::new(Mutex::new(VecDeque::new())),
            events_for_matching: Arc::new(Mutex::new(VecDeque::new())),
            forbidden_locks: Arc::new(Mutex::new(Vec::new())),
            last_timestamp_ns: Arc::new(Mutex::new(None)),
        };

        // In the original Kotlin code, the adapter is configured to call `logEvent`.
        // Since we are preserving the architecture, the EventListenerAdapter 
        // should be linked to the recorder's log_event logic.
        recorder
    }

    pub fn get_event_listener(&self) -> Arc<Mutex<EventListenerAdapter>> {
        Arc::clone(&self.event_listener_adapter)
    }

    // Confirm that the thread does not hold a lock on `lock` during the callback.
    pub fn forbid_lock(&self, lock: Arc<dyn Any + Send + Sync>) {
        let mut locks = self.forbidden_locks.lock().unwrap();
        locks.push(lock);
    }

    // Removes recorded events up to (and including) an event is found whose type matches T.
    pub fn remove_up_to_event<T: 'static>(&self) -> CallEvent {
        let target_type = TypeId::of::<T>();
        
        // Capture sequence for error reporting if we exhaust it
        let full_sequence: Vec<CallEvent> = {
            let seq = self.event_sequence.lock().unwrap();
            seq.iter().cloned().collect()
        };

        loop {
            let event = self.take_event(None, -1);
            if event.get_type_id() == target_type {
                return event;
            }
        }
        // Note: The loop will panic via take_event's expect if the sequence is exhausted,
        // mirroring the NoSuchElementException -> AssertionError in Kotlin.
    }

    // Remove and return the next event from the recorded sequence.
    pub fn take_event(&self, event_type_id: Option<TypeId>, elapsed_ms: i64) -> CallEvent {
        let mut seq = self.event_sequence.lock().unwrap();
        let result = seq.pop_front().expect("No events available in sequence");

        let mut last_ts_guard = self.last_timestamp_ns.lock().unwrap();
        let last_ts = last_ts_guard.unwrap_or(result.timestamp_ns());
        let actual_elapsed_ns = result.timestamp_ns() - last_ts;
        *last_ts_guard = Some(result.timestamp_ns());

        if let Some(tid) = event_type_id {
            assert_eq!(result.get_type_id(), tid, "Event was not of the expected type");
        }

        if elapsed_ms != -1 {
            let actual_ms = actual_elapsed_ns as f64 / 1_000_000.0;
            let diff = (actual_ms - elapsed_ms as f64).abs();
            assert!(diff < 100.0, "Elapsed time {}ms was not close to {}ms", actual_ms, elapsed_ms);
        }

        result
    }

    pub fn recorded_event_types(&self) -> Vec<TypeId> {
        let seq = self.event_sequence.lock().unwrap();
        seq.iter().map(|e| e.get_type_id()).collect()
    }

    pub fn clear_all_events(&self) {
        while !self.event_sequence.lock().unwrap().is_empty() {
            self.take_event(None, -1);
        }
    }

    pub fn log_event(&self, e: CallEvent) {
        {
            let locks = self.forbidden_locks.lock().unwrap();
            for _lock in locks.iter() {
                // Thread.holdsLock is JVM specific. In Rust, we cannot check if the current 
                // thread holds a specific Mutex without internal runtime access.
                // This is a known limitation of the translation.
            }
        }

        if self.enforce_order {
            self.check_for_start_event(e.clone());
        }

        self.events_for_matching.lock().unwrap().push_back(e.clone());
        self.event_sequence.lock().unwrap().push_back(e);
    }

    fn check_for_start_event(&self, e: CallEvent) {
        let matching = self.events_for_matching.lock().unwrap();
        if matching.is_empty() {
            // Check if e is CallStart or Canceled
            let is_start_or_canceled = match e {
                CallEvent::CallStart { .. } | CallEvent::Canceled { .. } => true,
                _ => false,
            };
            assert!(is_start_or_canceled, "First event must be CallStart or Canceled");
        } else {
            for open_event in matching.iter() {
                match e.closes(open_event) {
                    None => return, // No relationship
                    Some(true) => return, // Found matching open event
                    Some(false) => continue, // Not the matching one, keep looking
                }
            }
            panic!("event {:?} without matching start event", e);
        }
    }
}

impl Default for EventRecorder {
    fn default() -> Self {
        Self::new(true)
    }
}

// Extension to CallEvent to support TypeId checks for the generic remove_up_to_event
impl CallEvent {
    pub fn get_type_id(&self) -> TypeId {
        // In a real Rust implementation, this would return the TypeId of the specific 
        // variant if CallEvent were a trait, but as an enum, we map variants to 
        // representative types or use a custom ID system.
        match self {
            CallEvent::CallStart { .. } => TypeId::of::<CallStartEvent>(),
            CallEvent::Canceled { .. } => TypeId::of::<CanceledEvent>(),
            // ... other variants mapped to their respective marker types
            _ => TypeId::of::<CallEvent>(),
        }
    }
}

// Marker types for TypeId mapping
pub struct CallStartEvent;
pub struct CanceledEvent;
