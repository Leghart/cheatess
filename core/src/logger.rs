pub use log::LevelFilter;
use log::{Level, Log, Metadata, Record};
use std::sync::Once;
use text_colorizer::Colorize;

static INIT: Once = Once::new();

use crate::LOGGER;

pub struct Logger;

pub fn init(level: &LevelFilter) {
    INIT.call_once(|| {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(*level))
            .expect("Error with initialize logger");
    });
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
        }
    }

    fn flush(&self) {}
}
