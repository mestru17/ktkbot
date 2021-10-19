mod validate;

use std::{fmt, num::ParseIntError, str::FromStr, time::Duration};

use clap::{crate_authors, crate_name, crate_version, App, Arg, ArgMatches};

use crate::{Config, LogConfig, PushoverConfig};

macro_rules! all_args {
    () => {{
        &[
            Argument::LogLevel.into(),
            Argument::LogDirectory.into(),
            Argument::PushoverApiKey.into(),
            Argument::PushoverGroupKey.into(),
            Argument::EventsFile.into(),
            Argument::FetchInterval.into(),
        ]
    }};
}

pub fn parse_config() -> Config {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!(",\n"))
        .about("Sends KTK event push notifications.")
        .args(all_args!())
        .get_matches();

    Config {
        log: LogConfig {
            level: matches.parse_value(Argument::LogLevel),
            directory: matches.parse_value(Argument::LogDirectory),
        },
        pushover: PushoverConfig {
            api_key: matches.parse_value(Argument::PushoverApiKey),
            group_key: matches.parse_value(Argument::PushoverGroupKey),
        },
        events_file: matches.parse_value(Argument::EventsFile),
        fetch_interval: matches
            .parse_value::<DurationWrapper>(Argument::FetchInterval)
            .into(),
    }
}

enum Argument {
    LogLevel,
    LogDirectory,
    PushoverApiKey,
    PushoverGroupKey,
    EventsFile,
    FetchInterval,
}

impl Argument {
    fn name(&self) -> &'static str {
        match self {
            Self::LogLevel => "LogLevel",
            Self::LogDirectory => "LogDirectory",
            Self::PushoverApiKey => "PushoverApiKey",
            Self::PushoverGroupKey => "PushoverGroupKey",
            Self::EventsFile => "EventsFile",
            Self::FetchInterval => "FetchInterval",
        }
    }
}

impl From<Argument> for Arg<'_, '_> {
    fn from(argument: Argument) -> Self {
        match argument {
            Argument::LogLevel => Arg::with_name(argument.name())
                .short("l")
                .long("log-level")
                .value_name("LOG LEVEL")
                .help("Sets the level of logging.")
                .takes_value(true)
                .possible_values(&["error", "warn", "info", "debug", "trace"])
                .default_value("info"),
            Argument::LogDirectory => Arg::with_name(argument.name())
                .short("d")
                .long("log-directory")
                .value_name("DIRECTORY")
                .help("Sets the directory to put log files in.")
                .takes_value(true)
                .default_value("logs")
                .validator(validate::length(1, 64)),
            Argument::PushoverApiKey => Arg::with_name(argument.name())
                .value_name("PUSHOVER API KEY")
                .help("The API key to use for sending Pushover notifications.")
                .required(true)
                .index(1)
                .validator(validate::pushover_key),
            Argument::PushoverGroupKey => Arg::with_name(argument.name())
                .value_name("PUSHOVER GROUP KEY")
                .help("The group key to use for sending Pushover notifications.")
                .required(true)
                .index(2)
                .validator(validate::pushover_key),
            Argument::EventsFile => Arg::with_name(argument.name())
                .short("e")
                .long("events-file")
                .value_name("FILE")
                .help("Sets the file to save events to.")
                .takes_value(true)
                .default_value("events.json")
                .validator(validate::length(1, 64)),
            Argument::FetchInterval => Arg::with_name(argument.name())
                .short("f")
                .long("fetch-interval")
                .value_name("SECONDS")
                .help("Sets the delay in between fetching events.")
                .takes_value(true)
                .default_value("120")
                .validator(validate::uint),
        }
    }
}

trait ArgMatchesExt {
    fn value_of_unchecked(&self, name: Argument) -> &str;

    fn parse_value<T>(&self, name: Argument) -> T
    where
        T: FromStr,
        T::Err: fmt::Debug,
    {
        let value = self.value_of_unchecked(name);
        T::from_str(value).unwrap()
    }
}

impl ArgMatchesExt for ArgMatches<'_> {
    fn value_of_unchecked(&self, argument: Argument) -> &str {
        self.value_of(argument.name()).unwrap()
    }
}

struct DurationWrapper(Duration);

impl FromStr for DurationWrapper {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let secs: u64 = s.parse()?;
        Ok(Self(Duration::from_secs(secs)))
    }
}

impl From<DurationWrapper> for Duration {
    fn from(wrapper: DurationWrapper) -> Self {
        wrapper.0
    }
}
