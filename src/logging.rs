pub use log::{debug, error, info, warn};
use log::{Level, Metadata, Record};
pub use log::{LevelFilter, SetLoggerError};

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

pub fn init_logging(max_log_level: LevelFilter) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(max_log_level))
}
