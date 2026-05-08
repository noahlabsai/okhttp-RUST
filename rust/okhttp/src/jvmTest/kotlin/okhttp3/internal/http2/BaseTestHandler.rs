/*
 * Copyright (C) 2013 Square, Inc.
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

use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::settings_gradle::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::ErrorCode::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Reader::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::ResponseBodyTest::*;

use crate::okhttp3::internal::http2::{
    BufferedSource, ByteString, ErrorCode, Header, Http2Reader, Settings,
};

// BaseTestHandler is a base class for testing Http2Reader.Handler.
// In Rust, we implement the trait and provide a default implementation that panics,
// mimicking the behavior of `fail("")` in the Kotlin source.
pub struct BaseTestHandler;

impl BaseTestHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Http2Reader::Handler for BaseTestHandler {
    fn data(
        &mut self,
        _in_finished: bool,
        _stream_id: i32,
        _source: BufferedSource,
        _length: i32,
    ) {
        panic!("fail(\"\")");
    }

    fn headers(
        &mut self,
        _in_finished: bool,
        _stream_id: i32,
        _associated_stream_id: i32,
        _header_block: Vec<Header>,
    ) {
        panic!("fail(\"\")");
    }

    fn rst_stream(
        &mut self,
        _stream_id: i32,
        _error_code: ErrorCode,
    ) {
        panic!("fail(\"\")");
    }

    fn settings(
        &mut self,
        _clear_previous: bool,
        _settings: Settings,
    ) {
        panic!("fail(\"\")");
    }

    fn ack_settings(&mut self) {
        panic!("fail(\"\")");
    }

    fn ping(
        &mut self,
        _ack: bool,
        _payload1: i32,
        _payload2: i32,
    ) {
        panic!("fail(\"\")");
    }

    fn go_away(
        &mut self,
        _last_good_stream_id: i32,
        _error_code: ErrorCode,
        _debug_data: ByteString,
    ) {
        panic!("fail(\"\")");
    }

    fn window_update(
        &mut self,
        _stream_id: i32,
        _window_size_increment: i64,
    ) {
        panic!("fail(\"\")");
    }

    fn priority(
        &mut self,
        _stream_id: i32,
        _stream_dependency: i32,
        _weight: i32,
        _exclusive: bool,
    ) {
        panic!("fail(\"\")");
    }

    fn push_promise(
        &mut self,
        _stream_id: i32,
        _associated_stream_id: i32,
        _header_block: Vec<Header>,
    ) {
        panic!("fail(\"\")");
    }

    fn alternate_service(
        &mut self,
        _stream_id: i32,
        _origin: String,
        _protocol: ByteString,
        _host: String,
        _port: i32,
        _max_age: i64,
    ) {
        panic!("fail(\"\")");
    }
}