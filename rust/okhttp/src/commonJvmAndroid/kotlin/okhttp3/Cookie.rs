use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http::{MAX_DATE, HttpDateStringExt};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::{\
    can_parse_as_ip_address, delimiter_offset, index_of_control_or_non_ascii, to_canonical_host,\
    trim_substring,\
};
use chrono::{TimeZone, Utc};
use regex::Regex;
use std::collections::HashMap;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::http::DateFormatting::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub expires_at: i64,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub persistent: bool,
    pub host_only: bool,
    pub same_site: Option<String>,
}

impl Cookie {
    pub fn matches(&self, url: &HttpUrl) -> bool {
        let domain_match = if self.host_only {
            url.host() == self.domain
        } else {
            Self::domain_match(url.host(), &self.domain)
        };
        if !domain_match {
            return false;
        }

        if !Self::path_match(url, &self.path) {
            return false;
        }

        !self.secure || url.is_https()
    }

    pub fn to_string_internal(&self, for_obsolete_rfc2965: bool) -> String {
        let mut s = String::new();
        s.push_str(&self.name);
        s.push('=');
        s.push_str(&self.value);

        if self.persistent {
            if self.expires_at == i64::MIN {
                s.push_str("; max-age=0");
            } else {
                let date = Utc.timestamp_millis_opt(self.expires_at).unwrap();
                s.push_str("; expires=");
                s.push_str(&date.to_http_date_string());
            }
        }

        if !self.host_only {
            s.push_str("; domain=");
            if for_obsolete_rfc2965 {
                s.push('.');
            }
            s.push_str(&self.domain);
        }

        s.push_str("; path=");
        s.push_str(&self.path);

        if self.secure {
            s.push_str("; secure");
        }

        if self.http_only {
            s.push_str("; httponly");
        }

        if let Some(ref ss) = self.same_site {
            s.push_str("; samesite=");
            s.push_str(ss);
        }

        s
    }

    pub fn new_builder(&self) -> Builder {
        Builder::from_cookie(self)
    }

    fn domain_match(url_host: &str, domain: &str) -> bool {
        if url_host == domain {
            return true;
        }
        url_host.ends_with(domain)
            && url_host.as_bytes().get(url_host.len().saturating_sub(domain.len() + 1)) == Some(&b'.')
            && !can_parse_as_ip_address(url_host)
    }

    fn path_match(url: &HttpUrl, path: &str) -> bool {
        let url_path = url.encoded_path();
        if url_path == path {
            return true;
        }
        if url_path.starts_with(path) {
            if path.ends_with('/') {
                return true;
            }
            if url_path.as_bytes().get(path.len()) == Some(&b'/') {
                return true;
            }
        }
        false
    }

    pub fn parse(url: &HttpUrl, set_cookie: &str) -> Option<Self> {
        Self::parse_internal(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
            url,
            set_cookie,
        )
    }

