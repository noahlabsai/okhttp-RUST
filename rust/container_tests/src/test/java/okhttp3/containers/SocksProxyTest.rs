/*
 * Copyright (C) 2024 Square, Inc.
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

use std::net::InetSocketAddress;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::OkHttpClient::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrl;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::android_test::src::test::kotlin::okhttp::android::test::BaseOkHttpClientUnitTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::container_tests::src::test::java::okhttp3::containers::BasicLoomTest::*;

// Mocking the external Testcontainers and MockServer dependencies as they are Java-specific.
// In a real Rust translation, these would be replaced by equivalent Rust crates 
// (e.g., testcontainers-rs) or trait-based mocks.
pub struct Network;
impl Network {
    pub fn new_network() -> Self { Network }
}

pub struct DockerImageName(pub String);
impl DockerImageName {
    pub fn parse(name: &str) -> Self { DockerImageName(name.to_string()) }
    pub fn with_tag(self, tag: &str) -> Self {
        DockerImageName(format!("{}:{}", self.0, tag))
    }
}

impl MockServerContainer {
    pub fn new(image: DockerImageName) -> Self {
        MockServerContainer {
            host: "localhost".to_string(),
            server_port: 1080,
        }
    }
    pub fn with_network(self, _network: Network) -> Self { self }
    pub fn with_network_aliases(self, _aliases: &str) -> Self { self }
}

pub struct GenericContainer {
    pub host: String,
}
impl GenericContainer {
    pub fn new(image: DockerImageName) -> Self {
        GenericContainer {
            host: "localhost".to_string(),
        }
    }
    pub fn with_network(self, _network: Network) -> Self { self }
    pub fn with_exposed_ports(self, _port: i32) -> Self { self }
    pub fn first_mapped_port(&self) -> i32 { 1080 }
}

impl MockServerClient {
    pub fn new(host: String, port: i32) -> Self {
        MockServerClient { host, port }
    }
    pub fn when(self, _request: HttpRequest) -> MockServerClientExpectation {
        MockServerClientExpectation { client: self }
    }
}

pub struct HttpRequest;
impl HttpRequest {
    pub fn request() -> Self { HttpRequest }
    pub fn with_path(self, _path: &str) -> Self { self }
    pub fn with_query_string_parameter(self, _key: &str, _val: &str) -> Self { self }
}

pub struct HttpResponse;
impl HttpResponse {
    pub fn response() -> Self { HttpResponse }
    pub fn with_body(self, _body: &str) -> Self { self }
}

pub struct MockServerClientExpectation {
    pub client: MockServerClient,
}
impl MockServerClientExpectation {
    pub fn respond(self, _response: HttpResponse) {
        // Mock implementation of response setup
    }
}

// Java Proxy equivalent
pub enum ProxyType {
    SOCKS,
    HTTP,
    DIRECT,
}

impl Default for ProxyType {
    fn default() -> Self {
        ProxyType::SOCKS
    }
}

pub const SOCKS: ProxyType = ProxyType::SOCKS;
pub const HTTP: ProxyType = ProxyType::HTTP;
pub const DIRECT: ProxyType = ProxyType::DIRECT;

pub struct Proxy {
    pub proxy_type: ProxyType,
    pub socket_address: InetSocketAddress,
}

impl Proxy {
    pub fn new(proxy_type: ProxyType, socket_address: InetSocketAddress) -> Self {
        Proxy { proxy_type, socket_address }
    }
}

pub struct SocksProxyTest {
    pub network: Network,
    pub mock_server: MockServerContainer,
    pub socks5_proxy: GenericContainer,
}

impl SocksProxyTest {
    pub fn new() -> Self {
        let network = Network::new_network();
        let mock_server = MockServerContainer::new(DockerImageName::parse("mockserver/mockserver"))
            .with_network(network) // Note: In Rust, we'd need to clone or use Arc for network
            .with_network_aliases("mockserver");
        
        let socks5_proxy = GenericContainer::new(Self::SOCKS5_PROXY())
            .with_network(Network::new_network())
            .with_exposed_ports(1080);

        SocksProxyTest {
            network: Network::new_network(),
            mock_server,
            socks5_proxy,
        }
    }

    pub fn test_local(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mock_server_client = MockServerClient::new(
            self.mock_server.host.clone(), 
            self.mock_server.server_port
        );

        // .r#use block in Kotlin is handled by scope or explicit drop in Rust
        {
            let client_ref = &mock_server_client;
            client_ref.when(
                HttpRequest::request()
                    .with_path("/person")
                    .with_query_string_parameter("name", "peter"),
            ).respond(HttpResponse::response().with_body("Peter the person!"));
        }

        let proxy = Proxy::new(
            ProxyType::SOCKS, 
            InetSocketAddress::new(
                self.socks5_proxy.host.parse().unwrap_or("127.0.0.1".parse().unwrap()), 
                self.socks5_proxy.first_mapped_port() as u16
            )
        );

        let client = OkHttpClient::builder()
            .proxy(proxy)
            .build();

        // "http://mockserver:1080/person?name=peter".toHttpUrl()
        let url = HttpUrl::parse("http://mockserver:1080/person?name=peter")
            .expect("Invalid URL");

        let request = Request::builder()
            .url(url)
            .build();

        let response = client.new_call(request).execute()?;

        let body_string = response.body().and_then(|b| b.string()).unwrap_or_default();
        
        if !body_string.contains("Peter the person") {
            return Err("Response body did not contain expected string".into());
        }

        Ok(())
    }

    fn SOCKS5_PROXY() -> DockerImageName {
        DockerImageName::parse("serjs/go-socks5-proxy").with_tag("v0.0.3")
    }
}