use clap::Arg;

use super::validate;

pub fn log_level(name: &str) -> Arg {
    Arg::with_name(name)
        .short("l")
        .long("log-level")
        .value_name("LOG LEVEL")
        .help("Sets the level of logging.")
        .takes_value(true)
        .possible_values(&["info", "warn", "error"])
        .default_value("info")
}

pub fn log_directory(name: &str) -> Arg {
    Arg::with_name(name)
        .short("d")
        .long("log-directory")
        .value_name("DIRECTORY")
        .help("Sets the directory to put log files in.")
        .takes_value(true)
        .default_value("logs")
        .validator(validate::length(1, 64).unwrap())
}

pub fn pushover_api_key(name: &str) -> Arg {
    Arg::with_name(name)
        .value_name("PUSHOVER API KEY")
        .help("The API key to use for sending Pushover notifications.")
        .required(true)
        .index(1)
        .validator(validate::pushover_key)
}

pub fn pushover_group_key(name: &str) -> Arg {
    Arg::with_name(name)
        .value_name("PUSHOVER GROUP KEY")
        .help("The group key to use for sending Pushover notifications.")
        .required(true)
        .index(2)
        .validator(validate::pushover_key)
}

pub fn events_file(name: &str) -> Arg {
    Arg::with_name(name)
        .short("e")
        .long("events-file")
        .value_name("FILE")
        .help("Sets the file to save events to.")
        .takes_value(true)
        .default_value("events.json")
        .validator(validate::length(1, 64).unwrap())
}

pub fn fetch_interval(name: &str) -> Arg {
    Arg::with_name(name)
        .short("f")
        .long("fetch-interval")
        .value_name("SECONDS")
        .help("Sets the delay in between fetching events.")
        .takes_value(true)
        .default_value("120")
        .validator(validate::uint)
}
