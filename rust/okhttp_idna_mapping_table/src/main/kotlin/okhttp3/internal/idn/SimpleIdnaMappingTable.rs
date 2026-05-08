use okio::{Buffer, ByteString};
use okio::BufferedSink;
use okio::BufferedSource;
use std::cmp::Ordering;
use std::io::{Error, ErrorKind};
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::ResponseBodyTest::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::internal::idn::IdnaMappingTableTest::*;

// A decoded mapping table that can perform the mapping step of IDNA processing.
//
// This implementation is optimized for readability over efficiency.
//
// This implements non-transitional processing by preserving deviation characters.
//
// This implementation's STD3 rules are configured to `UseSTD3ASCIIRules=false`. This is
// permissive and permits the `_` character.


impl SimpleIdnaMappingTable {
    pub fn new(mappings: Vec<Mapping>) -> Self {
        Self { mappings }
    }

    // Returns true if the code_point was applied successfully. Returns false if it was disallowed.
    pub fn map(&self, code_point: i32, sink: &mut dyn BufferedSink) -> bool {
        let index = self.mappings.binary_search_by(|it| {
            if it.source_code_point1 < code_point {
                Ordering::Less
            } else if it.source_code_point0 > code_point {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });

        match index {
            Ok(idx) => {
                let mapping = &self.mappings[idx];
                let mut result = true;

                match mapping.mapping_type {
                    TYPE_IGNORED => {}
                    TYPE_MAPPED | TYPE_DISALLOWED_STD3_MAPPED => {
                        sink.write(&mapping.mapped_to);
                    }
                    TYPE_DEVIATION | TYPE_DISALLOWED_STD3_VALID | TYPE_VALID => {
                        sink.write_utf8_code_point(code_point);
                    }
                    TYPE_DISALLOWED => {
                        sink.write_utf8_code_point(code_point);
                        result = false;
                    }
                    _ => {}
                }
                result
            }
            Err(_) => {
                // Code points must be in 0..0x10ffff.
                panic!("unexpected code point: {}", code_point);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mapping {
    pub source_code_point0: i32,
    pub source_code_point1: i32,
    pub mapping_type: i32,
    pub mapped_to: ByteString,
}

impl Mapping {
    pub fn section(&self) -> i32 {
        self.source_code_point0 & 0x1fff80
    }

    pub fn range_start(&self) -> i32 {
        self.source_code_point0 & 0x7f
    }

    pub fn has_single_source_code_point(&self) -> bool {
        self.source_code_point0 == self.source_code_point1
    }

    pub fn spans_sections(&self) -> bool {
        (self.source_code_point0 & 0x1fff80) != (self.source_code_point1 & 0x1fff80)
    }
}

pub const TYPE_DEVIATION: i32 = 0;
pub const TYPE_DISALLOWED: i32 = 1;
pub const TYPE_DISALLOWED_STD3_MAPPED: i32 = 2;
pub const TYPE_DISALLOWED_STD3_VALID: i32 = 3;
pub const TYPE_IGNORED: i32 = 4;
pub const TYPE_MAPPED: i32 = 5;
pub const TYPE_VALID: i32 = 6;

const DELIMITER_DOT: i64 = 0;
const DELIMITER_SPACE: i64 = 1;
const DELIMITER_SEMICOLON: i64 = 2;
const DELIMITER_HASH: i64 = 3;
const DELIMITER_NEWLINE: i64 = 4;

fn select_delimiter(source: &mut dyn BufferedSource) -> i64 {
    let first = source.peek().first().cloned().unwrap_or(0);
    match first {
        b'.' => DELIMITER_DOT,
        b' ' => DELIMITER_SPACE,
        b';' => DELIMITER_SEMICOLON,
        b'#' => DELIMITER_HASH,
        b'\n' => DELIMITER_NEWLINE,
        _ => -1,
    }
}

fn select_type(source: &mut dyn BufferedSource) -> i32 {
    let peeked = source.peek();
    if peeked.starts_with(b"deviation ") {
        TYPE_DEVIATION
    } else if peeked.starts_with(b"disallowed_STD3_mapped ") {
        TYPE_DISALLOWED_STD3_MAPPED
    } else if peeked.starts_with(b"disallowed_STD3_valid ") {
        TYPE_DISALLOWED_STD3_VALID
    } else if peeked.starts_with(b"disallowed ") {
        TYPE_DISALLOWED
    } else if peeked.starts_with(b"ignored ") {
        TYPE_IGNORED
    } else if peeked.starts_with(b"mapped ") {
        TYPE_MAPPED
    } else if peeked.starts_with(b"valid ") {
        TYPE_VALID
    } else {
        -1
    }
}

pub trait BufferedSourceExt {
    fn skip_whitespace(&mut self) -> std::io::Result<()>;
    fn skip_rest_of_line(&mut self) -> std::io::Result<()>;
    fn read_hexadecimal_unsigned_long(&mut self) -> std::io::Result<u64>;
    fn read_plain_text_idna_mapping_table(&mut self) -> std::io::Result<SimpleIdnaMappingTable>;
}

impl<S: BufferedSource + ?Sized> BufferedSourceExt for S {
    fn skip_whitespace(&mut self) -> std::io::Result<()> {
        while !self.exhausted() {
            if self.peek().first() != Some(&b' ') {
                return Ok(());
            }
            self.skip(1);
        }
        Ok(())
    }

    fn skip_rest_of_line(&mut self) -> std::io::Result<()> {
        let buffer = self.peek();
        if let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            self.skip((pos + 1) as i64);
        } else {
            self.skip(buffer.len() as i64);
        }
        Ok(())
    }

    fn read_hexadecimal_unsigned_long(&mut self) -> std::io::Result<u64> {
        let mut result = 0u64;
        let mut count = 0;
        while !self.exhausted() {
            let b = self.peek().first().cloned().unwrap_or(0);
            let val = match b {
                b'0'..=b'9' => b - b'0',
                b'A'..=b'F' => b - b'A' + 10,
                b'a'..=b'f' => b - b'a' + 10,
                _ => break,
            };
            self.read_byte()?;
            result = (result << 4) | (val as u64);
            count += 1;
        }
        if count == 0 {
            return Err(Error::new(ErrorKind::InvalidData, "expected hexadecimal"));
        }
        Ok(result)
    }

    fn read_plain_text_idna_mapping_table(&mut self) -> std::io::Result<SimpleIdnaMappingTable> {
        let mut mapped_to_buffer = Buffer::new();
        let mut result = Vec::new();

        while !self.exhausted() {
            match select_delimiter(self) {
                DELIMITER_HASH => {
                    self.skip_rest_of_line()?;
                    continue;
                }
                DELIMITER_NEWLINE => {
                    self.skip(1); // consume newline
                    continue;
                }
                DELIMITER_DOT | DELIMITER_SPACE | DELIMITER_SEMICOLON => {
                    return Err(Error::new(ErrorKind::InvalidData, "unexpected delimiter"));
                }
                _ => {}
            }

            let source_code_point0 = self.read_hexadecimal_unsigned_long()?;
            let source_code_point1 = if select_delimiter(self) == DELIMITER_DOT {
                if self.read_byte()? != b'.' {
                    return Err(Error::new(ErrorKind::InvalidData, "expected '..'"));
                }
                if self.read_byte()? != b'.' {
                    return Err(Error::new(ErrorKind::InvalidData, "expected '..'"));
                }
                self.read_hexadecimal_unsigned_long()?
            } else {
                source_code_point0
            };

            self.skip_whitespace()?;
            if self.read_byte()? != b';' {
                return Err(Error::new(ErrorKind::InvalidData, "expected ';'"));
            }

            self.skip_whitespace()?;
            let mapping_type = select_type(self);

            match mapping_type {
                TYPE_DEVIATION | TYPE_MAPPED | TYPE_DISALLOWED_STD3_MAPPED => {
                    let type_str = match mapping_type {
                        TYPE_DEVIATION => b"deviation ",
                        TYPE_MAPPED => b"mapped ",
                        TYPE_DISALLOWED_STD3_MAPPED => b"disallowed_STD3_mapped ",
                        _ => b"",
                    };
                    self.skip(type_str.len() as i64);

                    self.skip_whitespace()?;
                    if self.read_byte()? != b';' {
                        return Err(Error::new(ErrorKind::InvalidData, "expected ';'"));
                    }

                    loop {
                        self.skip_whitespace()?;
                        match select_delimiter(self) {
                            DELIMITER_HASH => break,
                            DELIMITER_DOT | DELIMITER_SEMICOLON | DELIMITER_NEWLINE => {
                                return Err(Error::new(ErrorKind::InvalidData, "unexpected delimiter"));
                            }
                            _ => {}
                        }
                        let cp = self.read_hexadecimal_unsigned_long()? as i32;
                        mapped_to_buffer.write_utf8_code_point(cp);
                    }
                }
                TYPE_DISALLOWED | TYPE_DISALLOWED_STD3_VALID | TYPE_IGNORED | TYPE_VALID => {
                    let type_str = match mapping_type {
                        TYPE_DISALLOWED => b"disallowed ",
                        TYPE_DISALLOWED_STD3_VALID => b"disallowed_STD3_valid ",
                        TYPE_IGNORED => b"ignored ",
                        TYPE_VALID => b"valid ",
                        _ => b"",
                    };
                    self.skip(type_str.len() as i64);
                }
                _ => return Err(Error::new(ErrorKind::InvalidData, "unexpected type")),
            }

            self.skip_rest_of_line()?;

            result.push(Mapping {
                source_code_point0: source_code_point0 as i32,
                source_code_point1: source_code_point1 as i32,
                mapping_type,
                mapped_to: mapped_to_buffer.read_byte_string(),
            });
            mapped_to_buffer.clear();
        }

        Ok(SimpleIdnaMappingTable::new(result))
    }
}
