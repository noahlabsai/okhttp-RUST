use okio::Buffer;
use sha1::{Digest, Sha1};
use base64::{engine::general_purpose, Engine as _};

/// WebSocketProtocol provides constants and utility functions for the WebSocket protocol.
pub struct WebSocketProtocol;

impl WebSocketProtocol {
    /// Magic value which must be appended to the key in a response header.
    pub const ACCEPT_MAGIC: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    /*
    Each frame starts with two bytes of data.

     0 1 2 3 4 5 6 7    0 1 2 3 4 5 6 7
    +-+-+-+-+-------+  +-+-------------+
    |F|R|R|R| OP    |  |M| LENGTH      |
    |I|S|S|S| CODE  |  |A|             |
    |N|V|V|V|       |  |S|             |
    | |1|2|3|       |  |K|             |
    +-+-+-+-+-------+  +-+-------------+
     */

    /// Byte 0 flag for whether this is the final fragment in a message.
    pub const B0_FLAG_FIN: i32 = 128;

    /// Byte 0 reserved flag 1. Must be 0 unless negotiated otherwise.
    pub const B0_FLAG_RSV1: i32 = 64;

    /// Byte 0 reserved flag 2. Must be 0 unless negotiated otherwise.
    pub const B0_FLAG_RSV2: i32 = 32;

    /// Byte 0 reserved flag 3. Must be 0 unless negotiated otherwise.
    pub const B0_FLAG_RSV3: i32 = 16;

    /// Byte 0 mask for the frame opcode.
    pub const B0_MASK_OPCODE: i32 = 15;

    /// Flag in the opcode which indicates a control frame.
    pub const OPCODE_FLAG_CONTROL: i32 = 8;

    ///
    /// Byte 1 flag for whether the payload data is masked.
    ///
    /// If this flag is set, the next four
    /// bytes represent the mask key. These bytes appear after any additional bytes specified by [B1_MASK_LENGTH].
    pub const B1_FLAG_MASK: i32 = 128;

    ///
    /// Byte 1 mask for the payload length.
    ///
    /// If this value is [PAYLOAD_SHORT], the next two
    /// bytes represent the length. If this value is [PAYLOAD_LONG], the next eight bytes
    /// represent the length.
    pub const B1_MASK_LENGTH: i32 = 127;

    pub const OPCODE_CONTINUATION: i32 = 0x0;
    pub const OPCODE_TEXT: i32 = 0x1;
    pub const OPCODE_BINARY: i32 = 0x2;

    pub const OPCODE_CONTROL_CLOSE: i32 = 0x8;
    pub const OPCODE_CONTROL_PING: i32 = 0x9;
    pub const OPCODE_CONTROL_PONG: i32 = 0xa;

    ///
    /// Maximum length of frame payload. Larger payloads, if supported by the frame type, can use the
    /// special values [PAYLOAD_SHORT] or [PAYLOAD_LONG].
    pub const PAYLOAD_BYTE_MAX: i64 = 125;

    /// Maximum length of close message in bytes.
    pub const CLOSE_MESSAGE_MAX: i64 = 125 - 2;

    ///
    /// Value for [B1_MASK_LENGTH] which indicates the next two bytes are the unsigned length.
    pub const PAYLOAD_SHORT: i32 = 126;

    /// Maximum length of a frame payload to be denoted as [PAYLOAD_SHORT].
    pub const PAYLOAD_SHORT_MAX: i64 = 0xffff;

    ///
    /// Value for [B1_MASK_LENGTH] which indicates the next eight bytes are the unsigned
    /// length.
    pub const PAYLOAD_LONG: i32 = 127;

    /// Used when an unchecked exception was thrown in a listener.
    pub const CLOSE_CLIENT_GOING_AWAY: i32 = 1001;

    /// Used when an empty close frame was received (i.e., without a status code).
    pub const CLOSE_NO_STATUS_CODE: i32 = 1005;

    /// Toggles the mask of the data in the buffer using the provided key.
    pub fn toggle_mask(
        cursor: &mut Buffer::UnsafeCursor,
        key: &[u8],
    ) {
        let mut key_index = 0;
        let key_length = key.len();
        if key_length == 0 {
            return;
        }
        loop {
            if let Some(buffer) = cursor.data() {
                let mut i = cursor.start();
                let end = cursor.end();
                while i < end {
                    key_index %= key_length;

                    // XOR the buffer byte with the key byte
                    buffer[i] ^= key[key_index];

                    i += 1;
                    key_index += 1;
                }
            }
            if cursor.next() == -1 {
                break;
            }
        }
    }

    /// Returns an error message if the close code is invalid.
    pub fn close_code_exception_message(code: i32) -> Option<String> {
        if code < 1000 || code >= 5000 {
            Some(format!("Code must be in range [1000,5000): {}", code))
        } else if (1004..=1006).contains(&code) || (1015..=2999).contains(&code) {
            Some(format!("Code {} is reserved and may not be used.", code))
        } else {
            None
        }
    }

    /// Validates the close code and panics if it is invalid.
    pub fn validate_close_code(code: i32) {
        if let Some(message) = Self::close_code_exception_message(code) {
            panic!("{}", message);
        }
    }

    /// Computes the WebSocket accept header from the provided key.
    pub fn accept_header(key: &str) -> String {
        let combined = format!("{}{}", key, Self::ACCEPT_MAGIC);
        let mut hasher = Sha1::new();
        hasher.update(combined.as_bytes());
        let hash = hasher.finalize();
        general_purpose::STANDARD.encode(hash)
    }
}