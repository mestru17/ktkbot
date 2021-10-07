mod argument;
mod validate;

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use clap::{crate_authors, crate_name, crate_version, App, ArgMatches};

// const PUSHOVER_API_KEY: &str = "***REMOVED***"; // FIXME: Don't store api key in program
// const PUSHOVER_GROUP_KEY: &str = "***REMOVED***"; // FIXME: Don't store user key in program

#[derive(Debug)]
pub struct Config {
    pub log: LogConfig,
    pub pushover: PushoverConfig,
    events_file: PathBuf,
    pub fetch_interval: Duration,
}

impl Config {
    pub fn events_file(&self) -> &Path {
        self.events_file.as_path()
    }
}

#[derive(Debug)]
pub struct LogConfig {
    pub level: String,
    directory: PathBuf,
}

impl LogConfig {
    pub fn directory(&self) -> &Path {
        self.directory.as_path()
    }
}

#[derive(Debug)]
pub struct PushoverConfig {
    pub api_key: String,
    pub group_key: String,
}

pub fn parse_config() -> Config {
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
        log: LogConfig {
            level: matches.parse(argument::LOG_LEVEL_NAME),
            directory: matches.parse(argument::LOG_DIRECTORY_NAME),
        },
        pushover: PushoverConfig {
            api_key: matches.parse(argument::PUSHOVER_API_KEY_NAME),
            group_key: matches.parse(argument::PUSHOVER_GROUP_KEY_NAME),
        },
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
