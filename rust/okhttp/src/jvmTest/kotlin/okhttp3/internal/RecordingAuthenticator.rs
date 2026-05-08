use std::sync::Mutex;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::ContextAwarePlatform::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

// Equivalent to java.net.PasswordAuthentication
#[derive(Debug, Clone, PartialEq)]
pub struct PasswordAuthentication {
    pub username: String,
    pub password: Vec<char>,
}

impl PasswordAuthentication {
    pub fn new(username: &str, password: &[char]) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_vec(),
        }
    }
}

// Equivalent to java.net.Authenticator
// In Rust, we represent the base class as a trait or a struct.
// Since RecordingAuthenticator inherits from Authenticator, we define the behavior.
pub trait Authenticator {
    fn get_password_authentication(&self, context: &AuthenticationContext) -> Option<PasswordAuthentication>;
}

// Context providing the request details that java.net.Authenticator provides via protected fields.
pub struct AuthenticationContext {
    pub requesting_host: String,
    pub requesting_port: i32,
    pub requesting_site_host_name: String,
    pub requesting_url: String,
    pub requestor_type: String,
    pub requesting_prompt: String,
    pub requesting_protocol: String,
    pub requesting_scheme: String,
}

#[derive(Debug, Clone)]
pub struct RecordingAuthenticator {
    authentication: Option<PasswordAuthentication>,
    // Mutex is used because get_password_authentication is typically called 
    // in a context where &self is immutable, but we need to record calls.
    pub calls: Mutex<Vec<String>>,
}

impl RecordingAuthenticator {
    pub const BASE_64_CREDENTIALS: &'static str = "dXNlcm5hbWU6cGFzc3dvcmQ=";

    pub fn new(authentication: Option<PasswordAuthentication>) -> Self {
        Self {
            authentication,
            calls: Mutex::new(Vec::new()),
        }
    }

    // Default constructor matching Kotlin's default parameter
    pub fn default() -> Self {
        let default_auth = PasswordAuthentication::new(
            "username",
            "password".chars().collect::<Vec<char>>().as_slice(),
        );
        Self::new(Some(default_auth))
    }
}

impl Authenticator for RecordingAuthenticator {
    fn get_password_authentication(&self, context: &AuthenticationContext) -> Option<PasswordAuthentication> {
        let call_log = format!(
            "host={} port={} site={} url={} type={} prompt={} protocol={} scheme={}",
            context.requesting_host,
            context.requesting_port,
            context.requesting_site_host_name,
            context.requesting_url,
            context.requestor_type,
            context.requesting_prompt,
            context.requesting_protocol,
            context.requesting_scheme
        );

        if let Ok(mut calls) = self.calls.lock() {
            calls.push(call_log);
        }

        self.authentication.clone()
    }
}