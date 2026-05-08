use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::sync::OnceLock;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::cache::FaultHidingSink::*;

// Mocking okio types as they are external dependencies in the original code
// In a real project, these would be imported from the `okio` crate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ByteString(pub Vec<u8>);

impl ByteString {
    pub fn new(bytes: Vec<u8>) -> Self {
        ByteString(bytes)
    }
    pub fn size(&self) -> usize {
        self.0.len()
    }
    pub fn utf8(&self) -> String {
        String::from_utf8_lossy(&self.0).into_owned()
    }
    pub fn to_ascii_lowercase(&self) -> ByteString {
        ByteString(self.0.iter().map(|b| b.to_ascii_lowercase()).collect())
    }
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.0.starts_with(prefix.as_bytes())
    }
}

impl std::ops::Index<usize> for ByteString {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}


impl Buffer {
    pub fn new() -> Self {
        Buffer { data: Vec::new() }
    }
    pub fn write_byte(&mut self, b: u8) {
        self.data.push(b);
    }
    pub fn write(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }
    pub fn read_byte_string(&mut self, length: u64) -> ByteString {
        let len = length as usize;
        let bytes = self.data.drain(0..len).collect();
        ByteString(bytes)
    }
    pub fn read_byte_string_all(&mut self) -> ByteString {
        ByteString(self.data.drain(..).collect())
    }
}

impl Read for Buffer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.data.is_empty() {
            return Ok(0);
        }
        let len = std::cmp::min(buf.len(), self.data.len());
        let drained: Vec<u8> = self.data.drain(0..len).collect();
        buf[..len].copy_from_slice(&drained);
        Ok(len)
    }
}

impl Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.data.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// Constants from Header companion object
pub const RESPONSE_STATUS: &str = ":status";
pub const TARGET_AUTHORITY: &str = ":authority";
pub const TARGET_METHOD: &str = ":method";
pub const TARGET_PATH: &str = ":path";
pub const TARGET_SCHEME: &str = ":scheme";
pub const PSEUDO_PREFIX: &str = ":";

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub name: ByteString,
    pub value: ByteString,
}

impl Header {
    pub fn new(name: &str, value: &str) -> Self {
        Header {
            name: ByteString(name.as_bytes().to_vec()),
            value: ByteString(value.as_bytes().to_vec()),
        }
    }

    pub fn from_bytes(name: ByteString, value: ByteString) -> Self {
        Header { name, value }
    }

    pub fn hpack_size(&self) -> usize {
        self.name.size() + self.value.size() + 32
    }
}

// Mocking Huffman for completeness as it's used in the source
pub struct Huffman;
impl Huffman {
    pub fn decode<R: Read>(source: &mut R, length: u64, sink: &mut Buffer) -> io::Result<()> {
        // Implementation omitted for brevity, but signature preserved
        Ok(())
    }
    pub fn encode(data: ByteString, sink: &mut Buffer) {
        // Implementation omitted
    }
    pub fn encoded_length(data: ByteString) -> usize {
        data.size() // Simplified
    }
}

const HEADER_LIMIT: u64 = 8192;
const PREFIX_4_BITS: i32 = 0x0f;
const PREFIX_5_BITS: i32 = 0x1f;
const PREFIX_6_BITS: i32 = 0x3f;
const PREFIX_7_BITS: i32 = 0x7f;
const SETTINGS_HEADER_TABLE_SIZE: i32 = 4096;
const SETTINGS_HEADER_TABLE_SIZE_LIMIT: i32 = 16384;

pub struct Hpack;

