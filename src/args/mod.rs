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
        &self.events_file.as_path()
    }
}

#[derive(Debug)]
pub struct LogConfig {
    level: String,
    directory: PathBuf,
}

impl LogConfig {
    pub fn level(&self) -> &str {
        self.level.as_str()
    }

    pub fn directory(&self) -> &Path {
        &self.directory.as_path()
    }
}

#[derive(Debug)]
pub struct PushoverConfig {
    api_key: String,
    group_key: String,
}

impl PushoverConfig {
    pub fn api_key(&self) -> &str {
        self.api_key.as_str()
    }

    pub fn group_key(&self) -> &str {
        self.group_key.as_str()
    }
}

pub fn parse_config() -> Config {
    const LOG_LEVEL_NAME: &str = "log_level";
    const LOG_DIRECTORY_NAME: &str = "log_directory";
    const PUSHOVER_API_KEY_NAME: &str = "pushover_api_key";
    const PUSHOVER_GROUP_KEY_NAME: &str = "pushover_group_key";
    const EVENTS_FILE_NAME: &str = "events_file";
    const FETCH_INTERVAL_NAME: &str = "fetch_interval_seconds_name";

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!(",\n"))
        .about("Sends KTK event push notifications.")
        .args(&[
            argument::log_level(LOG_LEVEL_NAME),
            argument::log_directory(LOG_DIRECTORY_NAME),
            argument::pushover_api_key(PUSHOVER_API_KEY_NAME),
            argument::pushover_group_key(PUSHOVER_GROUP_KEY_NAME),
            argument::events_file(EVENTS_FILE_NAME),
            argument::fetch_interval(FETCH_INTERVAL_NAME),
        ])
        .get_matches();

    Config {
        log: LogConfig {
            level: matches.parse(LOG_LEVEL_NAME),
            directory: matches.parse(LOG_DIRECTORY_NAME),
        },
        pushover: PushoverConfig {
            api_key: matches.parse(PUSHOVER_API_KEY_NAME),
            group_key: matches.parse(PUSHOVER_GROUP_KEY_NAME),
        },
        events_file: matches.parse(EVENTS_FILE_NAME),
        fetch_interval: matches.parse(FETCH_INTERVAL_NAME),
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