    fn parse_internal(current_time_millis: i64, url: &HttpUrl, set_cookie: &str) -> Option<Self> {
        let cookie_pair_end = delimiter_offset(set_cookie, ';', None);
        let pair_equals_sign = delimiter_offset(set_cookie, '=', Some(cookie_pair_end));
        if pair_equals_sign == cookie_pair_end {
            return None;
        }

        let cookie_name = trim_substring(set_cookie, 0, pair_equals_sign);
        if cookie_name.is_empty() || index_of_control_or_non_ascii(&cookie_name) != -1 {
            return None;
        }

        let cookie_value = trim_substring(set_cookie, pair_equals_sign + 1, cookie_pair_end);
        if index_of_control_or_non_ascii(&cookie_value) != -1 {
            return None;
        }

        let mut expires_at = MAX_DATE;
        let mut delta_seconds = -1i64;
        let mut domain: Option<String> = None;
        let mut path: Option<String> = None;
        let mut secure_only = false;
        let mut http_only = false;
        let mut host_only = true;
        let mut persistent = false;
        let mut same_site: Option<String> = None;

        let mut pos = cookie_pair_end + 1;
        let limit = set_cookie.len();
        while pos < limit {
            let attribute_pair_end = delimiter_offset(set_cookie, ';', Some(pos));
            let attr_eq = delimiter_offset_range(set_cookie, '=', pos, attribute_pair_end);

            let attribute_name = trim_substring_range(set_cookie, pos, attr_eq);
            let attribute_value = if attr_eq < attribute_pair_end {
                trim_substring_range(set_cookie, attr_eq + 1, attribute_pair_end)
            } else {
                String::new()
            };

            let name_lower = attribute_name.to_lowercase();
            match name_lower.as_str() {
                "expires" => {
                    if let Ok(val) = Self::parse_expires(&attribute_value) {
                        expires_at = val;
                        persistent = true;
                    }
                }
                "max-age" => {
                    if let Ok(val) = Self::parse_max_age(&attribute_value) {
                        delta_seconds = val;
                        persistent = true;
                    }
                }
                "domain" => {
                    if let Ok(val) = Self::parse_domain(&attribute_value) {
                        domain = Some(val);
                        host_only = false;
                    }
                }
                "path" => {
                    path = Some(attribute_value);
                }
                "secure" => {
                    secure_only = true;
                }
                "httponly" => {
                    http_only = true;
                }
                "samesite" => {
                    same_site = Some(attribute_value);
                }
                _ => {}
            }
            pos = attribute_pair_end + 1;
        }

        if delta_seconds == i64::MIN {
            expires_at = i64::MIN;
        } else if delta_seconds != -1 {
            let delta_milliseconds = if delta_seconds <= i64::MAX / 1000 {
                delta_seconds * 1000
            } else {
                i64::MAX
            };
            expires_at = current_time_millis + delta_milliseconds;
            if expires_at < current_time_millis || expires_at > MAX_DATE {
                expires_at = MAX_DATE;
            }
        }

        let url_host = url.host();
        let final_domain = match domain {
            None => url_host.to_string(),
            Some(d) => {
                if !Self::domain_match(url_host, &d) {
                    return None;
                }
                d
            }
        };

        if url_host.len() != final_domain.len()
            && PublicSuffixDatabase::get().lock().unwrap().get_effective_tld_plus_one(&final_domain).is_none()
        {
            return None;
        }

        let final_path = match path {
            Some(ref p) if p.starts_with('/') => p.clone(),
            _ => {
                let encoded_path = url.encoded_path();
                match encoded_path.rfind('/') {
                    Some(last_slash) if last_slash != 0 => encoded_path[..last_slash].to_string(),
                    _ => "/".to_string(),
                }
            }
        };

        Some(Cookie {
            name: cookie_name,
            value: cookie_value,
            expires_at,
            domain: final_domain,
            path: final_path,
            secure: secure_only,
            http_only,
            persistent,
            host_only,
            same_site,
        })
    }

    fn parse_expires(s: &str) -> Result<i64, Box<dyn std::error::Error>> {
        let time_pattern = Regex::new(r"(\d{1,2}):(\d{1,2}):(\d{1,2})[^\d]*").unwrap();
        let day_pattern = Regex::new(r"(\d{1,2})[^\d]*").unwrap();
        let month_pattern = Regex::new(r"(?i)(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec).*").unwrap();
        let year_pattern = Regex::new(r"(\d{2,4})[^\d]*").unwrap();

        let mut pos = Self::date_character_offset(s, 0, s.len(), false);
        let mut hour = -1;
        let mut minute = -1;
        let mut second = -1;
        let mut day_of_month = -1;
        let mut month = -1;
        let mut year = -1;

        while pos < s.len() {
            let end = Self::date_character_offset(s, pos + 1, s.len(), true);
            let slice = &s[pos..end];

            if hour == -1 {
                if let Some(caps) = time_pattern.captures(slice) {
                    hour = caps[1].parse::<i32>()?;
                    minute = caps[2].parse::<i32>()?;
                    second = caps[3].parse::<i32>()?;
                }
            }
            if day_of_month == -1 {
                if let Some(caps) = day_pattern.captures(slice) {
                    day_of_month = caps[1].parse::<i32>()?;
                }
            }
            if month == -1 {
                if let Some(caps) = month_pattern.captures(slice) {
                    let m_str = caps[1].to_lowercase();
                    let months = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];
                    if let Some(pos) = months.iter().position(|&x| x == m_str) {
                        month = (pos + 1) as i32;
                    }
                }
            }
            if year == -1 {
                if let Some(caps) = year_pattern.captures(slice) {
                    year = caps[1].parse::<i32>()?;
                }
            }
            pos = Self::date_character_offset(s, end + 1, s.len(), false);
        }

        if year >= 70 && year <= 99 {
            year += 1900;
        } else if year >= 0 && year <= 69 {
            year += 2000;
        }

        if year < 1601 || month == -1 || day_of_month < 1 || day_of_month > 31
            || hour < 0 || hour > 23 || minute < 0 || minute > 59 || second < 0 || second > 59
        {
            return Err("Invalid date components".into());
        }

        let dt = Utc
            .with_ymd_and_hms(year as i32, month as u32, day_of_month as u32, hour as u32, minute as u32, second as u32)
            .single()
            .ok_or("Invalid date")?;

