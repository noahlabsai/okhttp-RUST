use std::net::IpAddr;
use std::sync::Arc;
use std::error::Error;
use std::fmt;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;

// Custom error type to represent UnknownHostException from Kotlin.
#[derive(Debug)]
pub struct UnknownHostException {
    pub message: String,
    pub cause: Option<Box<dyn Error + Send + Sync>>,
}

impl fmt::Display for UnknownHostException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for UnknownHostException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_ref().map(|e| e.as_ref() as &(dyn Error + 'static))
    }
}

// A domain name service that resolves IP addresses for host names. Most applications will use the
// [SYSTEM] DNS service, which is the default. Some applications may provide their own
// implementation to use a different DNS server, to prefer IPv6 addresses, to prefer IPv4 addresses,
// or to force a specific known IP address.
//
// Implementations of this trait must be safe for concurrent use.
pub trait Dns: Send + Sync {
    // Returns the IP addresses of `hostname`, in the order they will be attempted by OkHttp. If a
    // connection to an address fails, OkHttp will retry the connection with the next address until
    // either a connection is made, the set of IP addresses is exhausted, or a limit is exceeded.
    fn lookup(&self, hostname: &str) -> Result<Vec<IpAddr>, Box<dyn Error + Send + Sync>>;
}

// A DNS that uses the underlying operating system to lookup IP addresses.
struct DnsSystem;

impl Dns for DnsSystem {
    fn lookup(&self, hostname: &str) -> Result<Vec<IpAddr>, Box<dyn Error + Send + Sync>> {
        // In Rust, std::net::ToSocketAddrs is the equivalent of InetAddress.getAllByName.
        // Since ToSocketAddrs requires a port, we append a dummy port.
        let host_with_port = format!("{}:0", hostname);
        
        match std::net::ToSocketAddrs::get_addresses(&host_with_port) {
            Ok(addresses) => {
                // Extract only the IpAddr from the SocketAddr
                let ips = addresses.map(|addr| addr.ip()).collect();
                Ok(ips)
            }
            Err(e) => {
                // Kotlin code specifically catches NullPointerException and wraps it in UnknownHostException.
                // In Rust, we map the IO error to our UnknownHostException to preserve business behavior.
                Err(Box::new(UnknownHostException {
                    message: format!("Broken system behaviour for dns lookup of {}", hostname),
                    cause: Some(Box::new(e)),
                }))
            }
        }
    }
}

// Companion object equivalent for Dns.
pub struct DnsCompanion;

impl DnsCompanion {
    // A DNS that uses the underlying operating system to lookup IP addresses.
    // Most custom [Dns] implementations should delegate to this instance.
    pub fn system() -> Arc<dyn Dns> {
        Arc::new(DnsSystem)
    }
}

// Global constant for the system DNS, equivalent to Dns.Companion.SYSTEM.
pub static SYSTEM: std::sync::OnceLock<Arc<dyn Dns>> = std::sync::OnceLock::new();

// Helper function to get the system DNS instance.
pub fn get_system_dns() -> Arc<dyn Dns> {
    SYSTEM.get_or_init(|| DnsCompanion::system()).clone()
}