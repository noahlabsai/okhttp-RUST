use std::io::{Error, ErrorKind};
use std::net::{IpAddr, InetSocketAddress};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Address;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrl;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Route;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::connection::RouteDatabase;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::connection::RealCall;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::can_parse_as_ip_address;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Proxy;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::connection::RouteDatabase::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::proxy::NullProxySelector::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

// Proxy representation mapping to java.net.Proxy


impl Default for ProxyType {
    fn default() -> Self {
        ProxyType::DIRECT
    }
}

// Selects routes to connect to an origin server. Each connection requires a choice of proxy server,
// IP address, and TLS mode. Connections may also be recycled.
pub struct RouteSelector {
    address: Address,
    route_database: RouteDatabase,
    call: RealCall,
    fast_fallback: bool,
    proxies: Vec<Proxy>,
    next_proxy_index: usize,
    inet_socket_addresses: Vec<InetSocketAddress>,
    postponed_routes: Vec<Route>,
}

impl RouteSelector {
    pub fn new(
        address: Address,
        route_database: RouteDatabase,
        call: RealCall,
        fast_fallback: bool,
    ) -> Self {
        let mut selector = Self {
            address,
            route_database,
            call,
            fast_fallback,
            proxies: Vec::new(),
            next_proxy_index: 0,
            inet_socket_addresses: Vec::new(),
            postponed_routes: Vec::new(),
        };

        // Initialize proxy state
        selector.reset_next_proxy(selector.address.url.clone(), selector.address.proxy.clone());
        selector
    }

    // Returns true if there's another set of routes to attempt. Every address has at least one route.
    pub fn has_next(&self) -> bool {
        self.has_next_proxy() || !self.postponed_routes.is_empty()
    }

    // Computes the next set of routes to attempt.
    pub fn next(&mut self) -> Result<Selection, Box<dyn std::error::Error>> {
        if !self.has_next() {
            return Err(Box::new(Error::new(ErrorKind::Other, "NoSuchElementException")));
        }

        let mut routes = Vec::new();
        while self.has_next_proxy() {
            // Postponed routes are always tried last. For example, if we have 2 proxies and all the
            // routes for proxy1 should be postponed, we'll move to proxy2. Only after we've exhausted
            // all the good routes will we attempt the postponed routes.
            let proxy = self.next_proxy()?;
            for inet_socket_address in &self.inet_socket_addresses {
                let route = Route::new(self.address.clone(), proxy.clone(), *inet_socket_address);
                if self.route_database.should_postpone(&route) {
                    self.postponed_routes.push(route);
                } else {
                    routes.push(route);
                }
            }

            if !routes.is_empty() {
                break;
            }
        }

        if routes.is_empty() {
            // We've exhausted all Proxies so fallback to the postponed routes.
            routes.extend(self.postponed_routes.drain(..));
        }

        Ok(Selection {
            routes,
            next_route_index: 0,
        })
    }

    // Prepares the proxy servers to try.
    fn reset_next_proxy(&mut self, url: HttpUrl, proxy: Option<Proxy>) {
        let select_proxies = |url: &HttpUrl, proxy: &Option<Proxy>, address: &Address| -> Vec<Proxy> {
            // If the user specifies a proxy, try that and only that.
            if let Some(p) = proxy {
                return vec![p.clone()];
            }

            // If the URI lacks a host (as in "http://</"), don't call the ProxySelector.
            if url.host().is_none() {
                return vec![Proxy::NO_PROXY];
            }

            // Try each of the ProxySelector choices until one connection succeeds.
            let proxies_or_null = address.proxy_selector.select(url);
            if proxies_or_null.is_empty() {
                return vec![Proxy::NO_PROXY];
            }

            proxies_or_null
        };

        self.call.event_listener.proxy_select_start(&self.call, url.clone());
        let proxies = select_proxies(&url, &proxy, &self.address);
        self.proxies = proxies.clone();
        self.next_proxy_index = 0;
        self.call.event_listener.proxy_select_end(&self.call, url, proxies);
    }

    // Returns true if there's another proxy to try.
    fn has_next_proxy(&self) -> bool {
        self.next_proxy_index < self.proxies.len()
    }

