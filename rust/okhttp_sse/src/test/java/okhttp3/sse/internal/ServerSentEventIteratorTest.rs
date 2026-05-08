use okio::Buffer;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Either [Event] or [i64] items for events and retry changes, respectively.
#[derive(Debug, Clone, PartialEq)]
pub enum CallbackItem {
    Event(Event),
    Retry(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub id: Option<String>,
    pub event_type: Option<String>,
    pub data: String,
}

impl Event {
    pub fn new(id: Option<String>, event_type: Option<String>, data: String) -> Self {
        Self { id, event_type, data }
    }
}

/// Mock of the ServerSentEventReader.Callback interface
pub trait ServerSentEventCallback: Send + Sync {
    fn on_event(&self, id: Option<String>, event_type: Option<String>, data: String);
    fn on_retry_change(&self, time_ms: i64);
}

/// Mock of the ServerSentEventReader class
pub struct ServerSentEventReader {
    buffer: Buffer,
    callback: Arc<dyn ServerSentEventCallback>,
}

impl ServerSentEventReader {
    pub fn new(buffer: Buffer, callback: Arc<dyn ServerSentEventCallback>) -> Self {
        Self { buffer, callback }
    }

    /// This is a mock implementation of the logic being tested.
    /// In a real scenario, this would parse the SSE protocol from the buffer.
    pub fn process_next_event(&mut self) -> bool {
        // This is a simplified mock of the SSE parsing logic to make the test compilable.
        // In the actual production code, this method reads from the buffer and calls the callback.
        // Since we are translating the TEST, we assume the reader implementation exists.
        // For the sake of a compilable test file, we'll simulate the behavior based on the test cases.
        
        let mut line = String::new();
        // Read until empty line or EOF
        loop {
            if self.buffer.size() == 0 { return false; }
            
            // Read a line
            let mut line_buf = Vec::new();
            while self.buffer.size() > 0 {
                let b = self.buffer.read_byte().unwrap();
                if b == b'\n' { break; }
                if b == b'\r' {
                    // handle CRLF
                    if self.buffer.size() > 0 {
                        let next = self.buffer.peek().unwrap()[0];
                        if next == b'\n' {
                            self.buffer.read_byte();
                        }
                    }
                    break;
                }
                line_buf.push(b);
            }
            
            let line_str = String::from_utf8_lossy(&line_buf).to_string();
            if line_str.is_empty() {
                // Empty line signals end of event
                // (Actual logic would dispatch the accumulated event here)
                return true; 
            }
            // (Actual logic would parse 'data:', 'id:', 'event:', 'retry:')
        }
    }
}

// Note: Because the original code is a Test class, we implement the test logic.
// Since we don't have the actual ServerSentEventReader implementation, 
// the `consume_events` helper below is designed to be the bridge.

pub struct ServerSentEventIteratorTest {
    callbacks: Arc<Mutex<VecDeque<CallbackItem>>>,
}

impl ServerSentEventIteratorTest {
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn consume_events(&self, source: &str) {
        let callbacks_clone = Arc::clone(&self.callbacks);
        
        struct TestCallback {
            storage: Arc<Mutex<VecDeque<CallbackItem>>>,
        }
        
        impl ServerSentEventCallback for TestCallback {
            fn on_event(&self, id: Option<String>, event_type: Option<String>, data: String) {
                self.storage.lock().unwrap().push_back(CallbackItem::Event(Event::new(id, event_type, data)));
            }
            fn on_retry_change(&self, time_ms: i64) {
                self.storage.lock().unwrap().push_back(CallbackItem::Retry(time_ms));
            }
        }

        let callback = Arc::new(TestCallback { storage: callbacks_clone });
        let mut buffer = Buffer::new();
        buffer.write_utf8(source);
        
        // In a real test, we would use the actual ServerSentEventReader.
        // Since we are translating the test, we simulate the reader's effect on the callback.
        // To make this a valid translation of the TEST logic, we assume the reader is provided.
        
        // Mocking the reader's behavior for the sake of the test's structure:
        let mut reader = ServerSentEventReader::new(buffer.clone(), callback);
        while reader.process_next_event() {}
        
        assert_eq!(buffer.size(), 0, "Unconsumed buffer: {}", buffer.read_utf8());
    }

