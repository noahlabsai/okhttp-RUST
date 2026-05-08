use std::collections::{HashMap, HashSet};
use std::fmt;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::url::Url::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::android_test::build_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;

// Mocking internal helper functions as they are not provided in the source but required for compilation
mod internal {
    pub fn can_parse_as_ip_address(host: &str) -> bool {
        false 
    }
    pub fn delimiter_offset(input: &str, delimiters: &str, start: usize, limit: usize) -> usize {
        for i in start..limit {
            if delimiters.contains(input.as_bytes()[i] as char) {
                return i;
            }
        }
        limit
    }
    pub fn index_of_first_non_ascii_whitespace(input: &str) -> usize {
        input.find(|c: char| !c.is_whitespace()).unwrap_or(0)
    }
    pub fn index_of_last_non_ascii_whitespace(input: &str, start: usize) -> usize {
        input[start..].rfind(|c: char| !c.is_whitespace()).map(|i| i + start).unwrap_or(input.len())
    }
}


impl HttpUrl {
    pub fn is_https(&self) -> bool {
        self.scheme == "https"
    }

    pub fn encoded_username(&self) -> String {
        if self.username.is_empty() {
            return String::new();
        }
        let username_start = self.scheme.len() + 3; // "://"
        let username_end = internal::delimiter_offset(&self.url, ":@", username_start, self.url.len());
        self.url[username_start..username_end].to_string()
    }

    pub fn encoded_password(&self) -> String {
        if self.password.is_empty() {
            return String::new();
        }
        let password_start = self.url[self.scheme.len() + 3..].find(':').map(|i| i + self.scheme.len() + 4).unwrap_or(0);
        let password_end = self.url.find('@').unwrap_or(self.url.len());
        self.url[password_start..password_end].to_string()
    }

    pub fn path_size(&self) -> usize {
        self.path_segments.len()
    }

    pub fn encoded_path(&self) -> String {
        let path_start = self.url[self.scheme.len() + 3..].find('/').map(|i| i + self.scheme.len() + 3).unwrap_or(self.url.len());
        let path_end = internal::delimiter_offset(&self.url, "?#", path_start, self.url.len());
        self.url[path_start..path_end].to_string()
    }

    pub fn encoded_path_segments(&self) -> Vec<String> {
        let path_start = self.url[self.scheme.len() + 3..].find('/').map(|i| i + self.scheme.len() + 3).unwrap_or(self.url.len());
        let path_end = internal::delimiter_offset(&self.url, "?#", path_start, self.url.len());
        let mut result = Vec::new();
        let mut i = path_start;
        while i < path_end {
            i += 1; // Skip '/'
            let segment_end = internal::delimiter_offset(&self.url, "/", i, path_end);
            result.push(self.url[i..segment_end].to_string());
            i = segment_end;
        }
        result
    }

    pub fn encoded_query(&self) -> Option<String> {
        if self.query_names_and_values.is_none() {
            return None;
        }
        let query_start = self.url.find('?').map(|i| i + 1).unwrap_or(self.url.len());
        let query_end = internal::delimiter_offset(&self.url, "#", query_start, self.url.len());
        Some(self.url[query_start..query_end].to_string())
    }

    pub fn query(&self) -> Option<String> {
        let qnv = self.query_names_and_values.as_ref()?;
        let mut result = String::new();
        Self::to_query_string(qnv, &mut result);
        Some(result)
    }

    pub fn query_size(&self) -> usize {
        self.query_names_and_values.as_ref().map(|v| v.len() / 2).unwrap_or(0)
    }

    pub fn query_parameter(&self, name: &str) -> Option<String> {
        let qnv = self.query_names_and_values.as_ref()?;
        for i in (0..qnv.len()).step_by(2) {
            if qnv[i].as_deref() == Some(name) {
                return qnv[i + 1].clone();
            }
        }
        None
    }

