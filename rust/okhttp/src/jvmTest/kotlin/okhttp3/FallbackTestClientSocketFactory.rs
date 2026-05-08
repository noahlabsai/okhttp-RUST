use std::io;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::{DelegatingSSLSocketFactory, DelegatingSSLSocket, SSLSocket, SSLSocketFactory};
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test::src::test::kotlin::okhttp::android::test::DisabledInitialiserTest::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::SocketHandler::*;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::DelegatingSSLSocketFactory::*;

// The cipher suite used during TLS connection fallback to indicate a fallback. See
// https://tools.ietf.org/html/draft-ietf-tls-downgrade-scsv-00
pub const TLS_FALLBACK_SCSV: &str = "TLS_FALLBACK_SCSV";

/*
 * An SSLSocketFactory that delegates calls. Sockets created by the delegate are wrapped with ones
 * that will not accept the [TLS_FALLBACK_SCSV] cipher, thus bypassing server-side fallback
 * checks on platforms that support it. Unfortunately this wrapping will disable any
 * reflection-based calls to SSLSocket from Platform.
 */
pub struct FallbackTestClientSocketFactory {
    delegate: DelegatingSSLSocketFactory,
}

impl FallbackTestClientSocketFactory {
    pub fn new(delegate: DelegatingSSLSocketFactory) -> Self {
        Self { delegate }
    }

    pub fn configure_socket(&self, ssl_socket: SSLSocket) -> SSLSocket {
        TlsFallbackScsvDisabledSSLSocket::new(ssl_socket)
    }
}

impl SSLSocketFactory for FallbackTestClientSocketFactory {
    fn create_socket(&self) -> io::Result<SSLSocket> {
        self.delegate.create_socket().map(|socket| self.configure_socket(socket))
    }

    fn create_socket_host_port(&self, host: &str, port: i32) -> io::Result<SSLSocket> {
        self.delegate.create_socket_host_port(host, port).map(|socket| self.configure_socket(socket))
    }

    fn create_socket_host_port_local(&self, host: &str, port: i32, local_address: std::net::IpAddr, local_port: i32) -> io::Result<SSLSocket> {
        self.delegate.create_socket_host_port_local(host, port, local_address, local_port).map(|socket| self.configure_socket(socket))
    }

    fn create_socket_addr_port(&self, host: std::net::IpAddr, port: i32) -> io::Result<SSLSocket> {
        self.delegate.create_socket_addr_port(host, port).map(|socket| self.configure_socket(socket))
    }

    fn create_socket_addr_port_local(&self, host: std::net::IpAddr, port: i32, local_address: std::net::IpAddr, local_port: i32) -> io::Result<SSLSocket> {
        self.delegate.create_socket_addr_port_local(host, port, local_address, local_port).map(|socket| self.configure_socket(socket))
    }

    fn create_socket_from_socket(&self, socket: crate::okhttp_testing_support::src::main::kotlin::okhttp3::Socket, host: &str, port: i32, auto_close: bool) -> io::Result<SSLSocket> {
        self.delegate.create_socket_from_socket(socket, host, port, auto_close).map(|socket| self.configure_socket(socket))
    }

    fn get_default_cipher_suites(&self) -> Vec<String> {
        self.delegate.get_default_cipher_suites()
    }

    fn get_supported_cipher_suites(&self) -> Vec<String> {
        self.delegate.get_supported_cipher_suites()
    }
}

struct TlsFallbackScsvDisabledSSLSocket {
    delegate: DelegatingSSLSocket,
}

impl TlsFallbackScsvDisabledSSLSocket {
    fn new(socket: SSLSocket) -> SSLSocket {
        let wrapped = TlsFallbackScsvDisabledSSLSocket {
            delegate: DelegatingSSLSocket::new(socket),
        };
        wrapped.into()
    }
}

impl CipherSuiteConfigurable for TlsFallbackScsvDisabledSSLSocket {
    fn set_enabled_cipher_suites(&mut self, suites: Vec<String>) {
        let enabled_cipher_suites: Vec<String> = suites
            .into_iter()
            .filter(|suite| suite != TLS_FALLBACK_SCSV)
            .collect();
        
        self.delegate.set_enabled_cipher_suites(enabled_cipher_suites);
    }
}

pub trait CipherSuiteConfigurable {
    fn set_enabled_cipher_suites(&mut self, suites: Vec<String>);
}

impl From<TlsFallbackScsvDisabledSSLSocket> for SSLSocket {
    fn from(item: TlsFallbackScsvDisabledSSLSocket) -> Self {
        item.delegate.into()
    }
}
