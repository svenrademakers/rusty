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

#[no_mangle]
pub extern "C" fn log_error(message: *const ::std::os::raw::c_char) {
    unsafe {
        match std::ffi::CString::from_raw(message as *mut i8).into_string() {
            Ok(text) => error!("{}", text),
            Err(e) => println!("{}", e.to_string()),
        }
    }
}
