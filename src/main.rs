use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::thread;
use std::time;

use chrono::TimeZone;
use log::{error, info, warn};
use reqwest::blocking;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct Event {
    title: String,
    date_time: chrono::DateTime<chrono::FixedOffset>,
    class_info: Vec<String>,
}

impl Event {
    fn new() -> Event {
        Event {
            title: String::default(),
            date_time: chrono::FixedOffset::east(2 * 3600)
                .ymd(2021, 6, 30)
                .and_hms(0, 0, 0),
            class_info: Vec::default(),
        }
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Event) -> Option<cmp::Ordering> {
        self.date_time.partial_cmp(&other.date_time)
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.date_time.cmp(&other.date_time)
    }
}

struct EventParser {
    event_selector: scraper::Selector,
    main_info_selector: scraper::Selector,
    class_info_selector: scraper::Selector,
    month_lookup: HashMap<String, u32>,
}

impl EventParser {
    fn new() -> EventParser {
        EventParser {
            event_selector: scraper::Selector::parse(EVENT_SELECTOR).unwrap(),
            main_info_selector: scraper::Selector::parse(MAIN_INFO_SELECTOR).unwrap(),
            class_info_selector: scraper::Selector::parse(CLASS_INFO_SELECTOR).unwrap(),
            month_lookup: [
                "jan", "feb", "mar", "apr", "maj", "jun", "jul", "aug", "sep", "okt", "nov", "dec",
            ]
            .iter()
            .cloned()
            .map(|s| s.to_string())
            .zip((1..13).into_iter())
            .collect(),
        }
    }

    fn parse_all(&self, document: scraper::Html) -> Result<HashSet<Event>, ParseError> {
        document
            .select(&self.event_selector)
            .map(|row| self.parse_one(row))
            .collect()
    }

    fn parse_one(&self, row: scraper::ElementRef) -> Result<Event, ParseError> {
        let mut event: Event = Event::new();

        self.parse_main_info(row, &mut event)?;
        self.parse_class_info(row, &mut event);

        Ok(event)
    }

    fn parse_main_info(
        &self,
        row: scraper::ElementRef,
        event: &mut Event,
    ) -> Result<(), ParseError> {
        for line in row.select(&self.main_info_selector) {
            let text = EventParser::parse_text(line);

            let (title, date, time) = match text.len() {
                3 => (text[0], text[1], text[2]),
                5 => (text[0], text[3], text[4]),
                _ => {
                    return Err(format!(
                        "Expected event main info to have 3 or 5 lines, found {:?}",
                        text
                    ))?
                }
            };

            event.title = String::from(title);

            let (day, month, year): (u32, u32, i32) = match (&date[4..6]).parse() {
                Ok(day) => (
                    day,
                    self.month_lookup.get(&date[8..11]).unwrap().to_owned(),
                    (&date[12..]).parse().unwrap(),
                ),
                Err(_) => (
                    (&date[4..5]).parse().unwrap(),
                    self.month_lookup.get(&date[7..10]).unwrap().to_owned(),
                    (&date[11..]).parse().unwrap(),
                ),
            };

            let hours: u32 = (&time[..2]).parse().unwrap();
            let minutes: u32 = (&time[3..5]).parse().unwrap();

            event.date_time = chrono::FixedOffset::east(2 * 3600)
                .ymd(year, month, day)
                .and_hms(hours, minutes, 0);
        }

        Ok(())
    }

    fn parse_class_info(&self, row: scraper::ElementRef, event: &mut Event) {
        for line in row.select(&self.class_info_selector) {
            let text = EventParser::parse_text(line);
            event.class_info = text.iter().map(|t| t.to_string()).collect();
        }
    }

    fn parse_text(line: scraper::ElementRef) -> Vec<&str> {
        line.text()
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .collect()
    }
}

#[derive(Debug)]
struct ParseError {
    message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to parse event: {}", self.message)
    }
}

impl From<String> for ParseError {
    fn from(message: String) -> Self {
        ParseError { message }
    }
}

#[derive(Serialize)]
struct Notification {
    token: String,
    user: String,
    title: Option<String>,
    message: String,
    html: Option<u32>,      // Set to 1 to enable html in message
    monospace: Option<u32>, // Set to 1 to enable monospace font in message
}

