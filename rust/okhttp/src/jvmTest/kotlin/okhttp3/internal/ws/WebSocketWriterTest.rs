use okio::{Buffer, ByteString};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::cmp::min;

// Mocking the WebSocketProtocol constants as they are dependencies in the original source
pub mod web_socket_protocol {
    pub const OPCODE_BINARY: i8 = 2;
    pub const OPCODE_TEXT: i8 = 1;
}
use web_socket_protocol::*;

// Corrected imports based on the project structure and provided warnings
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::ws::WebSocketWriter;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::ws::WebSocketWriter::*;

// The target.rs previously attempted to implement WebSocketWriter inside the test file.
// According to the review focus, we should fix stubs and placeholders.
// Since WebSocketWriter is a dependency, we assume it is defined in the imported module.
// However, to make the test file self-contained and correct the "missing WebSocketWriter" warning,
// we must ensure the imports are correct and the test logic matches the Kotlin source.

pub struct WebSocketWriterTest {
    data: Buffer,
    random: ChaCha8Rng,
}

impl WebSocketWriterTest {
    pub fn new() -> Self {
        Self {
            data: Buffer::new(),
            random: ChaCha8Rng::seed_from_u64(0),
        }
    }

    fn server_writer(&mut self) -> WebSocketWriter {
        WebSocketWriter::new(
            false,
            self.data.clone(),
            self.random.clone(),
            false,
            false,
            0,
        )
    }

    fn client_writer(&mut self) -> WebSocketWriter {
        WebSocketWriter::new(
            true,
            self.data.clone(),
            self.random.clone(),
            false,
            false,
            0,
        )
    }

    fn assert_data_hex(&mut self, hex: &str) {
        let expected = ByteString::decode_hex(hex).expect("Invalid hex string");
        self.assert_data(expected);
    }

    fn assert_data(&mut self, expected: ByteString) {
        let size = min(expected.len() as i64, self.data.size());
        let actual = self.data.read_byte_string(size);
        assert_eq!(actual, expected, "Data mismatch");
    }

    fn binary_data(length: usize) -> ByteString {
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        let mut junk = vec![0u8; length];
        rng.fill(&mut junk[..]);
        ByteString::from(junk)
    }

    pub fn server_text_message(&mut self) {
        let mut writer = self.server_writer();
        writer.write_message_frame(OPCODE_TEXT, ByteString::from("Hello"));
        self.assert_data_hex("810548656c6c6f");
    }

    pub fn server_compressed_text_message(&mut self) {
        let mut writer = WebSocketWriter::new(
            false,
            self.data.clone(),
            self.random.clone(),
            true,
            false,
            0,
        );
        writer.write_message_frame(OPCODE_TEXT, ByteString::from("Hello"));
        self.assert_data_hex("c107f248cdc9c90700");
    }

    pub fn server_small_buffered_payload_written_as_one_frame(&mut self) {
        let length = 5;
        let payload = Self::binary_data(length);
        let mut writer = self.server_writer();
        writer.write_message_frame(OPCODE_TEXT, payload.clone());
        self.assert_data_hex("8105");
        self.assert_data(payload);
    }

    pub fn server_large_buffered_payload_written_as_one_frame(&mut self) {
        let length = 12345;
        let payload = Self::binary_data(length);
        let mut writer = self.server_writer();
        writer.write_message_frame(OPCODE_TEXT, payload.clone());
        self.assert_data_hex("817e");
        self.assert_data_hex(&format!("{:04x}", length));
        self.assert_data(payload);
    }

    pub fn client_text_message(&mut self) {
        let mut writer = self.client_writer();
        writer.write_message_frame(OPCODE_TEXT, ByteString::from("Hello"));
        self.assert_data_hex("818560b420bb28d14cd70f");
    }

    pub fn client_compressed_text_message(&mut self) {
        let mut writer = WebSocketWriter::new(
            false, // Preserving Kotlin source: clientCompressedTextMessage uses false for isClient
            self.data.clone(),
            self.random.clone(),
            true,
            false,
            0,
        );
        writer.write_message_frame(OPCODE_TEXT, ByteString::from("Hello"));
        self.assert_data_hex("c107f248cdc9c90700");
    }

    pub fn server_binary_message(&mut self) {
        let payload = ByteString::decode_hex(
            "60b420bb3851d9d47acb933dbe70399bf6c92da33af01d4fb770e98c0325f41d3ebaf8986da712c82bcd4d554bf0b54023c2"
        ).unwrap();
        let mut writer = self.server_writer();
        writer.write_message_frame(OPCODE_BINARY, payload.clone());
        self.assert_data_hex("8232");
        self.assert_data(payload);
    }

    pub fn server_message_length_short(&mut self) {
        let mut payload_buf = Buffer::new();
        while payload_buf.size() <= PAYLOAD_BYTE_MAX {
            payload_buf.write_u8(b'0');
        }
        let payload = payload_buf.snapshot();
        let mut writer = self.server_writer();
        writer.write_message_frame(OPCODE_BINARY, payload.clone());

        self.assert_data_hex("827e");
        self.assert_data_hex(&format!("{:04X}", payload_buf.size()));
        self.assert_data(payload);
    }

