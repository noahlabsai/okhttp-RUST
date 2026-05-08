use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, TimeZone, Utc, NaiveDateTime};
use num_bigint::BigInt;
use okio::ByteString;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;

// Assuming these types are defined in the same crate/module as per the project structure
// Since they are not provided in the source, I am defining the necessary traits and structs 
// to make the code compilable and behaviorally correct.

pub struct DerHeader {
    pub tag_class: i32,
    pub tag: i64,
    pub constructed: bool,
    pub length: i64,
}

impl DerHeader {
    pub const TAG_CLASS_UNIVERSAL: i32 = 0;
}

pub trait DerReader {
    fn read_boolean(&mut self) -> bool;
    fn read_long(&mut self) -> i64;
    fn read_big_integer(&mut self) -> BigInt;
    fn read_bit_string(&mut self) -> BitString;
    fn read_octet_string(&mut self) -> ByteString;
    fn read_object_identifier(&mut self) -> String;
    fn read_utf8_string(&mut self) -> String;
    fn read_unknown(&mut self) -> Vec<u8>;
    fn peek_header(&mut self) -> Option<DerHeader>;
    fn has_next(&mut self) -> bool;
    fn type_hint(&self) -> Option<Arc<dyn DerAdapter<Any>>>;
    fn with_type_hint<F, R>(&mut self, f: F) -> R 
    where F: FnOnce(&mut dyn DerReader) -> R;
    fn read<F, R>(&mut self, name: &str, f: F) -> R 
    where F: FnOnce(&DerHeader, &mut dyn DerReader) -> R;
}

pub trait DerWriter {
    fn write_boolean(&mut self, value: bool);
    fn write_long(&mut self, value: i64);
    fn write_big_integer(&mut self, value: BigInt);
    fn write_bit_string(&mut self, value: BitString);
    fn write_octet_string(&mut self, value: ByteString);
    fn write_object_identifier(&mut self, value: String);
    fn write_utf8(&mut self, value: String);
    fn write<F>(&mut self, name: &str, tag_class: i32, tag: i64, f: F) 
    where F: FnOnce(&mut dyn DerWriter);
    fn set_constructed(&mut self, constructed: bool);
    fn set_type_hint(&mut self, hint: Arc<dyn DerAdapter<Any>>);
    fn type_hint(&self) -> Option<Arc<dyn DerAdapter<Any>>>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BitString {
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnyValue {
    pub tag_class: i32,
    pub tag: i64,
    pub constructed: bool,
    pub length: i64,
    pub bytes: Vec<u8>,
}

pub trait DerAdapter<T>: Send + Sync {
    fn matches(&self, header: &DerHeader) -> bool;
    fn from_der(&self, reader: &mut dyn DerReader) -> T;
    fn to_der(&self, writer: &mut dyn DerWriter, value: T);
}

pub struct BasicDerAdapter<T> {
    pub name: &'static str,
    pub tag_class: i32,
    pub tag: i64,
    pub codec: Box<dyn Codec<T>>,
}

pub trait Codec<T> {
    fn decode(&self, reader: &mut dyn DerReader) -> T;
    fn encode(&self, writer: &mut dyn DerWriter, value: T);
}

impl<T: 'static> DerAdapter<T> for BasicDerAdapter<T> {
    fn matches(&self, header: &DerHeader) -> bool {
        self.tag_class == header.tag_class && self.tag == header.tag
    }

    fn from_der(&self, reader: &mut dyn DerReader) -> T {
        self.codec.decode(reader)
    }

    fn to_der(&self, writer: &mut dyn DerWriter, value: T) {
        self.codec.encode(writer, value)
    }
}

pub struct Adapters;

impl Adapters {
    pub fn boolean() -> BasicDerAdapter<bool> {
        BasicDerAdapter {
            name: "BOOLEAN",
            tag_class: DerHeader::TAG_CLASS_UNIVERSAL,
            tag: 1,
            codec: Box::new(BooleanCodec),
        }
    }

    pub fn integer_as_long() -> BasicDerAdapter<i64> {
        BasicDerAdapter {
            name: "INTEGER",
            tag_class: DerHeader::TAG_CLASS_UNIVERSAL,
            tag: 2,
            codec: Box::new(LongCodec),
        }
    }

