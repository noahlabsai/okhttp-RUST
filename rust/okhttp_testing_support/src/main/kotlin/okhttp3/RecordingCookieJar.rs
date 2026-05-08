use std::collections::VecDeque;
use std::sync::Mutex;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{Cookie, CookieJar, HttpUrl};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

// A `CookieJar` that records cookies for testing purposes.
// 
// It allows enqueuing expected request cookies and capturing response cookies
// to verify the behavior of the HTTP client.
pub struct RecordingCookieJar {
    // Using Mutex because CookieJar trait methods take &self, 
    // but we need to mutate the internal queues.
    request_cookies: Mutex<VecDeque<Vec<Cookie>>>,
    response_cookies: Mutex<VecDeque<Vec<Cookie>>>,
}

impl RecordingCookieJar {
    pub fn new() -> Self {
        Self {
            request_cookies: Mutex::new(VecDeque::new()),
            response_cookies: Mutex::new(VecDeque::new()),
        }
    }

    // Enqueues a list of cookies to be returned by `load_for_request`.
    pub fn enqueue_request_cookies(&self, cookies: Vec<Cookie>) {
        let mut lock = self.request_cookies.lock().unwrap();
        lock.push_back(cookies);
    }

    // Removes and returns the first list of cookies captured from a response.
    pub fn take_response_cookies(&self) -> Vec<Cookie> {
        let mut lock = self.response_cookies.lock().unwrap();
        lock.pop_front().unwrap_or_default()
    }

    // Asserts that the next captured response cookies match the provided strings.
    pub fn assert_response_cookies(&self, expected_cookies: Vec<Option<String>>) {
        let actual_cookies = self.take_response_cookies();
        let actual_strings: Vec<String> = actual_cookies
            .into_iter()
            .map(|c| format!("{}", c))
            .collect();

        let expected_strings: Vec<String> = expected_cookies
            .into_iter()
            .map(|opt| opt.unwrap_or_else(|| "null".to_string()))
            .collect();

        assert_eq!(actual_strings, expected_strings, "Response cookies did not match expected values");
    }
}

impl CookieJar for RecordingCookieJar {
    fn save_from_response(&self, _url: &HttpUrl, cookies: &[Cookie]) {
        let mut lock = self.response_cookies.lock().unwrap();
        lock.push_back(cookies.to_vec());
    }

    fn load_for_request(&self, _url: &HttpUrl) -> Vec<Cookie> {
        let mut lock = self.request_cookies.lock().unwrap();
        lock.pop_front().unwrap_or_default()
    }
}

impl Default for RecordingCookieJar {
    fn default() -> Self {
        Self::new()
    }
}
