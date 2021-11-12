use log::{LevelFilter, Level, Metadata, Record};
use syslog::{Facility, Formatter3164, BasicLogger};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init_std_log(logstd: bool, appname: &str) -> Result<(), Box<dyn std::error::Error>> {
    
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: appname.to_owned(),
        pid: 0,
    };

    if !logstd {
        let logger = syslog::unix(formatter)?;
        log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
            .map(|()| log::set_max_level(LevelFilter::Info))?
    } else {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Debug))?
    }
    Ok(())
}