    pub fn query_parameter_names(&self) -> HashSet<String> {
        let mut result = HashSet::new();
        if let Some(qnv) = &self.query_names_and_values {
            for i in (0..qnv.len()).step_by(2) {
                if let Some(name) = &qnv[i] {
                    result.insert(name.clone());
                }
            }
        }
        result
    }

    pub fn query_parameter_values(&self, name: &str) -> Vec<Option<String>> {
        let mut result = Vec::new();
        if let Some(qnv) = &self.query_names_and_values {
            for i in (0..qnv.len()).step_by(2) {
                if qnv[i].as_deref() == Some(name) {
                    result.push(qnv[i + 1].clone());
                }
            }
        }
        result
    }

    pub fn query_parameter_name(&self, index: usize) -> String {
        let qnv = self.query_names_and_values.as_ref().expect("IndexOutOfBoundsException");
        qnv.get(index * 2).and_then(|v| v.as_ref()).cloned().expect("IndexOutOfBoundsException")
    }

    pub fn query_parameter_value(&self, index: usize) -> Option<String> {
        let qnv = self.query_names_and_values.as_ref().expect("IndexOutOfBoundsException");
        qnv.get(index * 2 + 1).and_then(|v| v.clone())
    }

    pub fn encoded_fragment(&self) -> Option<String> {
        self.fragment.as_ref().map(|_| {
            let fragment_start = self.url.find('#').map(|i| i + 1).unwrap_or(self.url.len());
            self.url[fragment_start..].to_string()
        })
    }

    pub fn redact(&self) -> String {
        self.new_builder_with_link("/...")
            .map(|mut b| {
                b.username("");
                b.password("");
                b.build().to_string()
            })
            .unwrap_or_else(|| "redacted".to_string())
    }

    pub fn resolve(&self, link: &str) -> Option<HttpUrl> {
        self.new_builder_with_link(link).and_then(|b| b.build_opt())
    }

    pub fn new_builder(&self) -> Builder {
        let mut builder = Builder::default();
        builder.scheme = Some(self.scheme.clone());
        builder.encoded_username = self.encoded_username();
        builder.encoded_password = self.encoded_password();
        builder.host = Some(self.host.clone());
        builder.port = if self.port != Self::default_port(&self.scheme) { self.port } else { -1 };
        builder.encoded_path_segments = self.encoded_path_segments();
        builder.encoded_query(Some(self.encoded_query().unwrap_or_default().as_str()));
        builder.encoded_fragment = self.encoded_fragment();
        builder
    }

    pub fn new_builder_with_link(&self, link: &str) -> Option<Builder> {
        Builder::parse(Some(self), link).ok()
    }

    pub fn top_private_domain(&self) -> Option<String> {
        if internal::can_parse_as_ip_address(&self.host) {
            None
        } else {
            PublicSuffixDatabase::get().lock().unwrap().get_effective_tld_plus_one(&self.host)
        }
    }

    pub fn default_port(scheme: &str) -> i32 {
        match scheme {
            "http" => 80,
            "https" => 443,
            _ => -1,
        }
    }

    fn to_query_string(qnv: &[Option<String>], out: &mut String) {
        for i in (0..qnv.len()).step_by(2) {
            let name = qnv[i].as_deref().unwrap_or("");
            let value = qnv[i + 1].as_deref();
            if i > 0 {
                out.push('&');
            }
            out.push_str(name);
            if let Some(v) = value {
                out.push('=');
                out.push_str(v);
            }
        }
    }
}

impl fmt::Display for HttpUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.url)
    }
}

impl PartialEq for HttpUrl {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}


impl Builder {
    pub fn new() -> Self {
        let mut b = Self::default();
        b.encoded_path_segments.push("".to_string());
        b
    }

    pub fn scheme(mut self, scheme: &str) -> Self {
        let s = scheme.to_lowercase();
        if s == "http" {
            self.scheme = Some("http".to_string());
        } else if s == "https" {
            self.scheme = Some("https".to_string());
        } else {
            panic!("unexpected scheme: {}", scheme);
        }
        self
    }

