/*
 * Copyright (C) 2023 Square, Inc.
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

use okio::ByteString;

// Constants for MappedRange::Constant type
const TYPE_IGNORED: i32 = 0; // Assuming these are defined elsewhere or based on typical IDNA tables
const TYPE_VALID: i32 = 1;
const TYPE_DISALLOWED: i32 = 2;

#[derive(Debug, Clone, PartialEq)]
pub enum MappedRange {
    Constant {
        range_start: i32,
        range_type: i32,
    },
    Inline1 {
        range_start: i32,
        mapped_to: ByteString,
    },
    Inline2 {
        range_start: i32,
        mapped_to: ByteString,
    },
    InlineDelta {
        range_start: i32,
        codepoint_delta: i32,
    },
    External {
        range_start: i32,
        mapped_to: ByteString,
    },
}

impl MappedRange {
    pub fn range_start(&self) -> i32 {
        match self {
            MappedRange::Constant { range_start, .. } => *range_start,
            MappedRange::Inline1 { range_start, .. } => *range_start,
            MappedRange::Inline2 { range_start, .. } => *range_start,
            MappedRange::InlineDelta { range_start, .. } => *range_start,
            MappedRange::External { range_start, .. } => *range_start,
        }
    }

    pub fn b1(&self) -> i32 {
        match self {
            MappedRange::Constant { range_type, .. } => match *range_type {
                TYPE_IGNORED => 119,
                TYPE_VALID => 120,
                TYPE_DISALLOWED => 121,
                _ => panic!("unexpected type: {}", range_type),
            },
            MappedRange::Inline1 { mapped_to, .. } => {
                let b3bit8 = (mapped_to[0] & 0x80) != 0;
                if b3bit8 { 123 } else { 122 }
            }
            MappedRange::Inline2 { mapped_to, .. } => {
                let b2bit8 = (mapped_to[0] & 0x80) != 0;
                let b3bit8 = (mapped_to[1] & 0x80) != 0;
                if b2bit8 && b3bit8 {
                    127
                } else if b3bit8 {
                    126
                } else if b2bit8 {
                    125
                } else {
                    124
                }
            }
            MappedRange::InlineDelta { codepoint_delta, .. } => {
                let absolute_delta = (*codepoint_delta).abs();
                if *codepoint_delta < 0 {
                    (0x40 | (absolute_delta >> 14)) as i32
                } else if *codepoint_delta > 0 {
                    (0x50 | (absolute_delta >> 14)) as i32
                } else {
                    panic!("Unexpected codepointDelta of 0")
                }
            }
            MappedRange::External { .. } => panic!("b1 not defined for External"),
        }
    }

    pub fn b2(&self) -> i32 {
        match self {
            MappedRange::Inline1 { mapped_to, .. } => (mapped_to[0] & 0x7f) as i32,
            MappedRange::Inline2 { mapped_to, .. } => (mapped_to[0] & 0x7f) as i32,
            MappedRange::InlineDelta { codepoint_delta, .. } => {
                let absolute_delta = (*codepoint_delta).abs();
                ((absolute_delta >> 7) & 0x7f) as i32
            }
            _ => panic!("b2 not defined for this MappedRange variant"),
        }
    }

    pub fn b3(&self) -> i32 {
        match self {
            MappedRange::Inline2 { mapped_to, .. } => (mapped_to[1] & 0x7f) as i32,
            MappedRange::InlineDelta { codepoint_delta, .. } => {
                let absolute_delta = (*codepoint_delta).abs();
                (absolute_delta & 0x7f) as i32
            }
            _ => panic!("b3 not defined for this MappedRange variant"),
        }
    }
}

impl MappedRange {
    pub const MAX_VALUE: i32 = 0x3FFFF;
}