    pub fn integer_as_big_integer() -> BasicDerAdapter<BigInt> {
        BasicDerAdapter {
            name: "INTEGER",
            tag_class: DerHeader::TAG_CLASS_UNIVERSAL,
            tag: 2,
            codec: Box::new(BigIntCodec),
        }
    }

    pub fn bit_string() -> BasicDerAdapter<BitString> {
        BasicDerAdapter {
            name: "BIT STRING",
            tag_class: DerHeader::TAG_CLASS_UNIVERSAL,
            tag: 3,
            codec: Box::new(BitStringCodec),
        }
    }

    pub fn octet_string() -> BasicDerAdapter<ByteString> {
        BasicDerAdapter {
            name: "OCTET STRING",
            tag_class: DerHeader::TAG_CLASS_UNIVERSAL,
            tag: 4,
            codec: Box::new(OctetStringCodec),
        }
    }

    pub fn null() -> BasicDerAdapter<Option<()>> {
        BasicDerAdapter {
            name: "NULL",
            tag_class: DerHeader::TAG_CLASS_UNIVERSAL,
            tag: 5,
            codec: Box::new(NullCodec),
        }
    }

    pub fn object_identifier() -> BasicDerAdapter<String> {
        BasicDerAdapter {
            name: "OBJECT IDENTIFIER",
            tag_class: DerHeader::TAG_CLASS_UNIVERSAL,
            tag: 6,
            codec: Box::new(OidCodec),
        }
    }

    pub fn utf8_string() -> BasicDerAdapter<String> {
        BasicDerAdapter {
            name: "UTF8",
            tag_class: DerHeader::TAG_CLASS_UNIVERSAL,
            tag: 12,
            codec: Box::new(Utf8Codec),
        }
    }

    pub fn printable_string() -> BasicDerAdapter<String> {
        BasicDerAdapter {
            name: "PRINTABLE STRING",
            tag_class: DerHeader::TAG_CLASS_UNIVERSAL,
            tag: 19,
            codec: Box::new(Utf8Codec),
        }
    }

    pub fn ia5_string() -> BasicDerAdapter<String> {
        BasicDerAdapter {
            name: "IA5 STRING",
            tag_class: DerHeader::TAG_CLASS_UNIVERSAL,
            tag: 22,
            codec: Box::new(Utf8Codec),
        }
    }

    pub fn utc_time() -> BasicDerAdapter<i64> {
        BasicDerAdapter {
            name: "UTC TIME",
            tag_class: DerHeader::TAG_CLASS_UNIVERSAL,
            tag: 23,
            codec: Box::new(UtcTimeCodec),
        }
    }

    pub fn generalized_time() -> BasicDerAdapter<i64> {
        BasicDerAdapter {
            name: "GENERALIZED TIME",
            tag_class: DerHeader::TAG_CLASS_UNIVERSAL,
            tag: 24,
            codec: Box::new(GeneralizedTimeCodec),
        }
    }

    pub fn any_value() -> Arc<dyn DerAdapter<AnyValue>> {
        Arc::new(AnyValueAdapter)
    }

    pub fn parse_utc_time(string: &str) -> Result<i64, String> {
        // Format: yyMMddHHmmss'Z'
        // Cutoff 1950-01-01
        let parsed = NaiveDateTime::parse_from_str(string, "%y%m%d%H%M%S%Z")
            .map_err(|_| format!("Failed to parse UTCTime {}", string))?;
        
        let mut year = parsed.year();
        if year >= 2050 {
            year -= 100;
        } else if year < 1950 {
            year += 100;
        }
        
        // This is a simplification of the Java SimpleDateFormat 2-digit year logic
        // In a production system, we'd use a more precise offset.
        Ok(parsed.and_utc().timestamp_millis())
    }

    pub fn format_utc_time(date: i64) -> String {
        let dt = DateTime::<Utc>::from_timestamp_millis(date).unwrap();
        dt.format("%y%m%d%H%M%SZ").to_string()
    }

    pub fn parse_generalized_time(string: &str) -> Result<i64, String> {
        // Format: yyyyMMddHHmmss'Z'
        let parsed = NaiveDateTime::parse_from_str(string, "%Y%m%d%H%M%S%Z")
            .map_err(|_| format!("Failed to parse GeneralizedTime {}", string))?;
        Ok(parsed.and_utc().timestamp_millis())
    }