    pub fn username(mut self, username: &str) -> Self {
        self.encoded_username = username.canonicalize(USERNAME_ENCODE_SET, false);
        self
    }

    pub fn encoded_username(mut self, encoded_username: &str) -> Self {
        self.encoded_username = encoded_username.canonicalize(USERNAME_ENCODE_SET, true);
        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.encoded_password = password.canonicalize(PASSWORD_ENCODE_SET, false);
        self
    }

    pub fn encoded_password(mut self, encoded_password: &str) -> Self {
        self.encoded_password = encoded_password.canonicalize(PASSWORD_ENCODE_SET, true);
        self
    }

    pub fn host(mut self, host: &str) -> Self {
        let decoded = host.percent_decode().to_canonical_host();
        self.host = Some(decoded.expect("unexpected host"));
        self
    }

    pub fn port(mut self, port: i32) -> Self {
        if !(1..=65535).contains(&port) {
            panic!("unexpected port: {}", port);
        }
        self.port = port;
        self
    }

    pub fn add_path_segment(mut self, segment: &str) -> Self {
        self.push(segment, 0, segment.len(), false, false);
        self
    }

    pub fn add_path_segments(mut self, segments: &str) -> Self {
        let mut offset = 0;
        loop {
            let segment_end = internal::delimiter_offset(segments, "/\\", offset, segments.len());
            let add_trailing_slash = segment_end < segments.len();
            self.push(&segments[offset..segment_end], 0, segment_end - offset, add_trailing_slash, false);
            offset = segment_end + 1;
            if offset > segments.len() { break; }
        }
        self
    }

    pub fn add_encoded_path_segment(mut self, segment: &str) -> Self {
        self.push(segment, 0, segment.len(), false, true);
        self
    }

    pub fn add_encoded_path_segments(mut self, segments: &str) -> Self {
        let mut offset = 0;
        loop {
            let segment_end = internal::delimiter_offset(segments, "/\\", offset, segments.len());
            let add_trailing_slash = segment_end < segments.len();
            self.push(&segments[offset..segment_end], 0, segment_end - offset, add_trailing_slash, true);
            offset = segment_end + 1;
            if offset > segments.len() { break; }
        }
        self
    }

    pub fn set_path_segment(mut self, index: usize, segment: &str) -> Self {
        let canonical = segment.canonicalize(PATH_SEGMENT_ENCODE_SET, false);
        if self.is_dot(&canonical) || self.is_dot_dot(&canonical) {
            panic!("unexpected path segment: {}", segment);
        }
        if index < self.encoded_path_segments.len() {
            self.encoded_path_segments[index] = canonical;
        }
        self
    }

    pub fn remove_path_segment(mut self, index: usize) -> Self {
        if index < self.encoded_path_segments.len() {
            self.encoded_path_segments.remove(index);
        }
        if self.encoded_path_segments.is_empty() {
            self.encoded_path_segments.push("".to_string());
        }
        self
    }

    pub fn encoded_path(mut self, encoded_path: &str) -> Self {
        if !encoded_path.starts_with('/') {
            panic!("unexpected encodedPath: {}", encoded_path);
        }
        self.resolve_path(encoded_path, 0, encoded_path.len());
        self
    }

    pub fn query(mut self, query: Option<&str>) -> Self {
        self.encoded_query_names_and_values = query.map(|q| {
            q.canonicalize(QUERY_ENCODE_SET, false).to_query_names_and_values()
        });
        self
    }

    pub fn encoded_query(mut self, encoded_query: Option<&str>) -> Self {
        self.encoded_query_names_and_values = encoded_query.map(|q| {
            q.canonicalize(QUERY_ENCODE_SET, true).to_query_names_and_values()
        });
        self
    }

    pub fn add_query_parameter(mut self, name: &str, value: Option<&str>) -> Self {
        let mut qnv = self.encoded_query_names_and_values.get_or_insert_with(Vec::new);
        qnv.push(Some(name.canonicalize(QUERY_COMPONENT_ENCODE_SET, false)));
        qnv.push(value.map(|v| v.canonicalize(QUERY_COMPONENT_ENCODE_SET, false)));
        self
    }

