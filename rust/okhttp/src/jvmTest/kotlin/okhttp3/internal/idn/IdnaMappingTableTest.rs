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

use std::cmp::Ordering;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::cache::FaultHidingSink::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::idn::IdnaMappingTable::*;

// Mocking Okio Buffer for the test environment

impl Buffer {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn write_utf8(&mut self, s: &str) {
        self.data.extend_from_slice(s.as_bytes());
    }

    pub fn read_utf8(&mut self) -> String {
        let s = String::from_utf8_lossy(&self.data).into_owned();
        self.data.clear();
        s
    }
}

// The actual IdnaMappingTable structure as used in the tests

impl IdnaMappingTable {
    pub fn map(&self, code_point: i32, buffer: &mut Buffer) -> bool {
        // In a real scenario, this would be the actual implementation.
        // For the test unit, we assume the implementation exists in the target crate.
        true
    }

    pub fn read_14_bit_int(&self, offset: usize) -> i32 {
        let bytes = self.sections.as_bytes();
        if offset + 2 >= bytes.len() {
            return 0;
        }
        // 14-bit read logic: 2 bytes, 7 bits each
        ((bytes[offset] as i32 & 0x7F) << 7) | (bytes[offset + 1] as i32 & 0x7F)
    }
}

// Plain text representation of the table
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleIdnaMappingTable {
    // Fields representing the plain text table structure
    pub mappings: Vec<(i32, String)>,
}

impl SimpleIdnaMappingTable {
    pub fn map(&self, code_point: i32, buffer: &mut Buffer) -> bool {
        if code_point < 0 || code_point > 0x10FFFF {
            panic!("IllegalArgumentException: code point out of range");
        }
        if let Some((_, mapping)) = self.mappings.iter().find(|(cp, _)| *cp == code_point) {
            buffer.write_utf8(mapping);
            return true;
        }
        true
    }
}

pub struct IdnaMappingTableData {
    pub sections: String,
    pub ranges: String,
    pub mappings: String,
}

// Global constant as referenced in the test
pub const IDNA_MAPPING_TABLE: IdnaMappingTable = IdnaMappingTable {
    sections: String::new(),
    ranges: String::new(),
    mappings: String::new(),
};

// Mocking resource loading functions
fn read_plain_text_idna_mapping_table() -> SimpleIdnaMappingTable {
    SimpleIdnaMappingTable {
        mappings: Vec::new(),
    }
}

fn build_idna_mapping_table_data(_table: &SimpleIdnaMappingTable) -> IdnaMappingTableData {
    IdnaMappingTableData {
        sections: String::new(),
        ranges: String::new(),
        mappings: String::new(),
    }
}

pub struct IdnaMappingTableTest {
    table: Option<SimpleIdnaMappingTable>,
    compact_table: Option<IdnaMappingTable>,
}

impl IdnaMappingTableTest {
    pub fn new() -> Self {
        Self {
            table: None,
            compact_table: None,
        }
    }

    pub fn set_up(&mut self) {
        let plain_table = read_plain_text_idna_mapping_table();
        let data = build_idna_mapping_table_data(&plain_table);

        self.table = Some(plain_table);
        self.compact_table = Some(IdnaMappingTable {
            sections: data.sections,
            ranges: data.ranges,
            mappings: data.mappings,
        });
    }

    fn map_string(&self, s: &str) -> String {
        let mut buffer = Buffer::new();
        let table = self.table.as_ref().expect("table not initialized");
        for cp in s.chars().map(|c| c as i32) {
            if !table.map(cp, &mut buffer) {
                panic!("Mapping failed");
            }
        }
        buffer.read_utf8()
    }

    fn map_expecting_errors(&self, s: &str) -> String {
        let mut buffer = Buffer::new();
        let mut error_count = 0;
        let table = self.table.as_ref().expect("table not initialized");
        for cp in s.chars().map(|c| c as i32) {
            if !table.map(cp, &mut buffer) {
                error_count += 1;
            }
        }
        assert!(error_count > 0, "Expected errors but found none");
        buffer.read_utf8()
    }