impl Notification {
    fn builder(token: &str, user: &str, message: &str) -> NotificationBuilder {
        NotificationBuilder::new(token, user, message)
    }

    fn send(self) -> Result<blocking::Response, reqwest::Error> {
        blocking::Client::new()
            .post(PUSHOVER_API_URL)
            .form(&self)
            .send()
    }
}

struct NotificationBuilder {
    token: String,
    user: String,
    title: Option<String>,
    message: String,
    html: Option<u32>,      // Set to 1 to enable html in message
    monospace: Option<u32>, // Set to 1 to enable monospace font in message
}

impl NotificationBuilder {
    fn new(token: &str, user: &str, message: &str) -> NotificationBuilder {
        NotificationBuilder {
            token: String::from(token),
            user: String::from(user),
            title: None,
            message: String::from(message),
            html: None,
            monospace: None,
        }
    }

    fn title(mut self, title: &str) -> NotificationBuilder {
        self.title = Some(String::from(title));
        self
    }

    fn html(mut self, html: bool) -> NotificationBuilder {
        self.html = if html { Some(1) } else { None };
        self
    }

    fn monospace(mut self, monospace: bool) -> NotificationBuilder {
        self.monospace = if monospace { Some(1) } else { None };
        self
    }

    fn build(self) -> Notification {
        Notification {
            token: self.token,
            user: self.user,
            title: self.title,
            message: self.message,
            html: self.html,
            monospace: self.monospace,
        }
    }
}

const LOG_SPEC: &str = "info";
const LOG_DIRECTORY: &str = "logs";

const EVENTS_FILE: &str = "events.json";
const FETCH_INTERVAL_SECONDS: u64 = 120;

const PUSHOVER_API_URL: &str = "https://api.pushover.net/1/messages.json";
const PUSHOVER_API_KEY: &str = "***REMOVED***"; // FIXME: Don't store api key in program
const PUSHOVER_GROUP_KEY: &str = "***REMOVED***"; // FIXME: Don't store user key in program

const EVENT_SELECTOR: &str = "tr[class=\"infinite-item\"]";
const MAIN_INFO_SELECTOR: &str = "td[class=\"liste_wide min992\"]";
const CLASS_INFO_SELECTOR: &str = "td[class=\"liste_wide min992 holdinfo\"]";

fn main() {
    let _logger_handle = init_logger(LOG_SPEC, LOG_DIRECTORY)
        .unwrap_or_else(|error| panic!("Failed to initialize logger: {}", error));

    let events_file_path = Path::new(EVENTS_FILE);
    let fetch_interval = time::Duration::from_secs(FETCH_INTERVAL_SECONDS);

    let event_parser = EventParser::new();

    info!("Loading local list of events from {}...", EVENTS_FILE);

    // Try to load known events from local file. If it fails, then fetch the events and write them
    // to the file. If writing fails, then continue with in-memory list `stored_events`.
    let mut stored_events = deserialize_events(events_file_path).unwrap_or_else(|error| {
        warn!("Failed to load local list of events: {}", error);
        warn!(
            "Fetching events and creating new local list at {} instead...",
            EVENTS_FILE
        );

        let client = new_client().unwrap_or_else(crash(|error| {
            format!("Failed to create HTTP client: {}", error)
        }));
        let events = fetch_all_events(&client, &event_parser)
            .unwrap_or_else(crash(|error| format!("Failed to fetch events: {}", error)));

        info!("Fetched events. Writing them to {}...", EVENTS_FILE);

        if let Err(error) = serialize_events(&events, &events_file_path) {
            warn!(
                "Failed to save fetched events to {}: {}",
                EVENTS_FILE, error
            );
            warn!("Continuing without saving events to disk, only storing them in memory.");
        } else {
            info!("Wrote events to {}.", EVENTS_FILE);
        }

        events
    });

    // Continuously fetch events and compare to local list of events. If there are any new ones,
    // then send a push notification and update local list.
    let mut running = false;
    loop {
        if running {
            info!("Fetching again in 2 minutes.\n");
            thread::sleep(fetch_interval);
        } else {
            running = true;
            info!("Now running.");
        }

        info!("Creating HTTP client...");

        let client = new_client().unwrap_or_else(crash(|error| {
            format!("Failed to create HTTP client: {}", error)
        }));

        info!("Created HTTP client. Fetching events...");

        let events = fetch_all_events(&client, &event_parser)
            .unwrap_or_else(crash(|error| format!("Failed to fetch events: {}", error)));

        info!("Fetched events. Comparing to local list of events...");

        let mut diff: Vec<_> = events.difference(&stored_events).collect();

        if diff.is_empty() {
            info!("There are no new events.");
            continue;
        }

        info!(
            "There are {} new events. Sorting and sending push notification...",
            diff.len()
        );

        diff.sort();

        // TODO: Handle response - see [Being Friendly to our API](https://pushover.net/api#friendly)
        let response = send_push_notification(&diff).unwrap_or_else(crash(|error| {
            format!("Failed to send push notification: {}", error)
        }));

        info!("Sent push notification. Updating local list of events...");

        stored_events = events;
        if let Err(why) = serialize_events(&stored_events, &events_file_path) {
            error!("Failed to serialize events: {}", why);
            panic!();
        }

        info!("Updated local list of events.");
    }
}

