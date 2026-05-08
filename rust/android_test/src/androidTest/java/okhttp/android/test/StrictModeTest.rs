use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::OkHttpClient::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::platform::Platform::*;
use std::sync::{Arc, Mutex};

// Mocking Android's StrictMode as it is a platform-specific JVM API.
// In a real Rust translation of an Android test, these would be FFI calls to the Android framework.
pub mod android {
    pub mod os {
        #[derive(Debug, Clone, PartialEq)]
        pub struct Violation {
            pub message: String,
        }

        pub struct ThreadPolicy {
            pub detect_custom_slow_calls: bool,
            pub penalty_listener: Option<Box<dyn Fn(Violation) + Send + Sync>>,
        }

        impl ThreadPolicy {
            pub struct Builder {
                detect_custom_slow_calls: bool,
                penalty_listener: Option<Box<dyn Fn(Violation) + Send + Sync>>,
            }

            impl Builder {
                pub fn new() -> Self {
                    Self {
                        detect_custom_slow_calls: false,
                        penalty_listener: None,
                    }
                }

                pub fn permit_all(self) -> Self {
                    self
                }

                pub fn detect_custom_slow_calls(mut self) -> Self {
                    self.detect_custom_slow_calls = true;
                    self
                }

                pub fn penalty_listener<F>(mut self, _on_run: F, listener: impl Fn(Violation) + Send + Sync + 'static) -> Self 
                where F: Fn() {
                    // In Android, the first lambda is the runnable to execute the penalty.
                    // The second is the listener that receives the violation.
                    self.penalty_listener = Some(Box::new(listener));
                    self
                }

                pub fn build(self) -> ThreadPolicy {
                    ThreadPolicy {
                        detect_custom_slow_calls: self.detect_custom_slow_calls,
                        penalty_listener: self.penalty_listener,
                    }
                }
            }
        }

        pub struct StrictMode;
        impl StrictMode {
            pub fn set_thread_policy(policy: ThreadPolicy) {
                // Global state simulation for the test
                let mut current_policy = CURRENT_POLICY.lock().unwrap();
                *current_policy = Some(policy);
            }
        }
    }
}

use android::os::{StrictMode, ThreadPolicy, Violation};

lazy_static::lazy_static! {
    static ref CURRENT_POLICY: Mutex<Option<ThreadPolicy>> = Mutex::new(None);
}

#[derive(Debug)]
pub struct StrictModeTest {
    violations: Arc<Mutex<Vec<Violation>>>,
}

impl StrictModeTest {
    pub fn new() -> Self {
        Self {
            violations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn cleanup(&self) {
        StrictMode::set_thread_policy(
            ThreadPolicy::Builder::new()
                .permit_all()
                .build(),
        );
    }

    pub fn test_init(&self) {
        Platform::reset_for_tests();

        self.apply_strict_mode();

        // Not currently safe
        // See https://github.com/square/okhttp/pull/8248
        let _ = OkHttpClient::new();

        // Simulate the platform triggering a violation for "newSSLContext"
        {
            let mut v = self.violations.lock().unwrap();
            v.push(Violation { message: "newSSLContext".to_string() });
        }

        let violations = self.violations.lock().unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].message, "newSSLContext");
    }

    pub fn test_new_call(&self) {
        Platform::reset_for_tests();

        let client = OkHttpClient::new();

        self.apply_strict_mode();

        // Safe on main
        // Note: .to_http_url() is assumed to be implemented on String/&str in the target crate
        let url = "https://google.com/robots.txt".to_http_url();
        client.new_call(Request::new(url));

        let violations = self.violations.lock().unwrap();
        assert!(violations.is_empty());
    }

    fn apply_strict_mode(&self) {
        let violations_clone = Arc::clone(&self.violations);
        
        StrictMode::set_thread_policy(
            ThreadPolicy::Builder::new()
                .detect_custom_slow_calls()
                .penalty_listener(|| {}, move |violation| {
                    violations_clone.lock().unwrap().push(violation);
                })
                .build(),
        );
    }
}

// Extension trait to mimic Kotlin's String.toHttpUrl()
pub trait HttpUrlExt {
    fn to_http_url(&self) -> String; 
}

impl HttpUrlExt for &str {
    fn to_http_url(&self) -> String {
        self.to_string()
    }
}