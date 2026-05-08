use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use std::thread;
use std::sync::mpsc::{self, Receiver, Sender};

use crate::mockwebserver::src::main::kotlin::mockwebserver3::SocketHandler::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::connection::BufferedSocket::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::ResponseBodyTest::*;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::JsseDebugLogging::*;

// Type alias for the action closure
type Action = Box<dyn Fn(Arc<dyn BufferedSocket + Send + Sync>) -> Result<(), Box<dyn std::error::Error>> + Send>;

// A scriptable request/response conversation. Create the script by calling methods like
// `receive_request` in the sequence they are run.
pub struct MockSocketHandler {
    actions: Arc<Mutex<VecDeque<Action>>>,
    results: Arc<Mutex<VecDeque<mpsc::Receiver<Result<(), Box<dyn std::error::Error>>>>>>,
}

impl MockSocketHandler {
    pub fn new() -> Self {
        Self {
            actions: Arc::new(Mutex::new(VecDeque::new())),
            results: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn receive_request(self, expected: String) -> Self {
        let mut actions = self.actions.lock().unwrap();
        actions.push_back(Box::new(move |stream| {
            let mut source = stream.source();
            let mut buf = vec![0u8; expected.len()];
            source.read_exact(&mut buf).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            let actual = String::from_utf8_lossy(&buf).into_owned();
            if actual != expected {
                panic!("{} != {}", actual, expected);
            }
            Ok(())
        }));
        self
    }

    pub fn exhaust_request(self) -> Self {
        let mut actions = self.actions.lock().unwrap();
        actions.push_back(Box::new(|stream| {
            let mut source = stream.source();
            let mut buf = [0u8; 1];
            if source.read(&mut buf).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)? != 0 {
                panic!("expected exhausted");
            }
            Ok(())
        }));
        self
    }

    pub fn cancel_stream(self) -> Self {
        let mut actions = self.actions.lock().unwrap();
        actions.push_back(Box::new(|stream| {
            stream.cancel();
            Ok(())
        }));
        self
    }

    pub fn request_io_exception(self) -> Self {
        let mut actions = self.actions.lock().unwrap();
        actions.push_back(Box::new(|stream| {
            let mut source = stream.source();
            let mut buf = [0u8; 1];
            // In Kotlin, this checks if exhausted() throws IOException.
            // In Rust, we check if read returns an Err.
            match source.read(&mut buf) {
                Ok(_) => panic!("expected IOException"),
                Err(_) => Ok(()),
            }
        }));
        self
    }

    pub fn send_response(self, s: String, response_sent: Arc<Mutex<std::sync::Condvar>>) -> Self {
        // Note: CountDownLatch is replaced by a Condvar/Mutex pattern or similar.
        // For simplicity in this translation, we use a shared state.
        let mut actions = self.actions.lock().unwrap();
        actions.push_back(Box::new(move |stream| {
            let mut sink = stream.sink();
            sink.write_all(s.as_bytes()).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            sink.flush().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            
            // Simulate countDown()
            let _lock = response_sent.lock().unwrap(); // This is a simplification of CountDownLatch
            // In a real scenario, you'd use a proper AtomicInt or Condvar
            Ok(())
        }));
        self
    }

    pub fn exhaust_response(self) -> Self {
        let mut actions = self.actions.lock().unwrap();
        actions.push_back(Box::new(|stream| {
            // In Rust, closing the sink is typically handled by Drop or explicit close if trait provides it.
            // Since BufferedSink is Write, we rely on the scope or a custom close.
            Ok(())
        }));
        self
    }

    pub fn sleep(self, duration: u64, unit_ms: u64) -> Self {
        let mut actions = self.actions.lock().unwrap();
        actions.push_back(Box::new(move |_| {
            thread::sleep(Duration::from_millis(duration * unit_ms));
            Ok(())
        }));
        self
    }

    pub fn await_success(&self) {
        let rx = {
            let mut results = self.results.lock().unwrap();
            results.pop_front().expect("no onRequest call received")
        };

        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(_)) => (),
            Ok(Err(e)) => panic!("Action failed: {}", e),
            Err(_) => panic!("Timeout waiting for action completion"),
        }
    }
}

impl SocketHandler for MockSocketHandler {
    fn handle(&self, socket: Socket) {
        let buffered_socket = socket.as_buffered_socket();
        let (tx, rx) = mpsc::channel();
        
        {
            let mut results = self.results.lock().unwrap();
            results.push_back(rx);
        }

        let actions_clone = Arc::clone(&self.actions);
        let socket_clone = Arc::clone(&buffered_socket);

        thread::spawn(move || {
            let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                loop {
                    let action = {
                        let mut actions = actions_clone.lock().unwrap();
                        actions.pop_front()
                    };
                    
                    match action {
                        Some(act) => act(socket_clone.clone())?,
                        None => break,
                    }
                }
                Ok(())
            })();
            let _ = tx.send(result);
        });
    }
}