use log::{Record, Level, Metadata, SetLoggerError, LevelFilter};

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{}: {}", record.level(), record.args());
        }
    }

    /// Means writing the log to the file bro.
    /// This will write to the file if enabled, and the database.
    fn flush(&self) {

    }
}


static LOGGER: Logger = Logger;

pub(crate) fn init(log_level: LevelFilter) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log_level))
}
