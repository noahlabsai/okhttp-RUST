use std::collections::HashMap;
use std::io::{self, Read, Write};
use okio::{Buffer, ByteString};
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http2::Http2Reader::*;

// Mocking the Hpack internal structures as they are dependencies of the test
// In a real project, these would be imported from crate::okhttp3::internal::http2::Hpack

impl Header {
    pub fn new(name: &str, value: &str, size: i32) -> Self {
        Self {
            name: ByteString::from(name),
            value: ByteString::from(value),
            hpack_size: size,
        }
    }
}

pub struct Hpack;

impl Hpack {
    pub struct Reader {
        source: Buffer,
        dynamic_table: Vec<Header>,
        max_table_size: i32,
        header_count: usize,
        buffered_headers: Vec<Header>,
    }

    impl Hpack::Reader {
        pub fn new(source: Buffer, max_table_size: i32) -> Self {
            Self {
                source,
                dynamic_table: Vec::new(),
                max_table_size,
                header_count: 0,
                buffered_headers: Vec::new(),
            }
        }

        pub fn read_headers(&mut self) -> io::Result<()> {
            // Implementation would be in the actual Hpack::Reader
            Ok(())
        }

        pub fn get_and_reset_header_list(&mut self) -> Vec<Header> {
            std::mem::take(&mut self.buffered_headers)
        }

        pub fn read_int(&mut self, prefix: i32, _max: i32) -> io::Result<i32> {
            // Implementation would be in the actual Hpack::Reader
            Ok(prefix)
        }

        pub fn read_byte_string(&mut self) -> io::Result<ByteString> {
            // Implementation would be in the actual Hpack::Reader
            Ok(ByteString::EMPTY)
        }

        pub fn max_dynamic_table_byte_count(&self) -> i32 {
            self.max_table_size
        }

        pub fn dynamic_table_byte_count(&self) -> i32 {
            self.dynamic_table.iter().map(|h| h.hpack_size).sum()
        }
    }

    pub struct Writer {
        sink: Buffer,
        dynamic_table: Vec<Header>,
        max_table_size: i32,
        header_table_size_setting: i32,
        use_huffman: bool,
        header_count: usize,
    }

    impl Hpack::Writer {
        pub fn new(max_table_size: i32, use_huffman: bool, sink: Buffer) -> Self {
            Self {
                sink,
                dynamic_table: Vec::new(),
                max_table_size,
                header_table_size_setting: max_table_size,
                use_huffman,
                header_count: 0,
            }
        }

        pub fn write_headers(&mut self, headers: Vec<Header>) -> io::Result<()> {
            // Implementation would be in the actual Hpack::Writer
            Ok(())
        }

        pub fn write_int(&mut self, value: i32, prefix: i32, mask: i32) -> io::Result<()> {
            // Implementation would be in the actual Hpack::Writer
            Ok(())
        }

        pub fn write_byte_string(&mut self, bs: ByteString) -> io::Result<()> {
            // Implementation would be in the actual Hpack::Writer
            Ok(())
        }

        pub fn resize_header_table(&mut self, size: i32) {
            self.header_table_size_setting = size.min(16384);
        }
    }
}

pub struct TestUtil;
impl TestUtil {
    pub fn header_entries(pairs: &[(&str, &str)]) -> Vec<Header> {
        pairs.iter()
            .map(|(n, v)| Header::new(n, v, (n.len() + v.len() + 32) as i32))
            .collect()
    }
}

pub struct HpackTest {
    bytes_in: Buffer,
    hpack_reader: Option<Hpack::Reader>,
    bytes_out: Buffer,
    hpack_writer: Option<Hpack::Writer>,
}

impl HpackTest {
    pub fn new() -> Self {
        Self {
            bytes_in: Buffer::new(),
            hpack_reader: None,
            bytes_out: Buffer::new(),
            hpack_writer: None,
        }
    }

    pub fn reset(&mut self) {
        let bytes_in = Buffer::new();
        let bytes_out = Buffer::new();
        self.hpack_reader = Some(Hpack::Reader::new(bytes_in.clone(), 4096));
        self.hpack_writer = Some(Hpack::Writer::new(4096, false, bytes_out.clone()));
        self.bytes_in = bytes_in;
        self.bytes_out = bytes_out;
    }

