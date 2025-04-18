use log::{Level, Log, SetLoggerError};

use crate::println;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&Logger)?;
    log::set_max_level(log::LevelFilter::Info);
    Ok(())
}

struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let lvl = record.level();
        let lvl_color = match lvl {
            Level::Error => "160",
            Level::Warn => "172",
            Level::Info => "47",
            Level::Debug => "25",
            Level::Trace => "103",
        };
        let module = record.module_path().unwrap_or_default();
        let line = record.line().unwrap_or_default();
        println!(
            "\x1b[38;5;{lvl_color}m{lvl}\x1b[0m [{module}:{line}]: {}\r\n",
            record.args(),
        )
    }

    fn flush(&self) {}
}
