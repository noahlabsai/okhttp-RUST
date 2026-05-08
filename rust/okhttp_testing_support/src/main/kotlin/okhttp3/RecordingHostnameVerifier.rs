use std::sync::Mutex;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

// Mock of javax.net.ssl.SSLSession
#[derive(Debug, Clone, PartialEq)]
pub struct SSLSession;

// Mock of javax.net.ssl.HostnameVerifier
pub trait HostnameVerifier {
    fn verify(&self, hostname: &str, session: &SSLSession) -> bool;
}

// RecordingHostnameVerifier records all calls to verify() in a list.
// 
// In Kotlin, the `calls` property is a MutableList and the `verify` method is @Synchronized.
// In Rust, we use a Mutex to protect the Vec<String> to ensure thread-safety and 
// preserve the synchronized behavior.
pub struct RecordingHostnameVerifier {
    pub calls: Mutex<Vec<String>>,
}

impl Default for RecordingHostnameVerifier {
    fn default() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
        }
    }
}

impl RecordingHostnameVerifier {
    pub fn new() -> Self {
        Self::default()
    }
}

impl HostnameVerifier for RecordingHostnameVerifier {
    fn verify(&self, hostname: &str, _session: &SSLSession) -> bool {
        // @Synchronized in Kotlin locks the instance. 
        // Here we lock the mutex protecting the calls list.
        if let Ok(mut calls) = self.calls.lock() {
            calls.push(format!("verify {}", hostname));
        }
        true
    }
}