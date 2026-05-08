use std::io::Write;

// A trait to simulate the BufferedSink behavior from okio.
// In a real production environment, this would be a trait implemented by a buffer or stream.
pub trait BufferedSink: Write {
    fn write_utf8(&mut self, source: &str, begin: usize, end: usize) -> std::io::Result<()> {
        let slice = &source[begin..end];
        self.write_all(slice.as_bytes())
    }

    fn write_utf8_code_point(&mut self, code_point: i32) -> std::io::Result<()> {
        let s = std::char::from_u32(code_point as u32)
            .map(|c| c.to_string())
            .expect("Invalid Unicode code point");
        self.write_all(s.as_bytes())
    }
}

// An IDNA mapping table optimized for small code and data size.
#[derive(Debug, Clone, PartialEq)]
pub struct IdnaMappingTable {
    pub sections: String,
    pub ranges: String,
    pub mappings: String,
}

impl IdnaMappingTable {
    pub fn new(sections: String, ranges: String, mappings: String) -> Self {
        Self {
            sections,
            ranges,
            mappings,
        }
    }

    // Returns true if the [code_point] was applied successfully. Returns false if it was disallowed.
    pub fn map<S: BufferedSink>(&self, code_point: i32, sink: &mut S) -> bool {
        let sections_index = self.find_sections_index(code_point);

        let ranges_position = self.sections.read_14_bit_int(sections_index + 2);

        let ranges_limit = if sections_index + 4 < self.sections.len() {
            self.sections.read_14_bit_int(sections_index + 6)
        } else {
            (self.ranges.len() / 4) as i32
        };

        let ranges_index = self.find_ranges_offset(code_point, ranges_position, ranges_limit);

        // Rust strings are UTF-8. The Kotlin code treats them as arrays of 16-bit chars (UTF-16).
        // However, the documentation states the data is strictly ASCII (bit 0x80 not used).
        // Therefore, we can index into the bytes of the string.
        let ranges_bytes = self.ranges.as_bytes();
        let b1 = ranges_bytes[ranges_index + 1] as i32;

        match b1 {
            0..=63 => {
                // Length of the UTF-16 sequence that this range maps to. The offset is b2b3.
                let begin_index = self.ranges.read_14_bit_int(ranges_index + 2) as usize;
                let _ = sink.write_utf8(&self.mappings, begin_index, begin_index + b1 as usize);
            }
            64..=79 => {
                // Mapped inline as codePoint delta to subtract
                let b2 = ranges_bytes[ranges_index + 2] as i32;
                let b3 = ranges_bytes[ranges_index + 3] as i32;

                let codepoint_delta = ((b1 & 0xF) << 14) | (b2 << 7) | b3;
                let _ = sink.write_utf8_code_point(code_point - codepoint_delta);
            }
            80..=95 => {
                // Mapped inline as codePoint delta to add
                let b2 = ranges_bytes[ranges_index + 2] as i32;
                let b3 = ranges_bytes[ranges_index + 3] as i32;

                let codepoint_delta = ((b1 & 0xF) << 14) | (b2 << 7) | b3;
                let _ = sink.write_utf8_code_point(code_point + codepoint_delta);
            }
            119 => {
                // Ignored.
            }
            120 => {
                // Valid.
                let _ = sink.write_utf8_code_point(code_point);
            }
            121 => {
                // Disallowed.
                let _ = sink.write_utf8_code_point(code_point);
                return false;
            }
            122 => {
                // Mapped inline to the sequence: [b2].
                let _ = sink.write(&[ranges_bytes[ranges_index + 2]]);
            }
            123 => {
                // Mapped inline to the sequence: [b2a].
                let _ = sink.write(&[(ranges_bytes[ranges_index + 2] | 0x80)]);
            }
            124 => {
                // Mapped inline to the sequence: [b2, b3].
                let _ = sink.write(&[ranges_bytes[ranges_index + 2], ranges_bytes[ranges_index + 3]]);
            }
            125 => {
                // Mapped inline to the sequence: [b2a, b3].
                let _ = sink.write(&[(ranges_bytes[ranges_index + 2] | 0x80), ranges_bytes[ranges_index + 3]]);
            }
            126 => {
                // Mapped inline to the sequence: [b2, b3a].
                let _ = sink.write(&[ranges_bytes[ranges_index + 2], (ranges_bytes[ranges_index + 3] | 0x80)]);
            }
            127 => {
                // Mapped inline to the sequence: [b2a, b3a].
                let _ = sink.write(&[(ranges_bytes[ranges_index + 2] | 0x80), (ranges_bytes[ranges_index + 3] | 0x80)]);
            }
            _ => {
                panic!("unexpected rangesIndex for {}", code_point);
            }
        }

        true
    }

    fn find_sections_index(&self, code_point: i32) -> usize {
        let target = (code_point & 0x1fff80) >> 7;
        let offset = binary_search(
            0,
            (self.sections.len() / 4) as i32,
            |index| {
                let entry_index = (index * 4) as usize;
                let b0b1 = self.sections.read_14_bit_int(entry_index);
                target.cmp(&b0b1)
            },
        );

        if offset >= 0 {
            (offset * 4) as usize
        } else {
            ((-offset - 2) * 4) as usize
        }
    }

    fn find_ranges_offset(&self, code_point: i32, position: i32, limit: i32) -> usize {
        let target = code_point & 0x7f;
        let offset = binary_search(
            position,
            limit,
            |index| {
                let entry_index = (index * 4) as usize;
                let b0 = self.ranges.as_bytes()[entry_index] as i32;
                target.cmp(&b0)
            },
        );

        if offset >= 0 {
            (offset * 4) as usize
        } else {
            ((-offset - 2) * 4) as usize
        }
    }
}

pub trait Read14BitInt {
    fn read_14_bit_int(&self, index: usize) -> i32;
}

impl Read14BitInt for String {
    fn read_14_bit_int(&self, index: usize) -> i32 {
        let bytes = self.as_bytes();
        let b0 = bytes[index] as i32;
        let b1 = bytes[index + 1] as i32;
        (b0 << 7) + b1
    }
}

pub fn binary_search<F>(position: i32, limit: i32, compare: F) -> i32
where
    F: Fn(i32) -> std::cmp::Ordering,
{
    let mut low = position;
    let mut high = limit - 1;
    while low <= high {
        let mid = (low + high) / 2;
        match compare(mid) {
            std::cmp::Ordering::Less => high = mid - 1,
            std::cmp::Ordering::Greater => low = mid + 1,
            std::cmp::Ordering::Equal => return mid,
        }
    }
    -low - 1
}