fn init_logger(
    spec: &str,
    directory: &str,
) -> Result<flexi_logger::LoggerHandle, flexi_logger::FlexiLoggerError> {
    flexi_logger::Logger::try_with_env_or_str(spec)?
        .format(flexi_logger::colored_detailed_format)
        .log_to_file(flexi_logger::FileSpec::default().directory(directory))
        .duplicate_to_stdout(flexi_logger::Duplicate::Info)
        .print_message()
        .start()
}

fn crash<F, E, T>(f: F) -> impl Fn(E) -> T
where
    F: Fn(E) -> String,
    E: std::fmt::Display,
{
    move |error| {
        let message = f(error);
        error!("{}", message);
        panic!("{}", message);
    }
}

fn serialize_events(
    events: &HashSet<Event>,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(&events)?;

    let mut file = File::create(&path)?;
    file.write_all(json.as_bytes())?;

    Ok(())
}

fn deserialize_events(path: &Path) -> Result<HashSet<Event>, Box<dyn std::error::Error>> {
    let file = File::open(&path)?;
    let events: HashSet<Event> = serde_json::from_reader(file)?;
    Ok(events)
}

fn new_client() -> reqwest::Result<blocking::Client> {
    blocking::Client::builder()
        .cookie_store(true) // Required to properly fetch all events
        .build()
}

fn fetch_all_events(
    client: &blocking::Client,
    parser: &EventParser,
) -> Result<HashSet<Event>, ParseError> {
    let mut events: HashSet<Event> = HashSet::new();

    let mut i = 0;
    loop {
        // Fetch and parse event page as HTML
        let url = events_url(i);
        let new_events = fetch_events(url.as_str(), client, parser)?;
        if new_events.is_subset(&events) {
            // No new events, so stop
            break;
        }
        events.extend(new_events);

        i += 1;
    }

    Ok(events)
}

fn events_url(index: u32) -> String {
    if index == 0 {
        String::from("https://ktk-tennis.halbooking.dk/newlook/proc_liste.asp?pid=01")
    } else {
        format!("https://ktk-tennis.halbooking.dk/newlook/proc_liste.asp?liste=liste1&forrigetype=203&seson=0&scroll={}&pid=01", index - 1)
    }
}

fn fetch_events(
    url: &str,
    client: &blocking::Client,
    parser: &EventParser,
) -> Result<HashSet<Event>, ParseError> {
    let response = client.get(url).send().unwrap();
    let body = response.text().unwrap();
    let document = scraper::Html::parse_document(&body);
    parser.parse_all(document)
}

fn send_push_notification(events: &Vec<&Event>) -> Result<blocking::Response, reqwest::Error> {
    let mut message = String::from("<u>Der er blevet lagt nye tider op</u>:");
    for event in events {
        message.push_str(
            &format!(
                "\n- <b>{}</b>: {}",
                event.title,
                event.date_time.format("%a %e %b %Y"),
            )[..],
        );
    }
    Notification::builder(PUSHOVER_API_KEY, PUSHOVER_GROUP_KEY, &message[..])
        .title("Nye tider lagt op!")
        .html(true)
        .build()
        .send()
}
