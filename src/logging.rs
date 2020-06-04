use log::{LevelFilter, Log, Metadata, Record};

struct GlobalLogger;

static LOGGER: GlobalLogger = GlobalLogger;

pub(crate) fn initialize() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .unwrap()
}

impl Log for GlobalLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        crate::println!("{} > {}", record.level(), record.args());
    }

    fn flush(&self) {}
}
