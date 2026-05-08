use std::io::{Read, Write, Result as IoResult};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;

// Mocking the Java/javax.net.ssl types as they are dependencies of the original Kotlin code.
// In a real production environment, these would be traits or structs from a platform-specific crate.
pub trait SSLSession: Send + Sync {}
pub trait SSLParameters: Send + Sync {}
pub trait HandshakeCompletedListener: Send + Sync {}
pub trait SocketChannel: Send + Sync {}

pub trait SSLSocket: Read + Write + Send + Sync {
    fn shutdown_input(&mut self) -> IoResult<()>;
    fn shutdown_output(&mut self) -> IoResult<()>;
    fn supported_cipher_suites(&self) -> Vec<String>;
    fn enabled_cipher_suites(&self) -> Vec<String>;
    fn set_enabled_cipher_suites(&mut self, suites: Vec<String>);
    fn supported_protocols(&self) -> Vec<String>;
    fn enabled_protocols(&self) -> Vec<String>;
    fn set_enabled_protocols(&mut self, protocols: Vec<String>);
    fn session(&self) -> Arc<dyn SSLSession>;
    fn add_handshake_completed_listener(&mut self, listener: Arc<dyn HandshakeCompletedListener>);
    fn remove_handshake_completed_listener(&mut self, listener: Arc<dyn HandshakeCompletedListener>);
    fn start_handshake(&mut self) -> IoResult<()>;
    fn use_client_mode(&self) -> bool;
    fn set_use_client_mode(&mut self, mode: bool);
    fn need_client_auth(&self) -> bool;
    fn set_need_client_auth(&mut self, need: bool);
    fn want_client_auth(&self) -> bool;
    fn set_want_client_auth(&mut self, want: bool);
    fn enable_session_creation(&self) -> bool;
    fn set_enable_session_creation(&mut self, flag: bool);
    fn ssl_parameters(&self) -> Arc<dyn SSLParameters>;
    fn set_ssl_parameters(&mut self, p: Arc<dyn SSLParameters>);
    fn inet_address(&self) -> IpAddr;
    fn keep_alive(&self) -> IoResult<bool>;
    fn set_keep_alive(&mut self, keep_alive: bool) -> IoResult<()>;
    fn local_address(&self) -> IpAddr;
    fn local_port(&self) -> i32;
    fn port(&self) -> i32;
    fn so_linger(&self) -> IoResult<i32>;
    fn set_so_linger(&mut self, on: bool, timeout: i32) -> IoResult<()>;
    fn receive_buffer_size(&self) -> IoResult<i32>;
    fn set_receive_buffer_size(&mut self, size: i32) -> IoResult<()>;
    fn send_buffer_size(&self) -> IoResult<i32>;
    fn set_send_buffer_size(&mut self, size: i32) -> IoResult<()>;
    fn so_timeout(&self) -> IoResult<i32>;
    fn set_so_timeout(&mut self, timeout: i32) -> IoResult<()>;
    fn tcp_no_delay(&self) -> IoResult<bool>;
    fn set_tcp_no_delay(&mut self, on: bool) -> IoResult<()>;
    fn local_socket_address(&self) -> SocketAddr;
    fn remote_socket_address(&self) -> SocketAddr;
    fn is_bound(&self) -> bool;
    fn is_connected(&self) -> bool;
    fn is_closed(&self) -> bool;
    fn bind(&mut self, local_addr: SocketAddr) -> IoResult<()>;
    fn connect(&mut self, remote_addr: SocketAddr) -> IoResult<()>;
    fn connect_with_timeout(&mut self, remote_addr: SocketAddr, timeout: i32) -> IoResult<()>;
    fn is_input_shutdown(&self) -> bool;
    fn is_output_shutdown(&self) -> bool;
    fn reuse_address(&self) -> IoResult<bool>;
    fn set_reuse_address(&mut self, reuse: bool) -> IoResult<()>;
    fn oob_inline(&self) -> IoResult<bool>;
    fn set_oob_inline(&mut self, oobinline: bool) -> IoResult<()>;
    fn traffic_class(&self) -> IoResult<i32>;
    fn set_traffic_class(&mut self, value: i32) -> IoResult<()>;
    fn send_urgent_data(&mut self, value: i32) -> IoResult<()>;
    fn channel(&self) -> Arc<dyn SocketChannel>;
    fn handshake_session(&self) -> Arc<dyn SSLSession>;
    fn application_protocol(&self) -> String;
    fn handshake_application_protocol(&self) -> String;
    fn handshake_application_protocol_selector(&self) -> Option<Arc<dyn Fn(Arc<dyn SSLSocket>, Vec<String>) -> String + Send + Sync>>;
    fn set_handshake_application_protocol_selector(&mut self, selector: Option<Arc<dyn Fn(Arc<dyn SSLSocket>, Vec<String>) -> String + Send + Sync>>);
}

