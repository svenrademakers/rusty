use log::{Level, Metadata, Record};
use log::{SetLoggerError, LevelFilter};
pub use log::{debug, info, warn, error};

struct TerminalLogger;

impl log::Log for TerminalLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        println!("{} - {}", record.level(), record.args());
    }

    fn flush(&self) {}
}

static LOGGER: TerminalLogger = TerminalLogger;

pub fn init_logging() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Debug))
}
