/*
 * Copyright (C) 2021 Square, Inc.
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

use std::fmt;
use std::io::Closeable; // Note: In Rust, this is typically handled by the Drop trait or a custom trait.

/// Mocking the OkHttpDebugLogging.enable functionality as it is a dependency
/// that interacts with JVM system properties and logging handlers.
pub struct OkHttpDebugLogging;
impl OkHttpDebugLogging {
    pub fn enable<H: LogHandler>(logger_name: &str, handler: H) -> Box<dyn Closeable> {
        // In a real JVM translation, this would register the handler with java.util.logging.Logger
        // For the purpose of this translation, we return a dummy Closeable.
        Box::new(DummyCloseable)
    }
}

struct DummyCloseable;
impl Closeable for DummyCloseable {
    fn close(&mut self) {
        // No-op
    }
}

/// Trait representing the java.util.logging.Handler behavior
pub trait LogHandler {
    fn publish(&self, record: LogRecord);
    fn flush(&self);
    fn close(&self);
}

/// Representation of java.util.logging.LogRecord
pub struct LogRecord {
    pub message: String,
    pub parameters: Option<Vec<Box<dyn std::any::Any>>>,
}

pub struct JsseDebugLogging;

impl JsseDebugLogging {
    #[derive(Debug, Clone, PartialEq)]
    pub struct JsseDebugMessage {
        pub message: String,
        pub param: Option<String>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Type {
        Handshake,
        Plaintext,
        Encrypted,
        Setup,
        Unknown,
    }

    impl JsseDebugMessage {
        pub fn get_type(&self) -> Type {
            if self.message == "adding as trusted certificates" {
                Type::Setup
            } else if self.message == "Raw read" || self.message == "Raw write" {
                Type::Encrypted
            } else if self.message == "Plaintext before ENCRYPTION" || self.message == "Plaintext after DECRYPTION" {
                Type::Plaintext
            } else if self.message.starts_with("System property ") {
                Type::Setup
            } else if self.message.starts_with("Reload ") {
                Type::Setup
            } else if self.message == "No session to resume." {
                Type::Handshake
            } else if self.message.starts_with("Consuming ") {
                Type::Handshake
            } else if self.message.starts_with("Produced ") {
                Type::Handshake
            } else if self.message.starts_with("Negotiated ") {
                Type::Handshake
            } else if self.message.starts_with("Found resumable session") {
                Type::Handshake
            } else if self.message.starts_with("Resuming session") {
                Type::Handshake
            } else if self.message.starts_with("Using PSK to derive early secret") {
                Type::Handshake
            } else {
                Type::Unknown
            }
        }
    }

    impl fmt::Display for JsseDebugMessage {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if let Some(ref param) = self.param {
                write!(f, "{}\n{}", self.message, param)
            } else {
                write!(f, "{}", self.message)
            }
        }
    }

    fn quiet_debug(message: JsseDebugMessage) {
        if message.message.starts_with("Ignore") {
            return;
        }

        match message.get_type() {
            Type::Setup | Type::Encrypted | Type::Plaintext => {
                println!("{} (skipped output)", message.message);
            }
            _ => {
                println!("{}", message);
            }
        }
    }

    pub fn enable_jsse_debug_logging<F>(debug_handler: Option<F>) -> Box<dyn Closeable>
    where
        F: Fn(JsseDebugMessage) + 'static,
    {
        // System.setProperty("javax.net.debug", "")
        std::env::set_var("javax.net.debug", "");

        let handler_fn = debug_handler.unwrap_or_else(|| {
            // Wrap the private quiet_debug in a closure
            Box::new(|msg| Self::quiet_debug(msg)) as Box<dyn Fn(JsseDebugMessage)>
        });

        // We need to wrap the handler in a struct that implements LogHandler
        struct JsseHandler<F: Fn(JsseDebugMessage)> {
            callback: F,
        }

        impl<F: Fn(JsseDebugMessage)> LogHandler for JsseHandler<F> {
            fn publish(&self, record: LogRecord) {
                let param = record.parameters
                    .and_then(|p| p.first().cloned())
                    .and_then(|any| any.downcast::<String>().ok().map(|s| (*s).clone()));
                
                (self.callback)(JsseDebugMessage {
                    message: record.message,
                    param,
                });
            }

            fn flush(&self) {}
            fn close(&self) {}
        }

        let jsse_handler = JsseHandler { callback: handler_fn };

        OkHttpDebugLogging::enable("javax.net.ssl", jsse_handler)
    }
}