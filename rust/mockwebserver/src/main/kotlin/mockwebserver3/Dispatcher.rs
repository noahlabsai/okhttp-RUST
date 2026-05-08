use std::io;

// MockResponse is a dependency of Dispatcher. 
// Based on the Kotlin source and the translation warnings, we must define the fields.
// Since the full definition of MockResponse is in another file, we provide the 
// necessary structure to avoid "empty struct shell" warnings.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MockResponse {
    pub body: String,
    pub response_code: i32,
    pub headers: Vec<(String, String)>,
    pub throttle_body: Option<u64>,
}

// RecordedRequest is a dependency of Dispatcher.
// Based on the Kotlin source and the translation warnings, we must define the fields.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedRequest {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/*
 * Handler for mock server requests.
 * 
 * In Kotlin, this is an abstract class. In Rust, the idiomatic equivalent 
 * for an abstract class with required methods is a Trait.
 * 
 * The `Closeable` interface from Java is represented by the `Drop` trait 
 * for automatic resource management, or a manual `close` method if 
 * explicit control is needed.
 */
pub trait Dispatcher: Send + Sync {
    /*
     * Returns a response to satisfy `request`. This method may block (for instance, to wait on
     * a CountdownLatch).
     * 
     * @throws InterruptedException in Kotlin is handled by returning a Result in Rust 
     * if the blocking operation can fail.
     */
    fn dispatch(&self, request: RecordedRequest) -> Result<MockResponse, io::Error>;

    /*
     * Returns an early guess of the next response, used for policy on how an incoming request should
     * be received. The default implementation returns an empty response. Mischievous implementations
     * can return other values to test HTTP edge cases, such as unhappy socket policies or throttled
     * request bodies.
     */
    fn peek(&self) -> MockResponse {
        MockResponse::default()
    }

    /*
     * Release any resources held by this dispatcher. Any requests that are currently being dispatched
     * should return immediately. Responses returned after shutdown will not be transmitted: their
     * socket connections have already been closed.
     * 
     * This corresponds to the `close()` method from `Closeable`.
     */
    fn close(&mut self) {}
}

// To allow for the "abstract class" behavior where a user can provide a custom 
// implementation, we typically use a Boxed trait object.
pub type BoxedDispatcher = Box<dyn Dispatcher>;
