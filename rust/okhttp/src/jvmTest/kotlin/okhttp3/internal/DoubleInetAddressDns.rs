use std::net::IpAddr;
use std::sync::Arc;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

// Equivalent to okhttp3.Dns interface
pub trait Dns: Send + Sync {
    fn lookup(&self, hostname: &str) -> Result<Vec<IpAddr>, Box<dyn std::error::Error>>;
}

// Equivalent to Dns.SYSTEM singleton
pub struct SystemDns;

impl Dns for SystemDns {
    fn lookup(&self, hostname: &str) -> Result<Vec<IpAddr>, Box<dyn std::error::Error>> {
        // In a real production environment, this would call the OS resolver (e.g., getaddrinfo).
        // For the purpose of this translation, we implement the logic as a system lookup.
        let addresses = std::net::ToSocketAddrs::to_socket_addrs(&(hostname.to_string(), 80))?
            .map(|socket_addr| socket_addr.ip())
            .collect::<Vec<_>>();
        
        if addresses.is_empty() {
            return Err("Could not resolve hostname".into());
        }
        Ok(addresses)
    }
}

// Global access to the system DNS, mimicking Dns.SYSTEM
pub static SYSTEM_DNS: once_cell::sync::Lazy<Arc<dyn Dns>> = 
    once_cell::sync::Lazy::new(|| Arc::new(SystemDns));

/*
 * A network that always resolves two IP addresses per host. Use this when testing route selection
 * fallbacks to guarantee that a fallback address is available.
 */
pub struct DoubleInetAddressDns;

impl Default for DoubleInetAddressDns {
    fn default() -> Self {
        Self
    }
}

impl Dns for DoubleInetAddressDns {
    fn lookup(&self, hostname: &str) -> Result<Vec<IpAddr>, Box<dyn std::error::Error>> {
        // val addresses = Dns.SYSTEM.lookup(hostname)
        let addresses = SYSTEM_DNS.lookup(hostname)?;
        
        // return listOf(addresses[0], addresses[0])
        // Note: Kotlin's addresses[0] will throw IndexOutOfBoundsException if empty.
        // Rust's index access also panics, preserving the behavior.
        let first_address = addresses[0];
        Ok(vec![first_address, first_address])
    }
}