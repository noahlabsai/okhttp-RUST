use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::io::{Error, ErrorKind};

// --- Mocking the necessary framework types from the Kotlin source ---

// Represents the JUnit Description
pub struct Description {
    pub name: String,
}

impl Description {
    pub const EMPTY: Description = Description {
        name: String::new(),
    };
}

// Represents the JUnit Statement
pub trait Statement {
    fn evaluate(&self) -> Result<(), Box<dyn std::error::Error>>;
}

// Mock implementation of MockWebServer
pub struct MockWebServer {
    is_running: Arc<AtomicBool>,
}

impl MockWebServer {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self) {
        self.is_running.store(true, Ordering::SeqCst);
    }

    pub fn shutdown(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    pub fn url(&self, _path: &str) -> MockUrl {
        MockUrl {
            server: self.is_running.clone(),
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}

pub struct MockUrl {
    server: Arc<AtomicBool>,
}

impl MockUrl {
    pub fn to_url(&self) -> MockConnection {
        MockConnection {
            server_running: self.server.clone(),
        }
    }
}

pub struct MockConnection {
    server_running: Arc<AtomicBool>,
}

impl MockConnection {
    pub fn open_connection(&self) -> MockSocket {
        MockSocket {
            server_running: self.server_running.clone(),
        }
    }
}

pub struct MockSocket {
    server_running: Arc<AtomicBool>,
}

impl MockSocket {
    pub fn connect(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.server_running.load(Ordering::SeqCst) {
            return Err(Box::new(Error::new(
                ErrorKind::ConnectionRefused,
                "ConnectException: Connection refused",
            )));
        }
        Ok(())
    }
}

// Mock implementation of MockWebServerRule
pub struct MockWebServerRule {
    pub server: MockWebServer,
}

impl MockWebServerRule {
    pub fn new() -> Self {
        Self {
            server: MockWebServer::new(),
        }
    }

    pub fn apply<S: Statement + 'static>(
        &self,
        statement: S,
        _description: Description,
    ) -> Box<dyn Statement> {
        let server = Arc::new(self.server.clone()); // Simplified for mock
        // In a real JUnit rule, this would wrap the statement with start/stop logic
        // We simulate the behavior here.
        
        // Note: In Rust, we need a wrapper to handle the lifecycle
        Box::new(RuleStatement {
            inner: statement,
            server: server.clone(),
        })
    }
}

// Helper to simulate the Rule's behavior of starting and stopping the server
struct RuleStatement<S: Statement> {
    inner: S,
    server: Arc<MockWebServer>,
}

impl<S: Statement> Statement for RuleStatement<S> {
    fn evaluate(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.server.start();
        let result = self.inner.evaluate();
        self.server.shutdown();
        result
    }
}

// --- Test Implementation ---

pub struct MockWebServerRuleTest;

impl MockWebServerRuleTest {
    #[test]
    pub fn statement_starts_and_stops() {
        let rule = MockWebServerRule::new();
        let called = Arc::new(AtomicBool::new(false));
        
        let called_clone = called.clone();
        let rule_server_ref = Arc::new(rule.server.clone()); // To access inside the closure

        // Define the anonymous Statement
        struct TestStatement {
            called: Arc<AtomicBool>,
            server: Arc<MockWebServer>,
        }

        impl Statement for TestStatement {
            fn evaluate(&self) -> Result<(), Box<dyn std::error::Error>> {
                self.called.store(true, Ordering::SeqCst);
                self.server
                    .url("/")
                    .to_url()
                    .open_connection()
                    .connect()?;
                Ok(())
            }
        }

        let statement_impl = TestStatement {
            called: called_clone,
            server: rule_server_ref,
        };

        let statement = rule.apply(statement_impl, Description::EMPTY);
        
        // Execute the statement
        statement.evaluate().expect("Statement should evaluate successfully");

        // assertThat(called.get()).isTrue()
        assert!(called.load(Ordering::SeqCst), "The statement should have been called");

        // Verify that the server is stopped and connection fails
        let result = rule.server
            .url("/")
            .to_url()
            .open_connection()
            .connect();

        match result {
            Ok(_) => panic!("fail(): Expected ConnectException but connection succeeded"),
            Err(e) => {
                // Verify it is a "ConnectException" (ConnectionRefused in Rust)
                if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                    assert_eq!(io_err.kind(), ErrorKind::ConnectionRefused);
                } else {
                    panic!("Expected io::Error (ConnectException), got {:?}", e);
                }
            }
        }
    }
}

// To allow the test to run in a standard Rust environment
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statement_starts_and_stops() {
        MockWebServerRuleTest::statement_starts_and_stops();
    }
}

// Manual implementation of Clone for MockWebServer to support the Rule logic
impl Clone for MockWebServer {
    fn clone(&self) -> Self {
        Self {
            is_running: self.is_running.clone(),
        }
    }
}
}
