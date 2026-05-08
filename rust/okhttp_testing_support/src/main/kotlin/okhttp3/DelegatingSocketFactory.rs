use std::io;
use std::net::{IpAddr, TcpStream};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

// A trait representing the SocketFactory functionality.
// In Kotlin, SocketFactory is a class; in Rust, we use a trait to allow for delegation and polymorphism.
pub trait SocketFactory: Send + Sync {
    fn create_socket(&self) -> io::Result<TcpStream>;
    fn create_socket_host_port(&self, host: &str, port: i32) -> io::Result<TcpStream>;
    fn create_socket_host_port_local(&self, host: &str, port: i32, local_address: IpAddr, local_port: i32) -> io::Result<TcpStream>;
    fn create_socket_addr_port(&self, host: IpAddr, port: i32) -> io::Result<TcpStream>;
    fn create_socket_addr_port_local(&self, host: IpAddr, port: i32, local_address: IpAddr, local_port: i32) -> io::Result<TcpStream>;
}

// A SocketFactory that delegates calls. Sockets can be configured after creation by
// overriding `configure_socket`.
// 
// In Rust, since we cannot "override" a method in a struct in the same way as Kotlin's `open` class,
// we use a generic closure or a trait object to handle the configuration logic.
pub struct DelegatingSocketFactory<F>
where
    F: Fn(TcpStream) -> io::Result<TcpStream> + Send + Sync,
{
    delegate: Box<dyn SocketFactory>,
    configure_socket: F,
}

impl<F> DelegatingSocketFactory<F>
where
    F: Fn(TcpStream) -> io::Result<TcpStream> + Send + Sync,
{
    pub fn new(delegate: Box<dyn SocketFactory>, configure_socket: F) -> Self {
        Self {
            delegate,
            configure_socket,
        }
    }

    // Default configuration is a no-op.
    pub fn default_config() -> impl Fn(TcpStream) -> io::Result<TcpStream> + Send + Sync {
        |socket| Ok(socket)
    }
}

impl<F> SocketFactory for DelegatingSocketFactory<F>
where
    F: Fn(TcpStream) -> io::Result<TcpStream> + Send + Sync,
{
    fn create_socket(&self) -> io::Result<TcpStream> {
        let socket = self.delegate.create_socket()?;
        (self.configure_socket)(socket)
    }

    fn create_socket_host_port(&self, host: &str, port: i32) -> io::Result<TcpStream> {
        let socket = self.delegate.create_socket_host_port(host, port)?;
        (self.configure_socket)(socket)
    }

    fn create_socket_host_port_local(&self, host: &str, port: i32, local_address: IpAddr, local_port: i32) -> io::Result<TcpStream> {
        let socket = self.delegate.create_socket_host_port_local(host, port, local_address, local_port)?;
        (self.configure_socket)(socket)
    }

    fn create_socket_addr_port(&self, host: IpAddr, port: i32) -> io::Result<TcpStream> {
        let socket = self.delegate.create_socket_addr_port(host, port)?;
        (self.configure_socket)(socket)
    }

    fn create_socket_addr_port_local(&self, host: IpAddr, port: i32, local_address: IpAddr, local_port: i32) -> io::Result<TcpStream> {
        let socket = self.delegate.create_socket_addr_port_local(host, port, local_address, local_port)?;
        (self.configure_socket)(socket)
    }
}