impl Hpack {
    pub fn static_header_table() -> &'static [Header] {
        static TABLE: OnceLock<Vec<Header>> = OnceLock::new();
        TABLE.get_or_init(|| {
            vec![
                Header::new(TARGET_AUTHORITY, ""),
                Header::new(TARGET_METHOD, "GET"),
                Header::new(TARGET_METHOD, "POST"),
                Header::new(TARGET_PATH, "/"),
                Header::new(TARGET_PATH, "/index.html"),
                Header::new(TARGET_SCHEME, "http"),
                Header::new(TARGET_SCHEME, "https"),
                Header::new(RESPONSE_STATUS, "200"),
                Header::new(RESPONSE_STATUS, "204"),
                Header::new(RESPONSE_STATUS, "206"),
                Header::new(RESPONSE_STATUS, "304"),
                Header::new(RESPONSE_STATUS, "400"),
                Header::new(RESPONSE_STATUS, "404"),
                Header::new(RESPONSE_STATUS, "500"),
                Header::new("accept-charset", ""),
                Header::new("accept-encoding", "gzip, deflate"),
                Header::new("accept-language", ""),
                Header::new("accept-ranges", ""),
                Header::new("accept", ""),
                Header::new("access-control-allow-origin", ""),
                Header::new("age", ""),
                Header::new("allow", ""),
                Header::new("authorization", ""),
                Header::new("cache-control", ""),
                Header::new("content-disposition", ""),
                Header::new("content-encoding", ""),
                Header::new("content-language", ""),
                Header::new("content-length", ""),
                Header::new("content-location", ""),
                Header::new("content-range", ""),
                Header::new("content-type", ""),
                Header::new("cookie", ""),
                Header::new("date", ""),
                Header::new("etag", ""),
                Header::new("expect", ""),
                Header::new("expires", ""),
                Header::new("from", ""),
                Header::new("host", ""),
                Header::new("if-match", ""),
                Header::new("if-modified-since", ""),
                Header::new("if-none-match", ""),
                Header::new("if-range", ""),
                Header::new("if-unmodified-since", ""),
                Header::new("last-modified", ""),
                Header::new("link", ""),
                Header::new("location", ""),
                Header::new("max-forwards", ""),
                Header::new("proxy-authenticate", ""),
                Header::new("proxy-authorization", ""),
                Header::new("range", ""),
                Header::new("referer", ""),
                Header::new("refresh", ""),
                Header::new("retry-after", ""),
                Header::new("server", ""),
                Header::new("set-cookie", ""),
                Header::new("strict-transport-security", ""),
                Header::new("transfer-encoding", ""),
                Header::new("user-agent", ""),
                Header::new("vary", ""),
                Header::new("via", ""),
                Header::new("www-authenticate", ""),
            ]
        })
    }

    pub fn name_to_first_index() -> HashMap<ByteString, usize> {
        let mut result = HashMap::new();
        let table = Self::static_header_table();
        for (i, header) in table.iter().enumerate() {
            result.entry(header.name.clone()).or_insert(i);
        }
        result
    }

    pub fn check_lowercase(name: ByteString) -> io::Result<ByteString> {
        for &b in &name.0 {
            if b >= b'A' && b <= b'Z' {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("PROTOCOL_ERROR response malformed: mixed case name: {}", name.utf8()),
                ));
            }
        }
        Ok(name)
    }

    pub struct Reader<S: Read> {
        source: Buffer,
        header_table_size_setting: i32,
        max_dynamic_table_byte_count: i32,
        header_list: Vec<Header>,
        header_list_byte_count: u64,
        pub dynamic_table: Vec<Option<Header>>,
        next_header_index: usize,
        pub header_count: usize,
        pub dynamic_table_byte_count: usize,
        _phantom: std::marker::PhantomData<S>,
    }

    impl<S: Read> Hpack::Reader<S> {
        pub fn new(source: S, header_table_size_setting: i32) -> Self {
            let mut buffer = Buffer::new();
            let _ = source.read_to_end(&mut buffer.data); // Simplified for translation
            let dynamic_table_size = 8;
            Hpack::Reader {
                source: buffer,
                header_table_size_setting,
                max_dynamic_table_byte_count: header_table_size_setting,
                header_list: Vec::new(),
                header_list_byte_count: 0,
                dynamic_table: vec![None; dynamic_table_size],
                next_header_index: dynamic_table_size - 1,
                header_count: 0,
                dynamic_table_byte_count: 0,
                _phantom: std::marker::PhantomData,
            }
        }

        pub fn get_and_reset_header_list(&mut self) -> Vec<Header> {
            let result = self.header_list.clone();
            self.header_list.clear();
            self.header_list_byte_count = 0;
            result
        }

        pub fn max_dynamic_table_byte_count(&self) -> i32 {
            self.max_dynamic_table_byte_count
        }

        fn adjust_dynamic_table_byte_count(&mut self) {
            if (self.max_dynamic_table_byte_count as usize) < self.dynamic_table_byte_count {
                if self.max_dynamic_table_byte_count == 0 {
                    self.clear_dynamic_table();
                } else {
                    self.evict_to_recover_bytes(self.dynamic_table_byte_count - self.max_dynamic_table_byte_count as usize);
                }
            }
        }

        fn clear_dynamic_table(&mut self) {
            for item in self.dynamic_table.iter_mut() {
                *item = None;
            }
            self.next_header_index = self.dynamic_table.len() - 1;
            self.header_count = 0;
            self.dynamic_table_byte_count = 0;
        }

        fn evict_to_recover_bytes(&mut self, mut bytes_to_recover: usize) -> usize {
            let mut entries_to_evict = 0;
            if bytes_to_recover > 0 {
                let mut j = self.dynamic_table.len() - 1;
                while j >= self.next_header_index && bytes_to_recover > 0 {
                    if let Some(to_evict) = &self.dynamic_table[j] {
                        let size = to_evict.hpack_size();
                        bytes_to_recover = bytes_to_recover.saturating_sub(size);
                        self.dynamic_table_byte_count -= size;
                        self.header_count -= 1;
                        entries_to_evict += 1;
                    }
                    j = j.saturating_sub(1);
                }
                
                if entries_to_evict > 0 {
                    let start = self.next_header_index + 1;
                    let len = self.header_count;
                    let dest = start + entries_to_evict;
                    if dest + len <= self.dynamic_table.len() {
                        for k in 0..len {
                            self.dynamic_table[dest + k] = self.dynamic_table[start + k].clone();
                        }
                    }
                    for k in start..dest {
                        self.dynamic_table[k] = None;
                    }
                    self.next_header_index += entries_to_evict;
                }
            }
            entries_to_evict
        }

        pub fn read_headers(&mut self) -> io::Result<()> {
            while !self.source.data.is_empty() {
                let b = self.source.data[0] as i32;
                self.source.data.remove(0);
                
                if b == 0x80 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "index == 0"));
                } else if (b & 0x80) == 0x80 {
                    let index = self.read_int(b, PREFIX_7_BITS);
                    self.read_indexed_header(index - 1)?;
                } else if b == 0x40 {
                    self.read_literal_header_with_incremental_indexing_new_name()?;
                } else if (b & 0x40) == 0x40 {
                    let index = self.read_int(b, PREFIX_6_BITS);
                    self.read_literal_header_with_incremental_indexing_indexed_name(index - 1)?;
                } else if (b & 0x20) == 0x20 {
                    let new_max = self.read_int(b, PREFIX_5_BITS);
                    if new_max < 0 || new_max > self.header_table_size_setting {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid dynamic table size update {}", new_max)));
                    }
                    self.max_dynamic_table_byte_count = new_max;
                    self.adjust_dynamic_table_byte_count();
                } else if b == 0x10 || b == 0 {
                    self.read_literal_header_without_indexing_new_name()?;
                } else {
                    let index = self.read_int(b, PREFIX_4_BITS);
                    self.read_literal_header_without_indexing_indexed_name(index - 1)?;
                }
            }
            Ok(())
        }

        fn read_indexed_header(&mut self, index: i32) -> io::Result<()> {
            if self.is_static_header(index) {
                let static_entry = &Hpack::static_header_table()[index as usize];
                self.add_header(static_entry.clone())?;
            } else {
                let dynamic_idx = self.dynamic_table_index(index - Hpack::static_header_table().len() as i32);
                if dynamic_idx >= self.dynamic_table.len() {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Header index too large {}", index + 1)));
                }
                if let Some(header) = self.dynamic_table[dynamic_idx].clone() {
                    self.add_header(header)?;
                } else {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Dynamic table entry missing"));
                }
            }
            Ok(())
        }

        fn dynamic_table_index(&self, index: i32) -> usize {
            self.next_header_index + 1 + index as usize
        }

        fn read_literal_header_without_indexing_indexed_name(&mut self, index: i32) -> io::Result<()> {
            let name = self.get_name(index)?;
            let value = self.read_byte_string()?;
            self.add_header(Header::from_bytes(name, value))?;
            Ok(())
        }

        fn read_literal_header_without_indexing_new_name(&mut self) -> io::Result<()> {
            let name = Hpack::check_lowercase(self.read_byte_string()?)?;
            let value = self.read_byte_string()?;
            self.add_header(Header::from_bytes(name, value))?;
            Ok(())
        }

        fn read_literal_header_with_incremental_indexing_indexed_name(&mut self, name_index: i32) -> io::Result<()> {
            let name = self.get_name(name_index)?;
            let value = self.read_byte_string()?;
            self.insert_into_dynamic_table(-1, Header::from_bytes(name, value))?;
            Ok(())
        }

        fn read_literal_header_with_incremental_indexing_new_name(&mut self) -> io::Result<()> {
            let name = Hpack::check_lowercase(self.read_byte_string()?)?;
            let value = self.read_byte_string()?;
            self.insert_into_dynamic_table(-1, Header::from_bytes(name, value))?;
            Ok(())
        }

        fn get_name(&self, index: i32) -> io::Result<ByteString> {
            if self.is_static_header(index) {
                Ok(Hpack::static_header_table()[index as usize].name.clone())
            } else {
                let dynamic_idx = self.dynamic_table_index(index - Hpack::static_header_table().len() as i32);
                if dynamic_idx >= self.dynamic_table.len() {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Header index too large {}", index + 1)));
                }
                self.dynamic_table[dynamic_idx].as_ref().map(|h| h.name.clone()).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Dynamic table entry missing"))
            }
        }

        fn is_static_header(&self, index: i32) -> bool {
            index >= 0 && (index as usize) < Hpack::static_header_table().len()
        }

        fn insert_into_dynamic_table(&mut self, index: i32, entry: Header) -> io::Result<()> {
            let mut idx = index;
            self.add_header(entry.clone())?;

            let mut delta = entry.hpack_size();
            if idx != -1 {
                if let Some(existing) = &self.dynamic_table[self.dynamic_table_index(idx)] {
                    delta = delta.saturating_sub(existing.hpack_size());
                }
            }

            if delta > self.max_dynamic_table_byte_count as usize {
                self.clear_dynamic_table();
                return Ok(());
            }

            let bytes_to_recover = self.dynamic_table_byte_count.saturating_add(delta).saturating_sub(self.max_dynamic_table_byte_count as usize);
            let entries_evicted = self.evict_to_recover_bytes(bytes_to_recover);

            if idx == -1 {
                if self.header_count + 1 > self.dynamic_table.len() {
                    let old_size = self.dynamic_table.len();
                    let mut doubled = vec![None; old_size * 2];
                    for k in 0..old_size {
                        doubled[old_size + k] = self.dynamic_table[k].clone();
                    }
                    self.next_header_index = old_size - 1;
                    self.dynamic_table = doubled;
                }
                idx = self.next_header_index as i32;
                self.next_header_index = self.next_header_index.saturating_sub(1);
                self.dynamic_table[idx as usize] = Some(entry);
                self.header_count += 1;
            } else {
                let actual_idx = idx as usize + self.dynamic_table_index(idx) + entries_to_evict;
                if actual_idx < self.dynamic_table.len() {
                    self.dynamic_table[actual_idx] = Some(entry);
                }
            }
            self.dynamic_table_byte_count += delta;
            Ok(())
        }

        pub fn read_int(&mut self, first_byte: i32, prefix_mask: i32) -> i32 {
            let prefix = first_byte & prefix_mask;
            if prefix < prefix_mask {
                return prefix;
            }

            let mut result: i64 = prefix_mask as i64;
            let mut shift = 0;
            let mut byte_count = 0;
            loop {
                if byte_count == 5 {
                    panic!("HPACK integer overflow");
                }
                if self.source.data.is_empty() {
                    panic!("Unexpected end of stream");
                }
                let b = self.source.data[0] as i32;
                self.source.data.remove(0);
                byte_count += 1;
                let increment = ((b & 0x7f) as i64) << shift;
                if increment > (i32::MAX as i64) - result {
                    panic!("HPACK integer overflow");
                }
                result += increment;
                if (b & 0x80) == 0 {
                    break;
                }
                shift += 7;
            }
            result as i32
        }

        pub fn read_byte_string(&mut self) -> io::Result<ByteString> {
            if self.source.data.is_empty() {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
            }
            let first_byte = self.source.data[0] as i32;
            self.source.data.remove(0);
            let huffman_decode = (first_byte & 0x80) == 0x80;
            let length = self.read_int(first_byte, PREFIX_7_BITS) as u64;

            if self.header_list_byte_count + length > HEADER_LIMIT {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("header byte count limit of {} exceeded", HEADER_LIMIT)));
            }

            if huffman_decode {
                let mut decode_buffer = Buffer::new();
                Huffman::decode(&mut self.source, length, &mut decode_buffer)?;
                Ok(decode_buffer.read_byte_string_all())
            } else {
                Ok(self.source.read_byte_string(length))
            }
        }

        fn add_header(&mut self, header: Header) -> io::Result<()> {
            let header_size = (header.name.size() + header.value.size()) as u64;
            let new_size = self.header_list_byte_count + header_size;
            if new_size > HEADER_LIMIT {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("header byte count limit of {} exceeded", HEADER_LIMIT)));
            }
            self.header_list_byte_count = new_size;
            self.header_list.push(header);
            Ok(())
        }
    }

    pub struct Writer {
        pub header_table_size_setting: i32,
        use_compression: bool,
        out: Buffer,
        smallest_header_table_size_setting: i32,
        emit_dynamic_table_size_update: bool,
        pub max_dynamic_table_byte_count: i32,
        pub dynamic_table: Vec<Option<Header>>,
        next_header_index: usize,
        pub header_count: usize,
        pub dynamic_table_byte_count: usize,
    }

    impl Hpack::Writer {
        pub fn new(header_table_size_setting: i32, use_compression: bool, out: Buffer) -> Self {
            let dynamic_table_size = 8;
            Hpack::Writer {
                header_table_size_setting,
                use_compression,
                out,
                smallest_header_table_size_setting: i32::MAX,
                emit_dynamic_table_size_update: false,
                max_dynamic_table_byte_count: header_table_size_setting,
                dynamic_table: vec![None; dynamic_table_size],
                next_header_index: dynamic_table_size - 1,
                header_count: 0,
                dynamic_table_byte_count: 0,
            }
        }

        fn clear_dynamic_table(&mut self) {
            for item in self.dynamic_table.iter_mut() {
                *item = None;
            }
            self.next_header_index = self.dynamic_table.len() - 1;
            self.header_count = 0;
            self.dynamic_table_byte_count = 0;
        }

        fn evict_to_recover_bytes(&mut self, mut bytes_to_recover: usize) -> usize {
            let mut entries_to_evict = 0;
            if bytes_to_recover > 0 {
                let mut j = self.dynamic_table.len() - 1;
                while j >= self.next_header_index && bytes_to_recover > 0 {
                    if let Some(header) = &self.dynamic_table[j] {
                        let size = header.hpack_size();
                        bytes_to_recover = bytes_to_recover.saturating_sub(size);
                        self.dynamic_table_byte_count -= size;
                        self.header_count -= 1;
                        entries_to_evict += 1;
                    }
                    j = j.saturating_sub(1);
                }
                if entries_to_evict > 0 {
                    let start = self.next_header_index + 1;
                    let len = self.header_count;
                    let dest = start + entries_to_evict;
                    if dest + len <= self.dynamic_table.len() {
                        for k in 0..len {
                            self.dynamic_table[dest + k] = self.dynamic_table[start + k].clone();
                        }
                    }
                    for k in start..dest {
                        self.dynamic_table[k] = None;
                    }
                    self.next_header_index += entries_to_evict;
                }
            }
            entries_to_evict
        }

        fn insert_into_dynamic_table(&mut self, entry: Header) {
            let delta = entry.hpack_size();
            if delta > self.max_dynamic_table_byte_count as usize {
                self.clear_dynamic_table();
                return;
            }
            let bytes_to_recover = self.dynamic_table_byte_count.saturating_add(delta).saturating_sub(self.max_dynamic_table_byte_count as usize);
            self.evict_to_recover_bytes(bytes_to_recover);

            if self.header_count + 1 > self.dynamic_table.len() {
                let old_size = self.dynamic_table.len();
                let mut doubled = vec![None; old_size * 2];
                for k in 0..old_size {
                    doubled[old_size + k] = self.dynamic_table[k].clone();
                }
                self.next_header_index = old_size - 1;
                self.dynamic_table = doubled;
            }
            let index = self.next_header_index;
            self.next_header_index = self.next_header_index.saturating_sub(1);
            self.dynamic_table[index] = Some(entry);
            self.header_count += 1;
            self.dynamic_table_byte_count += delta;
        }

        pub fn write_headers(&mut self, header_block: &[Header]) -> io::Result<()> {
            if self.emit_dynamic_table_size_update {
                if self.smallest_header_table_size_setting < self.max_dynamic_table_byte_count {
                    self.write_int(self.smallest_header_table_size_setting, PREFIX_5_BITS, 0x20);
                }
                self.emit_dynamic_table_size_update = false;
                self.smallest_header_table_size_setting = i32::MAX;
                self.write_int(self.max_dynamic_table_byte_count, PREFIX_5_BITS, 0x20);
            }

            let name_to_idx = Hpack::name_to_first_index();
            let static_table = Hpack::static_header_table();

            for header in header_block {
                let name = header.name.to_ascii_lowercase();
                let value = &header.value;
                let mut header_index = -1;
                let mut header_name_index = -1;

                if let Some(&static_idx) = name_to_idx.get(&name) {
                    header_name_index = (static_idx + 1) as i32;
                    if header_name_index >= 2 && header_name_index <= 7 {
                        if static_table[(header_name_index - 1) as usize].value == *value {
                            header_index = header_name_index;
                        } else if static_table[header_name_index as usize].value == *value {
                            header_index = header_name_index + 1;
                        }
                    }
                }

                if header_index == -1 {
                    for j in (self.next_header_index + 1)..self.dynamic_table.len() {
                        if let Some(entry) = &self.dynamic_table[j] {
                            if entry.name == name {
                                if entry.value == *value {
                                    header_index = (j - (self.next_header_index + 1) + static_table.len()) as i32;
                                    break;
                                } else if header_name_index == -1 {
                                    header_name_index = (j - (self.next_header_index + 1) + static_table.len()) as i32;
                                }
                            }
                        }
                    }
                }

                if header_index != -1 {
                    self.write_int(header_index, PREFIX_7_BITS, 0x80);
                } else if header_name_index == -1 {
                    self.out.write_byte(0x40);
                    self.write_byte_string(name)?;
                    self.write_byte_string(value.clone())?;
                    self.insert_into_dynamic_table(header.clone());
                } else if name.starts_with(PSEUDO_PREFIX) && TARGET_AUTHORITY != name.utf8() {
                    self.write_int(header_name_index, PREFIX_4_BITS, 0);
                    self.write_byte_string(value.clone())?;
                } else {
                    self.write_int(header_name_index, PREFIX_6_BITS, 0x40);
                    self.write_byte_string(value.clone())?;
                    self.insert_into_dynamic_table(header.clone());
                }
            }
            Ok(())
        }

        pub fn write_int(&mut self, mut value: i32, prefix_mask: i32, bits: i32) {
            if value < prefix_mask {
                self.out.write_byte((bits | value) as u8);
                return;
            }
            self.out.write_byte((bits | prefix_mask) as u8);
            value -= prefix_mask;
            while value >= 0x80 {
                let b = value & 0x7f;
                self.out.write_byte((b | 0x80) as u8);
                value = value >> 7;
            }
            self.out.write_byte(value as u8);
        }

        pub fn write_byte_string(&mut self, data: ByteString) -> io::Result<()> {
            if self.use_compression && Huffman::encoded_length(data.clone()) < data.size() {
                let mut huffman_buffer = Buffer::new();
                Huffman::encode(data.clone(), &mut huffman_buffer);
                let huffman_bytes = huffman_buffer.read_byte_string_all();
                self.write_int(huffman_bytes.size() as i32, PREFIX_7_BITS, 0x80);
                self.out.write(&huffman_bytes.0)?;
            } else {
                self.write_int(data.size() as i32, PREFIX_7_BITS, 0);
}
