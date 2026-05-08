use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::{Arc, Mutex};
use okio::Buffer;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

// Equivalent to the Dns interface in okhttp3.
pub trait Dns: Send + Sync {
    fn lookup(&self, hostname: &str) -> Result<Vec<IpAddr>, Box<dyn std::error::Error>>;
}

#[derive(Debug, Clone)]
pub struct FakeDns {
    host_addresses: Arc<Mutex<HashMap<String, Vec<IpAddr>>>>,
    requested_hosts: Arc<Mutex<Vec<String>>>,
    next_address: Arc<Mutex<i64>>,
}

impl Default for FakeDns {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeDns {
    pub fn new() -> Self {
        Self {
            host_addresses: Arc::new(Mutex::new(HashMap::new())),
            requested_hosts: Arc::new(Mutex::new(Vec::new())),
            next_address: Arc::new(Mutex::new(0xff000064i64)), // 255.0.0.100 in IPv4; ::ff00:64 in IPv6.
        }
    }

    // Sets the results for `hostname`.
    pub fn set(&self, hostname: String, addresses: Vec<IpAddr>) -> &Self {
        let mut map = self.host_addresses.lock().unwrap();
        map.insert(hostname, addresses);
        self
    }

    // Clears the results for `hostname`.
    pub fn clear(&self, hostname: &str) -> &Self {
        let mut map = self.host_addresses.lock().unwrap();
        map.remove(hostname);
        self
    }

    // Returns the address at the specified index for the given hostname.
    // Panics if the hostname is not found or index is out of bounds, matching Kotlin's !! and index access.
    pub fn lookup_at(&self, hostname: &str, index: usize) -> IpAddr {
        let map = self.host_addresses.lock().unwrap();
        let addresses = map.get(hostname).expect("Hostname not found in FakeDns");
        addresses[index]
    }

    pub fn assert_requests(&self, expected_hosts: &[Option<String>]) {
        let mut requested = self.requested_hosts.lock().unwrap();
        
        // Convert requested hosts to Option<String> for comparison with expected_hosts
        let actual: Vec<Option<String>> = requested.iter().map(|s| Some(s.clone())).collect();
        
        assert_eq!(actual, expected_hosts, "Requested hosts did not match expected hosts");
        requested.clear();
    }

    // Allocates and returns `count` fake IPv4 addresses like [255.0.0.100, 255.0.0.101].
    pub fn allocate(&self, count: i32) -> Vec<IpAddr> {
        let mut next_addr_lock = self.next_address.lock().unwrap();
        let from = *next_addr_lock;
        *next_addr_lock += count as i64;

        (from..(from + count as i64))
            .map(|it| {
                let mut buffer = Buffer::new();
                buffer.writeInt(it as i32);
                let bytes = buffer.readByteArray();
                
                // InetAddress.getByAddress in Java for 4 bytes creates an IPv4 address
                let mut ipv4_bytes = [0u8; 4];
                ipv4_bytes.copy_from_slice(&bytes[..4]);
                IpAddr::V4(Ipv4Addr::from(ipv4_bytes))
            })
            .collect()
    }

    // Allocates and returns `count` fake IPv6 addresses like [::ff00:64, ::ff00:65].
    pub fn allocate_ipv6(&self, count: i32) -> Vec<IpAddr> {
        let mut next_addr_lock = self.next_address.lock().unwrap();
        let from = *next_addr_lock;
        *next_addr_lock += count as i64;

        (from..(from + count as i64))
            .map(|it| {
                let mut buffer = Buffer::new();
                buffer.writeLong(0i64);
                buffer.writeLong(it);
                let bytes = buffer.readByteArray();
                
                let mut ipv6_bytes = [0u8; 16];
                ipv6_bytes.copy_from_slice(&bytes[..16]);
                IpAddr::V6(Ipv6Addr::from(ipv6_bytes))
            })
            .collect()
    }
}

impl Dns for FakeDns {
    fn lookup(&self, hostname: &str) -> Result<Vec<IpAddr>, Box<dyn std::error::Error>> {
        {
            let mut requested = self.requested_hosts.lock().unwrap();
            requested.push(hostname.to_string());
        }

        let map = self.host_addresses.lock().unwrap();
        map.get(hostname)
            .cloned()
            .ok_or_else(|| {
                // UnknownHostException equivalent
                Box::new(std::io::Error::new(std::io::ErrorKind::AddrNotAvailable, "Unknown host"))
            })
    }
}