// An SSLSocket that delegates all calls.
// In Rust, since we cannot inherit from a class, we implement the SSLSocket trait
// by wrapping a delegate.
pub struct DelegatingSSLSocket {
    pub delegate: Option<Arc<dyn SSLSocket>>,
}

impl DelegatingSSLSocket {
    pub fn new(delegate: Option<Arc<dyn SSLSocket>>) -> Self {
        Self { delegate }
    }

    fn get_delegate(&self) -> &Arc<dyn SSLSocket> {
        self.delegate.as_ref().expect("delegate must not be null")
    }

    fn get_delegate_mut(&mut self) -> &mut Arc<dyn SSLSocket> {
        self.delegate.as_mut().expect("delegate must not be null")
    }
}

impl Read for DelegatingSSLSocket {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.get_delegate_mut().read(buf)
    }
}

impl Write for DelegatingSSLSocket {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.get_delegate_mut().write(buf)
    }
    fn flush(&mut self) -> IoResult<()> {
        self.get_delegate_mut().flush()
    }
}

impl SSLSocket for DelegatingSSLSocket {
    fn shutdown_input(&mut self) -> IoResult<()> {
        self.get_delegate_mut().shutdown_input()
    }

    fn shutdown_output(&mut self) -> IoResult<()> {
        self.get_delegate_mut().shutdown_output()
    }

    fn supported_cipher_suites(&self) -> Vec<String> {
        self.get_delegate().supported_cipher_suites()
    }

    fn enabled_cipher_suites(&self) -> Vec<String> {
        self.get_delegate().enabled_cipher_suites()
    }

    fn set_enabled_cipher_suites(&mut self, suites: Vec<String>) {
        self.get_delegate_mut().set_enabled_cipher_suites(suites)
    }

    fn supported_protocols(&self) -> Vec<String> {
        self.get_delegate().supported_protocols()
    }

    fn enabled_protocols(&self) -> Vec<String> {
        self.get_delegate().enabled_protocols()
    }

    fn set_enabled_protocols(&mut self, protocols: Vec<String>) {
        self.get_delegate_mut().set_enabled_protocols(protocols)
    }

    fn session(&self) -> Arc<dyn SSLSession> {
        self.get_delegate().session()
    }

    fn add_handshake_completed_listener(&mut self, listener: Arc<dyn HandshakeCompletedListener>) {
        self.get_delegate_mut().add_handshake_completed_listener(listener)
    }

    fn remove_handshake_completed_listener(&mut self, listener: Arc<dyn HandshakeCompletedListener>) {
        self.get_delegate_mut().remove_handshake_completed_listener(listener)
    }

    fn start_handshake(&mut self) -> IoResult<()> {
        self.get_delegate_mut().start_handshake()
    }

    fn use_client_mode(&self) -> bool {
        self.get_delegate().use_client_mode()
    }

    fn set_use_client_mode(&mut self, mode: bool) {
        self.get_delegate_mut().set_use_client_mode(mode)
    }

    fn need_client_auth(&self) -> bool {
        self.get_delegate().need_client_auth()
    }

    fn set_need_client_auth(&mut self, need: bool) {
        self.get_delegate_mut().set_need_client_auth(need)
    }

    fn want_client_auth(&self) -> bool {
        self.get_delegate().want_client_auth()
    }

    fn set_want_client_auth(&mut self, want: bool) {
        self.get_delegate_mut().set_want_client_auth(want)
    }

    fn enable_session_creation(&self) -> bool {
        self.get_delegate().enable_session_creation()
    }

    fn set_enable_session_creation(&mut self, flag: bool) {
        self.get_delegate_mut().set_enable_session_creation(flag)
    }

    fn ssl_parameters(&self) -> Arc<dyn SSLParameters> {
        self.get_delegate().ssl_parameters()
    }

    fn set_ssl_parameters(&mut self, p: Arc<dyn SSLParameters>) {
        self.get_delegate_mut().set_ssl_parameters(p)
    }

    fn inet_address(&self) -> IpAddr {
        self.get_delegate().inet_address()
    }

    fn keep_alive(&self) -> IoResult<bool> {
        self.get_delegate().keep_alive()
    }

    fn set_keep_alive(&mut self, keep_alive: bool) -> IoResult<()> {
        self.get_delegate_mut().set_keep_alive(keep_alive)
    }

    fn local_address(&self) -> IpAddr {
        self.get_delegate().local_address()
    }

    fn local_port(&self) -> i32 {
        self.get_delegate().local_port()
    }

    fn port(&self) -> i32 {
        self.get_delegate().port()
    }

