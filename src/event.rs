use chrono::{DateTime, FixedOffset, TimeZone};
use reqwest::blocking::Client;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fmt::{self, Formatter},
    fs::File,
    hash::{Hash, Hasher},
    io::Write,
    path::Path,
};

const EVENT_SELECTOR: &str = "tr[class=\"infinite-item\"]";
const MAIN_INFO_SELECTOR: &str = "td[class=\"liste_wide min992\"]";
const CLASS_INFO_SELECTOR: &str = "td[class=\"liste_wide min992 holdinfo\"]";

pub fn serialize_events(
    events: &HashSet<Event>,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(&events)?;

    let mut file = File::create(&path)?;
    file.write_all(json.as_bytes())?;

    Ok(())
}

pub fn deserialize_events(path: &Path) -> Result<HashSet<Event>, Box<dyn std::error::Error>> {
    let file = File::open(&path)?;
    let events: HashSet<Event> = serde_json::from_reader(file)?;
    Ok(events)
}

#[derive(Debug, Serialize, Deserialize, Eq)]
pub struct Event {
    pub id: String,
    pub title: String,
    pub date_time: DateTime<FixedOffset>,
    pub class_info: Vec<String>,
}

impl Event {
    pub fn new() -> Event {
        Event {
            id: String::default(),
            title: String::default(),
            date_time: FixedOffset::east(2 * 3600)
                .ymd(2021, 6, 30)
                .and_hms(0, 0, 0),
            class_info: Vec::default(),
        }
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Event {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Event) -> Option<Ordering> {
        if self.id == other.id {
            Some(Ordering::Equal)
        } else {
            self.date_time.partial_cmp(&other.date_time)
        }
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.id == other.id {
            Ordering::Equal
        } else {
            self.date_time.cmp(&other.date_time)
        }
    }
}

pub struct EventParser {
    event_selector: Selector,
    main_info_selector: Selector,
    class_info_selector: Selector,
    month_lookup: HashMap<String, u32>,
}

impl EventParser {
    pub fn new() -> EventParser {
        EventParser {
            event_selector: Selector::parse(EVENT_SELECTOR).unwrap(),
            main_info_selector: Selector::parse(MAIN_INFO_SELECTOR).unwrap(),
            class_info_selector: Selector::parse(CLASS_INFO_SELECTOR).unwrap(),
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

    pub fn parse_all(&self, document: Html) -> Result<HashSet<Event>, ParseError> {
        document
            .select(&self.event_selector)
            .map(|row| self.parse_one(row))
            .collect()
    }

    pub fn parse_one(&self, row: ElementRef) -> Result<Event, ParseError> {
        let mut event: Event = Event::new();

        event.id = row
            .value()
            .attr("id")
            .ok_or(ParseError::from("No 'id' attribute in event HTML."))?
            .to_string();

        self.parse_main_info(row, &mut event)?;
        self.parse_class_info(row, &mut event);

        Ok(event)
    }

    fn parse_main_info(&self, row: ElementRef, event: &mut Event) -> Result<(), ParseError> {
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

            let (day, month, year): (u32, u32, i32) = match self.parse_day(&date[4..6]) {
                Ok(day) => (
                    day,
                    self.parse_month(&date[8..11])?,
                    self.parse_year(&date[12..])?,
                ),
                Err(_) => (
                    self.parse_day(&date[4..5])?,
                    self.parse_month(&date[7..10])?,
                    self.parse_year(&date[11..])?,
                ),
            };

            let hours = self.parse_hours(&time[..2])?;
            let minutes = self.parse_minutes(&time[3..5])?;

            event.date_time = FixedOffset::east(2 * 3600)
                .ymd(year, month, day)
                .and_hms(hours, minutes, 0);
        }

        Ok(())
    }

    fn parse_day(&self, s: &str) -> Result<u32, ParseError> {
        let day = s
            .parse()
            .map_err(|_| ParseError::from(format!("Failed to parse day from: '{}'", s)))?;
        Ok(day)
    }

    fn parse_month(&self, s: &str) -> Result<u32, ParseError> {
        let month = self
            .month_lookup
            .get(s)
            .ok_or(ParseError::from(format!(
                "No month matching pattern: '{}'",
                s
            )))?
            .to_owned();
        Ok(month)
    }

    fn parse_year(&self, s: &str) -> Result<i32, ParseError> {
        let year = s
            .parse()
            .map_err(|_| ParseError::from(format!("Failed to parse year from: '{}'", s)))?;
        Ok(year)
    }

    fn parse_hours(&self, s: &str) -> Result<u32, ParseError> {
        let hours = s
            .parse()
            .map_err(|_| ParseError::from(format!("Failed to parse hours from: '{}'", s)))?;
        Ok(hours)
    }

    fn parse_minutes(&self, s: &str) -> Result<u32, ParseError> {
        let minutes = s
            .parse()
            .map_err(|_| ParseError::from(format!("Failed to parse minutes from: '{}'", s)))?;
        Ok(minutes)
    }

    fn parse_class_info(&self, row: ElementRef, event: &mut Event) {
        for line in row.select(&self.class_info_selector) {
            let text = EventParser::parse_text(line);
            event.class_info = text.iter().map(|t| t.to_string()).collect();
        }
    }

    fn parse_text(line: ElementRef) -> Vec<&str> {
        line.text()
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .collect()
    }
}

#[derive(Debug)]
pub enum EventError {
    Parse(ParseError),
    Request(reqwest::Error),
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EventError::Parse(error) => write!(f, "Parse error: {}", error),
            EventError::Request(error) => write!(f, "Request error: {}", error),
        }
    }
}

