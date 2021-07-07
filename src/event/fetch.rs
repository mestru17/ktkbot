use std::{
    collections::HashSet,
    fmt::{self, Formatter},
};

use reqwest::blocking::Client;
use scraper::Html;

use super::{
    parse::{EventParser, ParseError},
    Event,
};

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

    pub fn fetch_all(&self) -> Result<HashSet<Event>, FetchError> {
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

    pub fn fetch(&self, url: &str) -> Result<HashSet<Event>, FetchError> {
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

#[derive(Debug)]
pub enum FetchError {
    Parse(ParseError),
    Request(reqwest::Error),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FetchError::Parse(error) => write!(f, "Parse error: {}", error),
            FetchError::Request(error) => write!(f, "Request error: {}", error),
        }
    }
}

impl From<reqwest::Error> for FetchError {
    fn from(error: reqwest::Error) -> Self {
        FetchError::Request(error)
    }
}

impl From<ParseError> for FetchError {
    fn from(error: ParseError) -> Self {
        FetchError::Parse(error)
    }
}
