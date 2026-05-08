use okio::{Buffer, ByteString};
use std::sync::Arc;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::internal::BufferMockResponseBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;

// Encode and decode a model object like a Long or Certificate as DER bytes.
pub trait DerAdapter<T>: Send + Sync {
    // Returns true if this adapter can read [header] in a choice.
    fn matches(&self, header: DerHeader) -> bool;

    // Returns a value from this adapter.
    //
    // This must always return a value, though it doesn't necessarily need to consume data from
    // [reader]. For example, if the reader's peeked tag isn't readable by this adapter, it may return
    // a default value.
    //
    // If this does read a value, it starts with the tag and length, and reads an entire value,
    // including any potential composed values.
    //
    // If there's nothing to read and no default value, this will throw an exception.
    fn from_der(&self, reader: &mut DerReader) -> T;

    // Helper to decode from a ByteString.
    fn from_der_bytes(&self, byte_string: ByteString) -> T
    where
        T: 'static,
    {
        let mut buffer = Buffer::new();
        buffer.write(byte_string);
        let mut reader = DerReader::new(buffer);
        self.from_der(&mut reader)
    }

    // Writes [value] to this adapter, unless it is the default value and can be safely omitted.
    //
    // If this does write a value, it will write a tag and a length and a full value.
    fn to_der(&self, writer: &mut DerWriter, value: T);

    // Helper to encode a value to a ByteString.
    fn to_der_bytes(&self, value: T) -> ByteString
    where
        T: 'static,
    {
        let mut buffer = Buffer::new();
        let mut writer = DerWriter::new(buffer);
        self.to_der(&mut writer, value);
        // In okio-rust, we need to get the buffer back from the writer or use the writer's internal buffer
        writer.read_byte_string()
    }

    // Returns an adapter that expects this value wrapped by another value.
    fn with_explicit_box(
        self: Arc<Self>,
        tag_class: i32,
        tag: i64,
        force_constructed: Option<bool>,
    ) -> BasicDerAdapter<T>
    where
        T: 'static,
    {
        let adapter = Arc::clone(&self);
        let codec = BasicDerAdapterCodec {
            decode: move |reader: &mut DerReader| adapter.from_der(reader),
            encode: move |writer: &mut DerWriter, value: T| {
                // We need a way to call to_der on the original adapter.
                // Since we are in a closure, we use the captured Arc.
                // Note: This requires the trait to be implemented for the Arc or a helper.
                // In Rust, we'll implement the logic inside the codec.
                // To avoid circularity, we use a helper closure or a wrapper.
                // The actual implementation of to_der is called here.
                // Since we can't call trait methods on Arc<Self> without a specific impl,
                // we assume the BasicDerAdapter handles the dispatch.
            },
            // We need to handle the force_constructed logic.
            // Because the Kotlin code uses an anonymous object, we'll use a custom struct for the codec.
        };

        // To properly implement the Kotlin anonymous object behavior in Rust:
        // We define a specific codec implementation.
        let codec = ExplicitBoxCodec {
            inner: Arc::clone(&self),
            force_constructed,
        };

        BasicDerAdapter::new("EXPLICIT", tag_class, tag, codec)
    }

    // Returns an adapter that returns a list of values of this type.
    fn as_sequence_of(
        self: Arc<Self>,
        name: String,
        tag_class: i32,
        tag: i64,
    ) -> BasicDerAdapter<Vec<T>>
    where
        T: 'static,
    {
        let adapter = Arc::clone(&self);
        let codec = SequenceCodec {
            inner: adapter,
        };

        BasicDerAdapter::new(name, tag_class, tag, codec)
    }

    // Returns an adapter that returns a set of values of this type.
    fn as_set_of(self: Arc<Self>) -> BasicDerAdapter<Vec<T>>
    where
        T: 'static,
    {
        self.as_sequence_of("SET OF".to_string(), DerHeader::TAG_CLASS_UNIVERSAL, 17)
    }
}

// Internal codec for Explicit Box wrapping.
struct ExplicitBoxCodec<T, A: DerAdapter<T>> {
    inner: Arc<A>,
    force_constructed: Option<bool>,
}

impl<T: 'static, A: DerAdapter<T>> BasicDerAdapterCodec<T> for ExplicitBoxCodec<T, A> {
    fn decode(&self, reader: &mut DerReader) -> T {
        self.inner.from_der(reader)
    }

    fn encode(&self, writer: &mut DerWriter, value: T) {
        self.inner.to_der(writer, value);
        if let Some(constructed) = self.force_constructed {
            writer.set_constructed(constructed);
        }
    }
}

// Internal codec for Sequence wrapping.
struct SequenceCodec<T, A: DerAdapter<T>> {
    inner: Arc<A>,
}

impl<T: 'static, A: DerAdapter<T>> BasicDerAdapterCodec<Vec<T>> for SequenceCodec<T, A> {
    fn decode(&self, reader: &mut DerReader) -> Vec<T> {
        let mut result = Vec::new();
        while reader.has_next() {
            result.push(self.inner.from_der(reader));
        }
        result
    }

    fn encode(&self, writer: &mut DerWriter, value: Vec<T>) {
        for v in value {
            self.inner.to_der(writer, v);
        }
    }
}

// Trait for the codec used by BasicDerAdapter.
pub trait BasicDerAdapterCodec<T>: Send + Sync {
    fn decode(&self, reader: &mut DerReader) -> T;
    fn encode(&self, writer: &mut DerWriter, value: T);
}

// A general purpose DER adapter implementation.
#[derive(Clone)]
pub struct BasicDerAdapter<T> {
    name: String,
    tag_class: i32,
    tag: i64,
    codec: Arc<dyn BasicDerAdapterCodec<T>>,
}

impl<T: 'static> BasicDerAdapter<T> {
    pub fn new(
        name: &str,
        tag_class: i32,
        tag: i64,
        codec: impl BasicDerAdapterCodec<T> + 'static,
    ) -> Self {
        Self {
            name: name.to_string(),
            tag_class,
            tag,
            codec: Arc::new(codec),
        }
    }
}

impl<T: 'static> DerAdapter<T> for BasicDerAdapter<T> {
    fn matches(&self, header: DerHeader) -> bool {
        header.tag_class == self.tag_class && header.tag == self.tag
    }

    fn from_der(&self, reader: &mut DerReader) -> T {
        self.codec.decode(reader)
    }

    fn to_der(&self, writer: &mut DerWriter, value: T) {
        self.codec.encode(writer, value);
    }
}

// --- Mock/Stub definitions for dependencies to ensure compilability ---
// These would normally be imported from the rest of the okhttp-tls crate.

pub struct DerHeader {
    pub tag_class: i32,
    pub tag: i64,
}

impl DerHeader {
    pub const TAG_CLASS_UNIVERSAL: i32 = 0;
    pub const TAG_CLASS_CONTEXT_SPECIFIC: i32 = 1;
}

pub struct DerReader {
    buffer: Buffer,
}

impl DerReader {
    pub fn new(buffer: Buffer) -> Self {
        Self { buffer }
    }
    pub fn has_next(&mut self) -> bool {
        !self.buffer.empty()
    }
}

pub struct DerWriter {
    buffer: Buffer,
    constructed: bool,
}

impl DerWriter {
    pub fn new(buffer: Buffer) -> Self {
        Self {
            buffer,
            constructed: false,
        }
    }
    pub fn set_constructed(&mut self, value: bool) {
        self.constructed = value;
    }
    pub fn read_byte_string(self) -> ByteString {
        self.buffer.read_byte_string()
    }
}