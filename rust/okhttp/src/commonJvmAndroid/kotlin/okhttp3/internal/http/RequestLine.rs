use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrl;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request;
use std::fmt::Write;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

// Equivalent to java.net.Proxy.Type

impl Default for ProxyType {
    fn default() -> Self {
        ProxyType::DIRECT
    }
}


pub struct RequestLine;

impl RequestLine {
    /*
     * Returns the request status line, like "GET / HTTP/1.1". This is exposed to the application by
     * [HttpURLConnection.getHeaderFields], so it needs to be set even if the transport is
     * HTTP/2.
     */
    pub fn get(
        request: &Request,
        proxy_type: ProxyType,
    ) -> String {
        let mut result = String::new();
        
        // append(request.method)
        result.push_str(request.method());
        
        // append(' ')
        result.push(' ');
        
        if Self::include_authority_in_request_line(request, proxy_type) {
            // append(request.url)
            result.push_str(&request.url().to_string());
        } else {
            // append(requestPath(request.url))
            result.push_str(&Self::request_path(request.url()));
        }
        
        // append(" HTTP/1.1")
        result.push_str(" HTTP/1.1");
        
        result
    }

    /*
     * Returns true if the request line should contain the full URL with host and port (like "GET
     * http://android.com/foo HTTP/1.1") or only the path (like "GET /foo HTTP/1.1").
     */
    fn include_authority_in_request_line(
        request: &Request,
        proxy_type: ProxyType,
    ) -> bool {
        !request.is_https() && proxy_type == ProxyType::HTTP
    }

    /*
     * Returns the path to request, like the '/' in 'GET / HTTP/1.1'. Never empty, even if the request
     * URL is. Includes the query component if it exists.
     */
    pub fn request_path(url: &HttpUrl) -> String {
        let path = url.encoded_path();
        let query = url.encoded_query();
        
        if let Some(q) = query {
            format!("{}?{}", path, q)
        } else {
            path
        }
    }
}