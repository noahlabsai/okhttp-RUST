use std::net::IpAddr;
use std::io::{Error, ErrorKind};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;

// Equivalent to okhttp3.Dns interface
pub trait Dns: Send + Sync {
    fn lookup(&self, hostname: &str) -> Result<Vec<IpAddr>, Box<dyn std::error::Error>>;
}

// Internal Bootstrap DNS implementation for handling initial connection to DNS over HTTPS server.
//
// Returns hardcoded results for the known host.
pub struct BootstrapDns {
    dns_hostname: String,
    dns_servers: Vec<IpAddr>,
}

impl BootstrapDns {
    pub fn new(dns_hostname: String, dns_servers: Vec<IpAddr>) -> Self {
        Self {
            dns_hostname,
            dns_servers,
        }
    }
}

impl Dns for BootstrapDns {
    fn lookup(&self, hostname: &str) -> Result<Vec<IpAddr>, Box<dyn std::error::Error>> {
        if self.dns_hostname != hostname {
            // Equivalent to throwing UnknownHostException
            return Err(Box::new(Error::new(
                ErrorKind::AddrNotAvailable,
                format!(
                    "BootstrapDns called for {} instead of {}",
                    hostname, self.dns_hostname
                ),
            )));
        }

        // Return a clone of the hardcoded servers list
        Ok(self.dns_servers.clone())
    }
}