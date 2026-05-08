use std::io;
use std::net::{IpAddr, TcpListener};
use crate::mockwebserver_deprecated::src::test::java::okhttp3::mockwebserver::KotlinSourceModernTest::*;

// A trait representing the ServerSocketFactory functionality.
// In Rust, ServerSocket is equivalent to TcpListener.
pub trait ServerSocketFactory: Send + Sync {
    fn create_server_socket(&self) -> io::Result<TcpListener>;
    fn create_server_socket_port(&self, port: i32) -> io::Result<TcpListener>;
    fn create_server_socket_port_backlog(&self, port: i32, backlog: i32) -> io::Result<TcpListener>;
    fn create_server_socket_port_backlog_if_address(&self, port: i32, backlog: i32, if_address: IpAddr) -> io::Result<TcpListener>;
}

/*
 * A [ServerSocketFactory] that delegates calls. Sockets can be configured after creation by
 * overriding [configure_server_socket].
 */
pub struct DelegatingServerSocketFactory {
    delegate: Box<dyn ServerSocketFactory>,
}

impl DelegatingServerSocketFactory {
    pub fn new(delegate: Box<dyn ServerSocketFactory>) -> Self {
        Self { delegate }
    }

    // This method corresponds to the protected open fun configureServerSocket in Kotlin.
    // Since Rust doesn't have protected methods in the same way, this is a public method
    // that can be overridden if this struct were a trait or if using a different pattern.
    // To preserve the "open" behavior for potential subclasses (which in Rust would be 
    // composition or trait implementation), we provide it as a method.
    pub fn configure_server_socket(&self, server_socket: TcpListener) -> TcpListener {
        // No-op by default.
        server_socket
    }
}

impl ServerSocketFactory for DelegatingServerSocketFactory {
    fn create_server_socket(&self) -> io::Result<TcpListener> {
        let server_socket = self.delegate.create_server_socket()?;
        Ok(self.configure_server_socket(server_socket))
    }

    fn create_server_socket_port(&self, port: i32) -> io::Result<TcpListener> {
        let server_socket = self.delegate.create_server_socket_port(port)?;
        Ok(self.configure_server_socket(server_socket))
    }

    fn create_server_socket_port_backlog(&self, port: i32, backlog: i32) -> io::Result<TcpListener> {
        let server_socket = self.delegate.create_server_socket_port_backlog(port, backlog)?;
        Ok(self.configure_server_socket(server_socket))
    }

    fn create_server_socket_port_backlog_if_address(&self, port: i32, backlog: i32, if_address: IpAddr) -> io::Result<TcpListener> {
        let server_socket = self.delegate.create_server_socket_port_backlog_if_address(port, backlog, if_address)?;
        Ok(self.configure_server_socket(server_socket))
    }
}