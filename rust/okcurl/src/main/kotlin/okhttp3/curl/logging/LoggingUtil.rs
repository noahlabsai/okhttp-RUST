use std::sync::Mutex;
use std::sync::OnceLock;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Protocol::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;
use crate::okcurl::src::main::kotlin::okhttp3::curl::logging::OneLineLogFormat::*;

// Mocking the Java Logging API as it is not natively available in Rust.
// In a production system, this would map to a crate like `log` or `tracing`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    ALL,
    FINEST,
    FINER,
    FINE,
    CONFIG,
    INFO,
    WARNING,
    SEVERE,
    OFF,
}

impl Default for Level {
    fn default() -> Self {
        Level::ALL
    }
}

pub const ALL: Level = Level::ALL;
pub const FINEST: Level = Level::FINEST;
pub const FINER: Level = Level::FINER;
pub const FINE: Level = Level::FINE;
pub const CONFIG: Level = Level::CONFIG;
pub const INFO: Level = Level::INFO;
pub const WARNING: Level = Level::WARNING;
pub const SEVERE: Level = Level::SEVERE;
pub const OFF: Level = Level::OFF;


pub trait Formatter: Send + Sync {
    fn format(&self, record: &LogRecord) -> String;
}

pub trait Handler: Send + Sync {
    fn publish(&self, record: &LogRecord);
    fn set_level(&self, level: Level);
    fn get_level(&self) -> Level;
    fn set_formatter(&self, formatter: Box<dyn Formatter>);
}

pub struct Logger {
    pub name: String,
    pub level: Mutex<Level>,
    pub handlers: Mutex<Vec<std::sync::Arc<dyn Handler>>>,
}

impl Logger {
    pub fn set_level(&self, level: Level) {
        let mut l = self.level.lock().unwrap();
        *l = level;
    }

    pub fn add_handler(&self, handler: std::sync::Arc<dyn Handler>) {
        let mut h = self.handlers.lock().unwrap();
        h.push(handler);
    }
}

struct LogManagerSim {
    loggers: Mutex<Vec<std::sync::Arc<Logger>>>,
}

impl LogManagerSim {
    fn get_logger(&self, name: &str) -> std::sync::Arc<Logger> {
        let mut loggers = self.loggers.lock().unwrap();
        for logger in loggers.iter() {
            if logger.name == name {
                return logger.clone();
            }
        }
        let logger = std::sync::Arc::new(Logger {
            name: name.to_string(),
            level: Mutex::new(Level::INFO),
            handlers: Mutex::new(Vec::new()),
        });
        loggers.push(logger.clone());
        logger
    }

    fn reset(&self) {
        let mut loggers = self.loggers.lock().unwrap();
        for logger in loggers.iter() {
            let mut handlers = logger.handlers.lock().unwrap();
            handlers.clear();
        }
    }
}

pub static LOG_MANAGER: OnceLock<LogManagerSim> = OnceLock::new();

fn get_log_manager() -> &'static LogManagerSim {
    LOG_MANAGER.get_or_init(|| LogManagerSim {
        loggers: Mutex::new(Vec::new()),
    })
}

struct OneLineLogFormat;
impl Formatter for OneLineLogFormat {
    fn format(&self, record: &LogRecord) -> String {
        format!("[{:?}] {}: {}", record.level, record.logger_name, record.message)
    }
}

struct MessageFormatter;
impl Formatter for MessageFormatter {
    fn format(&self, record: &LogRecord) -> String {
        record.message.clone()
    }
}

struct ConsoleHandler {
    level: Mutex<Level>,
    formatter: Mutex<Option<Box<dyn Formatter>>>,
}

impl ConsoleHandler {
    fn new() -> Self {
        Self {
            level: Mutex::new(Level::INFO),
            formatter: Mutex::new(None),
        }
    }
}

impl Handler for ConsoleHandler {
    fn publish(&self, record: &LogRecord) {
        let level = self.level.lock().unwrap();
        if record.level >= *level {
            let fmt = self.formatter.lock().unwrap();
            let output = if let Some(ref f) = *fmt {
                f.format(record)
            } else {
                record.message.clone()
            };
            println!("{}", output);
        }
    }

    fn set_level(&self, level: Level) {
        *self.level.lock().unwrap() = level;
    }

    fn get_level(&self) -> Level {
        *self.level.lock().unwrap()
    }

    fn set_formatter(&self, formatter: Box<dyn Formatter>) {
        *self.formatter.lock().unwrap() = Some(formatter);
    }
}

struct CustomConsoleHandler {
    inner: std::sync::Arc<ConsoleHandler>,
    ssl_debug: bool,
}

impl Handler for CustomConsoleHandler {
    fn publish(&self, record: &LogRecord) {
        self.inner.publish(record);

        if self.ssl_debug 
            && record.logger_name == "javax.net.ssl" 
            && record.parameters.is_some() 
        {
            if let Some(ref params) = record.parameters {
                if !params.is_empty() {
                    eprintln!("{}", params[0]);
                }
            }
        }
    }

    fn set_level(&self, level: Level) {
        self.inner.set_level(level);
    }

    fn get_level(&self) -> Level {
        self.inner.get_level()
    }

    fn set_formatter(&self, formatter: Box<dyn Formatter>) {
        self.inner.set_formatter(formatter);
    }
}

pub struct LoggingUtil;

static ACTIVE_LOGGERS: OnceLock<Mutex<Vec<std::sync::Arc<Logger>>>> = OnceLock::new();

impl LoggingUtil {
    fn active_loggers() -> &'static Mutex<Vec<std::sync::Arc<Logger>>> {
        ACTIVE_LOGGERS.get_or_init(|| Mutex::new(Vec::new()))
    }

    pub fn configure_logging(
        debug: bool,
        show_http2_frames: bool,
        ssl_debug: bool,
    ) {
        if debug || show_http2_frames || ssl_debug {
            if ssl_debug {
                std::env::set_var("javax.net.debug", "");
            }

            get_log_manager().reset();

            let handler_inner = std::sync::Arc::new(ConsoleHandler::new());
            let handler: std::sync::Arc<dyn Handler> = std::sync::Arc::new(CustomConsoleHandler {
                inner: handler_inner.clone(),
                ssl_debug,
            });

            if debug {
                handler.set_level(Level::ALL);
                handler.set_formatter(Box::new(OneLineLogFormat));
                
                let active_logger = Self::get_logger("");
                active_logger.add_handler(handler);
                active_logger.set_level(Level::ALL);

                Self::get_logger("jdk.event.security").set_level(Level::INFO);
                Self::get_logger("org.conscrypt").set_level(Level::INFO);
            } else {
                if show_http2_frames {
                    handler.set_level(Level::FINE);
                    handler.set_formatter(Box::new(MessageFormatter));
                    
                    let active_logger = Self::get_logger("okhttp3.internal.http2.Http2");
                    active_logger.set_level(Level::FINE);
                    active_logger.add_handler(handler.clone());
                }

                if ssl_debug {
                    handler.set_level(Level::FINEST);
                    handler.set_formatter(Box::new(MessageFormatter));
                    
                    let active_logger = Self::get_logger("javax.net.ssl");
                    active_logger.set_level(Level::FINEST);
                    active_logger.add_handler(handler);
                }
            }
        }
    }

    pub fn get_logger(name: &str) -> std::sync::Arc<Logger> {
        let logger = get_log_manager().get_logger(name);
        Self::active_loggers().lock().unwrap().push(logger.clone());
        logger
    }
}