        Ok(dt.timestamp_millis())
    }

    fn date_character_offset(input: &str, pos: usize, limit: usize, invert: bool) -> usize {
        for i in pos..limit {
            let c = input.as_bytes()[i] as i32;
            let is_date_char = (c < 32 && c != 9) || (c >= 127) || (c >= 48 && c <= 57)
                || (c >= 97 && c <= 122) || (c >= 65 && c <= 90) || (c == 58);
            if is_date_char == !invert {
                return i;
            }
        }
        limit
    }

    fn parse_max_age(s: &str) -> Result<i64, Box<dyn std::error::Error>> {
        match s.parse::<i64>() {
            Ok(parsed) => Ok(if parsed <= 0 { i64::MIN } else { parsed }),
            Err(e) => {
                let re = Regex::new(r"^-?\d+$").unwrap();
                if re.is_match(s) {
                    Ok(if s.starts_with('-') { i64::MIN } else { i64::MAX })
                } else {
                    Err(Box::new(e))
                }
            }
        }
    }

    fn parse_domain(s: &str) -> Result<String, Box<dyn std::error::Error>> {
        if s.ends_with('.') {
            return Err("Domain ends with dot".into());
        }
        let stripped = s.strip_prefix('.').unwrap_or(s);
        to_canonical_host(stripped).ok_or_else(|| "Invalid domain".into())
    }

    pub fn parse_all(url: &HttpUrl, headers: &Headers) -> Vec<Cookie> {
        headers
            .values("Set-Cookie")
            .iter()
            .filter_map(|s| Self::parse(url, s))
            .collect()
    }
}


impl Builder {
    pub fn new() -> Self {
        Self {
            name: None,
            value: None,
            expires_at: MAX_DATE,
            domain: None,
            path: "/".to_string(),
            secure: false,
            http_only: false,
            persistent: false,
            host_only: false,
            same_site: None,
        }
    }

    pub fn from_cookie(cookie: &Cookie) -> Self {
        Self {
            name: Some(cookie.name.clone()),
            value: Some(cookie.value.clone()),
            expires_at: cookie.expires_at,
            domain: Some(cookie.domain.clone()),
            path: cookie.path.clone(),
            secure: cookie.secure,
            http_only: cookie.http_only,
            persistent: cookie.persistent,
            host_only: cookie.host_only,
            same_site: cookie.same_site.clone(),
        }
    }

    pub fn name(&mut self, name: String) -> &mut Self {
        assert!(name.trim() == name, "name is not trimmed");
        self.name = Some(name);
        self
    }

    pub fn value(&mut self, value: String) -> &mut Self {
        assert!(value.trim() == value, "value is not trimmed");
        self.value = Some(value);
        self
    }

    pub fn expires_at(&mut self, expires_at: i64) -> &mut Self {
        let mut val = expires_at;
        if val <= 0 {
            val = i64::MIN;
        }
        if val > MAX_DATE {
            val = MAX_DATE;
        }
        self.expires_at = val;
        self.persistent = true;
        self
    }

    pub fn domain(&mut self, domain: String) -> &mut Self {
        self.domain_internal(domain, false)
    }

    pub fn host_only_domain(&mut self, domain: String) -> &mut Self {
        self.domain_internal(domain, true)
    }

    fn domain_internal(&mut self, domain: String, host_only: bool) -> &mut Self {
        let canonical = to_canonical_host(&domain).expect("unexpected domain");
        self.domain = Some(canonical);
        self.host_only = host_only;
        self
    }

    pub fn path(&mut self, path: String) -> &mut Self {
        assert!(path.starts_with('/'), "path must start with '/'");
        self.path = path;
        self
    }

    pub fn secure(&mut self) -> &mut Self {
        self.secure = true;
        self
    }

    pub fn http_only(&mut self) -> &mut Self {
        self.http_only = true;
        self
    }

    pub fn same_site(&mut self, same_site: String) -> &mut Self {
        assert!(same_site.trim() == same_site, "sameSite is not trimmed");
        self.same_site = Some(same_site);
        self
    }

    pub fn build(&self) -> Cookie {
        Cookie {
            name: self.name.clone().expect("builder.name == null"),
            value: self.value.clone().expect("builder.value == null"),
            expires_at: self.expires_at,
            domain: self.domain.clone().expect("builder.domain == null"),
            path: self.path.clone(),
            secure: self.secure,
            http_only: self.http_only,
            persistent: self.persistent,
            host_only: self.host_only,
            same_site: self.same_site.clone(),
        }
    }
}

fn delimiter_offset_range(s: &str, delim: char, start: usize, end: usize) -> usize {
    if let Some(pos) = s[start..end].find(delim) {
        start + pos
    } else {
        end
    }
}

fn trim_substring_range(s: &str, start: usize, end: usize) -> String {
    let slice = &s[start..end];
    slice.trim().to_string()
}

pub struct HttpUrl(String);
impl HttpUrl {
    pub fn host(&self) -> &str {
        ""
    }
    pub fn encoded_path(&self) -> &str {
        ""
    }
    pub fn is_https(&self) -> bool {
        true
    }
}

pub struct Headers(HashMap<String, Vec<String>>);
impl Headers {
    pub fn values(&self, _name: &str) -> Vec<String> {
        Vec::new()
    }
}
