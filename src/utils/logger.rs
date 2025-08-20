pub use log::LevelFilter;
use log::{Level, Log, Metadata, Record};
use std::sync::{Mutex, OnceLock};
use text_colorizer::Colorize;

static LOGGER: OnceLock<Logger> = OnceLock::new();

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub level: Level,
    pub message: String,
}

pub struct Logger {
    buffer: Option<Mutex<Vec<LogEntry>>>,
}

unsafe impl Send for Logger {}
unsafe impl Sync for Logger {}

impl Logger {
    pub fn new(with_buffer: bool) -> Self {
        Logger {
            buffer: if with_buffer {
                Some(Mutex::new(Vec::new()))
            } else {
                None
            },
        }
    }

    fn add(&self, level: Level, msg: String) {
        if let Some(buf) = &self.buffer {
            buf.lock().unwrap().push(LogEntry {
                level,
                message: msg,
            });
        }
    }

    pub fn collect(&self) -> Vec<LogEntry> {
        if let Some(buf) = &self.buffer {
            std::mem::take(&mut *buf.lock().unwrap())
        } else {
            Vec::new()
        }
    }
}

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = format!("{}", record.args());

            match record.level() {
                Level::Info => println!("{message}"),
                Level::Warn => println!("{}", message.yellow()),
                Level::Error => println!("{}", message.red()),
                Level::Debug => println!("{}", message.magenta()),
                Level::Trace => println!("{}", message.blue()),
            }

            self.add(record.level(), message);
        }
    }

    fn flush(&self) {}
}

pub fn init_stdout(level: LevelFilter) {
    let logger = LOGGER.get_or_init(|| Logger::new(false));
    log::set_logger(logger)
        .map(|()| log::set_max_level(level))
        .expect("failed to init logger");
}

#[allow(dead_code)]
pub fn init_with_buffer(level: LevelFilter) {
    let logger = LOGGER.get_or_init(|| Logger::new(true));
    log::set_logger(logger)
        .map(|()| log::set_max_level(level))
        .expect("failed to init logger");
}

#[allow(dead_code)]
pub fn collect_logs() -> Vec<LogEntry> {
    LOGGER.get().map(|l| l.collect()).unwrap_or_default()
}
