use std::sync::OnceLock;

pub struct Http2;

impl Http2 {
    pub const CONNECTION_PREFACE: &'static [u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

    // The initial max frame size, applied independently writing to, or reading from the peer.
    pub const INITIAL_MAX_FRAME_SIZE: i32 = 0x4000; // 16384

    pub const TYPE_DATA: i32 = 0x0;
    pub const TYPE_HEADERS: i32 = 0x1;
    pub const TYPE_PRIORITY: i32 = 0x2;
    pub const TYPE_RST_STREAM: i32 = 0x3;
    pub const TYPE_SETTINGS: i32 = 0x4;
    pub const TYPE_PUSH_PROMISE: i32 = 0x5;
    pub const TYPE_PING: i32 = 0x6;
    pub const TYPE_GOAWAY: i32 = 0x7;
    pub const TYPE_WINDOW_UPDATE: i32 = 0x8;
    pub const TYPE_CONTINUATION: i32 = 0x9;

    pub const FLAG_NONE: i32 = 0x0;
    pub const FLAG_ACK: i32 = 0x1; // Used for settings and ping.
    pub const FLAG_END_STREAM: i32 = 0x1; // Used for headers and data.
    pub const FLAG_END_HEADERS: i32 = 0x4; // Used for headers and continuation.
    pub const FLAG_END_PUSH_PROMISE: i32 = 0x4;
    pub const FLAG_PADDED: i32 = 0x8; // Used for headers and data.
    pub const FLAG_PRIORITY: i32 = 0x20; // Used for headers.
    pub const FLAG_COMPRESSED: i32 = 0x20; // Used for data.

    // Lookup table for valid frame types.
    fn frame_names() -> &'static [&'static str] {
        static FRAME_NAMES: &[&str] = &[
            "DATA",
            "HEADERS",
            "PRIORITY",
            "RST_STREAM",
            "SETTINGS",
            "PUSH_PROMISE",
            "PING",
            "GOAWAY",
            "WINDOW_UPDATE",
            "CONTINUATION",
        ];
        FRAME_NAMES
    }

    // Lookup table for valid flags for DATA, HEADERS, CONTINUATION.
    fn flags_table() -> &'static [String] {
        static FLAGS: OnceLock<Vec<String>> = OnceLock::new();
        FLAGS.get_or_init(|| {
            let mut flags = vec![String::new(); 0x40];
            let mut binary = vec![String::new(); 256];
            for i in 0..256 {
                binary[i] = format!("{:08b}", i);
            }

            flags[Self::FLAG_NONE as usize] = "".to_string();
            flags[Self::FLAG_END_STREAM as usize] = "END_STREAM".to_string();

            let prefix_flags = [Self::FLAG_END_STREAM];

            flags[Self::FLAG_PADDED as usize] = "PADDED".to_string();
            for &prefix_flag in &prefix_flags {
                let combined = (prefix_flag | Self::FLAG_PADDED) as usize;
                flags[combined] = format!("{}|PADDED", flags[prefix_flag as usize]);
            }

            flags[Self::FLAG_END_HEADERS as usize] = "END_HEADERS".to_string();
            flags[Self::FLAG_PRIORITY as usize] = "PRIORITY".to_string();
            flags[(Self::FLAG_END_HEADERS | Self::FLAG_PRIORITY) as usize] = "END_HEADERS|PRIORITY".to_string();

            let frame_flags = [
                Self::FLAG_END_HEADERS,
                Self::FLAG_PRIORITY,
                Self::FLAG_END_HEADERS | Self::FLAG_PRIORITY,
            ];

            for &frame_flag in &frame_flags {
                for &prefix_flag in &prefix_flags {
                    let combined = (prefix_flag | frame_flag) as usize;
                    let combined_padded = (prefix_flag | frame_flag | Self::FLAG_PADDED) as usize;
                    
                    let base_flag_str = &flags[frame_flag as usize];
                    let prefix_flag_str = &flags[prefix_flag as usize];
                    
                    flags[combined] = format!("{}|{}", prefix_flag_str, base_flag_str);
                    flags[combined_padded] = format!("{}|{}|PADDED", prefix_flag_str, base_flag_str);
                }
            }

            for i in 0..flags.len() {
                if flags[i].is_empty() && i != Self::FLAG_NONE as usize {
                    flags[i] = binary[i].clone();
                }
            }
            flags
        })
    }

    fn binary_repr(val: i32) -> String {
        format!("{:08b}", val)
    }

    // Returns a human-readable representation of HTTP/2 frame headers.
    pub fn frame_log(
        inbound: bool,
        stream_id: i32,
        length: i32,
        frame_type: i32,
        flags: i32,
    ) -> String {
        let formatted_type = Self::formatted_type(frame_type);
        let formatted_flags = Self::format_flags(frame_type, flags);
        let direction = if inbound { "<<" } else { ">>" };
        format!(
            "{} 0x{:08x} {:5} {:-13} {}",
            direction, stream_id, length, formatted_type, formatted_flags
        )
    }

    // Returns a human-readable representation of a `WINDOW_UPDATE` frame.
    pub fn frame_log_window_update(
        inbound: bool,
        stream_id: i32,
        length: i32,
        window_size_increment: i64,
    ) -> String {
        let formatted_type = Self::formatted_type(Self::TYPE_WINDOW_UPDATE);
        let direction = if inbound { "<<" } else { ">>" };
        format!(
            "{} 0x{:08x} {:5} {:-13} {}",
            direction, stream_id, length, formatted_type, window_size_increment
        )
    }

    pub(crate) fn formatted_type(frame_type: i32) -> String {
        let names = Self::frame_names();
        if frame_type >= 0 && (frame_type as usize) < names.len() {
            names[frame_type as usize].to_string()
        } else {
            format!("0x{:02x}", frame_type)
        }
    }

    // Looks up valid string representing flags from the table.
    pub fn format_flags(frame_type: i32, flags: i32) -> String {
        if flags == 0 {
            return "".to_string();
        }
        match frame_type {
            Self::TYPE_SETTINGS | Self::TYPE_PING => {
                if flags == Self::FLAG_ACK {
                    return "ACK".to_string();
                } else {
                    return Self::binary_repr(flags);
                }
            }
            Self::TYPE_PRIORITY | Self::TYPE_RST_STREAM | Self::TYPE_GOAWAY | Self::TYPE_WINDOW_UPDATE => {
                return Self::binary_repr(flags);
            }
            _ => {}
        }

        let result = if flags >= 0 && (flags as usize) < Self::flags_table().len() {
            Self::flags_table()[flags as usize].clone()
        } else {
            Self::binary_repr(flags)
        };

        if frame_type == Self::TYPE_PUSH_PROMISE && (flags & Self::FLAG_END_PUSH_PROMISE) != 0 {
            result.replace("HEADERS", "PUSH_PROMISE")
        } else if frame_type == Self::TYPE_DATA && (flags & Self::FLAG_COMPRESSED) != 0 {
            result.replace("PRIORITY", "COMPRESSED")
        } else {
            result
        }
    }
}
]}