impl From<reqwest::Error> for EventError {
    fn from(error: reqwest::Error) -> Self {
        EventError::Request(error)
    }
}

impl From<ParseError> for EventError {
    fn from(error: ParseError) -> Self {
        EventError::Parse(error)
    }
}

#[derive(Debug)]
pub struct ParseError {
    message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Failed to parse event: {}", self.message)
    }
}

impl From<String> for ParseError {
    fn from(message: String) -> Self {
        ParseError { message }
    }
}

impl From<&str> for ParseError {
    fn from(message: &str) -> Self {
        ParseError::from(message.to_string())
    }
}

pub struct EventFetcher {
    client: Client,
    parser: EventParser,
}

impl EventFetcher {
    pub fn new() -> Result<EventFetcher, reqwest::Error> {
        Ok(EventFetcher {
            client: Client::builder()
                .cookie_store(true) // Required to properly fetch all events
                .build()?,
            parser: EventParser::new(),
        })
    }

    pub fn fetch_all(&self) -> Result<HashSet<Event>, EventError> {
        let mut events: HashSet<Event> = HashSet::new();

        let mut i = 0;
        loop {
            // Fetch and parse event page as HTML
            let url = Self::events_url(i);
            let new_events = self.fetch(url.as_str())?;
            if new_events.is_subset(&events) {
                // No new events, so stop
                break;
            }
            events.extend(new_events);

            i += 1;
        }

        Ok(events)
    }

    pub fn fetch(&self, url: &str) -> Result<HashSet<Event>, EventError> {
        let response = self.client.get(url).send()?;
        let body = response.text()?;
        let document = Html::parse_document(&body);
        let events = self.parser.parse_all(document)?;
        Ok(events)
    }

    fn events_url(index: u32) -> String {
        if index == 0 {
            String::from("https://ktk-tennis.halbooking.dk/newlook/proc_liste.asp?pid=01")
        } else {
            format!("https://ktk-tennis.halbooking.dk/newlook/proc_liste.asp?liste=liste1&forrigetype=203&seson=0&scroll={}&pid=01", index - 1)
        }
    }
}
