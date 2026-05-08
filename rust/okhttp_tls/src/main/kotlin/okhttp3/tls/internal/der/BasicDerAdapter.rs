/*
 * Copyright (C) 2020 Square, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::okhttp_tls::src::main::kotlin::okhttp3::tls::internal::der::{
    DerAdapter, DerHeader, DerReader, DerWriter,
};
use std::fmt;
use std::sync::Arc;

/// Reads and writes values without knowledge of the enclosing tag, length, or defaults.
pub trait Codec<T>: Send + Sync {
    fn decode(&self, reader: &mut DerReader) -> T;

    fn encode(&self, writer: &mut DerWriter, value: T);
}

/// Handles basic types that always use the same tag. This supports optional types and may set a type
/// hint for further adapters to process.
///
/// Types like ANY and CHOICE that don't have a consistent tag cannot use this.
#[derive(Clone)]
pub struct BasicDerAdapter<T> {
    name: String,
    /// The tag class this adapter expects, or -1 to match any tag class.
    pub tag_class: i32,
    /// The tag this adapter expects, or -1 to match any tag.
    pub tag: i64,
    /// Encode and decode the value once tags are handled.
    codec: Arc<dyn Codec<T>>,
    /// True if the default value should be used if this value is absent during decoding.
    pub is_optional: bool,
    /// The value to return if this value is absent. Undefined unless this is optional.
    pub default_value: Option<T>,
    /// True to set the encoded or decoded value as the type hint for the current SEQUENCE.
    type_hint: bool,
}

impl<T> BasicDerAdapter<T>
where
    T: 'static + Clone + PartialEq + Send + Sync,
{
    pub fn new(
        name: String,
        tag_class: i32,
        tag: i64,
        codec: Arc<dyn Codec<T>>,
        is_optional: bool,
        default_value: Option<T>,
        type_hint: bool,
    ) -> Self {
        assert!(tag_class >= 0, "tagClass must be >= 0");
        assert!(tag >= 0, "tag must be >= 0");
        Self {
            name,
            tag_class,
            tag,
            codec,
            is_optional,
            default_value,
            type_hint,
        }
    }

    /// Returns a copy with a context tag. This should be used when the type is ambiguous on its own.
    pub fn with_tag(self, tag_class: Option<i32>, tag: i64) -> Self {
        let mut new_adapter = self.clone();
        new_adapter.tag_class = tag_class.unwrap_or(DerHeader::TAG_CLASS_CONTEXT_SPECIFIC);
        new_adapter.tag = tag;
        new_adapter
    }

    /// Returns a copy of this adapter that doesn't encode values equal to [defaultValue].
    pub fn optional(mut self, default_value: Option<T>) -> Self {
        self.is_optional = true;
        self.default_value = default_value;
        self
    }

    /// Returns a copy of this adapter that sets the encoded or decoded value as the type hint for the
    /// other adapters on this SEQUENCE to interrogate.
    pub fn as_type_hint(mut self) -> Self {
        self.type_hint = true;
        self
    }
}

impl<T> DerAdapter<T> for BasicDerAdapter<T>
where
    T: 'static + Clone + PartialEq + Send + Sync,
{
    fn matches(&self, header: DerHeader) -> bool {
        header.tag_class == self.tag_class && header.tag == self.tag
    }

    fn from_der(&self, reader: &mut DerReader) -> T {
        let peeked_header = reader.peek_header();
        if peeked_header.is_none()
            || peeked_header.as_ref().unwrap().tag_class != self.tag_class
            || peeked_header.as_ref().unwrap().tag != self.tag
        {
            if self.is_optional {
                return self
                    .default_value
                    .clone()
                    .expect("defaultValue must be present if isOptional is true");
            }
            panic!(
                "expected {} but was {:?} at {}",
                self,
                peeked_header,
                reader
            );
        }

        let result = reader.read(&self.name, || self.codec.decode(reader));

        if self.type_hint {
            reader.set_type_hint(result.clone());
        }

        result
    }

    fn to_der(&self, writer: &mut DerWriter, value: T) {
        if self.type_hint {
            writer.set_type_hint(value.clone());
        }

        if self.is_optional && Some(value.clone()) == self.default_value {
            // Nothing to write!
            return;
        }

        writer.write(&self.name, self.tag_class, self.tag, || {
            self.codec.encode(writer, value);
        });
    }
}

impl<T> fmt::Debug for BasicDerAdapter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} [{}/{}]", self.name, self.tag_class, self.tag)
    }
}

impl<T> fmt::Display for BasicDerAdapter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}/{}]", self.name, self.tag_class, self.tag)
    }
}

impl<T> PartialEq for BasicDerAdapter<T>
where
    T: 'static + Clone + PartialEq + Send + Sync,
{
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.tag_class == other.tag_class
            && self.tag == other.tag
            && self.is_optional == other.is_optional
            && self.default_value == other.default_value
            && self.type_hint == other.type_hint
            // Note: Codec equality is not easily checkable in Rust via trait objects,
            // but in the original Kotlin data class, it would use the codec's hashCode/equals.
            // Since we use Arc<dyn Codec>, we assume identity or structural equality is handled
            // by the specific implementation if needed, but for BasicDerAdapter, we match the fields.
    }
}