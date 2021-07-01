use flexi_logger::{Duplicate, FileSpec, FlexiLoggerError, Logger, LoggerHandle};

pub fn init_logger(spec: &str, directory: &str) -> Result<LoggerHandle, FlexiLoggerError> {
    Logger::try_with_env_or_str(spec)?
        .format(flexi_logger::colored_detailed_format)
        .log_to_file(FileSpec::default().directory(directory))
        .duplicate_to_stdout(Duplicate::Info)
        .print_message()
        .start()
}