    pub fn set_query_parameter(mut self, name: &str, value: Option<&str>) -> Self {
        self.remove_all_query_parameters(name);
        self.add_query_parameter(name, value)
    }

    pub fn remove_all_query_parameters(mut self, name: &str) -> Self {
        if let Some(qnv) = &mut self.encoded_query_names_and_values {
            let canonical_name = name.canonicalize(QUERY_COMPONENT_ENCODE_SET, false);
            let mut i = 0;
            while i < qnv.len() {
                if qnv[i].as_deref() == Some(&canonical_name) {
                    qnv.remove(i + 1);
                    qnv.remove(i);
                } else {
                    i += 2;
                }
            }
            if qnv.is_empty() {
                self.encoded_query_names_and_values = None;
            }
        }
        self
    }

    pub fn fragment(mut self, fragment: Option<&str>) -> Self {
        self.encoded_fragment = fragment.map(|f| f.canonicalize(FRAGMENT_ENCODE_SET, false));
        self
    }

    pub fn build(&self) -> HttpUrl {
        self.build_opt().expect("IllegalStateException")
    }

    fn build_opt(&self) -> Option<HttpUrl> {
        let scheme = self.scheme.clone()?;
        let host = self.host.clone()?;
        let port = if self.port != -1 { self.port } else { HttpUrl::default_port(&scheme) };
        
        let path_segments = self.encoded_path_segments.iter()
            .map(|s| s.percent_decode()).collect();
        
        let query_names_and_values = self.encoded_query_names_and_values.as_ref().map(|qnv| {
            qnv.iter().map(|v| v.as_ref().map(|s| s.percent_decode())).collect()
        });

        let fragment = self.encoded_fragment.as_ref().map(|f| f.percent_decode());
        let url = self.to_string();

        Some(HttpUrl {
            scheme,
            username: self.encoded_username.percent_decode(),
            password: self.encoded_password.percent_decode(),
            host,
            port,
            path_segments,
            query_names_and_values,
            fragment,
            url,
        })
    }

    fn push(&mut self, input: &str, _pos: usize, _limit: usize, add_trailing_slash: bool, already_encoded: bool) {
        let segment = input.canonicalize(PATH_SEGMENT_ENCODE_SET, already_encoded);
        if self.is_dot(&segment) { return; }
        if self.is_dot_dot(&segment) {
            self.pop();
            return;
        }
        if self.encoded_path_segments.last().map(|s| s.is_empty()).unwrap_or(false) {
            if let Some(last) = self.encoded_path_segments.last_mut() {
                *last = segment;
            }
        } else {
            self.encoded_path_segments.push(segment);
        }
        if add_trailing_slash {
            self.encoded_path_segments.push("".to_string());
        }
    }

    fn pop(&mut self) {
        if !self.encoded_path_segments.is_empty() {
            let removed = self.encoded_path_segments.pop().unwrap();
            if removed.is_empty() && !self.encoded_path_segments.is_empty() {
                if let Some(last) = self.encoded_path_segments.last_mut() {
                    *last = "".to_string();
                }
            } else {
                self.encoded_path_segments.push("".to_string());
            }
        }
    }

    fn is_dot(&self, input: &str) -> bool {
        input == "." || input.eq_ignore_ascii_case("%2e")
    }

    fn is_dot_dot(&self, input: &str) -> bool {
        input == ".." || input.eq_ignore_ascii_case("%2e.") || 
        input.eq_ignore_ascii_case(".%2e") || input.eq_ignore_ascii_case("%2e%2e")
    }

