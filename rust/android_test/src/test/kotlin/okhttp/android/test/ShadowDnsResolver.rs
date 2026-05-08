/*
 * Copyright (C) 2024 Block, Inc.
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

use std::net::IpAddr;
use std::sync::Arc;

// Mocking Android framework types as they are dependencies in the original Kotlin code
pub struct Network;
pub struct CancellationSignal;
pub struct Executor;

// Equivalent to android.net.DnsResolver
pub struct DnsResolver;

impl DnsResolver {
    pub trait Callback<T> {
        fn onAnswer(&self, answers: T, errorCode: i32);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    pub network: Option<Arc<Network>>,
    pub domain: String,
    pub ns_type: i32,
    pub flags: i32,
    pub callback: Arc<dyn DnsResolver::Callback<Vec<IpAddr>> + Send + Sync>,
}

pub struct ShadowDnsResolver {
    pub responder: Box<dyn Fn(Request) + Send + Sync>,
}

impl Default for ShadowDnsResolver {
    fn default() -> Self {
        Self {
            responder: Box::new(|req: Request| {
                req.callback.onAnswer(Vec::new(), 0);
            }),
        }
    }
}

impl ShadowDnsResolver {
    pub fn new() -> Self {
        Self::default()
    }

    // Implementation of the query method
    pub fn query(
        &self,
        network: Option<Arc<Network>>,
        domain: String,
        ns_type: i32,
        flags: i32,
        _executor: Arc<Executor>,
        _cancellation_signal: Option<Arc<CancellationSignal>>,
        callback: Arc<dyn DnsResolver::Callback<Vec<IpAddr>> + Send + Sync>,
    ) {
        let request = Request {
            network,
            domain,
            ns_type,
            flags,
            callback,
        };
        (self.responder)(request);
    }

    // Companion object method: getInstance
    // In Rust, this is an associated function. 
    // Note: Shadow.newInstance is a Robolectric-specific reflection call.
    // In a Rust translation, we return a new instance of the target type.
    pub fn get_instance() -> DnsResolver {
        DnsResolver
    }
}