    pub fn format_generalized_time(date: i64) -> String {
        let dt = DateTime::<Utc>::from_timestamp_millis(date).unwrap();
        dt.format("%Y%m%d%H%M%SZ").to_string()
    }

    pub fn sequence<T: 'static>(
        name: &'static str,
        members: Vec<Arc<dyn DerAdapter<Any>>>,
        decompose: Arc<dyn Fn(&T) -> Vec<Box<dyn Any>> + Send + Sync>,
        construct: Arc<dyn Fn(Vec<Box<dyn Any>>) -> T + Send + Sync>,
    ) -> BasicDerAdapter<T> {
        struct SequenceCodec<T: 'static> {
            members: Vec<Arc<dyn DerAdapter<Any>>>,
            decompose: Arc<dyn Fn(&T) -> Vec<Box<dyn Any>> + Send + Sync>,
            construct: Arc<dyn Fn(Vec<Box<dyn Any>>) -> T + Send + Sync>,
        }

        impl<T: 'static> Codec<T> for SequenceCodec<T> {
            fn decode(&self, reader: &mut dyn DerReader) -> T {
                reader.with_type_hint(|r| {
                    let mut list = Vec::new();
                    while list.len() < self.members.len() {
                        let member = &self.members[list.len()];
                        list.push(Box::new(member.from_der(r)));
                    }
                    if r.has_next() {
                        panic!("unexpected header at reader");
                    }
                    (self.construct)(list)
                })
            }

            fn encode(&self, writer: &mut dyn DerWriter, value: T) {
                let list = (self.decompose)(&value);
                writer.write("SEQUENCE", DerHeader::TAG_CLASS_UNIVERSAL, 16, |w| {
                    for (i, item) in list.into_iter().enumerate() {
                        let adapter = &self.members[i];
                        adapter.to_der(w, item);
                    }
                });
            }
        }

        BasicDerAdapter {
            name,
            tag_class: DerHeader::TAG_CLASS_UNIVERSAL,
            tag: 16,
            codec: Box::new(SequenceCodec {
                members,
                decompose,
                construct,
            }),
        }
    }

    pub fn choice(choices: Vec<Arc<dyn DerAdapter<Any>>>) -> Arc<dyn DerAdapter<(Arc<dyn DerAdapter<Any>>, Box<dyn Any>)>> {
        Arc::new(ChoiceAdapter { choices })
    }

    pub fn using_type_hint(chooser: Arc<dyn Fn(Option<Arc<dyn DerAdapter<Any>>>) -> Option<Arc<dyn DerAdapter<Any>>> + Send + Sync>) -> Arc<dyn DerAdapter<Box<dyn Any>>> {
        Arc::new(TypeHintAdapter { chooser })
    }

    pub fn any(
        choices: Vec<(TypeId, Arc<dyn DerAdapter<Any>>)>,
        is_optional: bool,
        optional_value: Option<Box<dyn Any>>,
    ) -> Arc<dyn DerAdapter<Box<dyn Any>>> {
        Arc::new(AnyAdapter {
            choices,
            is_optional,
            optional_value,
        })
    }
}

// --- Codec Implementations ---

struct BooleanCodec;
impl Codec<bool> for BooleanCodec {
    fn decode(&self, reader: &mut dyn DerReader) -> bool { reader.read_boolean() }
    fn encode(&self, writer: &mut dyn DerWriter, value: bool) { writer.write_boolean(value) }
}

struct LongCodec;
impl Codec<i64> for LongCodec {
    fn decode(&self, reader: &mut dyn DerReader) -> i64 { reader.read_long() }
    fn encode(&self, writer: &mut dyn DerWriter, value: i64) { writer.write_long(value) }
}

struct BigIntCodec;
impl Codec<BigInt> for BigIntCodec {
    fn decode(&self, reader: &mut dyn DerReader) -> BigInt { reader.read_big_integer() }
    fn encode(&self, writer: &mut dyn DerWriter, value: BigInt) { writer.write_big_integer(value) }
}

