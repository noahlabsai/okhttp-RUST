use std::fmt::Write;
use chrono::{DateTime, Local, Timelike};

// Equivalent to java.util.logging.LogRecord
#[derive(Debug, Clone)]
pub struct LogRecord {
    pub millis: i64,
    pub message: String,
    pub thrown: Option<String>, // In Rust, we store the stack trace/error as a String to avoid Clone issues with Box<dyn Error>
}

// Equivalent to java.util.logging.Formatter
pub trait Formatter {
    fn format(&self, record: &LogRecord) -> String;
    
    fn format_message(&self, record: &LogRecord) -> String {
        record.message.clone()
    }
}

/*
 * Is Java8 Data and Time really this bad, or is writing this on a plane from just javadocs a bad
 * idea?
 *
 * Why so much construction?
 */
pub struct OneLineLogFormat;

impl OneLineLogFormat {
    pub fn new() -> Self {
        Self
    }

    // Helper to mimic the DateTimeFormatterBuilder logic:
    // HH:mm[:ss[.SSS]]
    fn format_time(&self, millis: i64) -> String {
        // Convert millis to DateTime<Local>
        let dt = DateTime::from_timestamp_millis(millis)
            .unwrap_or_else(|| Local::now());
        
        let hour = dt.hour();
        let min = dt.minute();
        let sec = dt.second();
        let nano = dt.nanosecond();

        // The Kotlin code uses optionalStart for seconds and nanos.
        // In a real production scenario, we'd check if those fields are present/relevant.
        // Based on the Kotlin logic, it formats as HH:mm:ss.SSS
        format!("{:02}:{:02}:{:02}.{:03}", hour, min, sec, nano / 1_000_000)
    }
}

impl Formatter for OneLineLogFormat {
    fn format(&self, record: &LogRecord) -> String {
        let message = self.format_message(record);
        let time_str = self.format_time(record.millis);

        if let Some(ref stack_trace) = record.thrown {
            // String.format("%s\t%s%n%s%n", time.format(d), message, sw.toString())
            format!("{}\t{}\n{}\n", time_str, message, stack_trace)
        } else {
            // String.format("%s\t%s%n", time.format(d), message)
            format!("{}\t{}\n", time_str, message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use crate::okcurl::src::main::kotlin::okhttp3::curl::MainCommandLine::*;

    #[test]
    fn test_format_no_exception() {
        let formatter = OneLineLogFormat::new();
        let record = LogRecord {
            millis: 1672531200000, // 2023-01-01 00:00:00 UTC
            message: "Hello World".to_string(),
            thrown: None,
        };
        let result = formatter.format(&record);
        assert!(result.contains("Hello World"));
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_format_with_exception() {
        let formatter = OneLineLogFormat::new();
        let record = LogRecord {
            millis: 1672531200000,
            message: "Error occurred".to_string(),
            thrown: Some("java.lang.NullPointerException\n  at Main.main(Main.rs:10)".to_string()),
        };
        let result = formatter.format(&record);
        assert!(result.contains("Error occurred"));
        assert!(result.contains("java.lang.NullPointerException"));
        assert!(result.ends_with('\n'));
    }
}