    pub fn large_header_value(&mut self) {
        let value = "!".repeat(4096);
        let header_block = TestUtil::header_entries(&[("cookie", &value)]);
        
        let writer = self.hpack_writer.as_mut().expect("Writer not initialized");
        writer.write_headers(header_block.clone()).unwrap();
        
        let mut temp_buf = Buffer::new();
        temp_buf.write_all(&self.bytes_out).unwrap();
        
        let reader = self.hpack_reader.as_mut().expect("Reader not initialized");
        reader.read_headers().unwrap();
        
        assert_eq!(reader.header_count, 0);
        assert_eq!(reader.get_and_reset_header_list(), header_block);
    }

    pub fn too_large_to_hpack_is_still_emitted(&mut self) {
        self.bytes_in.write_u8(0x21).unwrap();
        self.bytes_in.write_u8(0x00).unwrap();
        self.bytes_in.write_u8(0x0a).unwrap();
        self.bytes_in.write_all(b"custom-key").unwrap();
        self.bytes_in.write_u8(0x0d).unwrap();
        self.bytes_in.write_all(b"custom-header").unwrap();
        
        let reader = self.hpack_reader.as_mut().expect("Reader not initialized");
        reader.read_headers().unwrap();
        
        assert_eq!(reader.header_count, 0);
        assert_eq!(reader.get_and_reset_header_list(), TestUtil::header_entries(&[("custom-key", "custom-header")]));
    }

    pub fn writer_eviction(&mut self) {
        let header_block = TestUtil::header_entries(&[
            ("custom-foo", "custom-header"),
            ("custom-bar", "custom-header"),
            ("custom-baz", "custom-header"),
        ]);

        self.bytes_in.write_u8(0x40).unwrap();
        self.bytes_in.write_u8(0x0a).unwrap();
        self.bytes_in.write_all(b"custom-foo").unwrap();
        self.bytes_in.write_u8(0x0d).unwrap();
        self.bytes_in.write_all(b"custom-header").unwrap();
        
        self.bytes_in.write_u8(0x40).unwrap();
        self.bytes_in.write_u8(0x0a).unwrap();
        self.bytes_in.write_all(b"custom-bar").unwrap();
        self.bytes_in.write_u8(0x0d).unwrap();
        self.bytes_in.write_all(b"custom-header").unwrap();
        
        self.bytes_in.write_u8(0x40).unwrap();
        self.bytes_in.write_u8(0x0a).unwrap();
        self.bytes_in.write_all(b"custom-baz").unwrap();
        self.bytes_in.write_u8(0x0d).unwrap();
        self.bytes_in.write_all(b"custom-header").unwrap();

        let mut writer = Hpack::Writer::new(110, false, Buffer::new());
        writer.write_headers(header_block).unwrap();
        
        assert_eq!(writer.header_count, 2);
        let table_len = writer.dynamic_table.len();
        
        if let Some(entry) = writer.dynamic_table.get(table_len - 1) {
            self.check_entry(entry, "custom-bar", "custom-header", 55);
        }
        if let Some(entry) = writer.dynamic_table.get(table_len - 2) {
            self.check_entry(entry, "custom-baz", "custom-header", 55);
        }
    }

    pub fn reader_eviction(&mut self) {
        let header_block = TestUtil::header_entries(&[
            ("custom-foo", "custom-header"),
            ("custom-bar", "custom-header"),
            ("custom-baz", "custom-header"),
        ]);

        self.bytes_in.write_u8(0x3F).unwrap();
        self.bytes_in.write_u8(0x4F).unwrap();
        self.bytes_in.write_u8(0x40).unwrap();
        self.bytes_in.write_u8(0x0a).unwrap();
        self.bytes_in.write_all(b"custom-foo").unwrap();
        self.bytes_in.write_u8(0x0d).unwrap();
        self.bytes_in.write_all(b"custom-header").unwrap();
        
        self.bytes_in.write_u8(0x40).unwrap();
        self.bytes_in.write_u8(0x0a).unwrap();
        self.bytes_in.write_all(b"custom-bar").unwrap();
        self.bytes_in.write_u8(0x0d).unwrap();
        self.bytes_in.write_all(b"custom-header").unwrap();
        
        self.bytes_in.write_u8(0x40).unwrap();
        self.bytes_in.write_u8(0x0a).unwrap();
        self.bytes_in.write_all(b"custom-baz").unwrap();
        self.bytes_in.write_u8(0x0d).unwrap();
        self.bytes_in.write_all(b"custom-header").unwrap();

        let reader = self.hpack_reader.as_mut().expect("Reader not initialized");
        reader.read_headers().unwrap();
        
        assert_eq!(reader.header_count, 2);
        let table_len = reader.dynamic_table.len();
        if let Some(entry) = reader.dynamic_table.get(table_len - 1) {
            self.check_entry(entry, "custom-bar", "custom-header", 55);
        }
        if let Some(entry) = reader.dynamic_table.get(table_len - 2) {
            self.check_entry(entry, "custom-baz", "custom-header", 55);
        }

        assert_eq!(reader.get_and_reset_header_list(), header_block);

        self.bytes_in.write_u8(0x3F).unwrap();
        self.bytes_in.write_u8(0x18).unwrap();
        reader.read_headers().unwrap();
        assert_eq!(reader.header_count, 1);
    }