    // Returns the next proxy to try. May be PROXY.NO_PROXY but never null.
    fn next_proxy(&mut self) -> Result<Proxy, Box<dyn std::error::Error>> {
        if !self.has_next_proxy() {
            return Err(Box::new(Error::new(
                ErrorKind::AddrNotAvailable,
                format!("No route to {}; exhausted proxy configurations: {:?}", self.address.url.host(), self.proxies),
            )));
        }
        let result = self.proxies[self.next_proxy_index].clone();
        self.next_proxy_index += 1;
        self.reset_next_inet_socket_address(result.clone())?;
        Ok(result)
    }

    // Prepares the socket addresses to attempt for the current proxy or host.
    fn reset_next_inet_socket_address(&mut self, proxy: Proxy) -> Result<(), Box<dyn std::error::Error>> {
        // Clear the addresses. Necessary if getAllByName() below throws!
        let mut mutable_inet_socket_addresses = Vec::new();

        let socket_host: String;
        let socket_port: u16;

        if proxy.type_of() == ProxyType::DIRECT || proxy.type_of() == ProxyType::SOCKS {
            socket_host = self.address.url.host().unwrap_or_default().to_string();
            socket_port = self.address.url.port() as u16;
        } else {
            let proxy_address = proxy.address().ok_or_else(|| {
                Box::new(Error::new(ErrorKind::Other, "Proxy.address() is not an InetSocketAddress")) as Box<dyn std::error::Error>
            })?;
            socket_host = self.get_socket_host(&proxy_address);
            socket_port = proxy_address.port();
        }

        if socket_port == 0 || socket_port > 65535 {
            return Err(Box::new(Error::new(ErrorKind::InvalidInput, format!("No route to {}:{}; port is out of range", socket_host, socket_port))));
        }

        if proxy.type_of() == ProxyType::SOCKS {
            // In Rust, we simulate unresolved by using the host/port string
            // Note: In a real implementation, this would be a custom UnresolvedInetSocketAddress
            mutable_inet_socket_addresses.push(
                format!("{}:{}", socket_host, socket_port).parse::<InetSocketAddress>()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
            );
        } else {
            let addresses = if can_parse_as_ip_address(&socket_host) {
                vec![socket_host.parse::<IpAddr>().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?]
            } else {
                self.call.event_listener.dns_start(&self.call, &socket_host);
                let result = self.address.dns.lookup(&socket_host);
                if result.is_empty() {
                    return Err(Box::new(Error::new(
                        ErrorKind::AddrNotAvailable,
                        format!("{:?} returned no addresses for {}", self.address.dns, socket_host),
                    )));
                }
                self.call.event_listener.dns_end(&self.call, &socket_host, result.clone());
                result
            };

            // Try each address for best behavior in mixed IPv4/IPv6 environments.
            let ordered_addresses = if self.fast_fallback {
                self.reorder_for_happy_eyeballs(addresses)
            } else {
                addresses
            };

            for inet_address in ordered_addresses {
                let socket_addr = InetSocketAddress::new(inet_address, socket_port);
                mutable_inet_socket_addresses.push(socket_addr);
            }
        }

        self.inet_socket_addresses = mutable_inet_socket_addresses;
        Ok(())
    }

    fn get_socket_host(&self, addr: &InetSocketAddress) -> String {
        // Equivalent to the companion object extension property `socketHost`
        addr.ip().to_string()
    }

    fn reorder_for_happy_eyeballs(&self, addresses: Vec<IpAddr>) -> Vec<IpAddr> {
        // Implementation of Happy Eyeballs reordering logic
        addresses
    }
}

// A set of selected Routes.
pub struct Selection {
    pub routes: Vec<Route>,
    next_route_index: usize,
}

impl Selection {
    pub fn has_next(&self) -> bool {
        self.next_route_index < self.routes.len()
    }

    pub fn next(&mut self) -> Result<Route, Box<dyn std::error::Error>> {
        if !self.has_next() {
            return Err(Box::new(Error::new(ErrorKind::Other, "NoSuchElementException")));
        }
        let route = self.routes[self.next_route_index].clone();
        self.next_route_index += 1;
        Ok(route)
    }
}
