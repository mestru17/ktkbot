use clap::Arg;

use super::validate;

pub const LOG_LEVEL_NAME: &str = "log_level";
pub const LOG_DIRECTORY_NAME: &str = "log_directory";
pub const PUSHOVER_API_KEY_NAME: &str = "pushover_api_key";
pub const PUSHOVER_GROUP_KEY_NAME: &str = "pushover_group_key";
pub const EVENTS_FILE_NAME: &str = "events_file";
pub const FETCH_INTERVAL_NAME: &str = "fetch_interval_seconds_name";

pub fn log_level<'a>() -> Arg<'a, 'a> {
    Arg::with_name(LOG_LEVEL_NAME)
        .short("l")
        .long("log-level")
        .value_name("LOG LEVEL")
        .help("Sets the level of logging.")
        .takes_value(true)
        .possible_values(&["info", "warn", "error"])
        .default_value("info")
}

pub fn log_directory<'a>() -> Arg<'a, 'a> {
    Arg::with_name(LOG_DIRECTORY_NAME)
        .short("d")
        .long("log-directory")
        .value_name("DIRECTORY")
        .help("Sets the directory to put log files in.")
        .takes_value(true)
        .default_value("logs")
        .validator(validate::length(1, 64))
}

pub fn pushover_api_key<'a>() -> Arg<'a, 'a> {
    Arg::with_name(PUSHOVER_API_KEY_NAME)
        .value_name("PUSHOVER API KEY")
        .help("The API key to use for sending Pushover notifications.")
        .required(true)
        .index(1)
        .validator(validate::pushover_key)
}

pub fn pushover_group_key<'a>() -> Arg<'a, 'a> {
    Arg::with_name(PUSHOVER_GROUP_KEY_NAME)
        .value_name("PUSHOVER GROUP KEY")
        .help("The group key to use for sending Pushover notifications.")
        .required(true)
        .index(2)
        .validator(validate::pushover_key)
}

pub fn events_file<'a>() -> Arg<'a, 'a> {
    Arg::with_name(EVENTS_FILE_NAME)
        .short("e")
        .long("events-file")
        .value_name("FILE")
        .help("Sets the file to save events to.")
        .takes_value(true)
        .default_value("events.json")
        .validator(validate::length(1, 64))
}

pub fn fetch_interval<'a>() -> Arg<'a, 'a> {
    Arg::with_name(FETCH_INTERVAL_NAME)
        .short("f")
        .long("fetch-interval")
        .value_name("SECONDS")
        .help("Sets the delay in between fetching events.")
        .takes_value(true)
        .default_value("120")
        .validator(validate::uint)
}
