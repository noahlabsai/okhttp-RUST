use std::io;
use std::sync::Arc;
use regex::Regex;
use chrono::{TimeZone, Utc};

// These types are expected to be defined in the okhttp3 crate.
// We use imports or stubs to ensure the file is self-contained for review.
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{Request, Response, Headers, HttpUrl, WebSocket};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;

// Note: The original Kotlin code is a test utility class. 
// We implement the logic as a Rust struct with assertion methods.

#[derive(Debug)]
pub struct RecordedResponse {
    pub request: Request,
    pub response: Option<Response>,
    pub web_socket: Option<WebSocket>,
    pub body: Option<String>,
    pub failure: Option<Arc<io::Error>>,
}

impl RecordedResponse {
    pub fn new(
        request: Request,
        response: Option<Response>,
        web_socket: Option<WebSocket>,
        body: Option<String>,
        failure: Option<Arc<io::Error>>,
    ) -> Self {
        Self {
            request,
            response,
            web_socket,
            body,
            failure,
        }
    }

    pub fn assert_request_url(self, url: HttpUrl) -> Self {
        assert_eq!(self.request.url, url);
        self
    }

    pub fn assert_request_method(self, method: String) -> Self {
        assert_eq!(self.request.method, method);
        self
    }

    pub fn assert_request_header(self, name: String, values: &[String]) -> Self {
        assert_eq!(self.request.headers(name), values);
        self
    }

    pub fn assert_code(self, expected_code: i32) -> Self {
        let response = self.response.as_ref().expect("Response should not be null");
        assert_eq!(response.code, expected_code);
        self
    }

    pub fn assert_successful(self) -> Self {
        assert!(self.failure.is_none(), "Failure should be null");
        let response = self.response.as_ref().expect("Response should not be null");
        assert!(response.is_successful);
        self
    }

    pub fn assert_not_successful(self) -> Self {
        let response = self.response.as_ref().expect("Response should not be null");
        assert!(!response.is_successful);
        self
    }

    pub fn assert_header(self, name: String, values: &[Option<String>]) -> Self {
        let response = self.response.as_ref().expect("Response should not be null");
        let actual = response.headers(name);
        let expected: Vec<String> = values.iter().map(|v| v.clone().unwrap_or_default()).collect();
        assert_eq!(actual, expected);
        self
    }

    pub fn assert_headers(self, headers: Headers) -> Self {
        let response = self.response.as_ref().expect("Response should not be null");
        assert_eq!(response.headers, headers);
        self
    }

    pub fn assert_body(self, expected_body: String) -> Self {
        assert_eq!(self.body, Some(expected_body));
        self
    }

    pub fn assert_handshake(self) -> Self {
        let response = self.response.as_ref().expect("Response should not be null");
        let handshake = response.handshake.as_ref().expect("Handshake should not be null");
        assert!(handshake.tls_version.is_some());
        assert!(handshake.cipher_suite.is_some());
        assert!(handshake.peer_principal.is_some());
        assert_eq!(handshake.peer_certificates.len(), 1);
        assert!(handshake.local_principal.is_none());
        assert_eq!(handshake.local_certificates.len(), 0);
        self
    }

    // Asserts that the current response was redirected and returns the prior response.
    pub fn prior_response(self) -> RecordedResponse {
        let response = self.response.as_ref().expect("Response should not be null");
        let prior = response.prior_response.as_ref().expect("Prior response should not be null");
        RecordedResponse::new(
            prior.request.clone(),
            Some(prior.clone()),
            None,
            None,
            None,
        )
    }

    // Asserts that the current response used the network and returns the network response.
    pub fn network_response(self) -> RecordedResponse {
        let response = self.response.as_ref().expect("Response should not be null");
        let network = response.network_response.as_ref().expect("Network response should not be null");
        RecordedResponse::new(
            network.request.clone(),
            Some(network.clone()),
            None,
            None,
            None,
        )
    }

    // Asserts that the current response didn't use the network.
    pub fn assert_no_network_response(self) -> Self {
        let response = self.response.as_ref().expect("Response should not be null");
        assert!(response.network_response.is_none());
        self
    }

    // Asserts that the current response didn't use the cache.
    pub fn assert_no_cache_response(self) -> Self {
        let response = self.response.as_ref().expect("Response should not be null");
        assert!(response.cache_response.is_none());
        self
    }

    // Asserts that the current response used the cache and returns the cache response.
    pub fn cache_response(self) -> RecordedResponse {
        let response = self.response.as_ref().expect("Response should not be null");
        let cache = response.cache_response.as_ref().expect("Cache response should not be null");
        RecordedResponse::new(
            cache.request.clone(),
            Some(cache.clone()),
            None,
            None,
            None,
        )
    }

    pub fn assert_failure_type(self, allowed_types: &[std::any::TypeId]) -> Self {
        let mut found = false;
        if let Some(ref _failure) = self.failure {
            let actual_type = std::any::TypeId::of::<io::Error>();
            for expected in allowed_types {
                if expected == &actual_type {
                    found = true;
                    break;
                }
            }
        }
        assert!(found, "Expected exception type among allowed types, got {:?}", self.failure);
        self
    }

    pub fn assert_failure_messages(self, messages: &[String]) -> Self {
        let failure = self.failure.as_ref().expect("No failure found");
        let msg = failure.to_string();
        assert!(messages.contains(&msg), "Failure message {} not in allowed list", msg);
        self
    }

    pub fn assert_failure_matches(self, patterns: &[String]) -> Self {
        let failure = self.failure.as_ref().expect("No failure found");
        let message = failure.to_string();
        let found = patterns.iter().any(|p| {
            let re = Regex::new(p).expect("Invalid regex pattern");
            re.is_match(&message)
        });
        assert!(found, "Failure message {} did not match any patterns", message);
        self
    }

    pub fn assert_sent_request_at_millis(self, minimum: i64, maximum: i64) -> Self {
        let response = self.response.as_ref().expect("Response should not be null");
        self.assert_date_in_range(minimum, response.sent_request_at_millis, maximum);
        self
    }

    pub fn assert_received_response_at_millis(self, minimum: i64, maximum: i64) -> Self {
        let response = self.response.as_ref().expect("Response should not be null");
        self.assert_date_in_range(minimum, response.received_response_at_millis, maximum);
        self
    }

    fn assert_date_in_range(&self, minimum: i64, actual: i64, maximum: i64) {
        let min_fmt = self.format(minimum);
        let act_fmt = self.format(actual);
        let max_fmt = self.format(maximum);
        assert!(
            actual >= minimum && actual <= maximum,
            "{} <= {} <= {}",
            min_fmt, act_fmt, max_fmt
        );
    }

    fn format(&self, time: i64) -> String {
        let dt = Utc.timestamp_millis_opt(time).unwrap();
        dt.format("%H:%M:%S%.3f").to_string()
    }
}

impl PartialEq for RecordedResponse {
    fn eq(&self, other: &Self) -> bool {
        self.request == other.request &&
        self.response == other.response &&
        self.body == other.body &&
        self.failure.as_ref().map(|e| e.to_string()) == other.failure.as_ref().map(|e| e.to_string())
    }
}

impl Eq for RecordedResponse {}
