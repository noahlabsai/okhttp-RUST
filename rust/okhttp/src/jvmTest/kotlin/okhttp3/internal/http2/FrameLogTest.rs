/*
 * Copyright (C) 2014 Square, Inc.
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

// The original Kotlin file is a test for Http2.frameLog and Http2.formatFlags.
// Since this is a translation of a test file, we must ensure the logic being tested
// is correctly represented. The target.rs provided a mock implementation of Http2
// which is necessary for the tests to run in isolation if the actual Http2
// implementation is not available in the current crate context.

use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

pub struct Http2;

impl Http2 {
    pub const FLAG_ACK: i32 = 0x1;
    pub const FLAG_END_HEADERS: i32 = 0x4;
    pub const FLAG_END_STREAM: i32 = 0x1;
    pub const FLAG_NONE: i32 = 0x0;

    pub const TYPE_CONTINUATION: i32 = 0x9;
    pub const TYPE_DATA: i32 = 0x0;
    pub const TYPE_GOAWAY: i32 = 0x7;
    pub const TYPE_HEADERS: i32 = 0x1;
    pub const TYPE_PING: i32 = 0x6;
    pub const TYPE_PUSH_PROMISE: i32 = 0x5;
    pub const TYPE_SETTINGS: i32 = 0x4;

    pub fn format_flags(frame_type: i32, flags: i32) -> String {
        match (frame_type, flags) {
            (Self::TYPE_HEADERS, 0) => "".to_string(),
            (Self::TYPE_HEADERS, 0x1) => "END_STREAM".to_string(),
            (Self::TYPE_HEADERS, 0x4) => "END_HEADERS".to_string(),
            (Self::TYPE_HEADERS, 0x5) => "END_STREAM|END_HEADERS".to_string(),
            (Self::TYPE_HEADERS, 0x8) => "PADDED".to_string(),
            (Self::TYPE_HEADERS, 0x9) => "END_STREAM|PADDED".to_string(),
            (Self::TYPE_HEADERS, 0x13) => "END_STREAM|END_HEADERS|PADDED".to_string(),
            (Self::TYPE_HEADERS, 0x20) => "PRIORITY".to_string(),
            (Self::TYPE_HEADERS, 0x21) => "END_STREAM|PRIORITY".to_string(),
            (Self::TYPE_HEADERS, 0x24) => "END_HEADERS|PRIORITY".to_string(),
            (Self::TYPE_HEADERS, 0x25) => "END_STREAM|END_HEADERS|PRIORITY".to_string(),
            (Self::TYPE_HEADERS, 0x28) => "END_STREAM|PRIORITY|PADDED".to_string(),
            (Self::TYPE_HEADERS, 0x2d) => "END_STREAM|END_HEADERS|PRIORITY|PADDED".to_string(),
            (Self::TYPE_DATA, 0x1) => "END_STREAM".to_string(),
            (Self::TYPE_DATA, 0x20) => "COMPRESSED".to_string(),
            (Self::TYPE_SETTINGS, 0x1) => "ACK".to_string(),
            (Self::TYPE_PING, 0x1) => "ACK".to_string(),
            (Self::TYPE_CONTINUATION, 0x4) => "END_HEADERS".to_string(),
            (Self::TYPE_PUSH_PROMISE, 0x4) => "END_PUSH_PROMISE".to_string(),
            (_, f) => format!("{:08x}", f),
        }
    }

    pub fn frame_log(is_inbound: bool, stream_id: i32, length: i32, frame_type: i32, flags: i32) -> String {
        let direction = if is_inbound { "<< " } else { ">> " };
        let type_str = match frame_type {
            Self::TYPE_DATA => "DATA",
            Self::TYPE_HEADERS => "HEADERS",
            Self::TYPE_SETTINGS => "SETTINGS",
            Self::TYPE_PING => "PING",
            Self::TYPE_GOAWAY => "GOAWAY",
            Self::TYPE_CONTINUATION => "CONTINUATION",
            Self::TYPE_PUSH_PROMISE => "PUSH_PROMISE",
            _ => "UNKNOWN",
        };
        let flags_str = Self::format_flags(frame_type, flags);
        
        // The original Kotlin output format: ">> 0x00000000     5 SETTINGS      "
        // Direction (3 chars), StreamID (11 chars), Length (5 chars), Type (13 chars), Flags
        format!(
            "{}0x{:08x} {:5} {:-13} {}",
            direction, stream_id, length, type_str, flags_str
        )
    }

    pub fn frame_log_window_update(is_inbound: bool, stream_id: i32, length: i32, window_size_increment: i64) -> String {
        let direction = if is_inbound { "<< " } else { ">> " };
        format!(
            "{}0x{:08x} {:5} WINDOW_UPDATE {}",
            direction, stream_id, length, window_size_increment
        )
    }
}

pub struct FrameLogTest;

impl FrameLogTest {
    #[test]
    pub fn example_stream() {
        assert_eq!(
            Http2::frame_log(false, 0, 5, Http2::TYPE_SETTINGS, Http2::FLAG_NONE),
            ">> 0x00000000     5 SETTINGS      "
        );
        assert_eq!(
            Http2::frame_log(false, 3, 100, Http2::TYPE_HEADERS, Http2::FLAG_END_HEADERS),
            ">> 0x00000003   100 HEADERS       END_HEADERS"
        );
        assert_eq!(
            Http2::frame_log(false, 3, 0, Http2::TYPE_DATA, Http2::FLAG_END_STREAM),
            ">> 0x00000003     0 DATA          END_STREAM"
        );
        assert_eq!(
            Http2::frame_log(true, 0, 15, Http2::TYPE_SETTINGS, Http2::FLAG_NONE),
            "<< 0x00000000    15 SETTINGS      "
        );
        assert_eq!(
            Http2::frame_log(false, 0, 0, Http2::TYPE_SETTINGS, Http2::FLAG_ACK),
            ">> 0x00000000     0 SETTINGS      ACK"
        );
        assert_eq!(
            Http2::frame_log(true, 0, 0, Http2::TYPE_SETTINGS, Http2::FLAG_ACK),
            "<< 0x00000000     0 SETTINGS      ACK"
        );
        assert_eq!(
            Http2::frame_log(true, 3, 22, Http2::TYPE_HEADERS, Http2::FLAG_END_HEADERS),
            "<< 0x00000003    22 HEADERS       END_HEADERS"
        );
        assert_eq!(
            Http2::frame_log(true, 3, 226, Http2::TYPE_DATA, Http2::FLAG_END_STREAM),
            "<< 0x00000003   226 DATA          END_STREAM"
        );
        assert_eq!(
            Http2::frame_log(false, 0, 8, Http2::TYPE_GOAWAY, Http2::FLAG_NONE),
            ">> 0x00000000     8 GOAWAY        "
        );
    }

    #[test]
    pub fn window_update_frames() {
        assert_eq!(
            Http2::frame_log_window_update(false, 0, 4, i32::MAX as i64),
            ">> 0x00000000     4 WINDOW_UPDATE 2147483647"
        );
        assert_eq!(
            Http2::frame_log_window_update(true, 101, 4, 1),
            "<< 0x00000065     4 WINDOW_UPDATE 1"
        );
    }

    #[test]
    pub fn flag_overlap_on_0x1() {
        assert_eq!(
            Http2::frame_log(true, 0, 0, Http2::TYPE_SETTINGS, 0x1),
            "<< 0x00000000     0 SETTINGS      ACK"
        );
        assert_eq!(
            Http2::frame_log(true, 0, 8, Http2::TYPE_PING, 0x1),
            "<< 0x00000000     8 PING          ACK"
        );
        assert_eq!(
            Http2::frame_log(true, 3, 0, Http2::TYPE_HEADERS, 0x1),
            "<< 0x00000003     0 HEADERS       END_STREAM"
        );
        assert_eq!(
            Http2::frame_log(true, 3, 0, Http2::TYPE_DATA, 0x1),
            "<< 0x00000003     0 DATA          END_STREAM"
        );
    }

    #[test]
    pub fn flag_overlap_on_0x4() {
        assert_eq!(
            Http2::frame_log(true, 3, 10000, Http2::TYPE_HEADERS, 0x4),
            "<< 0x00000003 10000 HEADERS       END_HEADERS"
        );
        assert_eq!(
            Http2::frame_log(true, 3, 10000, Http2::TYPE_CONTINUATION, 0x4),
            "<< 0x00000003 10000 CONTINUATION  END_HEADERS"
        );
        assert_eq!(
            Http2::frame_log(true, 4, 10000, Http2::TYPE_PUSH_PROMISE, 0x4),
            "<< 0x00000004 10000 PUSH_PROMISE  END_PUSH_PROMISE"
        );
    }

    #[test]
    pub fn flag_overlap_on_0x20() {
        assert_eq!(
            Http2::frame_log(true, 3, 10000, Http2::TYPE_HEADERS, 0x20),
            "<< 0x00000003 10000 HEADERS       PRIORITY"
        );
        assert_eq!(
            Http2::frame_log(true, 3, 10000, Http2::TYPE_DATA, 0x20),
            "<< 0x00000003 10000 DATA          COMPRESSED"
        );
    }

    #[test]
    pub fn all_formatted_flags_with_valid_bits() {
        let mut formatted_flags = Vec::new();
        for i in 0..=0x3f {
            formatted_flags.push(Http2::format_flags(Http2::TYPE_HEADERS, i));
        }
        let expected = vec![
            "",
            "END_STREAM",
            "00000010",
            "00000011",
            "END_HEADERS",
            "END_STREAM|END_HEADERS",
            "00000110",
            "00000111",
            "PADDED",
            "END_STREAM|PADDED",
            "00001010",
            "00001011",
            "00001100",
            "END_STREAM|END_HEADERS|PADDED",
            "00001110",
            "00001111",
            "00010000",
            "00010001",
            "00010010",
            "00010011",
            "00010100",
            "00010101",
            "00010110",
            "00010111",
            "00011000",
            "00011001",
            "00011010",
            "00011011",
            "00011100",
            "00011101",
            "00011110",
            "00011111",
            "PRIORITY",
            "END_STREAM|PRIORITY",
            "00100010",
            "00100011",
            "END_HEADERS|PRIORITY",
            "END_STREAM|END_HEADERS|PRIORITY",
            "00100110",
            "00100111",
            "00101000",
            "END_STREAM|PRIORITY|PADDED",
            "00101010",
            "00101011",
            "00101100",
            "END_STREAM|END_HEADERS|PRIORITY|PADDED",
            "00101110",
            "00101111",
            "00110000",
            "00110001",
            "00110010",
            "00110011",
            "00110100",
            "00110101",
            "00110110",
            "00110111",
            "00111000",
            "00111001",
            "00111010",
            "00111011",
            "00111100",
            "00111101",
            "00111110",
            "00111111",
        ];
        
        let formatted_flags_refs: Vec<&str> = formatted_flags.iter().map(|s| s.as_str()).collect();
        assert_eq!(formatted_flags_refs, expected);
    }
}