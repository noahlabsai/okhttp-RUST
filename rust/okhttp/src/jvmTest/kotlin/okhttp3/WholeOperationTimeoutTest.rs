use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::mockwebserver::src::main::kotlin::mockwebserver3::{MockResponse, MockWebServer};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{
    Call, Callback, MediaType, OkHttpClient, Request, RequestBody, Response, 
    OkHttpClientTestRule, Timeout
};
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::testing::Flaky;
use okio::BufferedSink;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::MockResponse::*;
use crate::mockwebserver::src::test::java::mockwebserver3::CustomDispatcherTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::CallTagsTest::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::ResponseBodyTest::*;

// A large response body. Smaller bodies might successfully read after the socket is closed!
const BIG_ENOUGH_BODY: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"; // In real impl would be 64KB

pub struct WholeOperationTimeoutTest {
    client_test_rule: OkHttpClientTestRule,
    client: OkHttpClient,
    server: MockWebServer,
}

impl WholeOperationTimeoutTest {
    pub fn new() -> Self {
        let client_test_rule = OkHttpClientTestRule::new();
        let client = client_test_rule.new_client();
        let server = MockWebServer::new();
        
        Self {
            client_test_rule,
            client,
            server,
        }
    }

    pub async fn default_config_is_no_timeout(&self) {
        let request = Request::builder()
            .url(self.server.url("/"))
            .build();
        let call = self.client.new_call(request);
        assert_eq!(call.timeout().timeout_nanos(), 0);
    }

    pub async fn configure_client_default(&self) {
        let request = Request::builder()
            .url(self.server.url("/"))
            .build();
        let timeout_client = self.client
            .new_builder()
            .call_timeout(Duration::from_millis(456))
            .build();
        let call = timeout_client.new_call(request);
        assert_eq!(call.timeout().timeout_nanos(), 456 * 1_000_000);
    }

    pub async fn timeout_writing_request(&self) {
        self.server.enqueue(MockResponse::new());
        let request = Request::builder()
            .url(self.server.url("/"))
            .post(self.sleeping_request_body(500))
            .build();
        let call = self.client.new_call(request);
        call.timeout().timeout(Duration::from_millis(250));
        
        let result = call.execute();
        match result {
            Err(e) => {
                assert_eq!(e.to_string(), "timeout");
                assert!(call.is_canceled());
            }
            Ok(_) => panic!("Expected IOException"),
        }
    }

    pub async fn timeout_writing_request_with_enqueue(&self) {
        self.server.enqueue(MockResponse::new());
        let request = Request::builder()
            .url(self.server.url("/"))
            .post(self.sleeping_request_body(500))
            .build();
        
        let notify = Arc::new(tokio::sync::Notify::new());
        let exception_ref = Arc::new(Mutex::new(None));
        
        let call = self.client.new_call(request);
        call.timeout().timeout(Duration::from_millis(250));
        
        let notify_clone = Arc::clone(&notify);
        let exception_clone = Arc::clone(&exception_ref);
        
        call.enqueue(Box::new(object_callback(move |call, e| {
            let mut lock = exception_clone.lock().unwrap();
            *lock = Some(e);
            notify_clone.notify_one();
        }, move |call, response| {
            response.close();
            notify_clone.notify_one();
        })));

        let _ = tokio::time::timeout(Duration::from_secs(5), notify.notified()).r#await;
        assert!(call.is_canceled());
        assert!(exception_ref.lock().unwrap().is_some());
    }

    pub async fn timeout_processing(&self) {
        self.server.enqueue(
            MockResponse::builder()
                .headers_delay(Duration::from_millis(500))
                .build()
        );
        let request = Request::builder()
            .url(self.server.url("/"))
            .build();
        let call = self.client.new_call(request);
        call.timeout().timeout(Duration::from_millis(250));
        
        let result = call.execute();
        match result {
            Err(e) => {
                assert_eq!(e.to_string(), "timeout");
                assert!(call.is_canceled());
            }
            Ok(_) => panic!("Expected IOException"),
        }
    }

    pub async fn timeout_processing_with_enqueue(&self) {
        self.server.enqueue(
            MockResponse::builder()
                .headers_delay(Duration::from_millis(500))
                .build()
        );
        let request = Request::builder()
            .url(self.server.url("/"))
            .build();
        
        let notify = Arc::new(tokio::sync::Notify::new());
        let exception_ref = Arc::new(Mutex::new(None));
        
        let call = self.client.new_call(request);
        call.timeout().timeout(Duration::from_millis(250));
        
        let notify_clone = Arc::clone(&notify);
        let exception_clone = Arc::clone(&exception_ref);
        
        call.enqueue(Box::new(object_callback(move |call, e| {
            let mut lock = exception_clone.lock().unwrap();
            *lock = Some(e);
            notify_clone.notify_one();
        }, move |call, response| {
            response.close();
            notify_clone.notify_one();
        })));

        let _ = tokio::time::timeout(Duration::from_secs(5), notify.notified()).r#await;
        assert!(call.is_canceled());
        assert!(exception_ref.lock().unwrap().is_some());
    }

