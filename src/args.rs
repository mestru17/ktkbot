use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use clap::{crate_authors, crate_name, crate_version, App, ArgMatches};

#[derive(Debug)]
pub struct Config {
    pub log_level: String,
    log_directory: PathBuf,
    pub pushover_api_key: String,
    pub pushover_group_key: String,
    events_file: PathBuf,
    pub fetch_interval: Duration,
}

impl Config {
    pub fn log_directory(&self) -> &Path {
        &self.log_directory.as_path()
    }

    pub fn events_file(&self) -> &Path {
        &self.events_file.as_path()
    }
}

pub fn parse_args() -> Config {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!(",\n"))
        .about("Sends KTK event push notifications.")
        .args(&[
            argument::log_level(),
            argument::log_directory(),
            argument::pushover_api_key(),
            argument::pushover_group_key(),
            argument::events_file(),
            argument::fetch_interval(),
        ])
        .get_matches();

    Config {
        log_level: matches.parse(argument::LOG_LEVEL_NAME),
        log_directory: matches.parse(argument::LOG_DIRECTORY_NAME),
        pushover_api_key: matches.parse(argument::PUSHOVER_API_KEY_NAME),
        pushover_group_key: matches.parse(argument::PUSHOVER_GROUP_KEY_NAME),
        events_file: matches.parse(argument::EVENTS_FILE_NAME),
        fetch_interval: matches.parse(argument::FETCH_INTERVAL_NAME),
    }
}

trait Parse<T> {
    fn parse(&self, name: &str) -> T;
}

impl<'a> Parse<String> for ArgMatches<'a> {
    fn parse(&self, name: &str) -> String {
        self.value_of(name).unwrap().to_string()
    }
}

impl<'a> Parse<PathBuf> for ArgMatches<'a> {
    fn parse(&self, name: &str) -> PathBuf {
        PathBuf::from(self.value_of(name).unwrap())
    }
}

impl<'a> Parse<Duration> for ArgMatches<'a> {
    fn parse(&self, name: &str) -> Duration {
        Duration::from_secs(self.value_of(name).unwrap().parse().unwrap())
    }
}

mod argument {
    use clap::Arg;

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
            .validator(super::validate::length(1, 64))
    }

    pub fn pushover_api_key<'a>() -> Arg<'a, 'a> {
        Arg::with_name(PUSHOVER_API_KEY_NAME)
            .value_name("PUSHOVER API KEY")
            .help("The API key to use for sending Pushover notifications.")
            .required(true)
            .index(1)
            .validator(super::validate::pushover_key)
    }

    pub fn pushover_group_key<'a>() -> Arg<'a, 'a> {
        Arg::with_name(PUSHOVER_GROUP_KEY_NAME)
            .value_name("PUSHOVER GROUP KEY")
            .help("The group key to use for sending Pushover notifications.")
            .required(true)
            .index(2)
            .validator(super::validate::pushover_key)
    }

    pub fn events_file<'a>() -> Arg<'a, 'a> {
        Arg::with_name(EVENTS_FILE_NAME)
            .short("e")
            .long("events-file")
            .value_name("FILE")
            .help("Sets the file to save events to.")
            .takes_value(true)
            .default_value("events.json")
            .validator(super::validate::length(1, 64))
    }

    pub fn fetch_interval<'a>() -> Arg<'a, 'a> {
        Arg::with_name(FETCH_INTERVAL_NAME)
            .short("f")
            .long("fetch-interval")
            .value_name("SECONDS")
            .help("Sets the delay in between fetching events.")
            .takes_value(true)
            .default_value("120")
            .validator(super::validate::uint)
    }
}

mod validate {
    use lazy_static::lazy_static;
    use regex::Regex;

    pub fn length(min: usize, max: usize) -> impl Fn(String) -> Result<(), String> {
        move |s| {
            if s.len() < min || s.len() > max {
                return Err(format!(
                    "Invalid length - must be between {} and {} (inclusive) characters long",
                    min, max
                ));
            }
            Ok(())
        }
    }

    pub fn pushover_key(s: String) -> Result<(), String> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^[a-z0-9]{30}$").unwrap();
        }

        if !RE.is_match(s.as_str()) {
            return Err(String::from(
                "Invalid Pushover key - must consist of 30 alphanumeric characters",
            ));
        }
        Ok(())
    }

    pub fn uint(s: String) -> Result<(), String> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^[0-9]{1, 19}$").unwrap();
        }

        if !RE.is_match(s.as_str()) {
            return Err(String::from("Invalid uint - must consist of 1-19 digits"));
        }
        Ok(())
    }
}
