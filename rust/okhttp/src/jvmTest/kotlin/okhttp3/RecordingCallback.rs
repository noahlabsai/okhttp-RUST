use std::error::Error;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{Call, Response, HttpUrl, Request, Callback};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::CallTagsTest::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::InterceptorTest::*;

// Records received HTTP responses so they can be later retrieved by tests.


struct RecordingCallbackState {
    responses: Vec<RecordedResponse>,
}


impl RecordingCallback {
    pub const TIMEOUT_MILLIS: u64 = 10_000;

    pub fn new() -> Self {
        Self {
            state: Arc::new((
                Mutex::new(RecordingCallbackState {
                    responses: Vec::new(),
                }),
                Condvar::new(),
            )),
        }
    }

    // Returns the recorded response triggered by `url`. Panics if the response isn't
    // enqueued before the timeout.
    pub fn await(&self, url: HttpUrl) -> RecordedResponse {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().unwrap();
        
        let timeout = Duration::from_millis(Self::TIMEOUT_MILLIS);
        let start = Instant::now();

        loop {
            // Search for the response with the matching URL
            let index = state.responses.iter().position(|r| r.request.url() == url);
            
            if let Some(i) = index {
                return state.responses.remove(i);
            }

            let elapsed = start.elapsed();
            if elapsed >= timeout {
                break;
            }

            // Wait for notification or timeout
            let remaining = timeout - elapsed;
            let result = cvar.wait_timeout(state, remaining).unwrap();
            state = result.0;
        }

        panic!("Timed out waiting for response to {}", url);
    }
}

impl Callback for RecordingCallback {
    fn onFailure(&self, call: &Call, e: Box<dyn Error + Send + Sync>) {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().unwrap();
        
        state.responses.push(RecordedResponse {
            request: call.request().clone(),
            response: None,
            body: None,
            error: Some(e),
        });
        
        cvar.notify_all();
    }

    fn onResponse(&self, call: &Call, response: Response) {
        let (lock, cvar) = &*self.state;
        
        // In Kotlin, response.body.string() consumes the body.
        let body = response.body().and_then(|b| b.string().ok());
        
        let mut state = lock.lock().unwrap();
        state.responses.push(RecordedResponse {
            request: call.request().clone(),
            response: Some(response),
            body,
            error: None,
        });
        
        cvar.notify_all();
    }
}
