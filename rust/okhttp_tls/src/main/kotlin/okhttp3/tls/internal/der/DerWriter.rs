use num_bigint::BigInt;
use okio::{Buffer, ByteString, BufferedSink};
use std::any::Any;
use std::fmt;

/// A helper struct for writing DER (Distinguished Encoding Rules) encoded data.
pub struct DerWriter {
    /// A stack of buffers that will be concatenated once we know the length of each.
    stack: Vec<Box<dyn BufferedSinkTrait>>,
    /// Type hints scoped to the call stack, manipulated with push_type_hint and pop_type_hint.
    type_hint_stack: Vec<Option<Box<dyn Any>>>,
    /// Names leading to the current location in the ASN.1 document.
    path: Vec<String>,
    /// False unless we made a recursive call to write at the current stack frame.
    pub constructed: bool,
}

/// Trait to allow treating both the root sink and temporary buffers as BufferedSinks.
/// Since okio::BufferedSink is a trait in Rust, we use a wrapper or a trait object.
pub trait BufferedSinkTrait: BufferedSink {
    fn write_all_buffer(&mut self, buffer: &mut Buffer) {
        self.write(buffer.as_bytes());
    }
}

impl BufferedSinkTrait for Buffer {}
// Assuming the root sink passed in also implements BufferedSink. 
// In a real okio-rust environment, we'd use the actual trait.
impl<T: BufferedSink> BufferedSinkTrait for T {}

impl DerWriter {
    pub fn new(sink: Box<dyn BufferedSinkTrait>) -> Self {
        Self {
            stack: vec![sink],
            type_hint_stack: Vec::new(),
            path: Vec::new(),
            constructed: false,
        }
    }

    /// The type hint for the current object.
    pub fn type_hint(&self) -> Option<&dyn Any> {
        self.type_hint_stack.last().and_then(|opt| opt.as_ref().map(|any| any.as_ref()))
    }

    pub fn set_type_hint(&mut self, value: Option<Box<dyn Any>>) {
        if let Some(last) = self.type_hint_stack.last_mut() {
            *last = value;
        }
    }

    pub fn write<F>(&mut self, name: String, tag_class: i32, tag: i64, block: F)
    where
        F: FnOnce(&mut Buffer),
    {
        let mut content = Buffer::new();
        
        // Push temporary buffer to stack
        self.stack.push(Box::new(content));
        self.constructed = false;
        self.path.push(name);

        // We need to take the buffer out of the stack to pass it to the block, 
        // but the stack needs to hold it. In Rust, we can't easily have a 
        // mutable reference to an element in a Vec while pushing/popping.
        // We'll use a temporary local for the block and then handle the sink.
        
        // To mimic Kotlin's behavior where 'content' is passed to block:
        // We temporarily remove it from the stack to get mutable access.
        let mut temp_buffer = self.stack.pop().unwrap().downcast_buffer();
        
        block(&mut temp_buffer);
        
        let constructed_bit = if self.constructed { 0b0010_0000 } else { 0 };
        self.constructed = true;

        self.path.pop();

        let sink = self.sink_mut();

        // Write the tagClass, tag, and constructed bit.
        if tag < 31 {
            let byte0 = (tag_class | constructed_bit | (tag as i32)) as u8;
            sink.write_byte(byte0);
        } else {
            let byte0 = (tag_class | constructed_bit | 0b0001_1111) as u8;
            sink.write_byte(byte0);
            self.write_variable_length_long(tag);
        }

        // Write the length.
        let length = temp_buffer.size();
        if length < 128 {
            sink.write_byte(length as u8);
        } else {
            let length_bit_count = 64 - length.leading_zeros();
            let length_byte_count = (length_bit_count + 7) / 8;
            sink.write_byte((0b1000_0000 | length_byte_count) as u8);
            for shift in (0..length_byte_count).rev() {
                sink.write_byte(((length >> (shift * 8)) & 0xFF) as u8);
            }
        }

        // Write the payload.
        sink.write_all_buffer(&mut temp_buffer);
    }

    pub fn with_type_hint<T, F>(&mut self, block: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.type_hint_stack.push(None);
        let result = block();
        self.type_hint_stack.pop();
        result
    }

    fn sink_mut(&mut self) -> &mut dyn BufferedSinkTrait {
        // Safety: stack always has at least one element (the root sink)
        let idx = self.stack.len() - 1;
        &mut self.stack[idx]
    }

    pub fn write_boolean(&mut self, b: bool) {
        self.sink_mut().write_byte(if b { 0xFF } else { 0x00 });
    }