    pub fn dynamically_grows_beyond_64_entries(&mut self) {
        self.hpack_reader = Some(Hpack::Reader::new(self.bytes_in.clone(), 16384));
        self.bytes_in.write_u8(0x3F).unwrap();
        self.bytes_in.write_u8(0xE1).unwrap();
        self.bytes_in.write_u8(0x7F).unwrap();
        
        for _ in 0..=255 {
            self.bytes_in.write_u8(0x40).unwrap();
            self.bytes_in.write_u8(0x0a).unwrap();
            self.bytes_in.write_all(b"custom-foo").unwrap();
            self.bytes_in.write_u8(0x0d).unwrap();
            self.bytes_in.write_all(b"custom-header").unwrap();
        }
        
        let reader = self.hpack_reader.as_mut().unwrap();
        reader.read_headers().unwrap();
        assert_eq!(reader.header_count, 256);
    }

    pub fn huffman_decoding_supported(&mut self) {
        self.bytes_in.write_u8(0x44).unwrap();
        self.bytes_in.write_u8(0x8c).unwrap();
        self.bytes_in.write_all(&hex::decode("f1e3c2e5f23a6ba0ab90f4ff").unwrap()).unwrap();
        
        let reader = self.hpack_reader.as_mut().unwrap();
        reader.read_headers().unwrap();
        
        assert_eq!(reader.header_count, 1);
        assert_eq!(reader.dynamic_table_byte_count(), 52);
        let table_len = reader.dynamic_table.len();
        if let Some(entry) = reader.dynamic_table.get(table_len - 1) {
            self.check_entry(entry, ":path", "www.example.com", 52);
        }
    }

    pub fn read_literal_header_field_with_indexing(&mut self) {
        self.bytes_in.write_u8(0x40).unwrap();
        self.bytes_in.write_u8(0x0a).unwrap();
        self.bytes_in.write_all(b"custom-key").unwrap();
        self.bytes_in.write_u8(0x0d).unwrap();
        self.bytes_in.write_all(b"custom-header").unwrap();
        
        let reader = self.hpack_reader.as_mut().unwrap();
        reader.read_headers().unwrap();
        
        assert_eq!(reader.header_count, 1);
        assert_eq!(reader.dynamic_table_byte_count(), 55);
        let table_len = reader.dynamic_table.len();
        if let Some(entry) = reader.dynamic_table.get(table_len - 1) {
            self.check_entry(entry, "custom-key", "custom-header", 55);
        }
        assert_eq!(reader.get_and_reset_header_list(), TestUtil::header_entries(&[("custom-key", "custom-header")]));
    }

    pub fn literal_header_field_without_indexing_indexed_name(&mut self) {
        let header_block = TestUtil::header_entries(&[(":path", "/sample/path")]);
        self.bytes_in.write_u8(0x04).unwrap();
        self.bytes_in.write_u8(0x0c).unwrap();
        self.bytes_in.write_all(b"/sample/path").unwrap();
        
        let writer = self.hpack_writer.as_mut().unwrap();
        writer.write_headers(header_block.clone()).unwrap();
        
        let reader = self.hpack_reader.as_mut().unwrap();
        reader.read_headers().unwrap();
        
        assert_eq!(reader.header_count, 0);
        assert_eq!(reader.get_and_reset_header_list(), header_block);
    }

