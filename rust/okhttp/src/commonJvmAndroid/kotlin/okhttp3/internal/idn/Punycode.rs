use std::sync::OnceLock;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::cache::FaultHidingSink::*;

// A simple Buffer implementation to mimic okio.Buffer for the purpose of this translation.

impl Buffer {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.data.push(byte);
    }

    pub fn write(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub fn write_utf8(&mut self, s: &str, start: usize, end: usize) {
        let slice = &s[start..end];
        self.data.extend_from_slice(slice.as_bytes());
    }

    pub fn write_utf8_code_point(&mut self, code_point: i32) {
        let cp = std::char::from_u32(code_point as u32).unwrap_or('?');
        let mut buf = [0u8; 4];
        self.data.extend_from_slice(cp.encode_utf8(&mut buf).as_bytes());
    }

    pub fn read_utf8(&mut self) -> String {
        let s = String::from_utf8_lossy(&self.data).into_owned();
        self.data.clear();
        s
    }
}

// An [RFC 3492] punycode decoder for converting ASCII to Unicode domain name labels.
pub struct Punycode;

impl Punycode {
    pub const PREFIX_STRING: &'static str = "xn--";
    
    pub fn prefix() -> &'static [u8] {
        pub static PREFIX: OnceLock<Vec<u8>> = OnceLock::new();
        PREFIX.get_or_init(|| Self::PREFIX_STRING.as_bytes().to_vec()).as_slice()
    }

    const BASE: i32 = 36;
    const TMIN: i32 = 1;
    const TMAX: i32 = 26;
    const SKEW: i32 = 38;
    const DAMP: i32 = 700;
    const INITIAL_BIAS: i32 = 72;
    const INITIAL_N: i32 = 0x80;

    pub fn encode(string: &str) -> Option<String> {
        let mut pos = 0;
        let limit = string.len();
        let mut result = Buffer::new();

        while pos < limit {
            let dot = string[pos..].find('.').map(|i| i + pos).unwrap_or(limit);

            if !Self::encode_label(string, pos, dot, &mut result) {
                return None;
            }

            if dot < limit {
                result.write_byte(b'.');
                pos = dot + 1;
            } else {
                break;
            }
        }

        Some(result.read_utf8())
    }

    fn encode_label(string: &str, pos: usize, limit: usize, result: &mut Buffer) -> bool {
        if !Self::requires_encode(string, pos, limit) {
            result.write_utf8(string, pos, limit);
            return true;
        }

        result.write(Self::prefix());

        let input = Self::code_points(string, pos, limit);

        let mut b = 0;
        for &code_point in &input {
            if code_point < Self::INITIAL_N {
                result.write_byte(code_point as u8);
                b += 1;
            }
        }

        if b > 0 {
            result.write_byte(b'-');
        }

        let mut n = Self::INITIAL_N;
        let mut delta = 0;
        let mut bias = Self::INITIAL_BIAS;
        let mut h = b;

        while (h as usize) < input.len() {
            let m = *input.iter()
                .filter(|&&cp| cp >= n)
                .min()
                .unwrap_or(&i32::MAX);

            let increment = (m - n) * (h + 1);
            if delta > i32::MAX - increment {
                return false;
            }
            delta += increment;
            n = m;

            for &c in &input {
                if c < n {
                    if delta == i32::MAX {
                        return false;
                    }
                    delta += 1;
                } else if c == n {
                    let mut q = delta;
                    let mut k = Self::BASE;
                    while k < i32::MAX {
                        let t = if k <= bias {
                            Self::TMIN
                        } else if k >= bias + Self::TMAX {
                            Self::TMAX
                        } else {
                            k - bias
                        };
                        if q < t {
                            break;
                        }
                        result.write_byte(Self::punycode_digit(t + ((q - t) % (Self::BASE - t))));
                        q = (q - t) / (Self::BASE - t);
                        k += Self::BASE;
                    }

                    result.write_byte(Self::punycode_digit(q));
                    bias = Self::adapt(delta, h + 1, h == b);
                    delta = 0;
                    h += 1;
                }
            }
            delta += 1;
            n += 1;
        }

        true
    }

    pub fn decode(string: &str) -> Option<String> {
        let mut pos = 0;
        let limit = string.len();
        let mut result = Buffer::new();

        while pos < limit {
            let dot = string[pos..].find('.').map(|i| i + pos).unwrap_or(limit);

            if !Self::decode_label(string, pos, dot, &mut result) {
                return None;
            }

            if dot < limit {
                result.write_byte(b'.');
                pos = dot + 1;
            } else {
                break;
            }
        }

        Some(result.read_utf8())
    }

    fn decode_label(string: &str, pos: usize, limit: usize, result: &mut Buffer) -> bool {
        if string.len() < pos + 4 || !string[pos..].to_lowercase().starts_with(Self::PREFIX_STRING) {
            result.write_utf8(string, pos, limit);
            return true;
        }

        let mut pos = pos + 4;
        let mut code_points = Vec::new();

        let last_delimiter = string[pos..limit].rfind('-').map(|i| i + pos);
        if let Some(last_del) = last_delimiter {
            if last_del >= pos {
                while pos < last_del {
                    let c = string.as_bytes()[pos];
                    if (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || (c >= b'0' && c <= b'9') || c == b'-' {
                        code_points.push(c as i32);
                    } else {
                        return false;
                    }
                    pos += 1;
                }
                pos += 1; // Consume '-'
            }
        }

        let mut n = Self::INITIAL_N;
        let mut i = 0;
        let mut bias = Self::INITIAL_BIAS;

        while pos < limit {
            let old_i = i;
            let mut w = 1;
            let mut k = Self::BASE;
            while k < i32::MAX {
                if pos == limit {
                    return false;
                }
                let c = string.as_bytes()[pos];
                pos += 1;
                let digit = if c >= b'a' && c <= b'z' {
                    (c - b'a') as i32
                } else if c >= b'A' && c <= b'Z' {
                    (c - b'A') as i32
                } else if c >= b'0' && c <= b'9' {
                    (c - b'0') as i32 + 26
                } else {
                    return false;
                };

                let delta_i = digit * w;
                if i > i32::MAX - delta_i {
                    return false;
                }
                i += delta_i;

                let t = if k <= bias {
                    Self::TMIN
                } else if k >= bias + Self::TMAX {
                    Self::TMAX
                } else {
                    k - bias
                };

                if digit < t {
                    break;
                }
                let scale_w = Self::BASE - t;
                if w > i32::MAX / scale_w {
                    return false;
                }
                w *= scale_w;
                k += Self::BASE;
            }

            bias = Self::adapt(i - old_i, code_points.len() as i32 + 1, old_i == 0);
            let delta_n = i / (code_points.len() as i32 + 1);
            if n > i32::MAX - delta_n {
                return false;
            }
            n += delta_n;
            i %= code_points.len() as i32 + 1;

            if n > 0x10ffff {
                return false;
            }

            code_points.insert(i as usize, n);
            i += 1;
        }

        for cp in code_points {
            result.write_utf8_code_point(cp);
        }

        true
    }

    fn adapt(mut delta: i32, numpoints: i32, first: bool) -> i32 {
        delta = if first {
            delta / Self::DAMP
        } else {
            delta / 2
        };
        delta += delta / numpoints;
        let mut k = 0;
        while delta > ((Self::BASE - Self::TMIN) * Self::TMAX) / 2 {
            delta /= Self::BASE - Self::TMIN;
            k += Self::BASE;
        }
        k + (((Self::BASE - Self::TMIN + 1) * delta) / (delta + Self::SKEW))
    }

    fn requires_encode(string: &str, pos: usize, limit: usize) -> bool {
        for c in string[pos..limit].chars() {
            if (c as i32) >= Self::INITIAL_N {
                return true;
            }
        }
        false
    }

    fn code_points(string: &str, pos: usize, limit: usize) -> Vec<i32> {
        let mut result = Vec::new();
        let chars: Vec<char> = string[pos..limit].chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if (c as u32) >= 0xD800 && (c as u32) <= 0xDFFF {
                let low = if i + 1 < chars.len() {
                    chars[i + 1]
                } else {
                    '\u{0000}'
                };
                
                let is_low_surrogate = |ch: char| (ch as u32) >= 0xDC00 && (ch as u32) <= 0xDFFF;
                let is_high_surrogate = |ch: char| (ch as u32) >= 0xD800 && (ch as u32) <= 0xDBFF;

                if is_low_surrogate(c) || !is_low_surrogate(low) {
                    result.push('?'.encode_utf8_code_point()); // Helper logic below
                } else {
                    i += 1;
                    let cp = 0x010000 + (((c as u32 & 0x03FF) << 10) | (low as u32 & 0x03FF));
                    result.push(cp as i32);
                }
            } else {
                result.push(c as i32);
            }
            i += 1;
        }
        result
    }

    fn punycode_digit(digit: i32) -> u8 {
        if digit < 26 {
            (digit as u8) + b'a'
        } else if digit < 36 {
            ((digit - 26) as u8) + b'0'
        } else {
            panic!("unexpected digit: {}", digit);
        }
    }
}

trait CharExt {
    fn encode_utf8_code_point(self) -> i32;
}

impl CharExt for char {
    fn encode_utf8_code_point(self) -> i32 {
        self as i32
    }
}