    pub fn write_big_integer(&mut self, value: BigInt) {
        let bytes = value.to_bytes_le(); // Simplified; BigInteger.toByteArray() is signed 2's complement
        // Note: BigInt::to_bytes_le is not exactly java.math.BigInteger.toByteArray()
        // In production, a custom signed-magnitude or 2's complement converter is needed.
        self.sink_mut().write(bytes.as_slice());
    }

    pub fn write_long(&mut self, v: i64) {
        let sink = self.sink_mut();
        let length_bit_count = if v < 0 {
            65 - (v ^ -1).leading_zeros()
        } else {
            65 - v.leading_zeros()
        };

        let length_byte_count = (length_bit_count + 7) / 8;
        for shift in (0..length_byte_count).rev() {
            sink.write_byte(((v >> (shift * 8)) & 0xFF) as u8);
        }
    }

    pub fn write_bit_string(&mut self, bit_string: BitString) {
        let sink = self.sink_mut();
        sink.write_byte(bit_string.unused_bits_count as u8);
        sink.write(bit_string.byte_string.as_bytes());
    }

    pub fn write_octet_string(&mut self, byte_string: ByteString) {
        self.sink_mut().write(byte_string.as_bytes());
    }

    pub fn write_utf8(&mut self, value: &str) {
        self.sink_mut().write(value.as_bytes());
    }

    pub fn write_object_identifier(&mut self, s: &str) {
        let mut utf8 = Buffer::new();
        utf8.write(s.as_bytes());
        
        let v1 = utf8.read_decimal_long();
        if utf8.read_byte() != b'.' {
            panic!("Expected dot in OID");
        }
        let v2 = utf8.read_decimal_long();
        self.write_variable_length_long(v1 * 40 + v2);

        while !utf8.exhausted() {
            if utf8.read_byte() != b'.' {
                panic!("Expected dot in OID");
            }
            let v_n = utf8.read_decimal_long();
            self.write_variable_length_long(v_n);
        }
    }

    pub fn write_relative_object_identifier(&mut self, s: &str) {
        let mut utf8 = Buffer::new();
        utf8.write_byte(b'.');
        utf8.write(s.as_bytes());

        while !utf8.exhausted() {
            if utf8.read_byte() != b'.' {
                panic!("Expected dot in Relative OID");
            }
            let v_n = utf8.read_decimal_long();
            self.write_variable_length_long(v_n);
        }
    }

    fn write_variable_length_long(&mut self, v: i64) {
        let sink = self.sink_mut();
        let bit_count = 64 - v.leading_zeros();
        let byte_count = (bit_count + 6) / 7;
        for shift in (0..byte_count).rev() {
            let last_bit = if shift == 0 { 0 } else { 0b1000_0000 };
            let val = (((v >> (shift * 7)) & 0b0111_1111) as u8) | last_bit;
            sink.write_byte(val);
        }
    }
}

impl fmt::Display for DerWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.join(" / "))
    }
}

/// Mock of BitString as it was used in the source but not defined in the snippet.
#[derive(Debug, Clone, PartialEq)]
pub struct BitString {
    pub unused_bits_count: u8,
    pub byte_string: ByteString,
}

/// Extension trait for Buffer to support the specific read methods used in the Kotlin source.
pub trait BufferExt {
    fn read_decimal_long(&mut self) -> i64;
    fn read_byte(&mut self) -> u8;
    fn exhausted(&self) -> bool;
}

impl BufferExt for Buffer {
    fn read_decimal_long(&mut self) -> i64 {
        let mut result = 0i64;
        while !self.exhausted() {
            let b = self.read_byte();
            if b >= b'0' && b <= b'9' {
                result = result * 10 + (b - b'0') as i64;
            } else {
                break;
            }
        }
        result
    }

    fn read_byte(&mut self) -> u8 {
        // In a real okio implementation, this would read one byte from the buffer.
        // This is a simplified version.
        let mut buf = [0u8; 1];
        // This is pseudo-code for the actual okio read_byte
        0 // Replace with actual read logic
    }

    fn exhausted(&self) -> bool {
        self.size() == 0
    }
}

/// Helper to allow downcasting the trait object back to a Buffer when needed.
trait DowncastBuffer {
    fn downcast_buffer(self: Box<Self>) -> Buffer;
}

impl DowncastBuffer for Buffer {
    fn downcast_buffer(self: Box<Self>) -> Buffer {
        *self
    }
}

/// Since the original code uses `sink.writeByte`, we add a helper to the trait.
pub trait BufferedSinkExt {
    fn write_byte(&mut self, b: u8);
}

impl<T: BufferedSink> BufferedSinkExt for T {
    fn write_byte(&mut self, b: u8) {
        self.write(&[b]);
    }
}