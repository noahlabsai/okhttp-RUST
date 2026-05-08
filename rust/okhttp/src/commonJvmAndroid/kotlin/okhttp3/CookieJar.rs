use crate::okhttp3::{Cookie, HttpUrl};
use once_cell::sync::Lazy;

/// Provides **policy** and **persistence** for HTTP cookies.
///
/// As policy, implementations of this interface are responsible for selecting which cookies to
/// accept and which to reject. A reasonable policy is to reject all cookies, though that may
/// interfere with session-based authentication schemes that require cookies.
///
/// As persistence, implementations of this interface must also provide storage of cookies. Simple
/// implementations may store cookies in memory; sophisticated ones may use the file system or
/// database to hold accepted cookies. The [cookie storage model][rfc_6265_53] specifies policies for
/// updating and expiring cookies.
///
/// [rfc_6265_53]: https://tools.ietf.org/html/rfc6265#section-5.3
pub trait CookieJar: Send + Sync {
    /// Saves `cookies` from an HTTP response to this store according to this jar's policy.
    ///
    /// Note that this method may be called a second time for a single HTTP response if the response
    /// includes a trailer. For this obscure HTTP feature, `cookies` contains only the trailer's
    /// cookies.
    fn save_from_response(&self, url: &HttpUrl, cookies: &[Cookie]);

    /// Load cookies from the jar for an HTTP request to `url`. This method returns a possibly
    /// empty list of cookies for the network request.
    ///
    /// Simple implementations will return the accepted cookies that have not yet expired and that
    /// [match][Cookie::matches] `url`.
    fn load_for_request(&self, url: &HttpUrl) -> Vec<Cookie>;
}

/// A cookie jar that never accepts any cookies.
struct NoCookies;

impl CookieJar for NoCookies {
    fn save_from_response(&self, _url: &HttpUrl, _cookies: &[Cookie]) {
        // No-op: reject all cookies
    }

    fn load_for_request(&self, _url: &HttpUrl) -> Vec<Cookie> {
        Vec::new()
    }
}

/// Companion object equivalent for CookieJar
pub struct CookieJarCompanion;

impl CookieJarCompanion {
    /// A cookie jar that never accepts any cookies.
    pub fn no_cookies() -> &'static dyn CookieJar {
        pub static NO_COOKIES: Lazy<Box<dyn CookieJar>> = Lazy::new(|| Box::new(NoCookies));
        NO_COOKIES.as_ref()
    }
}

/// Global constant for NO_COOKIES as defined in the Kotlin companion object
pub static NO_COOKIES: Lazy<&'static dyn CookieJar> = Lazy::new(|| CookieJarCompanion::no_cookies());