    fn resolve_path(&mut self, input: &str, start_pos: usize, limit: usize) {
        let mut pos = start_pos;
        if pos == limit { return; }
        let c = input.as_bytes()[pos] as char;
        if c == '/' || c == '\\' {
            self.encoded_path_segments.clear();
            self.encoded_path_segments.push("".to_string());
            pos += 1;
        } else {
            if let Some(last) = self.encoded_path_segments.last_mut() {
                *last = "".to_string();
            }
        }

        let mut i = pos;
        while i < limit {
            let path_segment_delimiter_offset = internal::delimiter_offset(input, "/\\", i, limit);
            let segment_has_trailing_slash = path_segment_delimiter_offset < limit;
            self.push(&input[i..path_segment_delimiter_offset], i, path_segment_delimiter_offset, segment_has_trailing_slash, true);
            i = path_segment_delimiter_offset;
            if segment_has_trailing_slash { i += 1; }
        }
    }

    pub fn parse(base: Option<&HttpUrl>, input: &str) -> Result<Self, String> {
        let mut builder = Builder::new();
        let mut pos = internal::index_of_first_non_ascii_whitespace(input);
        let limit = internal::index_of_last_non_ascii_whitespace(input, pos);

        let scheme_delimiter_offset = Builder::scheme_delimiter_offset(input, pos, limit);
        if scheme_delimiter_offset != -1 {
            let scheme_part = &input[pos..scheme_delimiter_offset as usize];
            if scheme_part.eq_ignore_ascii_case("http") {
                builder.scheme = Some("http".to_string());
                pos += "http:".len();
            } else if scheme_part.eq_ignore_ascii_case("https") {
                builder.scheme = Some("https".to_string());
                pos += "https:".len();
            } else {
                return Err(format!("Expected URL scheme 'http' or 'https' but was '{}'", scheme_part));
            }
        } else if let Some(b) = base {
            builder.scheme = Some(b.scheme.clone());
        } else {
            return Err("Expected URL scheme 'http' or 'https' but no scheme was found".to_string());
        }

        let mut slash_count = 0;
        while pos < limit && (input.as_bytes()[pos] as char == '/' || input.as_bytes()[pos] as char == '\\') {
            slash_count += 1;
            pos += 1;
        }

        if slash_count >= 2 || base.is_none() || base.unwrap().scheme != builder.scheme.as_ref().unwrap() {
            loop {
                let component_delimiter_offset = internal::delimiter_offset(input, "@/\\?#", pos, limit);
                if component_delimiter_offset == limit { break; }
                let c = input.as_bytes()[component_delimiter_offset] as char;
                if c == '@' {
                    let password_colon_offset = internal::delimiter_offset(input, ":", pos, component_delimiter_offset);
                    let username = input[pos..password_colon_offset].canonicalize(USERNAME_ENCODE_SET, true);
                    builder.encoded_username = if builder.encoded_username.is_empty() { username } else { format!("{}%40{}", builder.encoded_username, username) };
                    if password_colon_offset != component_delimiter_offset {
                        builder.encoded_password = input[password_colon_offset + 1..component_delimiter_offset].canonicalize(PASSWORD_ENCODE_SET, true);
                    }
                    pos = component_delimiter_offset + 1;
                } else {
                    let port_colon_offset = Builder::port_colon_offset(input, pos, component_delimiter_offset);
                    let host_part = input[pos..port_colon_offset].percent_decode().to_canonical_host();
                    builder.host = Some(host_part.expect("Invalid host"));
                    if port_colon_offset < component_delimiter_offset {
                        builder.port = Builder::parse_port(input, port_colon_offset + 1, component_delimiter_offset);
                    } else {
                        builder.port = HttpUrl::default_port(builder.scheme.as_ref().unwrap());
                    }
                    pos = component_delimiter_offset;
                    break;
                }
            }
        } else if let Some(b) = base {
            builder.encoded_username = b.encoded_username();
            builder.encoded_password = b.encoded_password();
            builder.host = Some(b.host.clone());
            builder.port = b.port;
            builder.encoded_path_segments = b.encoded_path_segments();
            if pos == limit || input.as_bytes()[pos] as char == '#' {
                builder.encoded_query(b.encoded_query().as_deref());
            }
        }

        let path_delimiter_offset = internal::delimiter_offset(input, "?#", pos, limit);
        builder.resolve_path(input, pos, path_delimiter_offset);
        pos = path_delimiter_offset;

        if pos < limit && input.as_bytes()[pos] as char == '?' {
            let query_delimiter_offset = internal::delimiter_offset(input, "#", pos, limit);
            builder.encoded_query_names_and_values = Some(
                input[pos + 1..query_delimiter_offset].canonicalize(QUERY_ENCODE_SET, true).to_query_names_and_values()
            );
            pos = query_delimiter_offset;
        }

        if pos < limit && input.as_bytes()[pos] as char == '#' {
            builder.encoded_fragment = Some(
                input[pos + 1..limit].canonicalize(FRAGMENT_ENCODE_SET, true)
            );
        }

        Ok(builder)
    }

