use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{Protocol, Request, Response};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http::{has_body, HttpMethod};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http::HttpMethod::*;

pub struct PublicInternalApiTest;

impl PublicInternalApiTest {
    #[test]
    pub fn permits_request_body() {
        assert!(HttpMethod::permits_request_body("POST"));
        assert!(!HttpMethod::permits_request_body("GET"));
    }

    #[test]
    pub fn requires_request_body() {
        assert!(HttpMethod::requires_request_body("PUT"));
        assert!(!HttpMethod::requires_request_body("GET"));
    }

    #[test]
    pub fn has_body() {
        let request = Request::builder()
            .url("http://example.com")
            .build();

        let response = Response::builder()
            .code(200)
            .message("OK")
            .request(request)
            .protocol(Protocol::HTTP_2)
            .build();

        assert!(has_body(&response));
    }
}