    pub async fn timeout_reading_response(&self) {
        self.server.enqueue(
            MockResponse::builder()
                .body(BIG_ENOUGH_BODY.to_string())
                .build()
        );
        let request = Request::builder()
            .url(self.server.url("/"))
            .build();
        let call = self.client.new_call(request);
        call.timeout().timeout(Duration::from_millis(250));
        
        let response = call.execute().expect("Request should succeed");
        tokio::time::sleep(Duration::from_millis(500)).r#await;
        
        let result = response.body().source().read_utf8();
        match result {
            Err(e) => {
                assert_eq!(e.to_string(), "timeout");
                assert!(call.is_canceled());
            }
            Ok(_) => panic!("Expected IOException"),
        }
    }

    pub async fn timeout_reading_response_with_enqueue(&self) {
        self.server.enqueue(
            MockResponse::builder()
                .body(BIG_ENOUGH_BODY.to_string())
                .build()
        );
        let request = Request::builder()
            .url(self.server.url("/"))
            .build();
        
        let notify = Arc::new(tokio::sync::Notify::new());
        let exception_ref = Arc::new(Mutex::new(None));
        
        let call = self.client.new_call(request);
        call.timeout().timeout(Duration::from_millis(250));
        
        let notify_clone = Arc::clone(&notify);
        let exception_clone = Arc::clone(&exception_ref);
        
        call.enqueue(Box::new(object_callback(move |call, e| {
            notify_clone.notify_one();
        }, move |call, response| {
            std::thread::sleep(Duration::from_millis(500));
            let result = response.body().source().read_utf8();
            if let Err(e) = result {
                let mut lock = exception_clone.lock().unwrap();
                *lock = Some(e);
            }
            notify_clone.notify_one();
        })));

        let _ = tokio::time::timeout(Duration::from_secs(5), notify.notified()).r#await;
        assert!(call.is_canceled());
        assert!(exception_ref.lock().unwrap().is_some());
    }

    pub async fn single_timeout_for_all_follow_up_requests(&self) {
        for _ in 0..5 {
            self.server.enqueue(
                MockResponse::builder()
                    .code(302)
                    .set_header("Location", "/next")
                    .headers_delay(Duration::from_millis(100))
                    .build()
            );
        }
        self.server.enqueue(MockResponse::new());
        
        let request = Request::builder()
            .url(self.server.url("/a"))
            .build();
        let call = self.client.new_call(request);
        call.timeout().timeout(Duration::from_millis(250));
        
        let result = call.execute();
        match result {
            Err(e) => {
                assert_eq!(e.to_string(), "timeout");
                assert!(call.is_canceled());
            }
            Ok(_) => panic!("Expected IOException"),
        }
    }

    pub async fn timeout_following_redirect_on_new_connection(&self) {
        let mut other_server = MockWebServer::new();
        other_server.start();
        
        self.server.enqueue(
            MockResponse::builder()
                .code(302)
                .set_header("Location", other_server.url("/"))
                .build()
        );
        other_server.enqueue(
            MockResponse::builder()
                .headers_delay(Duration::from_millis(500))
                .build()
        );
        
        let request = Request::builder().url(self.server.url("/")).build();
        let call = self.client.new_call(request);
        call.timeout().timeout(Duration::from_millis(250));
        
        let result = call.execute();
        match result {
            Err(e) => {
                assert_eq!(e.to_string(), "timeout");
                assert!(call.is_canceled());
            }
            Ok(_) => panic!("Expected IOException"),
        }
    }

    #[Flaky]
    pub async fn no_timeout(&self) {
        self.server.enqueue(
            MockResponse::builder()
                .headers_delay(Duration::from_millis(250))
                .body(BIG_ENOUGH_BODY.to_string())
                .build()
        );
        let request = Request::builder()
            .url(self.server.url("/"))
            .post(self.sleeping_request_body(250))
            .build();
        let call = self.client.new_call(request);
        call.timeout().timeout(Duration::from_millis(2000));
        
        let response = call.execute().expect("Should not timeout");
        tokio::time::sleep(Duration::from_millis(250)).r#await;
        response.body().source().read_utf8().expect("Should read body");
        response.close();
        assert!(!call.is_canceled());
    }

    fn sleeping_request_body(&self, sleep_millis: i32) -> RequestBody {
        RequestBody::new(move |sink: &mut dyn BufferedSink| {
            sink.write_utf8("abc")?;
            sink.flush()?;
            std::thread::sleep(Duration::from_millis(sleep_millis as u64));
            sink.write_utf8("def")?;
            Ok(())
        })
    }
}

fn object_callback<F1, F2>(on_failure: F1, on_response: F2) -> Box<dyn Callback> 
where 
    F1: Fn(&Call, Box<dyn std::error::Error>) + Send + Sync + 'static,
    F2: Fn(&Call, Response) + Send + Sync + 'static,
{
    struct CallbackImpl<F1, F2> {
        on_failure: F1,
        on_response: F2,
    }
    impl<F1, F2> Callback for CallbackImpl<F1, F2> 
    where 
        F1: Fn(&Call, Box<dyn std::error::Error>) + Send + Sync + 'static,
        F2: Fn(&Call, Response) + Send + Sync + 'static,
    {
        fn on_failure(&self, call: &Call, e: Box<dyn std::error::Error>) {
            (self.on_failure)(call, e);
        }
        fn on_response(&self, call: &Call, response: Response) {
            (self.on_response)(call, response);
        }
    }
    Box::new(CallbackImpl { on_failure, on_response })
}
