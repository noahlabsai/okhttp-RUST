use num_bigint::BigInt;
use okio::{Buffer, BufferedSource, ByteString, Source};
use std::any::Any;
use std::io;

/// A synthetic value that indicates there's no more bytes.
const END_OF_DATA: DerHeader = DerHeader {
    tag_class: DerHeader::TAG_CLASS_UNIVERSAL,
    tag: DerHeader::TAG_END_OF_CONTENTS,
    constructed: false,
    length: -1,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DerHeader {
    pub tag_class: i32,
    pub tag: i64,
    pub constructed: bool,
    pub length: i64,
}

impl DerHeader {
    pub const TAG_CLASS_UNIVERSAL: i32 = 0;
    pub const TAG_END_OF_CONTENTS: i64 = 0;

    pub fn is_end_of_data(&self) -> bool {
        self.tag_class == Self::TAG_CLASS_UNIVERSAL
            && self.tag == Self::TAG_END_OF_CONTENTS
            && !self.constructed
            && self.length == -1
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BitString {
    pub data: ByteString,
    pub unused_bit_count: i32,
}

pub struct DerReader {
    counting_source: CountingSource,
    source: BufferedSource,
    limit: i64,
    type_hint_stack: Vec<Option<Box<dyn Any>>>,
    path: Vec<String>,
    constructed: bool,
    peeked_header: Option<DerHeader>,
}

impl DerReader {
    pub fn new(source: Box<dyn Source + Send>) -> Self {
        let counting_source = CountingSource::new(source);
        // In okio-rust, BufferedSource is typically created via .buffer()
        let source = BufferedSource::new(counting_source.clone());

        Self {
            counting_source,
            source,
            limit: -1,
            type_hint_stack: Vec::new(),
            path: Vec::new(),
            constructed: false,
            peeked_header: None,
        }
    }

    fn byte_count(&self) -> i64 {
        self.counting_source.bytes_read() - self.source.size() as i64
    }

    fn bytes_left(&self) -> i64 {
        if self.limit == -1 {
            -1
        } else {
            self.limit - self.byte_count()
        }
    }

    pub fn type_hint(&self) -> Option<&dyn Any> {
        self.type_hint_stack.last().and_then(|opt| opt.as_ref().map(|b| b.as_ref()))
    }

    pub fn set_type_hint(&mut self, value: Option<Box<dyn Any>>) {
        if let Some(last) = self.type_hint_stack.last_mut() {
            *last = value;
        }
    }

    pub fn has_next(&mut self) -> bool {
        self.peek_header().is_some()
    }

    pub fn peek_header(&mut self) -> Option<DerHeader> {
        let result = if let Some(header) = self.peeked_header {
            header
        } else {
            let header = self.read_header();
            self.peeked_header = Some(header);
            header
        };

        if result.is_end_of_data() {
            None
        } else {
            Some(result)
        }
    }

    pub fn read_header(&mut self) -> DerHeader {
        if self.peeked_header.is_some() {
            panic!("peekedHeader must be null to call readHeader");
        }

        if self.byte_count() == self.limit {
            return END_OF_DATA;
        }

        if self.limit == -1 && self.source.exhausted() {
            return END_OF_DATA;
        }

        let tag_and_class = (self.source.read_byte() as i32) & 0xff;
        let tag_class = tag_and_class & 0b1100_0000;
        let constructed = (tag_and_class & 0b0010_0000) == 0b0010_0000;

        let tag = match tag_and_class & 0b0001_1111 {
            0b0001_1111 => self.read_variable_length_long(),
            tag0 => tag0 as i64,
        };

        let length0 = (self.source.read_byte() as i32) & 0xff;
        let length = if length0 == 0b1000_0000 {
            panic!("indefinite length not permitted for DER");
        } else if (length0 & 0b1000_0000) == 0b1000_0000 {
            let length_bytes = length0 & 0b0111_1111;
            if length_bytes > 8 {
                panic!("length encoded with more than 8 bytes is not supported");
            }

            let mut length_bits = (self.source.read_byte() as i64) & 0xff;
            if length_bits == 0 || (length_bytes == 1 && (length_bits & 0b1000_0000) == 0) {
                panic!("invalid encoding for length");
            }

            for _ in 1..length_bytes {
                length_bits = (length_bits << 8) + ((self.source.read_byte() as i64) & 0xff);
            }

            if length_bits < 0 {
                panic!("length > Long.MAX_VALUE");
            }
            length_bits
        } else {
            (length0 & 0b0111_1111) as i64
        };

        DerHeader {
            tag_class,
            tag,
            constructed,
            length,
        }
    }

    pub fn read<T, F>(&mut self, name: Option<&str>, block: F) -> T
    where
        F: FnOnce(&mut Self, DerHeader) -> T,
    {
        if !self.has_next() {
            panic!("expected a value");
        }

        let header = self.peeked_header.take().expect("hasNext() was true");

        let pushed_limit = self.limit;
        let pushed_constructed = self.constructed;

        let new_limit = if header.length != -1 {
            self.byte_count() + header.length
        } else {
            -1
        };

        if pushed_limit != -1 && new_limit > pushed_limit {
            panic!("enclosed object too large");
        }

        self.limit = new_limit;
        self.constructed = header.constructed;
        if let Some(n) = name {
            self.path.push(n.to_string());
        }

        let result = block(self, header);

        if new_limit != -1 && self.byte_count() > new_limit {
            panic!("unexpected byte count at {}", self.to_string());
        }

        self.peeked_header = None;
        self.limit = pushed_limit;
        self.constructed = pushed_constructed;
        if name.is_some() {
            self.path.pop();
        }

        result
    }

    pub fn with_type_hint<T, F>(&mut self, block: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        self.type_hint_stack.push(None);
        let result = block(self);
        self.type_hint_stack.pop();
        result
    }

    pub fn read_boolean(&mut self) -> bool {
        if self.bytes_left() != 1 {
            panic!("unexpected length: {} at {}", self.bytes_left(), self.to_string());
        }
        self.source.read_byte() != 0
    }

    pub fn read_big_integer(&mut self) -> BigInt {
        let left = self.bytes_left();
        if left == 0 {
            panic!("unexpected length: {} at {}", left, self.to_string());
        }
        let bytes = self.source.read_byte_string(left).to_bytes();
        BigInt::from_bytes_be(num_bigint::Sign::Plus, &bytes)
    }

    pub fn read_long(&mut self) -> i64 {
        let left = self.bytes_left();
        if !(1..=8).contains(&left) {
            panic!("unexpected length: {} at {}", left, self.to_string());
        }

        let mut result = self.source.read_byte() as i64;
        while self.byte_count() < self.limit {
            result = (result << 8) + ((self.source.read_byte() as i64) & 0xff);
        }
        result
    }

    pub fn read_bit_string(&mut self) -> BitString {
        if self.bytes_left() == -1 || self.constructed {
            panic!("constructed bit strings not supported for DER");
        }
        if self.bytes_left() < 1 {
            panic!("malformed bit string");
        }
        let unused_bit_count = (self.source.read_byte() as i32) & 0xff;
        let byte_string = self.source.read_byte_string(self.bytes_left());
        BitString {
            data: byte_string,
            unused_bit_count,
        }
    }

    pub fn read_octet_string(&mut self) -> ByteString {
        if self.bytes_left() == -1 || self.constructed {
            panic!("constructed octet strings not supported for DER");
        }
        self.source.read_byte_string(self.bytes_left())
    }

    pub fn read_utf8_string(&mut self) -> String {
        if self.bytes_left() == -1 || self.constructed {
            panic!("constructed strings not supported for DER");
        }
        self.source.read_utf8(self.bytes_left())
    }

    pub fn read_object_identifier(&mut self) -> String {
        let mut result = Buffer::new();
        let dot = b'.';
        let xy = self.read_variable_length_long();
        if xy < 40 {
            result.write_decimal_long(0);
            result.write_byte(dot);
            result.write_decimal_long(xy);
        } else if xy < 80 {
            result.write_decimal_long(1);
            result.write_byte(dot);
            result.write_decimal_long(xy - 40);
        } else {
            result.write_decimal_long(2);
            result.write_byte(dot);
            result.write_decimal_long(xy - 80);
        }
        while self.byte_count() < self.limit {
            result.write_byte(dot);
            result.write_decimal_long(self.read_variable_length_long());
        }
        result.read_utf8()
    }

    pub fn read_relative_object_identifier(&mut self) -> String {
        let mut result = Buffer::new();
        let dot = b'.';
        while self.byte_count() < self.limit {
            if result.size() > 0 {
                result.write_byte(dot);
            }
            result.write_decimal_long(self.read_variable_length_long());
        }
        result.read_utf8()
    }

    fn read_variable_length_long(&mut self) -> i64 {
        let mut result = 0i64;
        loop {
            let byte_n = (self.source.read_byte() as i64) & 0xff;
            if (byte_n & 0b1000_0000) == 0b1000_0000 {
                result = (result + (byte_n & 0b0111_1111)) << 7;
            } else {
                return result + byte_n;
            }
        }
    }

    pub fn read_unknown(&mut self) -> ByteString {
        self.source.read_byte_string(self.bytes_left())
    }
}

impl std::fmt::Display for DerReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.join(" / "))
    }
}

