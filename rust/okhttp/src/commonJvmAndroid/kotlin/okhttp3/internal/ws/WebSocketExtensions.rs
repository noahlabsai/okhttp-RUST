use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Headers;
use std::io;
use crate::android_test_app::src::main::kotlin::okhttp::android::testapp::TestApplication::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;

// Helper trait to replicate Kotlin's String extension functions used in the source.
trait StringExtensions {
    fn delimiter_offset(&self, delimiter: char, start: usize, end: Option<usize>) -> usize;
    fn trim_substring(&self, start: usize, end: usize) -> String;
    fn remove_surrounding_quotes(&self) -> String;
}

impl StringExtensions for String {
    fn delimiter_offset(&self, delimiter: char, start: usize, end: Option<usize>) -> usize {
        let end = end.unwrap_or(self.len());
        if start >= self.len() {
            return self.len();
        }
        let bytes = self.as_bytes();
        for i in start..end {
            if bytes[i] as char == delimiter {
                return i;
            }
        }
        end
    }

    fn trim_substring(&self, start: usize, end: usize) -> String {
        if start >= end || start >= self.len() {
            return String::new();
        }
        let end = end.min(self.len());
        self[start..end].trim().to_string()
    }

    fn remove_surrounding_quotes(&self) -> String {
        if self.len() >= 2 && self.starts_with('\"') && self.ends_with('\"') {
            self[1..self.len() - 1].to_string()
        } else {
            self.clone()
        }
    }
}

// Models the contents of a `Sec-WebSocket-Extensions` response header.
#[derive(Debug, Clone, PartialEq)]
pub struct WebSocketExtensions {
    // True if the agreed upon extensions includes the permessage-deflate extension.
    pub per_message_deflate: bool,
    // Should be a value in [8..15]. Only 15 is acceptable by OkHttp as Java APIs are limited.
    pub client_max_window_bits: Option<i32>,
    // True if the agreed upon extension parameters includes "client_no_context_takeover".
    pub client_no_context_takeover: bool,
    // Should be a value in [8..15]. Any value in that range is acceptable by OkHttp.
    pub server_max_window_bits: Option<i32>,
    // True if the agreed upon extension parameters includes "server_no_context_takeover".
    pub server_no_context_takeover: bool,
    // True if the agreed upon extensions or parameters contained values unrecognized by OkHttp.
    pub unknown_values: bool,
}

impl WebSocketExtensions {
    pub fn new(
        per_message_deflate: bool,
        client_max_window_bits: Option<i32>,
        client_no_context_takeover: bool,
        server_max_window_bits: Option<i32>,
        server_no_context_takeover: bool,
        unknown_values: bool,
    ) -> Self {
        Self {
            per_message_deflate,
            client_max_window_bits,
            client_no_context_takeover,
            server_max_window_bits,
            server_no_context_takeover,
            unknown_values,
        }
    }

    pub fn no_context_takeover(&self, client_originated: bool) -> bool {
        if client_originated {
            self.client_no_context_takeover // Client is deflating.
        } else {
            self.server_no_context_takeover // Server is deflating.
        }
    }

    const HEADER_WEB_SOCKET_EXTENSION: &'static str = "Sec-WebSocket-Extensions";

    pub fn parse(response_headers: &Headers) -> io::Result<WebSocketExtensions> {
        // Note that this code does case-insensitive comparisons, even though the spec doesn't specify
        // whether extension tokens and parameters are case-insensitive or not.

        let mut compression_enabled = false;
        let mut client_max_window_bits: Option<i32> = None;
        let mut client_no_context_takeover = false;
        let mut server_max_window_bits: Option<i32> = None;
        let mut server_no_context_takeover = false;
        let mut unexpected_values = false;

        // Parse each header.
        for i in 0..response_headers.size() {
            if !response_headers.name(i).eq_ignore_ascii_case(Self::HEADER_WEB_SOCKET_EXTENSION) {
                continue; // Not a header we're interested in.
            }
            let header = response_headers.value(i);

            // Parse each extension.
            let mut pos = 0;
            while pos < header.len() {
                let extension_end = header.delimiter_offset(',', pos, None);
                let extension_token_end = header.delimiter_offset(';', pos, Some(extension_end));
                let extension_token = header.trim_substring(pos, extension_token_end);
                pos = extension_token_end + 1;

                if extension_token.eq_ignore_ascii_case("permessage-deflate") {
                    if compression_enabled {
                        unexpected_values = true; // Repeated extension!
                    }
                    compression_enabled = true;

                    // Parse each permessage-deflate parameter.
                    while pos < extension_end {
                        let parameter_end = header.delimiter_offset(';', pos, Some(extension_end));
                        let equals = header.delimiter_offset('=', pos, Some(parameter_end));
                        let name = header.trim_substring(pos, equals);
                        let value = if equals < parameter_end {
                            Some(header.trim_substring(equals + 1, parameter_end).remove_surrounding_quotes())
                        } else {
                            None
                        };
                        pos = parameter_end + 1;

                        if name.eq_ignore_ascii_case("client_max_window_bits") {
                            if client_max_window_bits.is_some() {
                                unexpected_values = true; // Repeated parameter!
                            }
                            client_max_window_bits = value.and_then(|v| v.parse::<i32>().ok());
                            if client_max_window_bits.is_none() {
                                unexpected_values = true; // Not an int!
                            }
                        } else if name.eq_ignore_ascii_case("client_no_context_takeover") {
                            if client_no_context_takeover {
                                unexpected_values = true; // Repeated parameter!
                            }
                            if value.is_some() {
                                unexpected_values = true; // Unexpected value!
                            }
                            client_no_context_takeover = true;
                        } else if name.eq_ignore_ascii_case("server_max_window_bits") {
                            if server_max_window_bits.is_some() {
                                unexpected_values = true; // Repeated parameter!
                            }
                            server_max_window_bits = value.and_then(|v| v.parse::<i32>().ok());
                            if server_max_window_bits.is_none() {
                                unexpected_values = true; // Not an int!
                            }
                        } else if name.eq_ignore_ascii_case("server_no_context_takeover") {
                            if server_no_context_takeover {
                                unexpected_values = true; // Repeated parameter!
                            }
                            if value.is_some() {
                                unexpected_values = true; // Unexpected value!
                            }
                            server_no_context_takeover = true;
                        } else {
                            unexpected_values = true; // Unexpected parameter.
                        }
                    }
                } else {
                    unexpected_values = true; // Unexpected extension.
                }
            }
        }

        Ok(WebSocketExtensions {
            per_message_deflate: compression_enabled,
            client_max_window_bits,
            client_no_context_takeover,
            server_max_window_bits,
            server_no_context_takeover,
            unknown_values: unexpected_values,
        })
    }
}
