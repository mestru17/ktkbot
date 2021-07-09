use flexi_logger::{
    Age, Cleanup, Criterion, Duplicate, FileSpec, FlexiLoggerError, Logger, LoggerHandle, Naming,
};

pub fn init_logger(spec: &str, directory: &str) -> Result<LoggerHandle, FlexiLoggerError> {
    Logger::try_with_env_or_str(spec)?
        .format(flexi_logger::colored_detailed_format)
        .log_to_file(FileSpec::default().directory(directory))
        .duplicate_to_stdout(Duplicate::Info)
        .print_message()
        .rotate(
            Criterion::Age(Age::Day), // Create a new log file every day
            Naming::Timestamps,       // Name log files using timestamps
            Cleanup::KeepLogFiles(7), // Keep only the 7 newest log files
        )
        .start()
}
