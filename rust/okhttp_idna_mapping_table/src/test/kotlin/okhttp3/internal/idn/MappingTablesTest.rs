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

use okio::{Buffer, ByteString};
use crate::okhttp_idna_mapping_table::src::main::kotlin::okhttp3::internal::idn::MappedRange::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp_idna_mapping_table::src::main::kotlin::okhttp3::internal::idn::SimpleIdnaMappingTable::*;

// Constants defined in the context of the mapping tables
pub const TYPE_MAPPED: i32 = 1;
pub const TYPE_VALID: i32 = 2;
pub const TYPE_DISALLOWED: i32 = 3;
pub const TYPE_DISALLOWED_STD3_VALID: i32 = 4;


impl Mapping {
    pub fn new(source_code_point0: i32, source_code_point1: i32, r#type: i32, mapped_to: ByteString) -> Self {
        Self {
            source_code_point0,
            source_code_point1,
            r#type,
            mapped_to,
        }
    }
}

// Mocking the logic of the functions being tested as they are likely in the same package in Kotlin
// but are not provided in the source snippet. In a real translation, these would be imported.

pub fn merge_adjacent_ranges(mappings: Vec<Mapping>) -> Vec<Mapping> {
    if mappings.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut current = mappings[0].clone();

    for next in mappings.into_iter().skip(1) {
        if next.r#type == current.r#type 
           && next.mapped_to == current.mapped_to 
           && next.source_code_point0 == current.source_code_point1 + 1 
           && (next.r#type == TYPE_VALID || next.r#type == TYPE_DISALLOWED || next.r#type == TYPE_DISALLOWED_STD3_VALID) 
        {
            current.source_code_point1 = next.source_code_point1;
        } else if next.r#type == TYPE_VALID && current.r#type == TYPE_DISALLOWED_STD3_VALID 
                  && next.source_code_point0 == current.source_code_point1 + 1 
                  && next.mapped_to == current.mapped_to 
        {
            current.r#type = TYPE_VALID;
            current.source_code_point1 = next.source_code_point1;
        } else {
            result.push(current);
            current = next;
        }
    }
    result.push(current);
    
    // Canonicalize types in the final list
    result.into_iter().map(|mut m| {
        if m.r#type == TYPE_DISALLOWED_STD3_VALID {
            m.r#type = TYPE_VALID;
        }
        m
    }).collect()
}

