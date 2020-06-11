use log::{Level, LevelFilter, Log, Metadata, Record};

use crate::io::vga_buffer::color::*;

pub struct GlobalLogger;

static LOGGER: GlobalLogger = GlobalLogger;

pub fn initialize_logging() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Trace))
        .unwrap()
}

impl Log for GlobalLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    // TODO: Does creating color codes on the fly have a significant performance impact?
    fn log(&self, record: &Record) {
        match record.level() {
            Level::Trace => {
                crate::print_colored!(ColorCode::new(Color::Green, Color::Black), "TRACE > ")
            }
            Level::Debug => {
                crate::print_colored!(ColorCode::new(Color::Magenta, Color::Black), "DEBUG > ")
            }
            Level::Info => {
                crate::print_colored!(ColorCode::new(Color::Cyan, Color::Black), "INFO  > ")
            }
            Level::Warn => {
                crate::print_colored!(ColorCode::new(Color::Yellow, Color::Black), "WARN  > ")
            }
            Level::Error => {
                crate::print_colored!(ColorCode::new(Color::Red, Color::Black), "ERROR > ")
            }
        }
        crate::println!("{}", record.args());
    }

    fn flush(&self) {}
}
