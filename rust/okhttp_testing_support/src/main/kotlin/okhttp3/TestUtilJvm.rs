/*
 * Copyright (C) 2018 Square, Inc.
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

use std::net::{InetSocketAddress, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;
use std::thread;
use std::env;

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Header;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Reader::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::UrlComponentEncodingTester::*;

// Mocking Okio Buffer as it's a dependency in the source
// In a real project, this would be imported from the okio crate.

impl Buffer {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    pub fn exhausted(&self) -> bool {
        self.data.is_empty()
    }
    pub fn write(&mut self, other: &mut Buffer, byte_count: usize) {
        let len = std::cmp::min(byte_count, other.data.len());
        self.data.extend_from_slice(&other.data[..len]);
        other.data.drain(..len);
    }
    pub fn copy(&self) -> Buffer {
        Buffer { data: self.data.clone() }
    }
}

// Mocking FileSystem.SYSTEM
pub struct FileSystem;
impl FileSystem {
}

pub struct TestUtil;

impl TestUtil {
    pub const UNREACHABLE_ADDRESS_IPV4: InetSocketAddress = 
        match "198.51.100.1:8080".parse() {
            Ok(addr) => addr,
            Err(_) => panic!("Invalid static address"),
        };

    pub fn unreachable_address_ipv6() -> InetSocketAddress {
        match "[::ffff:198.51.100.1]:8080".parse() {
            Ok(addr) => addr,
            Err(_) => panic!("Invalid static address"),
        }
    }

    pub fn is_graal_vm_image() -> bool {
        env::var("org.graalvm.nativeimage.imagecode").is_ok()
    }

    pub fn header_entries(elements: &[Option<&str>]) -> Vec<Header> {
        let mut headers = Vec::with_capacity(elements.len() / 2);
        for i in (0..elements.len()).step_by(2) {
            if i + 1 < elements.len() {
                let name = elements[i].expect("Header name cannot be null");
                let value = elements[i + 1].expect("Header value cannot be null");
                headers.push(Header::from_strings(name, value));
            }
        }
        headers
    }

    pub fn repeat(c: char, count: usize) -> String {
        std::iter::repeat(c).take(count).collect()
    }

    pub fn fragment_buffer(mut buffer: Buffer) -> Buffer {
        let mut result = Buffer::new();
        while !buffer.exhausted() {
            let mut box_buf = Buffer::new();
            // Simulate writing 1 byte from buffer to box_buf
            let byte = buffer.data[0];
            box_buf.data.push(byte);
            buffer.data.remove(0);
            
            let cloned = box_buf.copy();
            let mut cloned_mut = cloned;
            result.write(&mut cloned_mut, 1);
        }
        result
    }

    pub fn await_garbage_collection() {
        // Rust does not have a direct equivalent to System.gc() as it uses RAII.
        // We simulate the delay and the intent.
        thread::sleep(Duration::from_millis(100));
    }

    pub fn assume_network() {
        match "www.google.com".to_socket_addrs() {
            Ok(_) => {},\n            Err(_) => {
                panic!("requires network");
            }
        }
    }

    pub fn assume_not_windows() {
        if Self::windows() {
            panic!("This test fails on Windows.");
        }
    }

    pub fn windows() -> bool {
        #[cfg(windows)]
        { true }
        #[cfg(not(windows))]
        { false }
    }

    pub fn assert_suppressed<F>(error: &dyn std::error::Error, block: F) 
    where F: Fn(Vec<Box<dyn std::error::Error>>) {
        if Self::is_graal_vm_image() {
            return;
        }
        // Rust's std::error::Error doesn't have a standard 'suppressed' list like Java.
        block(Vec::new());
    }

    pub fn thread_factory(name: String) -> Box<dyn ThreadFactory> {
        Box::new(CustomThreadFactory {
            name,
            next_id: AtomicI32::new(1),
        })
    }
}

// Extension trait for Path to mimic File.isDescendentOf
pub trait FileExt {
    fn is_descendent_of(&self, directory: &Path) -> bool;
}

impl FileExt for Path {
    fn is_descendent_of(&self, directory: &Path) -> bool {
        if let Some(parent) = self.parent() {
            if parent == directory {
                return true;
            }
            return parent.is_descendent_of(directory);
        }
        false
    }
}

pub trait ThreadFactory: Send + Sync {
    fn new_thread(&self, runnable: Box<dyn FnOnce() + Send>);
}

struct CustomThreadFactory {
    name: String,
    next_id: AtomicI32,
}

impl ThreadFactory for CustomThreadFactory {
    fn new_thread(&self, runnable: Box<dyn FnOnce() + Send>) {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let thread_name = format!("{}-{}", self.name, id);
        thread::Builder::new()
            .name(thread_name)
            .spawn(runnable)
            .expect("Failed to spawn thread");
    }
}

pub fn get_env(name: &str) -> Option<String> {
    env::var(name).ok()
}

pub static SYSTEM_FILE_SYSTEM: FileSystem = FileSystem;

pub const IS_JVM: bool = true;
