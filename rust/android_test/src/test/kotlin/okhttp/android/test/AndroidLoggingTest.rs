use std::error::Error;
use regex::Regex;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::ConnectionSpec;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::OkHttpClient;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request;
use crate::okhttp::src::androidMain::kotlin::okhttp3::internal::platform::AndroidPlatform;
use crate::okhttp_logging_interceptor::src::main::kotlin::okhttp3::logging::HttpLoggingInterceptor;
use crate::okhttp_logging_interceptor::src::main::kotlin::okhttp3::logging::LoggingEventListener;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;

// Mocking Android Log and ShadowLog as they are platform-specific dependencies
pub mod android {
    pub mod util {
        pub mod Log {
            pub const INFO: i32 = 4;
        }
    }
}

pub struct LogEntry {
    pub type_: i32,
    pub msg: String,
    pub throwable: Option<Box<dyn Error + Send + Sync>>,
}

pub struct ShadowLog;
impl ShadowLog {
    pub fn get_logs_for_tag(_tag: &str) -> Vec<LogEntry> {
        // This would return the captured logs in a Robolectric environment
        Vec::new()
    }
}

pub struct AndroidLoggingTest {
    pub client_builder: OkHttpClient::Builder,
    pub request: Request,
}

impl AndroidLoggingTest {
    pub fn new() -> Self {
        let builder = OkHttpClient::builder()
            .connection_specs(vec![ConnectionSpec::CLEARTEXT])
            .dns(|_hostname| {
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "shortcircuit",
                )))
            });

        let request = Request::new("http://google.com/robots.txt".to_string());

        AndroidLoggingTest {
            client_builder: builder,
            request,
        }
    }

    pub fn test_http_logging_interceptor(&self) -> Result<(), Box<dyn Error>> {
        let mut interceptor = HttpLoggingInterceptor::new();
        interceptor.set_level(HttpLoggingInterceptor::Level::BASIC);

        let client = self.client_builder
            .clone()
            .add_interceptor(interceptor)
            .build();

        // try-catch block for UnknownHostException
        let _ = client.new_call(self.request.clone()).execute();

        let logs = ShadowLog::get_logs_for_tag(AndroidPlatform::TAG);
        
        let log_types: Vec<i32> = logs.iter().map(|l| l.type_).collect();
        for t in &log_types {
            if *t != android::util::Log::INFO {
                return Err("Log type was not INFO".into());
            }
        }

        let re_digits = Regex::new(r"\d+").unwrap();
        let processed_msgs: Vec<String> = logs.iter()
            .map(|l| re_digits.replace_all(&l.msg, "").to_string())
            .collect();

        let expected_msgs = vec![
            "--> GET http://google.com/robots.txt".to_string(),
            format!("<-- HTTP FAILED: java.net.UnknownHostException: shortcircuit. {} (ms)", self.request.url()),
        ];

        if processed_msgs != expected_msgs {
            return Err("Log messages did not match expected output".into());
        }

        if let Some(last_log) = logs.last() {
            if last_log.throwable.is_some() {
                return Err("Last log throwable should be null".into());
            }
        }

        Ok(())
    }

    pub fn test_logging_event_listener(&self) -> Result<(), Box<dyn Error>> {
        let client = self.client_builder
            .clone()
            .event_listener_factory(LoggingEventListener::Factory::new())
            .build();

        // try-catch block for UnknownHostException
        let _ = client.new_call(self.request.clone()).execute();

        let logs = ShadowLog::get_logs_for_tag(AndroidPlatform::TAG);

        let log_types: Vec<i32> = logs.iter().map(|l| l.type_).collect();
        for t in &log_types {
            if *t != android::util::Log::INFO {
                return Err("Log type was not INFO".into());
            }
        }

        let re_ms = Regex::new(r"\[\d+ ms\] ").unwrap();
        let processed_msgs: Vec<String> = logs.iter()
            .map(|l| re_ms.replace_all(&l.msg, "").to_string())
            .collect();

        let expected_msgs = vec![
            format!("callStart: Request{{method=GET, url={}}}", self.request.url()),
            "proxySelectStart: http://google.com/".to_string(),
            "proxySelectEnd: [DIRECT]".to_string(),
            "dnsStart: google.com".to_string(),
            "callFailed: java.net.UnknownHostException: shortcircuit".to_string(),
        ];

        if processed_msgs != expected_msgs {
            return Err("Log messages did not match expected output".into());
        }

        if let Some(last_log) = logs.last() {
            if last_log.throwable.is_some() {
                return Err("Last log throwable should be null".into());
            }
        }

        Ok(())
    }
}