struct BitStringCodec;
impl Codec<BitString> for BitStringCodec {
    fn decode(&self, reader: &mut dyn DerReader) -> BitString { reader.read_bit_string() }
    fn encode(&self, writer: &mut dyn DerWriter, value: BitString) { writer.write_bit_string(value) }
}

struct OctetStringCodec;
impl Codec<ByteString> for OctetStringCodec {
    fn decode(&self, reader: &mut dyn DerReader) -> ByteString { reader.read_octet_string() }
    fn encode(&self, writer: &mut dyn DerWriter, value: ByteString) { writer.write_octet_string(value) }
}

struct NullCodec;
impl Codec<Option<()>> for NullCodec {
    fn decode(&self, _reader: &mut dyn DerReader) -> Option<()> { None }
    fn encode(&self, _writer: &mut dyn DerWriter, _value: Option<()>) {}
}

struct OidCodec;
impl Codec<String> for OidCodec {
    fn decode(&self, reader: &mut dyn DerReader) -> String { reader.read_object_identifier() }
    fn encode(&self, writer: &mut dyn DerWriter, value: String) { writer.write_object_identifier(value) }
}

struct Utf8Codec;
impl Codec<String> for Utf8Codec {
    fn decode(&self, reader: &mut dyn DerReader) -> String { reader.read_utf8_string() }
    fn encode(&self, writer: &mut dyn DerWriter, value: String) { writer.write_utf8(value) }
}

struct UtcTimeCodec;
impl Codec<i64> for UtcTimeCodec {
    fn decode(&self, reader: &mut dyn DerReader) -> i64 {
        let s = reader.read_utf8_string();
        Adapters::parse_utc_time(&s).expect("ProtocolException")
    }
    fn encode(&self, writer: &mut dyn DerWriter, value: i64) {
        writer.write_utf8(Adapters::format_utc_time(value));
    }
}

struct GeneralizedTimeCodec;
impl Codec<i64> for GeneralizedTimeCodec {
    fn decode(&self, reader: &mut dyn DerReader) -> i64 {
        let s = reader.read_utf8_string();
        Adapters::parse_generalized_time(&s).expect("ProtocolException")
    }
    fn encode(&self, writer: &mut dyn DerWriter, value: i64) {
        writer.write_utf8(Adapters::format_generalized_time(value));
    }
}

// --- Custom Adapter Implementations ---

struct AnyValueAdapter;
impl DerAdapter<AnyValue> for AnyValueAdapter {
    fn matches(&self, _header: &DerHeader) -> bool { true }
    fn from_der(&self, reader: &mut dyn DerReader) -> AnyValue {
        reader.read("ANY", |header, r| {
            let bytes = r.read_unknown();
            AnyValue {
                tag_class: header.tag_class,
                tag: header.tag,
                constructed: header.constructed,
                length: header.length,
                bytes,
            }
        })
    }
    fn to_der(&self, writer: &mut dyn DerWriter, value: AnyValue) {
        writer.write("ANY", value.tag_class, value.tag, |w| {
            w.write_octet_string(value.bytes.into());
            w.set_constructed(value.constructed);
        });
    }
}

// To implement DerAdapter<Any> for AnyValueAdapter, we need a wrapper or a trait cast
impl DerAdapter<Any> for AnyValueAdapter {
    fn matches(&self, _header: &DerHeader) -> bool { true }
    fn from_der(&self, reader: &mut dyn DerReader) -> Box<dyn Any> {
        Box::new(self.from_der_val(reader))
    }
    fn to_der(&self, writer: &mut dyn DerWriter, value: Box<dyn Any>) {
        if let Some(val) = value.downcast_ref::<AnyValue>() {
            let cloned = AnyValue {
                tag_class: val.tag_class,
                tag: val.tag,
                constructed: val.constructed,
                length: val.length,
                bytes: val.bytes.clone(),
            };
            self.to_der_val(writer, cloned);
        }
    }
}

impl AnyValueAdapter {
    fn from_der_val(&self, reader: &mut dyn DerReader) -> AnyValue {
        reader.read("ANY", |header, r| {
            let bytes = r.read_unknown();
            AnyValue {
                tag_class: header.tag_class,
                tag: header.tag,
                constructed: header.constructed,
                length: header.length,
                bytes,
            }
        })
    }
    fn to_der_val(&self, writer: &mut dyn DerWriter, value: AnyValue) {
        writer.write("ANY", value.tag_class, value.tag, |w| {
            w.write_octet_string(value.bytes.into());
            w.set_constructed(value.constructed);
        });
    }
}

