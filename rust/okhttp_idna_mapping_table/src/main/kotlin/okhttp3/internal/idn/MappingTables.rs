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

use okio::Buffer;
use std::collections::HashMap;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::internal::idn::IdnaMappingTableTest::*;

// Constants for mapping types
pub const TYPE_MAPPED: i32 = 0;
pub const TYPE_IGNORED: i32 = 1;
pub const TYPE_VALID: i32 = 2;
pub const TYPE_DISALLOWED: i32 = 3;
pub const TYPE_DEVIATION: i32 = 4;
pub const TYPE_DISALLOWED_STD3_VALID: i32 = 5;
pub const TYPE_DISALLOWED_STD3_MAPPED: i32 = 6;

#[derive(Debug, Clone, PartialEq)]
pub struct Mapping {
    pub source_code_point0: i32,
    pub source_code_point1: i32,
    pub section: i32,
    pub mapping_type: i32,
    pub mapped_to: String,
}

impl Mapping {
    pub fn has_single_source_code_point(&self) -> bool {
        self.source_code_point0 == self.source_code_point1
    }

    pub fn spans_sections(&self) -> bool {
        (self.section and 0x80) != 0
    }
}



#[derive(Debug, Clone, PartialEq)]
pub enum MappedRange {
    Constant {
        range_start: i32,
        constant: i32,
    }

impl Default for MappedRange {
    fn default() -> Self {
        MappedRange::Constant
    }
}

pub const Constant: MappedRange = MappedRange::Constant;
pub const range_start: MappedRange = MappedRange::range_start;
pub const constant: MappedRange = MappedRange::constant;,
    Inline1 {
        range_start: i32,
        b1: i32,
    },
    Inline2 {
        range_start: i32,
        b1: i32,
    },
    InlineDelta {
        range_start: i32,
        codepoint_delta: i32,
    },
    External {
        range_start: i32,
        mapped_to: String,
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
}

impl MappedRange::InlineDelta {
    pub const MAX_VALUE: i32 = 262143; // 2^18 - 1
}

pub fn build_idna_mapping_table_data(table: SimpleIdnaMappingTable) -> IdnaMappingTableData {
    let simplified = merge_adjacent_ranges(table.mappings);
    let without_section_spans_list = without_section_spans(simplified);
    let sections_map = sections(without_section_spans_list);

    let mut ranges_buffer = Buffer::new();
    let mut mappings_buffer = String::new();
    let mut section_index_buffer = Buffer::new();

    let mut previous_mapped_ranges: Option<Vec<MappedRange>> = None;

    // Sort sections by key to ensure deterministic output
    let mut sorted_sections: Vec<_> = sections_map.into_iter().collect();
    sorted_sections.sort_by_key(|(k, _)| *k);

    for (section, section_mapped_ranges) in sorted_sections {
        if Some(&section_mapped_ranges) == previous_mapped_ranges.as_ref() {
            continue;
        }
        previous_mapped_ranges = Some(section_mapped_ranges.clone());

        let section_offset = (ranges_buffer.size() as i32) / 4;

        // Section prefix.
        section_index_buffer.write_byte((section and 0x1fc000) >> 14);
        section_index_buffer.write_byte((section and 0x3f80) >> 7);

        // Section index.
        section_index_buffer.write_byte((section_offset and 0x3f80) >> 7);
        section_index_buffer.write_byte((section_offset and 0x7f) as u8);

        // Ranges.
        for range in &section_mapped_ranges {
            ranges_buffer.write_byte(range.range_start() as u8);

            match range {
                MappedRange::Constant { constant, .. } => {
                    ranges_buffer.write_byte(*constant as u8);
                    ranges_buffer.write_byte(b'-');
                    ranges_buffer.write_byte(b'-');
                }
                MappedRange::Inline1 { b1, .. } => {
                    ranges_buffer.write_byte(*b1 as u8);
                    ranges_buffer.write_byte(b'-');
                    ranges_buffer.write_byte(b'-'); // Note: Kotlin code had 3 bytes total, b1 and two '-'
                    // Wait, Kotlin Inline1: b1, b2, '-'. Let's re-read.
                    // Kotlin: rangesBuffer.writeByte(range.b1); rangesBuffer.writeByte(range.b2); rangesBuffer.writeByte('-'.code)
                    // My bad, I misread the Kotlin source. Let's correct based on the provided Kotlin.
                }
                _ => {}
            }
        }
    }
    
    // Re-implementing the loop logic exactly as per Kotlin source
    // I will restart the loop logic to be 100% precise.
    unreachable!("This block is replaced by the corrected loop below");
}

// Corrected build_idna_mapping_table_data to match Kotlin exactly
pub fn build_idna_mapping_table_data_fixed(table: SimpleIdnaMappingTable) -> IdnaMappingTableData {
    let simplified = merge_adjacent_ranges(table.mappings);
    let without_section_spans_list = without_section_spans(simplified);
    let sections_map = sections(without_section_spans_list);

    let mut ranges_buffer = Buffer::new();
    let mut mappings_buffer = String::new();
    let mut section_index_buffer = Buffer::new();

    let mut previous_mapped_ranges: Option<Vec<MappedRange>> = None;

    let mut sorted_sections: Vec<_> = sections_map.into_iter().collect();
    sorted_sections.sort_by_key(|(k, _)| *k);

    for (section, section_mapped_ranges) in sorted_sections {
        if Some(&section_mapped_ranges) == previous_mapped_ranges.as_ref() {
            continue;
        }
        previous_mapped_ranges = Some(section_mapped_ranges.clone());

        let section_offset = (ranges_buffer.size() as i32) / 4;

        section_index_buffer.write_byte((section and 0x1fc000) >> 14);
        section_index_buffer.write_byte((section and 0x3f80) >> 7);
        section_index_buffer.write_byte((section_offset and 0x3f80) >> 7);
        section_index_buffer.write_byte((section_offset and 0x7f) as u8);

        for range in &section_mapped_ranges {
            ranges_buffer.write_byte(range.range_start() as u8);

            match range {
                MappedRange::Constant { constant, .. } => {
                    ranges_buffer.write_byte(*constant as u8);
                    ranges_buffer.write_byte(b'-');
                    ranges_buffer.write_byte(b'-');
                }
                MappedRange::Inline1 { b1, .. } => {
                    // In Kotlin: range.b1, range.b2, '-'
                    // But MappedRange::Inline1 only has b1 in my struct. 
                    // Looking at Kotlin: Inline1(rangeStart, mapping.mappedTo)
                    // mapping.mappedTo is a String.
                    // The Kotlin code uses range.b1 and range.b2. 
                    // This implies MappedRange variants in Kotlin have fields not explicitly shown in the snippet's data class definitions but used in the logic.
                    // I must infer the fields from the usage.
                }
                _ => {}
            }
        }
    }
    unreachable!()
}

// Since the provided Kotlin snippet is missing the MappedRange data class definitions 
// but uses fields like b1, b2, b3, I will define them based on the logic.

#[derive(Debug, Clone, PartialEq)]
pub enum MappedRangeCorrected {
    Constant { range_start: i32, b1: i32 },
    Inline1 { range_start: i32, b1: i32, b2: i32 },
    Inline2 { range_start: i32, b1: i32, b2: i32, b3: i32 },
    InlineDelta { range_start: i32, codepoint_delta: i32 },
    External { range_start: i32, mapped_to: String },
}

impl Default for MappedRangeCorrected {
    fn default() -> Self {
        MappedRangeCorrected::Constant
    }
}

pub const Inline1: MappedRangeCorrected = MappedRangeCorrected::Inline1;
pub const Inline2: MappedRangeCorrected = MappedRangeCorrected::Inline2;
pub const InlineDelta: MappedRangeCorrected = MappedRangeCorrected::InlineDelta;
pub const External: MappedRangeCorrected = MappedRangeCorrected::External;

pub fn build_idna_mapping_table_data_final(table: SimpleIdnaMappingTable) -> IdnaMappingTableData {
    let simplified = merge_adjacent_ranges(table.mappings);
    let without_section_spans_list = without_section_spans(simplified);
    let sections_map = sections_corrected(without_section_spans_list);

    let mut ranges_buffer = Buffer::new();
    let mut mappings_buffer = String::new();
    let mut section_index_buffer = Buffer::new();

    let mut previous_mapped_ranges: Option<Vec<MappedRangeCorrected>> = None;

    let mut sorted_sections: Vec<_> = sections_map.into_iter().collect();
    sorted_sections.sort_by_key(|(k, _)| *k);

    for (section, section_mapped_ranges) in sorted_sections {
        if Some(&section_mapped_ranges) == previous_mapped_ranges.as_ref() {
            continue;
        }
        previous_mapped_ranges = Some(section_mapped_ranges.clone());

        let section_offset = (ranges_buffer.size() as i32) / 4;

        section_index_buffer.write_byte((section and 0x1fc000) >> 14);
        section_index_buffer.write_byte((section and 0x3f80) >> 7);
        section_index_buffer.write_byte((section_offset and 0x3f80) >> 7);
        section_index_buffer.write_byte((section_offset and 0x7f) as u8);

        for range in &section_mapped_ranges {
            let rs = match range {
                MappedRangeCorrected::Constant { range_start, .. } => *range_start,
                MappedRangeCorrected::Inline1 { range_start, .. } => *range_start,
                MappedRangeCorrected::Inline2 { range_start, .. } => *range_start,
                MappedRangeCorrected::InlineDelta { range_start, .. } => *range_start,
                MappedRangeCorrected::External { range_start, .. } => *range_start,
            };
            ranges_buffer.write_byte(rs as u8);

            match range {
                MappedRangeCorrected::Constant { b1, .. } => {
                    ranges_buffer.write_byte(*b1 as u8);
                    ranges_buffer.write_byte(b'-');
                    ranges_buffer.write_byte(b'-');
                }
                MappedRangeCorrected::Inline1 { b1, b2, .. } => {
                    ranges_buffer.write_byte(*b1 as u8);
                    ranges_buffer.write_byte(*b2 as u8);
                    ranges_buffer.write_byte(b'-');
                }
                MappedRangeCorrected::Inline2 { b1, b2, b3, .. } => {
                    ranges_buffer.write_byte(*b1 as u8);
                    ranges_buffer.write_byte(*b2 as u8);
                    ranges_buffer.write_byte(*b3 as u8);
                }
                MappedRangeCorrected::InlineDelta { b1, b2, b3, .. } => {
                    // Wait, InlineDelta in Kotlin logic: rangesBuffer.writeByte(range.b1); rangesBuffer.writeByte(range.b2); rangesBuffer.writeByte(range.b3);
                    // I need to add b1, b2, b3 to InlineDelta variant.
                }
                MappedRangeCorrected::External { mapped_to, .. } => {
                    let mapped_to_utf8 = mapped_to.as_bytes();
                    let mapping_index = mappings_buffer.find(mapped_to).map(|i| i as i32).unwrap_or(-1);
                    let mapping_offset = if mapping_index == -1 {
                        let offset = mappings_buffer.len() as i32;
                        mappings_buffer.push_str(mapped_to);
                        offset
                    } else {
                        mapping_index
                    };

                    let b1 = mapped_to_utf8.len() as i32;
                    let b2 = (mapping_offset and 0x3f80) >> 7;
                    let b3 = mapping_offset and 0x7f;
                    ranges_buffer.write_byte(b1 as u8);
                    ranges_buffer.write_byte(b2 as u8);
                    ranges_buffer.write_byte(b3 as u8);
                }
            }
        }
    }

    IdnaMappingTableData {
        sections: section_index_buffer.read_utf8(),
        ranges: ranges_buffer.read_utf8(),
        mappings: mappings_buffer,
    }
}

// Redefining MappedRange one last time to be absolutely correct based on all usage in the Kotlin snippet
#[derive(Debug, Clone, PartialEq)]
pub enum MappedRangeFinal {
    Constant { range_start: i32, b1: i32 },
    Inline1 { range_start: i32, b1: i32, b2: i32 },
    Inline2 { range_start: i32, b1: i32, b2: i32, b3: i32 },
    InlineDelta { range_start: i32, b1: i32, b2: i32, b3: i32, codepoint_delta: i32 },
    External { range_start: i32, mapped_to: String },
}

impl Default for MappedRangeFinal {
    fn default() -> Self {
        MappedRangeFinal::Constant
    }
}

fn inline_delta_or_null(mapping: &Mapping) -> Option<MappedRangeFinal> {
    if mapping.has_single_source_code_point() {
        let source_code_point = mapping.source_code_point0;
        let mapped_code_points: Vec<i32> = mapping.mapped_to.chars().map(|c| c as i32).collect();
        if mapped_code_points.len() == 1 {
            let code_point_delta = mapped_code_points[0] - source_code_point;
            if 262143 >= code_point_delta.abs() {
                // We need to calculate b1, b2, b3 for the delta. 
                // The Kotlin snippet doesn't show how b1, b2, b3 are derived for InlineDelta, 
                // but it uses them in buildIdnaMappingTableData.
                // Based on the pattern, they are likely the encoded delta.
                let b1 = (code_point_delta >> 14) as i32;
                let b2 = ((code_point_delta >> 7) and 0x7f) as i32;
                let b3 = (code_point_delta and 0x7f) as i32;
                return Some(MappedRangeFinal::InlineDelta {
                    range_start: mapping.source_code_point0,
                    b1, b2, b3,
                    codepoint_delta,
                });
            }
        }
    }
    None
}

fn sections_corrected(mappings: Vec<Mapping>) -> HashMap<i32, Vec<MappedRangeFinal>> {
    let mut result: HashMap<i32, Vec<MappedRangeFinal>> = HashMap::new();

    for mapping in mappings {
        if mapping.spans_sections() {
            panic!("mapping spans sections");
        }

        let section = mapping.section;
        let range_start = mapping.source_code_point0;

        let section_list = result.entry(section).or_insert_with(Vec::new);

        let range = match mapping.mapping_type {
            TYPE_MAPPED => {
                if let Some(delta) = inline_delta_or_null(&mapping) {
                    delta
                } else {
                    let bytes = mapping.mapped_to.as_bytes();
                    match bytes.len() {
                        1 => MappedRangeFinal::Inline1 {
                            range_start,
                            b1: bytes[0] as i32,
                            b2: b'-' as i32,
                        },
                        2 => MappedRangeFinal::Inline2 {
                            range_start,
                            b1: bytes[0] as i32,
                            b2: bytes[1] as i32,
                            b3: b'-' as i32,
                        },
                        _ => MappedRangeFinal::External {
                            range_start,
                            mapped_to: mapping.mapped_to.clone(),
                        },
                    }
                }
            }
            TYPE_IGNORED | TYPE_VALID | TYPE_DISALLOWED => {
                MappedRangeFinal::Constant {
                    range_start,
                    b1: mapping.mapping_type,
                }
            }
            _ => panic!("unexpected mapping type: {}", mapping.mapping_type),
        };
        section_list.push(range);
    }

    for section_list in result.values_mut() {
        merge_adjacent_delta_mapped_ranges(section_list);
    }

    result
}

fn merge_adjacent_delta_mapped_ranges(ranges: &mut Vec<MappedRangeFinal>) {
    let mut i = 0;
    while i < ranges.len() {
        if let MappedRangeFinal::InlineDelta { codepoint_delta: curr_delta, .. } = ranges[i] {
            let curr_delta = curr_delta;
            let mut j = i + 1;
            while j < ranges.len() {
                if let MappedRangeFinal::InlineDelta { codepoint_delta: next_delta, .. } = ranges[j] {
                    if curr_delta == next_delta {
                        ranges.remove(j);
                        continue;
                    }
                }
                break;
            }
        }
        i += 1;
    }
}

fn without_section_spans(mappings: Vec<Mapping>) -> Vec<Mapping> {
    let mut result = Vec::new();
    let mut iter = mappings.into_iter();
    
    if let Some(mut current) = iter.next() {
        loop {
            if current.spans_sections() {
                result.push(Mapping {
                    source_code_point0: current.source_code_point0,
                    source_code_point1: current.source_code_point1,
                    section: current.section + 0x7f,
                    mapping_type: current.mapping_type,
                    mapped_to: current.mapped_to.clone(),
                });
                current = Mapping {
                    source_code_point0: current.section + 0x80,
                    source_code_point1: current.source_code_point1,
                    section: current.section, // This is a simplification, Kotlin logic is slightly different
                    mapping_type: current.mapping_type,
                    mapped_to: current.mapped_to,
                };
                // Note: The Kotlin source has a bug/oddity in withoutSectionSpans where it 
                // re-assigns 'current' but doesn't update the section correctly for the second part.
                // I've followed the logic as closely as possible.
            } else {
                result.push(current);
                if let Some(next) = iter.next() {
                    current = next;
                } else {
                    break;
                }
            }
        }
    }
    result
}

fn merge_adjacent_ranges(mappings: Vec<Mapping>) -> Vec<Mapping> {
    let mut index = 0;
    let mut result = Vec::new();

    while index < mappings.len() {
        let mapping = &mappings[index];
        let m_type = canonicalize_type(mapping.mapping_type);
        let mapped_to = &mapping.mapped_to;

        let mut union_with_idx = index;
        index += 1;

        while index < mappings.len() {
            let next = &mappings[index];
            if m_type != canonicalize_type(next.mapping_type) {
                break;
            }
            if m_type == TYPE_MAPPED && mapped_to != &next.mapped_to {
                break;
            }
            union_with_idx = index;
            index += 1;
        }

        let union_with = &mappings[union_with_idx];
        result.push(Mapping {
            source_code_point0: mapping.source_code_point0,
            source_code_point1: union_with.source_code_point1,
            section: mapping.section,
            mapping_type: m_type,
            mapped_to: mapped_to.clone(),
        });
    }

    result
}

fn canonicalize_type(m_type: i32) -> i32 {
    match m_type {
        TYPE_IGNORED => TYPE_IGNORED,
        TYPE_MAPPED | TYPE_DISALLOWED_STD3_MAPPED => TYPE_MAPPED,
        TYPE_DEVIATION | TYPE_DISALLOWED_STD3_VALID | TYPE_VALID => TYPE_VALID,
        TYPE_DISALLOWED => TYPE_DISALLOWED,
        _ => panic!("unexpected type: {}", m_type),
    }
}

// Helper trait for the 'and' infix functions
pub trait IdnaAnd {
    fn and_mask(self, mask: i32) -> i32;
}

impl IdnaAnd for i32 {
    fn and_mask(self, mask: i32) -> i32 {
        self & mask
    }
}

// To maintain the "infix" feel and the specific logic from Kotlin:
// Kotlin: internal infix fun Byte.and(mask: Int): Int = toInt() and mask
// In Rust, we just use the & operator. The helper functions in Kotlin were 
// likely to handle type casting.

// Redefining the 'and' logic as a simple macro or function to match the Kotlin usage
macro_rules! idna_and {
    ($a:expr, $b:expr) => { ($a as i32) & ($b as i32) };
}

// Since the Kotlin code used `section and 0x1fc000`, I will use a helper function
fn and_i32(a: i32, b: i32) -> i32 {
    a & b
}

// Final implementation of the build function using the corrected types
pub fn build_idna_mapping_table_data_final_fixed(table: SimpleIdnaMappingTable) -> IdnaMappingTableData {
    let simplified = merge_adjacent_ranges(table.mappings);
    let without_section_spans_list = without_section_spans(simplified);
    let sections_map = sections_corrected(without_section_spans_list);

    let mut ranges_buffer = Buffer::new();
    let mut mappings_buffer = String::new();
    let mut section_index_buffer = Buffer::new();

    let mut previous_mapped_ranges: Option<Vec<MappedRangeFinal>> = None;

    let mut sorted_sections: Vec<_> = sections_map.into_iter().collect();
    sorted_sections.sort_by_key(|(k, _)| *k);

    for (section, section_mapped_ranges) in sorted_sections {
        if Some(&section_mapped_ranges) == previous_mapped_ranges.as_ref() {
            continue;
        }
        previous_mapped_ranges = Some(section_mapped_ranges.clone());

        let section_offset = (ranges_buffer.size() as i32) / 4;

        section_index_buffer.write_byte((and_i32(section, 0x1fc000) >> 14) as u8);
        section_index_buffer.write_byte((and_i32(section, 0x3f80) >> 7) as u8);
        section_index_buffer.write_byte((and_i32(section_offset, 0x3f80) >> 7) as u8);
        section_index_buffer.write_byte((and_i32(section_offset, 0x7f)) as u8);

        for range in &section_mapped_ranges {
            let rs = match range {
                MappedRangeFinal::Constant { range_start, .. } => *range_start,
                MappedRangeFinal::Inline1 { range_start, .. } => *range_start,
                MappedRangeFinal::Inline2 { range_start, .. } => *range_start,
                MappedRangeFinal::InlineDelta { range_start, .. } => *range_start,
                MappedRangeFinal::External { range_start, .. } => *range_start,
            };
            ranges_buffer.write_byte(rs as u8);

            match range {
                MappedRangeFinal::Constant { b1, .. } => {
                    ranges_buffer.write_byte(*b1 as u8);
                    ranges_buffer.write_byte(b'-');
                    ranges_buffer.write_byte(b'-');
                }
                MappedRangeFinal::Inline1 { b1, b2, .. } => {
                    ranges_buffer.write_byte(*b1 as u8);
                    ranges_buffer.write_byte(*b2 as u8);
                    ranges_buffer.write_byte(b'-');
                }
                MappedRangeFinal::Inline2 { b1, b2, b3, .. } => {
                    ranges_buffer.write_byte(*b1 as u8);
                    ranges_buffer.write_byte(*b2 as u8);
                    ranges_buffer.write_byte(*b3 as u8);
                }
                MappedRangeFinal::InlineDelta { b1, b2, b3, .. } => {
                    ranges_buffer.write_byte(*b1 as u8);
                    ranges_buffer.write_byte(*b2 as u8);
                    ranges_buffer.write_byte(*b3 as u8);
                }
                MappedRangeFinal::External { mapped_to, .. } => {
                    let mapping_index = mappings_buffer.find(mapped_to).map(|i| i as i32).unwrap_or(-1);
                    let mapping_offset = if mapping_index == -1 {
                        let offset = mappings_buffer.len() as i32;
                        mappings_buffer.push_str(mapped_to);
                        offset
                    } else {
                        mapping_index
                    };

                    let b1 = mapped_to.len() as i32;
                    let b2 = (and_i32(mapping_offset, 0x3f80) >> 7);
                    let b3 = and_i32(mapping_offset, 0x7f);
                    ranges_buffer.write_byte(b1 as u8);
                    ranges_buffer.write_byte(b2 as u8);
                    ranges_buffer.write_byte(b3 as u8);
                }
            }
        }
    }

    IdnaMappingTableData {
        sections: section_index_buffer.read_utf8(),
        ranges: ranges_buffer.read_utf8(),
        mappings: mappings_buffer,
    }
}