    pub fn literal_header_field_without_indexing_new_name(&mut self) {
        let header_block = TestUtil::header_entries(&[("custom-key", "custom-header")]);
        self.bytes_in.write_u8(0x00).unwrap();
        self.bytes_in.write_u8(0x0a).unwrap();
        self.bytes_in.write_all(b"custom-key").unwrap();
        self.bytes_in.write_u8(0x0d).unwrap();
        self.bytes_in.write_all(b"custom-header").unwrap();
        
        let reader = self.hpack_reader.as_mut().unwrap();
        reader.read_headers().unwrap();
        
        assert_eq!(reader.header_count, 0);
        assert_eq!(reader.get_and_reset_header_list(), header_block);
    }

    pub fn literal_header_field_never_indexed_indexed_name(&mut self) {
        self.bytes_in.write_u8(0x14).unwrap();
        self.bytes_in.write_u8(0x0c).unwrap();
        self.bytes_in.write_all(b"/sample/path").unwrap();
        
        let reader = self.hpack_reader.as_mut().unwrap();
        reader.read_headers().unwrap();
        
        assert_eq!(reader.header_count, 0);
        assert_eq!(reader.get_and_reset_header_list(), TestUtil::header_entries(&[(":path", "/sample/path")]));
    }

    pub fn literal_header_field_never_indexed_new_name(&mut self) {
        let header_block = TestUtil::header_entries(&[("custom-key", "custom-header")]);
        self.bytes_in.write_u8(0x10).unwrap();
        self.bytes_in.write_u8(0x0a).unwrap();
        self.bytes_in.write_all(b"custom-key").unwrap();
        self.bytes_in.write_u8(0x0d).unwrap();
        self.bytes_in.write_all(b"custom-header").unwrap();
        
        let reader = self.hpack_reader.as_mut().unwrap();
        reader.read_headers().unwrap();
        
        assert_eq!(reader.header_count, 0);
        assert_eq!(reader.get_and_reset_header_list(), header_block);
    }

    pub fn literal_header_field_with_incremental_indexing_indexed_name(&mut self) {
        let header_block = TestUtil::header_entries(&[(":path", "/sample/path")]);
        self.bytes_in.write_u8(0x44).unwrap();
        self.bytes_in.write_u8(0x0c).unwrap();
        self.bytes_in.write_all(b"/sample/path").unwrap();
        
        let reader = self.hpack_reader.as_mut().unwrap();
        reader.read_headers().unwrap();
        
        assert_eq!(reader.header_count, 1);
        assert_eq!(reader.get_and_reset_header_list(), header_block);
    }

    pub fn literal_header_field_with_incremental_indexing_new_name(&mut self) {
        let header_block = TestUtil::header_entries(&[("custom-key", "custom-header")]);
        self.bytes_in.write_u8(0x40).unwrap();
        self.bytes_in.write_u8(0x0a).unwrap();
        self.bytes_in.write_all(b"custom-key").unwrap();
        self.bytes_in.write_u8(0x0d).unwrap();
        self.bytes_in.write_all(b"custom-header").unwrap();
        
        let writer = self.hpack_writer.as_mut().unwrap();
        writer.write_headers(header_block.clone()).unwrap();
        
        assert_eq!(writer.header_count, 1);
        let table_len = writer.dynamic_table.len();
        if let Some(entry) = writer.dynamic_table.get(table_len - 1) {
            self.check_entry(entry, "custom-key", "custom-header", 55);
        }
        
        let reader = self.hpack_reader.as_mut().unwrap();
        reader.read_headers().unwrap();
        
        assert_eq!(reader.header_count, 1);
        assert_eq!(reader.get_and_reset_header_list(), header_block);
    }

    pub fn the_same_header_after_one_incremental_indexed(&mut self) {
        let header_block = TestUtil::header_entries(&[
            ("custom-key", "custom-header"),
            ("custom-key", "custom-header"),
        ]);
        self.bytes_in.write_u8(0x40).unwrap();
        self.bytes_in.write_u8(0x0a).unwrap();
        self.bytes_in.write_all(b"custom-key").unwrap();
        self.bytes_in.write_u8(0x0d).unwrap();
        self.bytes_in.write_all(b"custom-header").unwrap();
        self.bytes_in.write_u8(0xbe).unwrap();
        
        let writer = self.hpack_writer.as_mut().unwrap();
        writer.write_headers(header_block.clone()).unwrap();
        
        assert_eq!(writer.header_count, 1);
        let table_len = writer.dynamic_table.len();
        if let Some(entry) = writer.dynamic_table.get(table_len - 1) {
            self.check_entry(entry, "custom-key", "custom-header", 55);
        }
        
        let reader = self.hpack_reader.as_mut().unwrap();
        reader.read_headers().unwrap();
        
        assert_eq!(reader.header_count, 1);
        assert_eq!(reader.get_and_reset_header_list(), header_block);
    }