    fn so_linger(&self) -> IoResult<i32> {
        self.get_delegate().so_linger()
    }

    fn set_so_linger(&mut self, on: bool, timeout: i32) -> IoResult<()> {
        self.get_delegate_mut().set_so_linger(on, timeout)
    }

    fn receive_buffer_size(&self) -> IoResult<i32> {
        self.get_delegate().receive_buffer_size()
    }

    fn set_receive_buffer_size(&mut self, size: i32) -> IoResult<()> {
        self.get_delegate_mut().set_receive_buffer_size(size)
    }

    fn send_buffer_size(&self) -> IoResult<i32> {
        self.get_delegate().send_buffer_size()
    }

    fn set_send_buffer_size(&mut self, size: i32) -> IoResult<()> {
        self.get_delegate_mut().set_send_buffer_size(size)
    }

    fn so_timeout(&self) -> IoResult<i32> {
        self.get_delegate().so_timeout()
    }

    fn set_so_timeout(&mut self, timeout: i32) -> IoResult<()> {
        self.get_delegate_mut().set_so_timeout(timeout)
    }

    fn tcp_no_delay(&self) -> IoResult<bool> {
        self.get_delegate().tcp_no_delay()
    }

    fn set_tcp_no_delay(&mut self, on: bool) -> IoResult<()> {
        self.get_delegate_mut().set_tcp_no_delay(on)
    }

    fn local_socket_address(&self) -> SocketAddr {
        self.get_delegate().local_socket_address()
    }

    fn remote_socket_address(&self) -> SocketAddr {
        self.get_delegate().remote_socket_address()
    }

    fn is_bound(&self) -> bool {
        self.get_delegate().is_bound()
    }

    fn is_connected(&self) -> bool {
        self.get_delegate().is_connected()
    }

    fn is_closed(&self) -> bool {
        self.get_delegate().is_closed()
    }

    fn bind(&mut self, local_addr: SocketAddr) -> IoResult<()> {
        self.get_delegate_mut().bind(local_addr)
    }

    fn connect(&mut self, remote_addr: SocketAddr) -> IoResult<()> {
        self.get_delegate_mut().connect(remote_addr)
    }

    fn connect_with_timeout(&mut self, remote_addr: SocketAddr, timeout: i32) -> IoResult<()> {
        self.get_delegate_mut().connect_with_timeout(remote_addr, timeout)
    }

    fn is_input_shutdown(&self) -> bool {
        self.get_delegate().is_input_shutdown()
    }

    fn is_output_shutdown(&self) -> bool {
        self.get_delegate().is_output_shutdown()
    }

    fn reuse_address(&self) -> IoResult<bool> {
        self.get_delegate().reuse_address()
    }

    fn set_reuse_address(&mut self, reuse: bool) -> IoResult<()> {
        self.get_delegate_mut().set_reuse_address(reuse)
    }

    fn oob_inline(&self) -> IoResult<bool> {
        self.get_delegate().oob_inline()
    }

    fn set_oob_inline(&mut self, oobinline: bool) -> IoResult<()> {
        self.get_delegate_mut().set_oob_inline(oobinline)
    }

    fn traffic_class(&self) -> IoResult<i32> {
        self.get_delegate().traffic_class()
    }

    fn set_traffic_class(&mut self, value: i32) -> IoResult<()> {
        self.get_delegate_mut().set_traffic_class(value)
    }

    fn send_urgent_data(&mut self, value: i32) -> IoResult<()> {
        self.get_delegate_mut().send_urgent_data(value)
    }

    fn channel(&self) -> Arc<dyn SocketChannel> {
        self.get_delegate().channel()
    }

    fn handshake_session(&self) -> Arc<dyn SSLSession> {
        self.get_delegate().handshake_session()
    }

    fn application_protocol(&self) -> String {
        self.get_delegate().application_protocol()
    }

    fn handshake_application_protocol(&self) -> String {
        self.get_delegate().handshake_application_protocol()
    }

    fn handshake_application_protocol_selector(&self) -> Option<Arc<dyn Fn(Arc<dyn SSLSocket>, Vec<String>) -> String + Send + Sync>> {
        self.get_delegate().handshake_application_protocol_selector()
    }

    fn set_handshake_application_protocol_selector(&mut self, selector: Option<Arc<dyn Fn(Arc<dyn SSLSocket>, Vec<String>) -> String + Send + Sync>>) {
        self.get_delegate_mut().set_handshake_application_protocol_selector(selector)
    }
}

impl std::fmt::Debug for DelegatingSSLSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Since we don't have a Debug impl for the trait object, we represent the delegate's presence
        f.debug_struct("DelegatingSSLSocket")
            .field("delegate", &self.delegate.as_ref().map(|_| "SSLSocket"))
            .finish()
    }
}
)}
