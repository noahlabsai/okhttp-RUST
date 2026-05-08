use std::cmp::Ordering;
use std::io::{Read, Write};
use std::path::PathBuf;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;

// Mocking okio types as they are external dependencies in the Kotlin source
// In a real production environment, these would be imported from a crate like `okio`


impl Buffer {
    pub fn exhausted(&self) -> bool {
        self.data.is_empty()
    }
    pub fn read_byte(&mut self) -> u8 {
        if self.data.is_empty() {
            panic!("Buffer is exhausted");
        }
        self.data.remove(0)
    }
    pub fn write_byte(&mut self, b: u8) {
        self.data.push(b);
    }
}

// Implementing Index for Buffer to support this[0]
impl std::ops::Index<usize> for Buffer {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

pub trait BufferedSink: Write {
    fn write_byte(&mut self, b: u8) -> std::io::Result<()>;
}

pub trait BufferedSource: Read {
    fn read_byte(&mut self) -> std::io::Result<u8>;
}

pub struct ByteString(Vec<u8>);
impl ByteString {
    pub fn decode_hex(hex: &str) -> Self {
        let bytes = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        ByteString(bytes)
    }
}

pub struct Options(Vec<ByteString>);
impl Options {
    pub fn of(boms: Vec<ByteString>) -> Self {
        Options(boms)
    }
}

pub trait Closeable {
    fn close(&mut self) -> std::io::Result<()>;
}

pub trait FileSystem {
    fn sink(&self, path: PathBuf) -> std::io::Result<Box<dyn BufferedSink>>;
    fn delete(&self, path: PathBuf) -> std::io::Result<()>;
    fn list(&self, path: PathBuf) -> std::io::Result<Vec<PathBuf>>;
    fn metadata(&self, path: PathBuf) -> std::io::Result<FileMetadata>;
}

pub struct FileMetadata {
    pub is_directory: bool,
}

pub const CONST_VERSION: &str = "4.12.0"; // Example version
pub const USER_AGENT: &str = "okhttp/4.12.0";

pub static EMPTY_BYTE_ARRAY: [u8; 0] = [];

// In a real implementation, this would be lazily initialized.
pub static UNICODE_BOMS: Option<Options> = None;

// Returns an array containing only elements found in this array and also in other.
pub fn intersect<C>(this: &[String], other: &[String], comparator: C) -> Vec<String>
where
    C: Fn(&String, &String) -> Ordering,
{
    let mut result = Vec::new();
    for a in this {
        for b in other {
            if comparator(a, b) == Ordering::Equal {
                result.push(a.clone());
                break;
            }
        }
    }
    result
}

// Returns true if there is an element in this array that is also in other.
pub fn has_intersection<C>(this: &[String], other: Option<&[String]>, comparator: C) -> bool
where
    C: Fn(&String, &String) -> Ordering,
{
    if this.is_empty() || other.is_none() || other.unwrap().is_empty() {
        return false;
    }
    let other_ref = other.unwrap();
    for a in this {
        for b in other_ref {
            if comparator(a, b) == Ordering::Equal {
                return true;
            }
        }
    }
    false
}

pub fn index_of<C>(this: &[String], value: &str, comparator: C) -> Option<usize>
where
    C: Fn(&String, &str) -> Ordering,
{
    this.iter().position(|it| comparator(it, value) == Ordering::Equal)
}

pub fn concat(this: &[String], value: String) -> Vec<String> {
    let mut result = this.to_vec();
    result.push(value);
    result
}

pub fn index_of_first_non_ascii_whitespace(s: &str, start_index: usize, end_index: usize) -> usize {
    let bytes = s.as_bytes();
    let end = end_index.min(bytes.len());
    for i in start_index..end {
        match bytes[i] as char {
            '\t' | '\n' | '\x0C' | '\r' | ' ' => continue,
            _ => return i,
        }
    }
    end
}

pub fn index_of_last_non_ascii_whitespace(s: &str, start_index: usize, end_index: usize) -> usize {
    let bytes = s.as_bytes();
    let end = end_index.min(bytes.len());
    if end == 0 { return start_index; }
    for i in (start_index..end).rev() {
        match bytes[i] as char {
            '\t' | '\n' | '\x0C' | '\r' | ' ' => continue,
            _ => return i + 1,
        }
    }
    start_index
}

pub fn trim_substring(s: &str, start_index: usize, end_index: usize) -> String {
    let start = index_of_first_non_ascii_whitespace(s, start_index, end_index);
    let end = index_of_last_non_ascii_whitespace(s, start, end_index);
    if start >= end {
        return String::new();
    }
    s[start..end].to_string()
}

pub fn delimiter_offset_str(s: &str, delimiters: &str, start_index: usize, end_index: usize) -> usize {
    let bytes = s.as_bytes();
    let end = end_index.min(bytes.len());
    for i in start_index..end {
        if delimiters.contains(bytes[i] as char) {
            return i;
        }
    }
    end
}

pub fn delimiter_offset_char(s: &str, delimiter: char, start_index: usize, end_index: usize) -> usize {
    let bytes = s.as_bytes();
    let end = end_index.min(bytes.len());
    for i in start_index..end {
        if bytes[i] as char == delimiter {
            return i;
        }
    }
    end
}

pub fn index_of_control_or_non_ascii(s: &str) -> Option<usize> {
    for (i, c) in s.chars().enumerate() {
        if c <= '\u{001f}' || c >= '\u{007f}' {
            return Some(i);
        }
    }
    None
}

pub fn is_sensitive_header(name: &str) -> bool {
    let n = name.to_lowercase();
    n == "authorization" || n == "cookie" || n == "proxy-authorization" || n == "set-cookie"
}

pub fn parse_hex_digit(c: char) -> i32 {
    match c {
        '0'..='9' => (c as i32) - ('0' as i32),
        'a'..='f' => (c as i32) - ('a' as i32) + 10,
        'A'..='F' => (c as i32) - ('A' as i32) + 10,
        _ => -1,
    }
}

pub fn byte_and(b: u8, mask: i32) -> i32 {
    (b as i32) & mask
}

pub fn short_and(s: i16, mask: i32) -> i32 {
    (s as i32) & mask
}

pub fn int_and(i: i32, mask: i64) -> i64 {
    (i as i64) & mask
}

pub fn write_medium(sink: &mut dyn BufferedSink, medium: i32) -> std::io::Result<()> {
    sink.write_byte(((medium >> 16) & 0xff) as u8)?;
    sink.write_byte(((medium >> 8) & 0xff) as u8)?;
    sink.write_byte((medium & 0xff) as u8)?;
    Ok(())
}

pub fn read_medium(source: &mut dyn BufferedSource) -> std::io::Result<i32> {
    let b1 = (source.read_byte()? as i32) & 0xff;
    let b2 = (source.read_byte()? as i32) & 0xff;
    let b3 = (source.read_byte()? as i32) & 0xff;
    Ok((b1 << 16) | (b2 << 8) | b3)
}

pub fn ignore_io_exceptions<F>(block: F)
where
    F: FnOnce() -> std::io::Result<()>,
{
    let _ = block();
}

pub fn skip_all(buffer: &mut Buffer, b: u8) -> usize {
    let mut count = 0;
    while !buffer.exhausted() && buffer[0] == b {
        count += 1;
        buffer.read_byte();
    }
    count
}

pub fn index_of_non_whitespace(s: &str, start_index: usize) -> usize {
    let bytes = s.as_bytes();
    for i in start_index..bytes.len() {
        let c = bytes[i] as char;
        if c != ' ' && c != '\t' {
            return i;
        }
    }
    bytes.len()
}

pub fn to_long_or_default(s: &str, default_value: i64) -> i64 {
    s.parse::<i64>().unwrap_or(default_value)
}

pub fn to_non_negative_int(s: Option<&str>, default_value: i32) -> i32 {
    match s {
        None => default_value,
        Some(val) => match val.parse::<i64>() {
            Ok(value) => {
                if value > i32::MAX as i64 {
                    i32::MAX
                } else if value < 0 {
                    0
                } else {
                    value as i32
                }
            }
            Err(_) => default_value,
        },
    }
}

pub fn close_quietly(closeable: &mut dyn Closeable) {
    let _ = closeable.close();
}

pub fn is_civilized(fs: &dyn FileSystem, file: PathBuf) -> bool {
    match fs.sink(file.clone()) {
        Ok(sink) => {
            let res = fs.delete(file.clone());
            drop(sink);
            if res.is_ok() {
                return true;
            }
        }
        Err(_) => {}
    }
    let _ = fs.delete(file);
    false
}

pub fn delete_if_exists(fs: &dyn FileSystem, path: PathBuf) {
    let _ = fs.delete(path);
}

pub fn delete_contents(fs: &dyn FileSystem, directory: PathBuf) -> std::io::Result<()> {
    let files = match fs.list(directory) {
        Ok(f) => f,
        Err(_) => return Ok(()),
    };

    let mut last_exception = None;
    for file in files {
        let res = (|| {
            if let Ok(meta) = fs.metadata(file.clone()) {
                if meta.is_directory {
                    delete_contents(fs, file.clone())?;
                }
            }
            fs.delete(file)
        })();

        if let Err(e) = res {
            if last_exception.is_none() {
                last_exception = Some(e);
            }
        }
    }

    if let Some(e) = last_exception {
        return Err(e);
    }
    Ok(())
}

pub fn add_if_absent<E: PartialEq>(list: &mut Vec<E>, element: E) {
    if !list.contains(&element) {
        list.push(element);
    }
}

pub fn filter_list<T, P>(iterable: impl IntoIterator<Item = T>, predicate: P) -> Vec<T>
where
    P: Fn(&T) -> bool,
{
    iterable.into_iter().filter(predicate).collect()
}

pub fn check_offset_and_count(array_length: i64, offset: i64, count: i64) {
    if offset < 0 || count < 0 || offset > array_length || array_length - offset < count {
        panic!("length={}, offset={}, count={}", array_length, offset, count);
    }
}

pub fn interleave<T: Clone>(a: impl IntoIterator<Item = T>, b: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut ia = a.into_iter();
    let mut ib = b.into_iter();
    let mut result = Vec::new();

    loop {
        let a_next = ia.next();
        let b_next = ib.next();
        if a_next.is_none() && b_next.is_none() {
            break;
        }
        if let Some(val) = a_next {
            result.push(val);
        }
        if let Some(val) = b_next {
            result.push(val);
        }
    }
    result
}
