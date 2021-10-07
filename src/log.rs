use flexi_logger::{
    Age, Cleanup, Criterion, Duplicate, FileSpec, FlexiLoggerError, Logger, LoggerHandle, Naming,
};

use crate::args::LogConfig;

pub fn init_logger(config: &LogConfig) -> Result<LoggerHandle, FlexiLoggerError> {
    Logger::try_with_env_or_str(&config.level)?
        .format(flexi_logger::colored_detailed_format)
        .log_to_file(FileSpec::default().directory(&config.directory()))
        .duplicate_to_stdout(Duplicate::Info)
        .print_message()
        .rotate(
            Criterion::Age(Age::Day), // Create a new log file every day
            Naming::Timestamps,       // Name log files using timestamps
            Cleanup::KeepLogFiles(7), // Keep only the 7 newest log files
        )
        .start()
}
