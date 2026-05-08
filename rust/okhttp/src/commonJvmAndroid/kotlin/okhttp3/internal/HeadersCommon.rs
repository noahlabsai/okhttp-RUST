use std::collections::HashMap;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers::Builder;

// This is the same as Chrome's limit.
pub const HEADER_LIMIT: i64 = 256 * 1024;

pub trait HeadersCommon {
    fn common_name(&self, index: i32) -> String;
    fn common_value(&self, index: i32) -> String;
    fn common_values(&self, name: &str) -> Vec<String>;
    fn common_iterator(&self) -> Vec<(String, String)>;
    fn common_new_builder(&self) -> Builder;
    fn common_equals(&self, other: &dyn std::any::Any) -> bool;
    fn common_hash_code(&self) -> i32;
    fn common_to_string(&self) -> String;
}

impl HeadersCommon for Headers {
    fn common_name(&self, index: i32) -> String {
        // Kotlin: namesAndValues.getOrNull(index * 2) ?: throw IndexOutOfBoundsException("name[$index]")
        self.name(index as usize)
    }

    fn common_value(&self, index: i32) -> String {
        // Kotlin: namesAndValues.getOrNull(index * 2 + 1) ?: throw IndexOutOfBoundsException("value[$index]")
        self.value(index as usize)
    }

    fn common_values(&self, name: &str) -> Vec<String> {
        let mut result: Option<Vec<String>> = None;
        for i in 0..self.size() {
            if name.eq_ignore_ascii_case(&self.name(i)) {
                let res = result.get_or_insert_with(|| Vec::with_capacity(2));
                res.push(self.value(i));
            }
        }
        result.unwrap_or_default()
    }

    fn common_iterator(&self) -> Vec<(String, String)> {
        let mut iter = Vec::with_capacity(self.size());
        for i in 0..self.size() {
            iter.push((self.name(i), self.value(i)));
        }
        iter
    }

    fn common_new_builder(&self) -> Builder {
        let mut builder = Builder::new();
        builder.add_all(self);
        builder
    }

    fn common_equals(&self, other: &dyn std::any::Any) -> bool {
        if let Some(other_headers) = other.downcast_ref::<Headers>() {
            if self.size() != other_headers.size() {
                return false;
            }
            for i in 0..self.size() {
                if self.name(i) != other_headers.name(i) || self.value(i) != other_headers.value(i) {
                    return false;
                }
            }
            return true;
        }
        false
    }

    fn common_hash_code(&self) -> i32 {
        let mut h = 0i32;
        for i in 0..self.size() {
            h = 31 * h + self.name(i).get_hash();
        }
        h
    }

    fn common_to_string(&self) -> String {
        let mut sb = String::new();
        for i in 0..self.size() {
            let name = self.name(i);
            let value = self.value(i);
            sb.push_str(&name);
            sb.push_str(": ");
            if is_sensitive_header(&name) {
                sb.push_str("\u{2588}\u{2588}");
            } else {
                sb.push_str(&value);
            }
            sb.push('\n');
        }
        sb
    }
}

pub fn common_headers_get(names_and_values: &[String], name: &str) -> Option<String> {
    if names_and_values.len() < 2 {
        return None;
    }
    for i in (0..names_and_values.len() - 1).step_by(2).rev() {
        if name.eq_ignore_ascii_case(&names_and_values[i]) {
            return Some(names_and_values[i + 1].clone());
        }
    }
    None
}

pub trait BuilderCommon {
    fn common_add(&mut self, name: String, value: String) -> &mut Self;
    fn common_add_all(&mut self, headers: &Headers) -> &mut Self;
    fn common_add_lenient(&mut self, name: String, value: String) -> &mut Self;
    fn common_remove_all(&mut self, name: String) -> &mut Self;
    fn common_set(&mut self, name: String, value: String) -> &mut Self;
    fn common_get(&self, name: String) -> Option<String>;
    fn common_build(&self) -> Headers;
}

impl BuilderCommon for Builder {
    fn common_add(&mut self, name: String, value: String) -> &mut Self {
        headers_check_name(&name);
        headers_check_value(&value, &name);
        self.common_add_lenient(name, value);
        self
    }

    fn common_add_all(&mut self, headers: &Headers) -> &mut Self {
        for i in 0..headers.size() {
            self.common_add_lenient(headers.name(i), headers.value(i));
        }
        self
    }

    fn common_add_lenient(&mut self, name: String, value: String) -> &mut Self {
        self.add(name, value.trim());
        self
    }

    fn common_remove_all(&mut self, name: String) -> &mut Self {
        self.remove_all(&name);
        self
    }

    fn common_set(&mut self, name: String, value: String) -> &mut Self {
        headers_check_name(&name);
        headers_check_value(&value, &name);
        self.remove_all(&name);
        self.common_add_lenient(name, value);
        self
    }

    fn common_get(&self, name: String) -> Option<String> {
        self.get(&name)
    }

    fn common_build(&self) -> Headers {
        self.build()
    }
}

pub fn headers_check_name(name: &str) {
    if name.is_empty() {
        panic!("name is empty");
    }
    for (i, c) in name.chars().enumerate() {
        if !('\u{0021}'..='\u{007e}').contains(&c) {
            panic!("Unexpected char 0x{} at {} in header name: {}", char_to_hex(c), i, name);
        }
    }
}

pub fn headers_check_value(value: &str, name: &str) {
    for (i, c) in value.chars().enumerate() {
        if c != '\t' && !('\u{0020}'..='\u{007e}').contains(&c) {
            let msg = format!("Unexpected char 0x{} at {} in {} value", char_to_hex(c), i, name);
            let full_msg = if is_sensitive_header(name) {
                msg
            } else {
                format!("{}: {}", msg, value)
            };
            panic!("{}", full_msg);
        }
    }
}

fn char_to_hex(c: char) -> String {
    let hex = format!("{:x}", c as u32);
    if hex.len() < 2 {
        format!("0{}", hex)
    } else {
        hex
    }
}

pub fn common_headers_of(input_names_and_values: &[String]) -> Headers {
    if input_names_and_values.len() % 2 != 0 {
        panic!("Expected alternating header names and values");
    }

    let mut builder = Builder::new();
    for i in (0..input_names_and_values.len()).step_by(2) {
        let name = input_names_and_values[i].trim().to_string();
        let value = input_names_and_values[i + 1].trim().to_string();
        headers_check_name(&name);
        headers_check_value(&value, &name);
        builder.add(&name, &value);
    }
    builder.build()
}

pub fn common_to_headers(map: &HashMap<String, String>) -> Headers {
    let mut builder = Builder::new();
    for (k, v) in map {
        let name = k.trim().to_string();
        let value = v.trim().to_string();
        headers_check_name(&name);
        headers_check_value(&value, &name);
        builder.add(&name, &value);
    }
    builder.build()
}

fn is_sensitive_header(name: &str) -> bool {
    name.eq_ignore_ascii_case("Authorization") || 
    name.eq_ignore_ascii_case("Cookie") || 
    name.eq_ignore_ascii_case("Set-Cookie")
}

trait Hashable {
    fn get_hash(&self) -> i32;
}

impl Hashable for String {
    fn get_hash(&self) -> i32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;

        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish() as i32
    }
}