#[derive(Clone)]
struct CountingSource {
    inner: std::sync::Arc<std::sync::Mutex<Box<dyn Source + Send>>>,
    bytes_read: std::sync::Arc<std::sync::atomic::AtomicI64>,
}

impl CountingSource {
    fn new(source: Box<dyn Source + Send>) -> Self {
        Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(source)),
            bytes_read: std::sync::Arc::new(std::sync::atomic::AtomicI64::new(0)),
        }
    }

    fn bytes_read(&self) -> i64 {
        self.bytes_read.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Source for CountingSource {
    fn read(&mut self, sink: &mut Buffer, byte_count: i64) -> io::Result<i64> {
        let mut inner = self.inner.lock().unwrap();
        let result = inner.read(sink, byte_count)?;
        if result != -1 {
            self.bytes_read.fetch_add(result, std::sync::atomic::Ordering::SeqCst);
        }
        Ok(result)
    }

    fn timeout(&self, timeout: std::time::Duration) -> io::Result<()> {
        let inner = self.inner.lock().unwrap();
        inner.timeout(timeout)
    }

    fn close(&mut self) -> io::Result<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.close()
    }
}

trait BufferExt {
    fn write_decimal_long(&mut self, value: i64);
}

impl BufferExt for Buffer {
    fn write_decimal_long(&mut self, value: i64) {
        self.write_utf8(&value.to_string());
    }
}