    pub fn static_header_is_not_copied_into_the_indexed_table(&mut self) {
        self.bytes_in.write_u8(0x82).unwrap();
        
        let reader = self.hpack_reader.as_mut().unwrap();
        reader.read_headers().unwrap();
        
        assert_eq!(reader.header_count, 0);
        assert_eq!(reader.dynamic_table_byte_count(), 0);
        let table_len = reader.dynamic_table.len();
        if table_len > 0 {
            panic!("Dynamic table should be empty");
        }
        assert_eq!(reader.get_and_reset_header_list(), TestUtil::header_entries(&[(":method", "GET")]));
    }

    pub fn read_indexed_header_field_index_0(&mut self) {
        self.bytes_in.write_u8(0x80).unwrap();
        let reader = self.hpack_reader.as_mut().unwrap();
        let result = reader.read_headers();
        assert!(result.is_err());
    }

    pub fn read_indexed_header_field_too_large_index(&mut self) {
        self.bytes_in.write_all(&[0xff, 0x00]).unwrap();
        let reader = self.hpack_reader.as_mut().unwrap();
        let result = reader.read_headers();
        assert!(result.is_err());
    }

    pub fn read_indexed_header_field_insidious_index(&mut self) {
        self.bytes_in.write_u8(0xff).unwrap();
        self.bytes_in.write_all(&hex::decode("8080808008").unwrap()).unwrap();
        let reader = self.hpack_reader.as_mut().unwrap();
        let result = reader.read_headers();
        assert!(result.is_err());
    }

    pub fn min_max_header_table_size(&mut self) {
        self.bytes_in.write_u8(0x20).unwrap();
        let reader = self.hpack_reader.as_mut().unwrap();
        reader.read_headers().unwrap();
        assert_eq!(reader.max_dynamic_table_byte_count(), 0);
        
        self.bytes_in.write_u8(0x3f).unwrap();
        self.bytes_in.write_u8(0xe1).unwrap();
        self.bytes_in.write_u8(0x1f).unwrap();
        reader.read_headers().unwrap();
        assert_eq!(reader.max_dynamic_table_byte_count(), 4096);
    }

    pub fn cannot_set_table_size_larger_than_settings_value(&mut self) {
        self.bytes_in.write_u8(0x3f).unwrap();
        self.bytes_in.write_u8(0xe2).unwrap();
        self.bytes_in.write_u8(0x1f).unwrap();
        let reader = self.hpack_reader.as_mut().unwrap();
        let result = reader.read_headers();
        assert!(result.is_err());
    }

    pub fn read_header_table_state_change_insidious_max_header_byte_count(&mut self) {
        self.bytes_in.write_u8(0x3f).unwrap();
        self.bytes_in.write_all(&hex::decode("e1ffffff07").unwrap()).unwrap();
        let reader = self.hpack_reader.as_mut().unwrap();
        let result = reader.read_headers();
        assert!(result.is_err());
    }

    pub fn dynamic_table_size_update_rejects_wrapped_small_positive(&mut self) {
        self.bytes_in.write_u8(0x3f).unwrap();
        self.bytes_in.write_all(&hex::decode("ffffffff8f00").unwrap()).unwrap();
        let reader = self.hpack_reader.as_mut().unwrap();
        let result = reader.read_headers();
        assert!(result.is_err());
    }

    pub fn read_indexed_header_field_from_static_table_without_buffering(&mut self) {
        self.bytes_in.write_u8(0x20).unwrap();
        self.bytes_in.write_u8(0x82).unwrap();
        
        let reader = self.hpack_reader.as_mut().unwrap();
        reader.read_headers().unwrap();
        
        assert_eq!(reader.header_count, 0);
        assert_eq!(reader.get_and_reset_header_list(), TestUtil::header_entries(&[(":method", "GET")]));
    }

