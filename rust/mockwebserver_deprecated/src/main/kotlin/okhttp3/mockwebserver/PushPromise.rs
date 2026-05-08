use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers;
use crate::mockwebserver_deprecated::src::main::kotlin::okhttp3::mockwebserver::MockResponse;
use crate::mockwebserver_deprecated::src::main::kotlin::okhttp3::mockwebserver::Dispatcher::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::PushPromise::*;


impl PushPromise {
    pub fn new(method: String, path: String, headers: Headers, response: MockResponse) -> Self {
        Self {
            method,
            path,
            headers,
            response,
        }
    }

    // @deprecated moved to val
    pub fn method(&self) -> &str {
        &self.method
    }

    // @deprecated moved to val
    pub fn path(&self) -> &str {
        &self.path
    }

    // @deprecated moved to val
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    // @deprecated moved to val
    pub fn response(&self) -> &MockResponse {
        &self.response
    }
}
