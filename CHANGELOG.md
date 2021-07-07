# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Thorough error handling when parsing events.
- This CHANGELOG.md file to hopefully better keep track of changes.

### Changed
- Change event comparison to go by event `id` instead of all fields.
- Continue running instead of crashing when there are network errors.
- Refactor event parsing to be more readable.
- Refactor whole event module to have clearer separation of concerns and a cleaner
  structure overall.

### Fixed
- Fix bug causing ktkbot to misclassify changed events as new events - i.e. when the
  number of available spots on an event would decrease because someone registered for
  it, ktkbot would see that as a new event and send a push notification.

## [0.10.1] - 2021-07-02
### Added
- .gitignore file to not track compiled binaries in the target/ folder.
- [reqwest](https://crates.io/crates/reqwest),
  [scraper](https://crates.io/crates/scraper), and
  [select](https://crates.io/crates/select) dependencies for web scraping.
- README.md file to provide info and track TODOs.
- Event parsing from HTML.
- Serialization and deserialization of events to JSON file using
  [serde](https://crates.io/crates/serde) with
  [serde\_json](https://crates.io/crates/serde_json).
- Detection and identification of new events compared to previously fetched ones.
- Scraping of all events - even dynamically loaded ones - from ktk web page.
- Sending push notifications via [Pushover](https://pushover.net/).
- Logging to both console and log files via the [log](https://crates.io/crates/log) and
  [flexi\_logger](https://crates.io/crates/flexi_logger) dependencies.
- In-memory caching of events to avoid having to deserialize previous ones on each fetch.
- Core loop consisting of continuously fetching events, checking if there are any new
  ones, and promptly sending push notifications when there are.
- Handling of errors instead of crashing with `unwrap()`.
- Sorting of events put in push notifications so that it is easy for people to get an
  overview.
- Handling of error codes from Pushover to be
  [friendly](https://pushover.net/api#friendly) to their API.
- `FIXME` comments where there are still unhandled errors.

[Unreleased]: https://github.com/mestru17/ktkbot/compare/v0.10.1...HEAD
[0.10.1]: https://github.com/mestru17/ktkbot/releases/tag/v0.10.1

