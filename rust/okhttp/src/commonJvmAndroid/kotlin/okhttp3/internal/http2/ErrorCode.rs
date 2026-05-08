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

// http://tools.ietf.org/html/draft-ietf-httpbis-http2-17#section-7
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // Not an error!
    NoError,
    ProtocolError,
    InternalError,
    FlowControlError,
    SettingsTimeout,
    StreamClosed,
    FrameSizeError,
    RefusedStream,
    Cancel,
    CompressionError,
    ConnectError,
    EnhanceYourCalm,
    InadequateSecurity,
    Http11Required,
}

impl Default for ErrorCode {
    fn default() -> Self {
        ErrorCode::NoError
    }
}

pub const NoError: ErrorCode = ErrorCode::NoError;
pub const ProtocolError: ErrorCode = ErrorCode::ProtocolError;
pub const InternalError: ErrorCode = ErrorCode::InternalError;
pub const FlowControlError: ErrorCode = ErrorCode::FlowControlError;
pub const SettingsTimeout: ErrorCode = ErrorCode::SettingsTimeout;
pub const StreamClosed: ErrorCode = ErrorCode::StreamClosed;
pub const FrameSizeError: ErrorCode = ErrorCode::FrameSizeError;
pub const RefusedStream: ErrorCode = ErrorCode::RefusedStream;
pub const Cancel: ErrorCode = ErrorCode::Cancel;
pub const CompressionError: ErrorCode = ErrorCode::CompressionError;
pub const ConnectError: ErrorCode = ErrorCode::ConnectError;
pub const EnhanceYourCalm: ErrorCode = ErrorCode::EnhanceYourCalm;
pub const InadequateSecurity: ErrorCode = ErrorCode::InadequateSecurity;
pub const Http11Required: ErrorCode = ErrorCode::Http11Required;

impl ErrorCode {
    // Returns the HTTP/2 error code associated with the variant.
    pub fn http_code(&self) -> i32 {
        match self {
            ErrorCode::NoError => 0,
            ErrorCode::ProtocolError => 1,
            ErrorCode::InternalError => 2,
            ErrorCode::FlowControlError => 3,
            ErrorCode::SettingsTimeout => 4,
            ErrorCode::StreamClosed => 5,
            ErrorCode::FrameSizeError => 6,
            ErrorCode::RefusedStream => 7,
            ErrorCode::Cancel => 8,
            ErrorCode::CompressionError => 9,
            ErrorCode::ConnectError => 0xa,
            ErrorCode::EnhanceYourCalm => 0xb,
            ErrorCode::InadequateSecurity => 0xc,
            ErrorCode::Http11Required => 0xd,
        }
    }

    // Returns the ErrorCode corresponding to the given HTTP/2 code, or None if not found.
    pub fn from_http2(code: i32) -> Option<ErrorCode> {
        match code {
            0 => Some(ErrorCode::NoError),
            1 => Some(ErrorCode::ProtocolError),
            2 => Some(ErrorCode::InternalError),
            3 => Some(ErrorCode::FlowControlError),
            4 => Some(ErrorCode::SettingsTimeout),
            5 => Some(ErrorCode::StreamClosed),
            6 => Some(ErrorCode::FrameSizeError),
            7 => Some(ErrorCode::RefusedStream),
            8 => Some(ErrorCode::Cancel),
            9 => Some(ErrorCode::CompressionError),
            0xa => Some(ErrorCode::ConnectError),
            0xb => Some(ErrorCode::EnhanceYourCalm),
            0xc => Some(ErrorCode::InadequateSecurity),
            0xd => Some(ErrorCode::Http11Required),
            _ => None,
        }
    }

    // Returns all possible ErrorCode variants.
    pub fn values() -> Vec<ErrorCode> {
        vec![
            ErrorCode::NoError,
            ErrorCode::ProtocolError,
            ErrorCode::InternalError,
            ErrorCode::FlowControlError,
            ErrorCode::SettingsTimeout,
            ErrorCode::StreamClosed,
            ErrorCode::FrameSizeError,
            ErrorCode::RefusedStream,
            ErrorCode::Cancel,
            ErrorCode::CompressionError,
            ErrorCode::ConnectError,
            ErrorCode::EnhanceYourCalm,
            ErrorCode::InadequateSecurity,
            ErrorCode::Http11Required,
        ]
    }
}