    pub fn read_literal_header_with_incremental_indexing_static_name(&mut self) {
        self.bytes_in.write_u8(0x7d).unwrap();
        self.bytes_in.write_u8(0x05).unwrap();
        self.bytes_in.write_all(b"Basic").unwrap();
        
        let reader = self.hpack_reader.as_mut().unwrap();
        reader.read_headers().unwrap();
        
        let list = reader.get_and_reset_header_list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, ByteString::from("www-authenticate"));
        assert_eq!(list[0].value, ByteString::from("Basic"));
    }

    pub fn read_literal_header_with_incremental_indexing_dynamic_name(&mut self) {
        self.bytes_in.write_u8(0x40).unwrap();
        self.bytes_in.write_u8(0x0a).unwrap();
        self.bytes_in.write_all(b"custom-foo").unwrap();
        self.bytes_in.write_u8(0x05).unwrap();
        self.bytes_in.write_all(b"Basic").unwrap();
        self.bytes_in.write_u8(0x7e).unwrap();
        self.bytes_in.write_u8(0x06).unwrap();
        self.bytes_in.write_all(b"Basic2").unwrap();
        
        let reader = self.hpack_reader.as_mut().unwrap();
        reader.read_headers().unwrap();
        
        let list = reader.get_and_reset_header_list();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, ByteString::from("custom-foo"));
        assert_eq!(list[0].value, ByteString::from("Basic"));
        assert_eq!(list[1].name, ByteString::from("custom-foo"));
        assert_eq!(list[1].value, ByteString::from("Basic2"));
    }

    pub fn read_request_examples_without_huffman(&mut self) {
        self.first_request_without_huffman();
        self.hpack_reader.as_mut().unwrap().read_headers().unwrap();
        self.check_read_first_request_without_huffman();
        
        self.second_request_without_huffman();
        self.hpack_reader.as_mut().unwrap().read_headers().unwrap();
        self.check_read_second_request_without_huffman();
        
        self.third_request_without_huffman();
        self.hpack_reader.as_mut().unwrap().read_headers().unwrap();
        self.check_read_third_request_without_huffman();
    }

    fn first_request_without_huffman(&mut self) {
        self.bytes_in.write_u8(0x82).unwrap();
        self.bytes_in.write_u8(0x86).unwrap();
        self.bytes_in.write_u8(0x84).unwrap();
        self.bytes_in.write_u8(0x41).unwrap();
        self.bytes_in.write_u8(0x0f).unwrap();
        self.bytes_in.write_all(b"www.example.com").unwrap();
    }

    fn check_read_first_request_without_huffman(&mut self) {
        let reader = self.hpack_reader.as_mut().unwrap();
        assert_eq!(reader.header_count, 1);
        let table_len = reader.dynamic_table.len();
        if let Some(entry) = reader.dynamic_table.get(table_len - 1) {
            self.check_entry(entry, ":authority", "www.example.com", 57);
        }
        assert_eq!(reader.dynamic_table_byte_count(), 57);
        assert_eq!(reader.get_and_reset_header_list(), TestUtil::header_entries(&[
            (":method", "GET"),
            (":scheme", "http"),
            (":path", "/"),
            (":authority", "www.example.com"),
        ]));
    }

    fn second_request_without_huffman(&mut self) {
        self.bytes_in.write_u8(0x82).unwrap();
        self.bytes_in.write_u8(0x86).unwrap();
        self.bytes_in.write_u8(0x84).unwrap();
        self.bytes_in.write_u8(0xbe).unwrap();
        self.bytes_in.write_u8(0x58).unwrap();
        self.bytes_in.write_u8(0x08).unwrap();
        self.bytes_in.write_all(b"no-cache").unwrap();
    }

    fn check_read_second_request_without_huffman(&mut self) {
        let reader = self.hpack_reader.as_mut().unwrap();
        assert_eq!(reader.header_count, 2);
        let table_len = reader.dynamic_table.len();
        if let Some(entry) = reader.dynamic_table.get(table_len - 2) {
            self.check_entry(entry, "cache-control", "no-cache", 53);
        }
        if let Some(entry) = reader.dynamic_table.get(table_len - 1) {
            self.check_entry(entry, ":authority", "www.example.com", 57);
        }
        assert_eq!(reader.dynamic_table_byte_count(), 110);
        assert_eq!(reader.get_and_reset_header_list(), TestUtil::header_entries(&[
            (":method", "GET"),
            (":scheme", "http"),
            (":path", "/"),
            (":authority", "www.example.com"),
            ("cache-control", "no-cache"),
        ]));
    }

    fn third_request_without_huffman(&mut self) {
        self.bytes_in.write_u8(0x82).unwrap();
        self.bytes_in.write_u8(0x87).unwrap();
        self.bytes_in.write_u8(0x85).unwrap();
        self.bytes_in.
}}