    pub fn regular_mappings(&self) {
        assert_eq!(self.map_string("hello"), "hello");
        assert_eq!(self.map_string("hello-world"), "hello-world");
        assert_eq!(self.map_string("HELLO"), "hello");
        assert_eq!(self.map_string("Hello"), "hello");
        assert_eq!(self.map_string("\u{00bc}"), "1\u{2044}4");
        assert_eq!(self.map_string("\u{2122}"), "tm");
    }

    pub fn validate_compact_table_invariants(&self) {
        let compact = self.compact_table.as_ref().expect("compactTable not initialized");

        assert!(compact.sections.len() < (1 << 14));
        assert!(compact.ranges.len() < (1 << 14) * 4);
        assert!(compact.mappings.len() < (1 << 14));

        for data_string in &[&compact.sections, &compact.ranges] {
            for b in data_string.as_bytes() {
                assert_eq!(*b & 0x7f, *b);
            }
        }

        let mut ranges_indices = Vec::new();
        let mut ranges_offsets = Vec::new();
        for i in (0..compact.sections.len()).step_by(4) {
            ranges_indices.push(compact.read_14_bit_int(i));
            ranges_offsets.push(compact.read_14_bit_int(i + 2));
        }

        let mut sorted_indices = ranges_indices.clone();
        sorted_indices.sort();
        assert_eq!(ranges_indices, sorted_indices);

        for r in 0..ranges_offsets.len() {
            let range_pos = (ranges_offsets[r] * 4) as usize;
            let range_limit = if r + 1 < ranges_offsets.len() {
                (ranges_offsets[r + 1] * 4) as usize
            } else {
                (ranges_offsets.len() * 4) as usize
            };

            assert_eq!(compact.ranges.as_bytes()[range_pos], 0);

            let mut range_starts = Vec::new();
            for i in (range_pos..range_limit).step_by(4) {
                if i < compact.ranges.len() {
                    range_starts.push(compact.ranges.as_bytes()[i] as i32);
                }
            }
            let mut sorted_starts = range_starts.clone();
            sorted_starts.sort();
            assert_eq!(range_starts, sorted_starts);
        }
    }

    pub fn deviations(&self) {
        assert_eq!(self.map_string("\u{00df}"), "\u{00df}");
        assert_eq!(self.map_string("\u{03c2}"), "\u{03c2}");
        assert_eq!(self.map_string("\u{200c}"), "\u{200c}");
        assert_eq!(self.map_string("\u{200d}"), "\u{200d}");
    }

    pub fn ignored(&self) {
        assert_eq!(self.map_string("\u{200b}"), "");
        assert_eq!(self.map_string("\u{feff}"), "");
    }

    pub fn disallowed(&self) {
        assert_eq!(self.map_expecting_errors("\u{0080}"), "\u{0080}");
    }

    pub fn disallowed_std3_valid(&self) {
        assert_eq!(self.map_string("_"), "_");
        assert_eq!(self.map_string("/"), "/");
        assert_eq!(self.map_string("\u{2260}"), "\u{2260}");
    }

    pub fn disallowed_std3_mapped(&self) {
        assert_eq!(self.map_string("\u{00b8}"), "\u{0020}\u{0327}");
        assert_eq!(self.map_string("\u{2474}"), "(1)");
    }

    pub fn out_of_bounds(&self) {
        let table = self.table.as_ref().expect("table not initialized");

        let result = std::panic::catch_unwind(|| {
            table.map(-1, &mut Buffer::new());
        });
        assert!(result.is_err());

        table.map(0, &mut Buffer::new());
        table.map(0x10ffff, &mut Buffer::new());

        let result_high = std::panic::catch_unwind(|| {
            table.map(0x110000, &mut Buffer::new());
        });
        assert!(result_high.is_err());
    }

