/*
 * Copyright (C) 2022 Square, Inc.
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

// Equivalent to java.util.logging.LogRecord
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

#[derive(Debug, Clone, PartialEq)]
pub struct LogRecord {
    pub message: Option<String>,
}

// Equivalent to java.util.logging.SimpleFormatter
pub trait SimpleFormatter {
    fn format(&self, record: &LogRecord) -> String;
}

// MessageFormatter is a singleton object in Kotlin.
// In Rust, we implement this as a struct with a trait implementation.
pub struct MessageFormatter;

impl SimpleFormatter for MessageFormatter {
    fn format(&self, record: &LogRecord) -> String {
        // record.message is nullable in Kotlin (String?), so we handle it as Option<String>.
        // String.format("%s%n", record.message) in Kotlin handles nulls by printing "null".
        let msg = record.message.as_deref().unwrap_or("null");
        format!("{}\n", msg)
    }
}

// To maintain the "object" (singleton) behavior, we can provide a static instance.
pub static MESSAGE_FORMATTER: MessageFormatter = MessageFormatter;