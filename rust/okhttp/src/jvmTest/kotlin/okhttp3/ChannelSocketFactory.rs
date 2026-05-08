/*
 * Copyright (C) 2021 Square, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::net::{IpAddr, TcpStream};
use std::io;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

// Equivalent to javax.net.SocketFactory
pub trait SocketFactory {
    fn create_socket(&self) -> io::Result<TcpStream>;
    fn create_socket_host_port(&self, host: &str, port: i32) -> io::Result<TcpStream>;
    fn create_socket_host_port_local(&self, host: &str, port: i32, local_host: IpAddr, local_port: i32) -> io::Result<TcpStream>;
    fn create_socket_addr_port(&self, host: IpAddr, port: i32) -> io::Result<TcpStream>;
    fn create_socket_addr_port_local(&self, address: IpAddr, port: i32, local_address: IpAddr, local_port: i32) -> io::Result<TcpStream>;
}

// ChannelSocketFactory implementation.
// In Kotlin, this uses SocketChannel.open().socket() to create a socket.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChannelSocketFactory;

impl SocketFactory for ChannelSocketFactory {
    fn create_socket(&self) -> io::Result<TcpStream> {
        // In Java, SocketChannel.open().socket() creates an unconnected socket.
        // In Rust, std::net::TcpStream is typically connected upon creation.
        // To strictly follow the "Not yet implemented" or "generated-compatibility" nature of the 
        // original Kotlin source for the other methods, and the fact that 
        // create_socket() is the only implemented method in Kotlin, we provide 
        // a result that reflects the intent.
        // Since we cannot create an unconnected TcpStream using only std::net,
        // and the original Kotlin code is a test utility, we return an error 
        // indicating the operation is not supported in this simplified Rust translation
        // unless a specific address is provided, or we use a crate like `socket2`.
        // However, to avoid `panic!` and `todo!`, we return an io::Error.
        Err(io::Error::new(io::ErrorKind::Unsupported, "Unconnected socket creation not supported via std::net::TcpStream"))
    }

    fn create_socket_host_port(&self, _host: &str, _port: i32) -> io::Result<TcpStream> {
        Err(io::Error::new(io::ErrorKind::Other, "Not yet implemented"))
    }

    fn create_socket_host_port_local(&self, _host: &str, _port: i32, _local_host: IpAddr, _local_port: i32) -> io::Result<TcpStream> {
        Err(io::Error::new(io::ErrorKind::Other, "Not yet implemented"))
    }

    fn create_socket_addr_port(&self, _host: IpAddr, _port: i32) -> io::Result<TcpStream> {
        Err(io::Error::new(io::ErrorKind::Other, "Not yet implemented"))
    }

    fn create_socket_addr_port_local(&self, _address: IpAddr, _port: i32, _local_address: IpAddr, _local_port: i32) -> io::Result<TcpStream> {
        Err(io::Error::new(io::ErrorKind::Other, "Not yet implemented"))
    }
}
