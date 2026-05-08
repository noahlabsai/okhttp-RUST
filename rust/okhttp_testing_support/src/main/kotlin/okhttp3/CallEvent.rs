/*
 * Copyright (C) 2017 Square, Inc.
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

use std::net::{InetAddress, SocketAddr};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::SuppressSignatureCheck;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::container_tests::src::test::java::okhttp3::containers::SocksProxyTest::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Request::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::CallTagsTest::*;

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::{
    Call, Connection, Dispatcher, Handshake, HttpUrl, Protocol, Proxy, Request, Response,
};

// Data classes that correspond to each of the methods of [EventListener].
#[derive(Debug, Clone, PartialEq)]
pub enum CallEvent {
    DispatcherQueueStart {
        timestamp_ns: i64,
        call: Call,
        dispatcher: Dispatcher,
    }

impl Default for CallEvent {
    fn default() -> Self {
        CallEvent::DispatcherQueueStart
    }
}

pub const DispatcherQueueStart: CallEvent = CallEvent::DispatcherQueueStart;
pub const timestamp_ns: CallEvent = CallEvent::timestamp_ns;
pub const call: CallEvent = CallEvent::call;
pub const dispatcher: CallEvent = CallEvent::dispatcher;,
    DispatcherQueueEnd {
        timestamp_ns: i64,
        call: Call,
        dispatcher: Dispatcher,
    },
    ProxySelectStart {
        timestamp_ns: i64,
        call: Call,
        url: HttpUrl,
    },
    ProxySelectEnd {
        timestamp_ns: i64,
        call: Call,
        url: HttpUrl,
        proxies: Option<Vec<Proxy>>,
    },
    DnsStart {
        timestamp_ns: i64,
        call: Call,
        domain_name: String,
    },
    DnsEnd {
        timestamp_ns: i64,
        call: Call,
        domain_name: String,
        inet_address_list: Vec<InetAddress>,
    },
    ConnectStart {
        timestamp_ns: i64,
        call: Call,
        inet_socket_address: SocketAddr,
        proxy: Option<Proxy>,
    },
    ConnectEnd {
        timestamp_ns: i64,
        call: Call,
        inet_socket_address: SocketAddr,
        proxy: Option<Proxy>,
        protocol: Option<Protocol>,
    },
    ConnectFailed {
        timestamp_ns: i64,
        call: Call,
        inet_socket_address: SocketAddr,
        proxy: Proxy,
        protocol: Option<Protocol>,
        ioe: Box<dyn std::error::Error>,
    },
    SecureConnectStart {
        timestamp_ns: i64,
        call: Call,
    },
    SecureConnectEnd {
        timestamp_ns: i64,
        call: Call,
        handshake: Option<Handshake>,
    },
    ConnectionAcquired {
        timestamp_ns: i64,
        call: Call,
        connection: Connection,
    },
    ConnectionReleased {
        timestamp_ns: i64,
        call: Call,
        connection: Connection,
    },
    CallStart {
        timestamp_ns: i64,
        call: Call,
    },
    CallEnd {
        timestamp_ns: i64,
        call: Call,
    },
    CallFailed {
        timestamp_ns: i64,
        call: Call,
        ioe: Box<dyn std::error::Error>,
    },
    Canceled {
        timestamp_ns: i64,
        call: Call,
    },
    RequestHeadersStart {
        timestamp_ns: i64,
        call: Call,
    },
    RequestHeadersEnd {
        timestamp_ns: i64,
        call: Call,
        header_length: i64,
    },
    RequestBodyStart {
        timestamp_ns: i64,
        call: Call,
    },
    RequestBodyEnd {
        timestamp_ns: i64,
        call: Call,
        bytes_written: i64,
    },
    RequestFailed {
        timestamp_ns: i64,
        call: Call,
        ioe: Box<dyn std::error::Error>,
    },
    ResponseHeadersStart {
        timestamp_ns: i64,
        call: Call,
    },
    ResponseHeadersEnd {
        timestamp_ns: i64,
        call: Call,
        header_length: i64,
    },
    ResponseBodyStart {
        timestamp_ns: i64,
        call: Call,
    },
    ResponseBodyEnd {
        timestamp_ns: i64,
        call: Call,
        bytes_read: i64,
    },
    ResponseFailed {
        timestamp_ns: i64,
        call: Call,
        ioe: Box<dyn std::error::Error>,
    },
    SatisfactionFailure {
        timestamp_ns: i64,
        call: Call,
    },
    CacheHit {
        timestamp_ns: i64,
        call: Call,
    },
    CacheMiss {
        timestamp_ns: i64,
        call: Call,
    },
    CacheConditionalHit {
        timestamp_ns: i64,
        call: Call,
    },
    RetryDecision {
        timestamp_ns: i64,
        call: Call,
        exception: Box<dyn std::error::Error>,
        retry: bool,
    },
    FollowUpDecision {
        timestamp_ns: i64,
        call: Call,
        network_response: Response,
        next_request: Option<Request>,
    },
}

impl CallEvent {
    pub fn timestamp_ns(&self) -> i64 {
        match self {
            CallEvent::DispatcherQueueStart { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::DispatcherQueueEnd { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::ProxySelectStart { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::ProxySelectEnd { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::DnsStart { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::DnsEnd { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::ConnectStart { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::ConnectEnd { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::ConnectFailed { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::SecureConnectStart { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::SecureConnectEnd { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::ConnectionAcquired { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::ConnectionReleased { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::CallStart { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::CallEnd { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::CallFailed { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::Canceled { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::RequestHeadersStart { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::RequestHeadersEnd { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::RequestBodyStart { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::RequestBodyEnd { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::RequestFailed { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::ResponseHeadersStart { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::ResponseHeadersEnd { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::ResponseBodyStart { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::ResponseBodyEnd { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::ResponseFailed { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::SatisfactionFailure { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::CacheHit { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::CacheMiss { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::CacheConditionalHit { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::RetryDecision { timestamp_ns, .. } => *timestamp_ns,
            CallEvent::FollowUpDecision { timestamp_ns, .. } => *timestamp_ns,
        }
    }

    pub fn call(&self) -> &Call {
        match self {
            CallEvent::DispatcherQueueStart { call, .. } => call,
            CallEvent::DispatcherQueueEnd { call, .. } => call,
            CallEvent::ProxySelectStart { call, .. } => call,
            CallEvent::ProxySelectEnd { call, .. } => call,
            CallEvent::DnsStart { call, .. } => call,
            CallEvent::DnsEnd { call, .. } => call,
            CallEvent::ConnectStart { call, .. } => call,
            CallEvent::ConnectEnd { call, .. } => call,
            CallEvent::ConnectFailed { call, .. } => call,
            CallEvent::SecureConnectStart { call, .. } => call,
            CallEvent::SecureConnectEnd { call, .. } => call,
            CallEvent::ConnectionAcquired { call, .. } => call,
            CallEvent::ConnectionReleased { call, .. } => call,
            CallEvent::CallStart { call, .. } => call,
            CallEvent::CallEnd { call, .. } => call,
            CallEvent::CallFailed { call, .. } => call,
            CallEvent::Canceled { call, .. } => call,
            CallEvent::RequestHeadersStart { call, .. } => call,
            CallEvent::RequestHeadersEnd { call, .. } => call,
            CallEvent::RequestBodyStart { call, .. } => call,
            CallEvent::RequestBodyEnd { call, .. } => call,
            CallEvent::RequestFailed { call, .. } => call,
            CallEvent::ResponseHeadersStart { call, .. } => call,
            CallEvent::ResponseHeadersEnd { call, .. } => call,
            CallEvent::ResponseBodyStart { call, .. } => call,
            CallEvent::ResponseBodyEnd { call, .. } => call,
            CallEvent::ResponseFailed { call, .. } => call,
            CallEvent::SatisfactionFailure { call, .. } => call,
            CallEvent::CacheHit { call, .. } => call,
            CallEvent::CacheMiss { call, .. } => call,
            CallEvent::CacheConditionalHit { call, .. } => call,
            CallEvent::RetryDecision { call, .. } => call,
            CallEvent::FollowUpDecision { call, .. } => call,
        }
    }

    pub fn name(&self) -> String {
        match self {
            CallEvent::DispatcherQueueStart { .. } => "DispatcherQueueStart".to_string(),
            CallEvent::DispatcherQueueEnd { .. } => "DispatcherQueueEnd".to_string(),
            CallEvent::ProxySelectStart { .. } => "ProxySelectStart".to_string(),
            CallEvent::ProxySelectEnd { .. } => "ProxySelectEnd".to_string(),
            CallEvent::DnsStart { .. } => "DnsStart".to_string(),
            CallEvent::DnsEnd { .. } => "DnsEnd".to_string(),
            CallEvent::ConnectStart { .. } => "ConnectStart".to_string(),
            CallEvent::ConnectEnd { .. } => "ConnectEnd".to_string(),
            CallEvent::ConnectFailed { .. } => "ConnectFailed".to_string(),
            CallEvent::SecureConnectStart { .. } => "SecureConnectStart".to_string(),
            CallEvent::SecureConnectEnd { .. } => "SecureConnectEnd".to_string(),
            CallEvent::ConnectionAcquired { .. } => "ConnectionAcquired".to_string(),
            CallEvent::ConnectionReleased { .. } => "ConnectionReleased".to_string(),
            CallEvent::CallStart { .. } => "CallStart".to_string(),
            CallEvent::CallEnd { .. } => "CallEnd".to_string(),
            CallEvent::CallFailed { .. } => "CallFailed".to_string(),
            CallEvent::Canceled { .. } => "Canceled".to_string(),
            CallEvent::RequestHeadersStart { .. } => "RequestHeadersStart".to_string(),
            CallEvent::RequestHeadersEnd { .. } => "RequestHeadersEnd".to_string(),
            CallEvent::RequestBodyStart { .. } => "RequestBodyStart".to_string(),
            CallEvent::RequestBodyEnd { .. } => "RequestBodyEnd".to_string(),
            CallEvent::RequestFailed { .. } => "RequestFailed".to_string(),
            CallEvent::ResponseHeadersStart { .. } => "ResponseHeadersStart".to_string(),
            CallEvent::ResponseHeadersEnd { .. } => "ResponseHeadersEnd".to_string(),
            CallEvent::ResponseBodyStart { .. } => "ResponseBodyStart".to_string(),
            CallEvent::ResponseBodyEnd { .. } => "ResponseBodyEnd".to_string(),
            CallEvent::ResponseFailed { .. } => "ResponseFailed".to_string(),
            CallEvent::SatisfactionFailure { .. } => "SatisfactionFailure".to_string(),
            CallEvent::CacheHit { .. } => "CacheHit".to_string(),
            CallEvent::CacheMiss { .. } => "CacheMiss".to_string(),
            CallEvent::CacheConditionalHit { .. } => "CacheConditionalHit".to_string(),
            CallEvent::RetryDecision { .. } => "RetryDecision".to_string(),
            CallEvent::FollowUpDecision { .. } => "FollowUpDecision".to_string(),
        }
    }

    // Returns if the event closes this event, or null if this is no open event.
    pub fn closes(&self, event: &CallEvent) -> Option<bool> {
        match self {
            CallEvent::DispatcherQueueEnd { call, .. } => {
                if let CallEvent::DispatcherQueueStart { call: start_call, .. } = event {
                    Some(call == start_call)
                } else {
                    Some(false)
                }
            }
            CallEvent::ProxySelectEnd { call, url, .. } => {
                if let CallEvent::ProxySelectStart { call: start_call, url: start_url, .. } = event {
                    Some(call == start_call && url == start_url)
                } else {
                    Some(false)
                }
            }
            CallEvent::DnsEnd { call, domain_name, .. } => {
                if let CallEvent::DnsStart { call: start_call, domain_name: start_domain, .. } = event {
                    Some(call == start_call && domain_name == start_domain)
                } else {
                    Some(false)
                }
            }
            CallEvent::ConnectEnd { call, inet_socket_address, proxy, .. } => {
                if let CallEvent::ConnectStart { call: start_call, inet_socket_address: start_addr, proxy: start_proxy, .. } = event {
                    Some(call == start_call && inet_socket_address == start_addr && proxy == start_proxy)
                } else {
                    Some(false)
                }
            }
            CallEvent::ConnectFailed { call, inet_socket_address, proxy, .. } => {
                if let CallEvent::ConnectStart { call: start_call, inet_socket_address: start_addr, proxy: start_proxy, .. } = event {
                    Some(call == start_call && inet_socket_address == start_addr && Some(proxy) == start_proxy)
                } else {
                    Some(false)
                }
            }
            CallEvent::SecureConnectEnd { call, .. } => {
                if let CallEvent::SecureConnectStart { call: start_call, .. } = event {
                    Some(call == start_call)
                } else {
                    Some(false)
                }
            }
            CallEvent::ConnectionReleased { call, connection, .. } => {
                if let CallEvent::ConnectionAcquired { call: start_call, connection: start_conn, .. } = event {
                    Some(call == start_call && connection == start_conn)
                } else {
                    Some(false)
                }
            }
            CallEvent::CallEnd { call, .. } => {
                if let CallEvent::CallStart { call: start_call, .. } = event {
                    Some(call == start_call)
                } else {
                    Some(false)
                }
            }
            CallEvent::CallFailed { call, .. } => {
                if let CallEvent::CallStart { call: start_call, .. } = event {
                    Some(call == start_call)
                } else {
                    Some(false)
                }
            }
            CallEvent::RequestHeadersEnd { call, .. } => {
                if let CallEvent::RequestHeadersStart { call: start_call, .. } = event {
                    Some(call == start_call)
                } else {
                    Some(false)
                }
            }
            CallEvent::RequestBodyEnd { call, .. } => {
                if let CallEvent::RequestBodyStart { call: start_call, .. } = event {
                    Some(call == start_call)
                } else {
                    Some(false)
                }
            }
            CallEvent::RequestFailed { call, .. } => {
                if let CallEvent::RequestHeadersStart { call: start_call, .. } = event {
                    Some(call == start_call)
                } else {
                    Some(false)
                }
            }
            CallEvent::ResponseHeadersEnd { call, .. } => {
                if let CallEvent::ResponseHeadersStart { call: start_call, .. } = event {
                    Some(call == start_call)
                } else {
                    Some(false)
                }
            }
            CallEvent::ResponseBodyEnd { call, .. } => {
                if let CallEvent::ResponseBodyStart { call: start_call, .. } = event {
                    Some(call == start_call)
                } else {
                    Some(false)
                }
            }
            _ => None,
        }
    }
}