    pub fn server_message_length_long(&mut self) {
        let mut payload_buf = Buffer::new();
        while payload_buf.size() <= PAYLOAD_SHORT_MAX {
            payload_buf.write_u8(b'0');
        }
        let payload = payload_buf.snapshot();
        let mut writer = self.server_writer();
        writer.write_message_frame(OPCODE_BINARY, payload.clone());

        self.assert_data_hex("827f");
        self.assert_data_hex(&format!("{:016X}", payload_buf.size()));
        self.assert_data(payload);
    }

    pub fn client_binary(&mut self) {
        let payload = ByteString::decode_hex(
            "60b420bb3851d9d47acb933dbe70399bf6c92da33af01d4fb770e98c0325f41d3ebaf8986da712c82bcd4d554bf0b54023c2"
        ).unwrap();
        let mut writer = self.client_writer();
        writer.write_message_frame(OPCODE_BINARY, payload);
        self.assert_data_hex("82b2");
        self.assert_data_hex("60b420bb");
        self.assert_data_hex("0000000058e5f96f1a7fb386dec41920967d0d185a443df4d7c4c9376391d4a65e0ed8230d1332734b796dee2b4495fb4376");
    }

    pub fn server_empty_close(&mut self) {
        let mut writer = self.server_writer();
        writer.write_close(0, None).unwrap();
        self.assert_data_hex("8800");
    }

    pub fn server_close_with_code(&mut self) {
        let mut writer = self.server_writer();
        writer.write_close(1001, None).unwrap();
        self.assert_data_hex("880203e9");
    }

    pub fn server_close_with_code_and_reason(&mut self) {
        let mut writer = self.server_writer();
        writer.write_close(1001, Some(ByteString::from("Hello"))).unwrap();
        self.assert_data_hex("880703e948656c6c6f");
    }

    pub fn client_empty_close(&mut self) {
        let mut writer = self.client_writer();
        writer.write_close(0, None).unwrap();
        self.assert_data_hex("888060b420bb");
    }

    pub fn client_close_with_code(&mut self) {
        let mut writer = self.client_writer();
        writer.write_close(1001, None).unwrap();
        self.assert_data_hex("888260b420bb635d");
    }

    pub fn client_close_with_code_and_reason(&mut self) {
        let mut writer = self.client_writer();
        writer.write_close(1001, Some(ByteString::from("Hello"))).unwrap();
        self.assert_data_hex("888760b420bb635d68de0cd84f");
    }

    pub fn close_with_only_reason_throws(&mut self) {
        let mut writer = self.client_writer();
        writer.write_close(0, Some(ByteString::from("Hello"))).unwrap();
        self.assert_data_hex("888760b420bb60b468de0cd84f");
    }

    pub fn close_code_out_of_range_throws(&mut self) {
        let mut writer = self.client_writer();
        let result = writer.write_close(98724976, Some(ByteString::from("Hello")));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Code must be in range [1000,5000): 98724976");
    }

    pub fn close_reserved_throws(&mut self) {
        let mut writer = self.client_writer();
        let result = writer.write_close(1005, Some(ByteString::from("Hello")));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Code 1005 is reserved and may not be used.");
    }

    pub fn server_empty_ping(&mut self) {
        let mut writer = self.server_writer();
        writer.write_ping(ByteString::new()).unwrap();
        self.assert_data_hex("8900");
    }

    pub fn client_empty_ping(&mut self) {
        let mut writer = self.client_writer();
        writer.write_ping(ByteString::new()).unwrap();
        self.assert_data_hex("898060b420bb");
    }

    pub fn server_ping_with_payload(&mut self) {
        let mut writer = self.server_writer();
        writer.write_ping(ByteString::from("Hello")).unwrap();
        self.assert_data_hex("890548656c6c6f");
    }

    pub fn client_ping_with_payload(&mut self) {
        let mut writer = self.client_writer();
        writer.write_ping(ByteString::from("Hello")).unwrap();
        self.assert_data_hex("898560b420bb28d14cd70f");
    }

    pub fn server_empty_pong(&mut self) {
        let mut writer = self.server_writer();
        writer.write_pong(ByteString::new()).unwrap();
        self.assert_data_hex("8a00");
    }

    pub fn client_empty_pong(&mut self) {
        let mut writer = self.client_writer();
        writer.write_pong(ByteString::new()).unwrap();
        self.assert_data_hex("8a8060b420bb");
    }

    pub fn server_pong_with_payload(&mut self) {
        let mut writer = self.server_writer();
        writer.write_pong(ByteString::from("Hello")).unwrap();
        self.assert_data_hex("8a0548656c6c6f");
    }

    pub fn client_pong_with_payload(&mut self) {
        let mut writer = self.client_writer();
        writer.write_pong(ByteString::from("Hello")).unwrap();
        self.assert_data_hex("8a8560b420bb28d14cd70f");
    }

    pub fn ping_too_long_throws(&mut self) {
        let mut writer = self.server_writer();
        let result = writer.write_ping(Self::binary_data(1000));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Payload size must be less than or equal to 125");
    }

    pub fn pong_too_long_throws(&mut self) {
        let mut writer = self.server_writer();
        let result = writer.write_pong(Self::binary_data(1000));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Payload size must be less than or equal to 125");
    }

    pub fn close_too_long_throws(&mut self) {
        let mut writer = self.server_writer();
        let long_reason = ByteString::from(vec![b'X'; 126]); // 126 > 125
        let result = writer.write_close(1000, Some(long_reason));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Payload size must be less than or equal to 125");
    }
}
