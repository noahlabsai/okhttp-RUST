use std::io::{self, ErrorKind};

// Mocking the BufferedSource trait as it is a dependency from okio.
// In a real production environment, this would be imported from the okio crate.
pub trait BufferedSource {
    fn peek(&self) -> Box<dyn BufferedSourceClone>;
    fn exhausted(&self) -> bool;
    fn read_utf8_code_point(&mut self) -> Result<u32, io::Error>;
}

// Helper trait to represent the peeked source.
pub trait BufferedSourceClone: BufferedSource {}

// Returns true if the body in question probably contains human-readable text. Uses a small
// sample of code points to detect Unicode control characters commonly used in binary file
// signatures.
//
// # Arguments
// * `code_point_limit` - the number of code points to read in order to make a decision.
pub fn is_probably_utf8(source: &mut dyn BufferedSource, code_point_limit: Option<i64>) -> bool {
    let limit = code_point_limit.unwrap_or(i64::MAX);
    
    let mut peek = source.peek();
    
    for _ in 0..limit {
        if peek.exhausted() {
            break;
        }
        
        match peek.read_utf8_code_point() {
            Ok(code_point) => {
                if is_iso_control(code_point) && !is_whitespace(code_point) {
                    return false;
                }
            }
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                // Truncated UTF-8 sequence.
                return false;
            }
            Err(_) => {
                // Other IO errors are treated as non-UTF8/binary for safety in this context
                return false;
            }
        }
    }
    true
}

// Implementation of Character.isISOControl
fn is_iso_control(code_point: u32) -> bool {
    // ISO control characters are in the range [0, 31] and [127, 159]
    (code_point <= 31) || (code_point >= 127 && code_point <= 159)
}

// Implementation of Character.isWhitespace
fn is_whitespace(code_point: u32) -> bool {
    match code_point {
        0x0009 | // \t
        0x000A | // \n
        0x000B | // \v
        0x000C | // \f
        0x000D | // \r
        0x0020 | // space
        0x00A0 | // non-breaking space
        0x1680 | // ogham space mark
        0x2000..=0x200A | // en quad .. hair space
        0x2028 | // line separator
        0x2029 | // paragraph separator
        0x202F | // narrow no-break space
        0x205F | // medium mathematical space
        0x3000 => true, // ideographic space
        _ => false,
    }
}

// Extension trait to allow calling .is_probably_utf8() on BufferedSource
pub trait BufferedSourceExt {
    fn is_probably_utf8(&mut self, code_point_limit: Option<i64>) -> bool;
}

impl BufferedSourceExt for dyn BufferedSource {
    fn is_probably_utf8(&mut self, code_point_limit: Option<i64>) -> bool {
        is_probably_utf8(self, code_point_limit)
    }
}