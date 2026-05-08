use std::fmt;
use std::sync::Mutex;

// WindowCounter tracks the flow control window for a specific HTTP/2 stream.
pub struct WindowCounter {
    pub stream_id: i32,
    // We use a Mutex to preserve the @Synchronized behavior from Kotlin.
    // The state is grouped into a inner struct to be protected by a single lock.
    state: Mutex<WindowCounterState>,
}

#[derive(Debug, Clone, PartialEq)]
struct WindowCounterState {
    // The total number of bytes consumed.
    total: i64,
    // The total number of bytes acknowledged by outgoing `WINDOW_UPDATE` frames.
    acknowledged: i64,
}

impl WindowCounter {
    pub fn new(stream_id: i32) -> Self {
        Self {
            stream_id,
            state: Mutex::new(WindowCounterState {
                total: 0,
                acknowledged: 0,
            }),
        }
    }

    // The total number of bytes consumed.
    pub fn total(&self) -> i64 {
        let state = self.state.lock().unwrap();
        state.total
    }

    // The total number of bytes acknowledged by outgoing `WINDOW_UPDATE` frames.
    pub fn acknowledged(&self) -> i64 {
        let state = self.state.lock().unwrap();
        state.acknowledged
    }

    // The total number of bytes consumed minus the acknowledged bytes.
    pub fn unacknowledged(&self) -> i64 {
        let state = self.state.lock().unwrap();
        state.total - state.acknowledged
    }

    // Updates the total and acknowledged byte counts.
    // 
    // # Panics
    // Panics if `total` or `acknowledged` are negative, or if the resulting 
    // acknowledged count exceeds the total count.
    pub fn update(&self, total: i64, acknowledged: i64) {
        assert!(total >= 0, "total must be non-negative");
        assert!(acknowledged >= 0, "acknowledged must be non-negative");

        let mut state = self.state.lock().unwrap();
        state.total += total;
        state.acknowledged += acknowledged;

        assert!(
            state.acknowledged <= state.total,
            "acknowledged bytes cannot exceed total bytes"
        );
    }
}

impl fmt::Display for WindowCounter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.state.lock().unwrap();
        let unacknowledged = state.total - state.acknowledged;
        write!(
            f,
            "WindowCounter(streamId={}, total={}, acknowledged={}, unacknowledged={})",
            self.stream_id, state.total, state.acknowledged, unacknowledged
        )
    }
}

impl fmt::Debug for WindowCounter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use the Display implementation to match Kotlin's toString() behavior
        fmt::write(f, format_args!("{}", self))
    }
}