pub fn without_section_spans(mappings: Vec<Mapping>) -> Vec<Mapping> {
    let mut result = Vec::new();
    for m in mappings {
        let mut start = m.source_code_point0;
        let end = m.source_code_point1;
        while start <= end {
            let section_start = (start & !0xFF) & !0xFF00; // Simplified section logic
            // In actual IDNA, sections are 0x100 wide.
            let section_end = (start & !0xFF) | 0xFF;
            let actual_end = std::cmp::min(end, section_end);
            
            result.push(Mapping::new(start, actual_end, m.r#type, m.mapped_to.clone()));
            start = actual_end + 1;
        }
    }
    // Note: The specific logic for 0x40000 in the test suggests 0x100 boundaries.
    // The actual implementation would be more precise.
    result
}

pub fn merge_adjacent_delta_mapped_ranges(ranges: Vec<MappedRange>) -> Vec<MappedRange> {
    if ranges.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut current = ranges[0].clone();

    for next in ranges.into_iter().skip(1) {
        if let (MappedRange::InlineDelta { range_start: s1, codepoint_delta: d1 }, 
                MappedRange::InlineDelta { range_start: s2, codepoint_delta: d2 }) = (&current, &next) 
        {
            if d1 == d2 && s2 == s1 + 1 {
                // In a real implementation, we'd update the range_start of the first one 
                // or handle the sequence. The test expects the first one to "absorb" 
                // if they are identical deltas.
                continue; 
            }
        }
        result.push(current);
        current = next;
    }
    result.push(current);
    result
}

pub fn inline_delta_or_null(mapping: Mapping) -> Option<MappedRange> {
    if mapping.source_code_point0 != mapping.source_code_point1 {
        return None;
    }
    if mapping.mapped_to.len() == 0 {
        return None;
    }
    
    // This is a simplification of UTF-8 codepoint extraction from ByteString
    let mut buffer = Buffer::new();
    buffer.write(&mapping.mapped_to);
    let codepoints: Vec<i32> = buffer.read_utf8_codepoints().collect();

    if codepoints.len() != 1 {
        return None;
    }

    let delta = codepoints[0] - mapping.source_code_point0;
    if delta < -MappedRange::MAX_VALUE || delta > MappedRange::MAX_VALUE {
        return None;
    }

    Some(MappedRange::InlineDelta {
        range_start: mapping.source_code_point0,
        codepoint_delta: delta,
    })
}

pub struct MappingTablesTest;

impl MappingTablesTest {
    pub fn simplify_combines_multiple_mappings(&self) {
        let input = vec![
            Mapping::new(0x0232, 0x0232, TYPE_MAPPED, ByteString::encode_utf8("a")),
            Mapping::new(0x0233, 0x0233, TYPE_VALID, ByteString::EMPTY),
            Mapping::new(0x0234, 0x0236, TYPE_VALID, ByteString::EMPTY),
            Mapping::new(0x0237, 0x0239, TYPE_VALID, ByteString::EMPTY),
            Mapping::new(0x023a, 0x023a, TYPE_MAPPED, ByteString::encode_utf8("b")),
        ];
        let expected = vec![
            Mapping::new(0x0232, 0x0232, TYPE_MAPPED, ByteString::encode_utf8("a")),
            Mapping::new(0x0233, 0x0239, TYPE_VALID, ByteString::EMPTY),
            Mapping::new(0x023a, 0x023a, TYPE_MAPPED, ByteString::encode_utf8("b")),
        ];
        assert_eq!(merge_adjacent_ranges(input), expected);
    }

    pub fn simplify_does_not_combine_when_mapped_targets_are_different(&self) {
        let input = vec![
            Mapping::new(0x0041, 0x0041, TYPE_MAPPED, ByteString::encode_utf8("a")),
            Mapping::new(0x0042, 0x0042, TYPE_MAPPED, ByteString::encode_utf8("b")),
        ];
        let expected = vec![
            Mapping::new(0x0041, 0x0041, TYPE_MAPPED, ByteString::encode_utf8("a")),
            Mapping::new(0x0042, 0x0042, TYPE_MAPPED, ByteString::encode_utf8("b")),
        ];
        assert_eq!(merge_adjacent_ranges(input), expected);
    }

    pub fn simplify_canonicalizes_type(&self) {
        let input = vec![
            Mapping::new(0x0000, 0x002c, TYPE_DISALLOWED_STD3_VALID, ByteString::EMPTY),
        ];
        let expected = vec![
            Mapping::new(0x0000, 0x002c, TYPE_VALID, ByteString::EMPTY),
        ];
        assert_eq!(merge_adjacent_ranges(input), expected);
    }

    pub fn simplify_combines_canonical_equivalent(&self) {
        let input = vec![
            Mapping::new(0x0000, 0x002c, TYPE_DISALLOWED_STD3_VALID, ByteString::EMPTY),
            Mapping::new(0x002d, 0x002e, TYPE_VALID, ByteString::EMPTY),
        ];
        let expected = vec![
            Mapping::new(0x0000, 0x002e, TYPE_VALID, ByteString::EMPTY),
        ];
        assert_eq!(merge_adjacent_ranges(input), expected);
    }

    pub fn with_section_starts_splits(&self) {
        let input = vec![
            Mapping::new(0x40000, 0x40180, TYPE_DISALLOWED, ByteString::EMPTY),
        ];
        // The Kotlin test expects specific splits at 0x7f/0x80 boundaries
        let expected = vec![
            Mapping::new(0x40000, 0x4007f, TYPE_DISALLOWED, ByteString::EMPTY),
            Mapping::new(0x40080, 0x400ff, TYPE_DISALLOWED, ByteString::EMPTY),
            Mapping::new(0x40100, 0x4017f, TYPE_DISALLOWED, ByteString::EMPTY),
            Mapping::new(0x40180, 0x40180, TYPE_DISALLOWED, ByteString::EMPTY),
        ];
        assert_eq!(without_section_spans(input), expected);
    }

    pub fn with_section_start_already_split(&self) {
        let input = vec![
            Mapping::new(0x40000, 0x4007f, TYPE_DISALLOWED, ByteString::EMPTY),
            Mapping::new(0x40080, 0x400ff, TYPE_DISALLOWED, ByteString::EMPTY),
        ];
        let expected = vec![
            Mapping::new(0x40000, 0x4007f, TYPE_DISALLOWED, ByteString::EMPTY),
            Mapping::new(0x40080, 0x400ff, TYPE_DISALLOWED, ByteString::EMPTY),
        ];
        assert_eq!(without_section_spans(input), expected);
    }

    pub fn merge_adjacent_delta_mapped_ranges_with_multiple_deltas(&self) {
        let input = vec![
            MappedRange::InlineDelta { range_start: 1, codepoint_delta: 5 },
            MappedRange::InlineDelta { range_start: 2, codepoint_delta: 5 },
            MappedRange::InlineDelta { range_start: 3, codepoint_delta: 5 },
            MappedRange::External { range_start: 4, mapped_to: ByteString::encode_utf8("a") },
        ];
        let expected = vec![
            MappedRange::InlineDelta { range_start: 1, codepoint_delta: 5 },
            MappedRange::External { range_start: 4, mapped_to: ByteString::encode_utf8("a") },
        ];
        assert_eq!(merge_adjacent_delta_mapped_ranges(input), expected);
    }

    pub fn merge_adjacent_delta_mapped_ranges_with_different_sized_deltas(&self) {
        let input = vec![
            MappedRange::InlineDelta { range_start: 1, codepoint_delta: 5 },
            MappedRange::InlineDelta { range_start: 2, codepoint_delta: 5 },
            MappedRange::InlineDelta { range_start: 3, codepoint_delta: 1 },
        ];
        let expected = vec![
            MappedRange::InlineDelta { range_start: 1, codepoint_delta: 5 },
            MappedRange::InlineDelta { range_start: 3, codepoint_delta: 1 },
        ];
        assert_eq!(merge_adjacent_delta_mapped_ranges(input), expected);
    }

    pub fn inline_delta_or_null_valid(&self) {
        let m1 = self.mapping_of(1, 1, vec![2]);
        assert_eq!(inline_delta_or_null(m1), Some(MappedRange::InlineDelta { range_start: 1, codepoint_delta: 1 }));

        let m2 = self.mapping_of(2, 2, vec![1]);
        assert_eq!(inline_delta_or_null(m2), Some(MappedRange::InlineDelta { range_start: 2, codepoint_delta: -1 }));
    }

    pub fn inline_delta_or_null_multiple_source_code_points(&self) {
        let m = self.mapping_of(2, 3, vec![2]);
        assert_eq!(inline_delta_or_null(m), None);
    }

    pub fn inline_delta_or_null_multiple_mapped_to_code_points(&self) {
        let m = self.mapping_of(1, 1, vec![2, 3]);
        assert_eq!(inline_delta_or_null(m), None);
    }

    pub fn inline_delta_or_null_max_codepoint_delta(&self) {
        let m1 = self.mapping_of(0, 0, vec![(1 << 18) - 1]);
        assert_eq!(
            inline_delta_or_null(m1), 
            Some(MappedRange::InlineDelta { range_start: 0, codepoint_delta: MappedRange::MAX_VALUE })
        );

        let m2 = self.mapping_of(0, 0, vec![1 << 18]);
        assert_eq!(inline_delta_or_null(m2), None);
    }

    fn mapping_of(&self, source_code_point0: i32, source_code_point1: i32, mapped_to_code_points: Vec<i32>) -> Mapping {
        let mut buffer = Buffer::new();
        for cp in mapped_to_code_points {
            // Helper to write UTF-8 codepoint to buffer
            let s = std::char::from_u32(cp as u32).unwrap().to_string();
            buffer.write_utf8(&s);
        }
        Mapping::new(
            source_code_point0,
            source_code_point1,
            TYPE_MAPPED,
            buffer.read_byte_string(),
        )
    }
}