    pub fn binary_search_even_sized_range(&self) {
        let table = vec![1, 3, 5, 7, 9, 11];

        assert_eq!(binary_search(0, 6, |index| 1.cmp(&table[index])), 0);
        assert_eq!(binary_search(0, 6, |index| 3.cmp(&table[index])), 1);
        assert_eq!(binary_search(0, 6, |index| 5.cmp(&table[index])), 2);
        assert_eq!(binary_search(0, 6, |index| 7.cmp(&table[index])), 3);
        assert_eq!(binary_search(0, 6, |index| 9.cmp(&table[index])), 4);
        assert_eq!(binary_search(0, 6, |index| 11.cmp(&table[index])), 5);

        assert_eq!(binary_search(0, 6, |index| 0.cmp(&table[index])), -1);
        assert_eq!(binary_search(0, 6, |index| 2.cmp(&table[index])), -2);
        assert_eq!(binary_search(0, 6, |index| 4.cmp(&table[index])), -3);
        assert_eq!(binary_search(0, 6, |index| 6.cmp(&table[index])), -4);
        assert_eq!(binary_search(0, 6, |index| 8.cmp(&table[index])), -5);
        assert_eq!(binary_search(0, 6, |index| 10.cmp(&table[index])), -6);
        assert_eq!(binary_search(0, 6, |index| 12.cmp(&table[index])), -7);
    }

    pub fn binary_search_odd_sized_range(&self) {
        let table = vec![1, 3, 5, 7, 9];

        assert_eq!(binary_search(0, 5, |index| 1.cmp(&table[index])), 0);
        assert_eq!(binary_search(0, 5, |index| 3.cmp(&table[index])), 1);
        assert_eq!(binary_search(0, 5, |index| 5.cmp(&table[index])), 2);
        assert_eq!(binary_search(0, 5, |index| 7.cmp(&table[index])), 3);
        assert_eq!(binary_search(0, 5, |index| 9.cmp(&table[index])), 4);

        assert_eq!(binary_search(0, 5, |index| 0.cmp(&table[index])), -1);
        assert_eq!(binary_search(0, 5, |index| 2.cmp(&table[index])), -2);
        assert_eq!(binary_search(0, 5, |index| 4.cmp(&table[index])), -3);
        assert_eq!(binary_search(0, 5, |index| 6.cmp(&table[index])), -4);
        assert_eq!(binary_search(0, 5, |index| 8.cmp(&table[index])), -5);
        assert_eq!(binary_search(0, 5, |index| 10.cmp(&table[index])), -6);
    }

    pub fn binary_search_single_element_range(&self) {
        let table = vec![1];
        assert_eq!(binary_search(0, 1, |index| 1.cmp(&table[index])), 0);
        assert_eq!(binary_search(0, 1, |index| 0.cmp(&table[index])), -1);
        assert_eq!(binary_search(0, 1, |index| 2.cmp(&table[index])), -2);
    }

    pub fn binary_search_empty_range(&self) {
        assert_eq!(binary_search(0, 0, |_| panic!("unexpected call")), -1);
    }

    pub fn compare_plain_and_compact_tables(&self) {
        let table = self.table.as_ref().expect("table not initialized");
        let compact_table = self.compact_table.as_ref().expect("compactTable not initialized");

        for code_point in 0..=0x10ffff {
            let mut buffer = Buffer::new();
            let allowed_by_table = table.map(code_point, &mut buffer);
            let table_mapped_to = buffer.read_utf8();

            let mut buffer_compact = Buffer::new();
            let allowed_by_compact_table = compact_table.map(code_point, &mut buffer_compact);
            let compact_table_mapped_to = buffer_compact.read_utf8();

            assert_eq!(allowed_by_compact_table, allowed_by_table);
            assert_eq!(compact_table_mapped_to, table_mapped_to);
        }
    }

    pub fn compare_constructed_and_generated_compact_tables(&self) {
        let compact = self.compact_table.as_ref().expect("compactTable not initialized");
        assert_eq!(IDNA_MAPPING_TABLE.sections, compact.sections);
        assert_eq!(IDNA_MAPPING_TABLE.ranges, compact.ranges);
        assert_eq!(IDNA_MAPPING_TABLE.mappings, compact.mappings);
    }
}

fn binary_search<F>(low: usize, high: usize, compare: F) -> i32
where
    F: Fn(usize) -> Ordering,
{
    let mut l = low as i32;
    let mut r = high as i32;
    while l < r {
        let mid = l + (r - l) / 2;
        match compare(mid as usize) {
            Ordering::Less => r = mid,
            Ordering::Greater => l = mid + 1,
            Ordering::Equal => return mid,
        }
    }
    -(l + 1)
}