    fn scheme_delimiter_offset(input: &str, pos: usize, limit: usize) -> i32 {
        if limit - pos < 2 { return -1; }
        let c0 = input.as_bytes()[pos] as char;
        if !c0.is_ascii_alphabetic() { return -1; }
        for i in pos + 1..limit {
            let c = input.as_bytes()[i] as char;
            if c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.' { continue; }
            if c == ':' { return i as i32; }
            return -1;
        }
        -1
    }

    fn port_colon_offset(input: &str, pos: usize, limit: usize) -> usize {
        let mut i = pos;
        while i < limit {
            let c = input.as_bytes()[i] as char;
            if c == '[' {
                while i < limit && input.as_bytes()[i] as char != ']' { i += 1; }
            } else if c == ':' {
                return i;
            }
            i += 1;
        }
        limit
    }

    fn parse_port(input: &str, pos: usize, limit: usize) -> i32 {
        let port_str = input[pos..limit].canonicalize("", false);
        port_str.parse::<i32>().unwrap_or(-1)
    }
}

impl fmt::Display for Builder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref s) = self.scheme {
            write!(f, "://{}", s)?;
        } else {
            write!(f, "//")?;
        }
        if !self.encoded_username.is_empty() || !self.encoded_password.is_empty() {
            write!(f, "{}", self.encoded_username)?;
            if !self.encoded_password.is_empty() {
                write!(f, ":{}", self.encoded_password)?;
            }
            write!(f, "@")?;
        }
        if let Some(ref h) = self.host {
            if h.contains(':') {
                write!(f, "[{}]", h)?;
            } else {
                write!(f, "{}", h)?;
            }
        }
        let effective_port = if self.port != -1 { self.port } else { 
            self.scheme.as_ref().map(|s| HttpUrl::default_port(s)).unwrap_or(-1) 
        };
        if self.port != -1 || self.scheme.is_some() {
            if self.scheme.is_none() || effective_port != HttpUrl::default_port(self.scheme.as_ref().unwrap()) {
                write!(f, ":{}", effective_port)?;
            }
        }
        for seg in &self.encoded_path_segments {
            write!(f, "/{}", seg)?;
        }
        if let Some(ref qnv) = self.encoded_query_names_and_values {
            write!(f, "?")?;
            let mut q_str = String::new();
            HttpUrl::to_query_string(qnv, &mut q_str);
            write!(f, "{}", q_str)?;
        }
        if let Some(ref frag) = self.encoded_fragment {
            write!(f, "#{}", frag)?;
        }
        Ok(())
    }
}

pub trait HttpUrlExt {
    fn to_http_url(&self) -> HttpUrl;
    fn to_http_url_or_null(&self) -> Option<HttpUrl>;
}

impl HttpUrlExt for str {
    fn to_http_url(&self) -> HttpUrl {
        Builder::parse(None, self).expect("Invalid URL").build()
    }
    fn to_http_url_or_null(&self) -> Option<HttpUrl> {
        Builder::parse(None, self).ok().map(|b| b.build())
    }
}
