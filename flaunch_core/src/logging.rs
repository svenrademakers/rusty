pub use log::{debug, error, warn};
use log::{Level, Metadata, Record};
pub use log::{LevelFilter, SetLoggerError};

struct TerminalLogger;

impl log::Log for TerminalLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        println!(
            "{} {}:{}\t{}",
            &record.level().as_str()[..1],
            record.file().unwrap_or_default(),
            record.line().unwrap_or_default(),
            record.args()
        );
    }

    fn flush(&self) {}
}

static LOGGER: TerminalLogger = TerminalLogger;

pub fn init_logging(max_log_level: LevelFilter) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(max_log_level))
}

// #[macro_export]
// macro_rules! info {
//     ($msg:literal) => {
//         log::info!(concat!(file!(), ":", line!(), " | ", $msg));
//     };
//     ($msg:literal, $($opt:expr),*) => {
//         let message = format!($msg $(,$opt)*);
//         let logline = format!("{}:{} | {}", file!(), line!(), message);
//         log::info!("{}", logline);
//     };
// }