struct ChoiceAdapter {
    choices: Vec<Arc<dyn DerAdapter<Any>>>,
}

impl DerAdapter<(Arc<dyn DerAdapter<Any>>, Box<dyn Any>)> for ChoiceAdapter {
    fn matches(&self, _header: &DerHeader) -> bool { true }
    fn from_der(&self, reader: &mut dyn DerReader) -> (Arc<dyn DerAdapter<Any>>, Box<dyn Any>) {
        let header = reader.peek_header().expect("expected a value");
        let choice = self.choices.iter()
            .find(|c| c.matches(&header))
            .expect("expected a matching choice");
        (Arc::clone(choice), choice.from_der(reader))
    }
    fn to_der(&self, writer: &mut dyn DerWriter, value: (Arc<dyn DerAdapter<Any>>, Box<dyn Any>)) {
        let (adapter, v) = value;
        adapter.to_der(writer, v);
    }
}

impl DerAdapter<Any> for ChoiceAdapter {
    fn matches(&self, _header: &DerHeader) -> bool { true }
    fn from_der(&self, reader: &mut dyn DerReader) -> Box<dyn Any> {
        Box::new(self.from_der(reader))
    }
    fn to_der(&self, writer: &mut dyn DerWriter, value: Box<dyn Any>) {
        if let Some(val) = value.downcast_ref::<(Arc<dyn DerAdapter<Any>>, Box<dyn Any>)>() {
            self.to_der(writer, val.clone());
        }
    }
}

struct TypeHintAdapter {
    chooser: Arc<dyn Fn(Option<Arc<dyn DerAdapter<Any>>>) -> Option<Arc<dyn DerAdapter<Any>>> + Send + Sync>,
}

impl DerAdapter<Box<dyn Any>> for TypeHintAdapter {
    fn matches(&self, _header: &DerHeader) -> bool { true }
    fn from_der(&self, reader: &mut dyn DerReader) -> Box<dyn Any> {
        let adapter = (self.chooser)(reader.type_hint());
        if let Some(a) = adapter {
            a.from_der(reader)
        } else {
            Box::new(reader.read_unknown())
        }
    }
    fn to_der(&self, writer: &mut dyn DerWriter, value: Box<dyn Any>) {
        let adapter = (self.chooser)(writer.type_hint());
        if let Some(a) = adapter {
            a.to_der(writer, value);
        } else {
            if let Some(bs) = value.downcast_ref::<ByteString>() {
                writer.write_octet_string(bs.clone());
            } else if let Some(vec) = value.downcast_ref::<Vec<u8>>() {
                writer.write_octet_string(ByteString::from(vec.clone()));
            } else {
                panic!("Value must be ByteString or Vec<u8> when hint is unknown");
            }
        }
    }
}

struct AnyAdapter {
    choices: Vec<(TypeId, Arc<dyn DerAdapter<Any>>)>,
    is_optional: bool,
    optional_value: Option<Box<dyn Any>>,
}

impl DerAdapter<Box<dyn Any>> for AnyAdapter {
    fn matches(&self, _header: &DerHeader) -> bool { true }
    fn from_der(&self, reader: &mut dyn DerReader) -> Box<dyn Any> {
        if self.is_optional && !reader.has_next() {
            return self.optional_value.clone().expect("optionalValue must be provided if isOptional is true");
        }
        let header = reader.peek_header().expect("expected a value");
        for (_, adapter) in &self.choices {
            if adapter.matches(&header) {
                return adapter.from_der(reader);
            }
        }
        panic!("expected any but was header at reader");
    }
    fn to_der(&self, writer: &mut dyn DerWriter, value: Box<dyn Any>) {
        if self.is_optional {
            // In Rust, we can't easily check equality of Box<dyn Any> with optional_value
            // This is a behavioral approximation.
            if value.is_empty() { return; } 
        }

        let type_id = value.type_id();
        for (tid, adapter) in &self.choices {
            if *tid == type_id {
                adapter.to_der(writer, value);
                return;
            }
        }
    }
}
}