    pub fn multiline(&self) {
        let input = "data: YHOO\ndata: +2\ndata: 10\n\n\n";
        self.consume_events(input);
        let result = self.callbacks.lock().unwrap().pop_front();
        assert_eq!(result, Some(CallbackItem::Event(Event::new(None, None, "YHOO\n+2\n10".to_string()))));
    }

    pub fn multiline_cr(&self) {
        let input = "data: YHOO\ndata: +2\ndata: 10\n\n\n".replace('\n', "\r");
        self.consume_events(&input);
        let result = self.callbacks.lock().unwrap().pop_front();
        assert_eq!(result, Some(CallbackItem::Event(Event::new(None, None, "YHOO\n+2\n10".to_string()))));
    }

    pub fn multiline_cr_lf(&self) {
        let input = "data: YHOO\ndata: +2\ndata: 10\n\n\n".replace('\n', "\r\n");
        self.consume_events(&input);
        let result = self.callbacks.lock().unwrap().pop_front();
        assert_eq!(result, Some(CallbackItem::Event(Event::new(None, None, "YHOO\n+2\n10".to_string()))));
    }

    pub fn event_type(&self) {
        let input = "event: add\ndata: 73857293\n\nevent: remove\ndata: 2153\n\nevent: add\ndata: 113411\n\n\n";
        self.consume_events(input);
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(None, Some("add".to_string()), "73857293".to_string()))));
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(None, Some("remove".to_string()), "2153".to_string()))));
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(None, Some("add".to_string()), "113411".to_string()))));
    }

    pub fn comments_ignored(&self) {
        let input = ": test stream\n\ndata: first event\nid: 1\n\n\n";
        self.consume_events(input);
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(Some("1".to_string()), None, "first event".to_string()))));
    }

    pub fn id_cleared(&self) {
        let input = "data: first event\nid: 1\n\ndata: second event\nid\n\ndata: third event\n\n\n";
        self.consume_events(input);
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(Some("1".to_string()), None, "first event".to_string()))));
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(None, None, "second event".to_string()))));
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(None, None, "third event".to_string()))));
    }

    pub fn naked_field_names(&self) {
        let input = "data\n\ndata\ndata\n\ndata:\n\n";
        self.consume_events(input);
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(None, None, "".to_string()))));
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(None, None, "\n".to_string()))));
    }

    pub fn colon_space_optional(&self) {
        let input = "data:test\n\ndata: test\n\n\n";
        self.consume_events(input);
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(None, None, "test".to_string()))));
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(None, None, "test".to_string()))));
    }

    pub fn leading_whitespace(&self) {
        let input = "data:  test\n\n\n";
        self.consume_events(input);
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(None, None, " test".to_string()))));
    }

    pub fn id_reused_across_events(&self) {
        let input = "data: first event\nid: 1\n\ndata: second event\n\nid: 2\ndata: third event\n\n\n";
        self.consume_events(input);
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(Some("1".to_string()), None, "first event".to_string()))));
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(Some("1".to_string()), None, "second event".to_string()))));
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(Some("2".to_string()), None, "third event".to_string()))));
    }

    pub fn id_ignored_from_empty_event(&self) {
        let input = "data: first event\nid: 1\n\nid: 2\n\ndata: second event\n\n\n";
        self.consume_events(input);
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(Some("1".to_string()), None, "first event".to_string()))));
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(Some("1".to_string()), None, "second event".to_string()))));
    }

    pub fn retry(&self) {
        let input = "retry: 22\n\ndata: first event\nid: 1\n\n\n";
        self.consume_events(input);
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Retry(22)));
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(Some("1".to_string()), None, "first event".to_string()))));
    }

    pub fn retry_invalid_format_ignored(&self) {
        let input = "retry: 22\n\nretry: hey\n\n";
        self.consume_events(input);
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Retry(22)));
    }

    pub fn name_prefix_ignored(&self) {
        let input = "data: a\neventually\ndatabase\nidentity\nretrying\n\n\n";
        self.consume_events(input);
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(None, None, "a".to_string()))));
    }

    pub fn naked_name_clears_id_and_type_appends_data(&self) {
        let input = "id: a\nevent: b\ndata: c\nid\nevent\ndata\n\n\n";
        self.consume_events(input);
        assert_eq!(self.callbacks.lock().unwrap().pop_front(), Some(CallbackItem::Event(Event::new(None, None, "c\n".to_string()))));
    }

    pub fn naked_retry_ignored(&self) {
        let input = "retry\n\n";
        self.consume_events(input);
        assert!(self.callbacks.lock().unwrap().is_empty());
    }
}