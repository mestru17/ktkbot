extern crate reqwest;
extern crate scraper;
extern crate serde;

use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::thread;
use std::time;

use reqwest::blocking;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct Event {
    title: String,
    date: String,
    time: String,
    class_info: Vec<String>,
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

const EVENTS_FILE: &str = "events.json";
const FETCH_INTERVAL_SECONDS: u64 = 120;

const EVENT_SELECTOR: &str = "tr[class=\"infinite-item\"]";
const MAIN_INFO_SELECTOR: &str = "td[class=\"liste_wide min992\"]";
const CLASS_INFO_SELECTOR: &str = "td[class=\"liste_wide min992 holdinfo\"]";

fn main() {
    let file_path = Path::new(EVENTS_FILE);
    let fetch_interval = time::Duration::from_secs(FETCH_INTERVAL_SECONDS);

    loop {
        let client = blocking::Client::builder()
            .cookie_store(true) // Required to properly fetch all events
            .build()
            .unwrap();

        println!("Fetching events...");
        let events = match fetch_all_events(&client) {
            Err(why) => panic!("Failed to fetch events: {}", why),
            Ok(events) => events,
        };
        println!("Fetched events.");

        println!("Loading local list of events from {}...", EVENTS_FILE);
        match deserialize_events(&file_path) {
            Err(_) => {
                println!("Did not find a local list of events.");
                println!("Saving fetched events to {}...", EVENTS_FILE);
                if let Err(why) = serialize_events(&events, &file_path) {
                    panic!("Failed to serialize events: {}", why);
                }
            }
            Ok(stored_events) => {
                println!("Loaded local list of events from {}.", EVENTS_FILE);

                let diff: HashSet<_> = events.difference(&stored_events).collect();

                if !diff.is_empty() {
                    println!("New events: {:#?}", diff);
                    println!("Updating list of events...");
                    if let Err(why) = serialize_events(&events, &file_path) {
                        panic!("Failed to serialize events: {}", why);
                    }
                } else {
                    println!("No new events");
                }
            }
        };

        println!("Fetching again in 2 minutes...\n");
        thread::sleep(fetch_interval);
    }
}

fn fetch_all_events(client: &blocking::Client) -> Result<HashSet<Event>, ParseError> {
    let mut events: HashSet<Event> = HashSet::new();

    let mut i = 0;
    loop {
        // Fetch and parse event page as HTML
        let url = events_url(i);
        let new_events = fetch_events(&url[..], &client)?;
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

fn fetch_events(url: &str, client: &blocking::Client) -> Result<HashSet<Event>, ParseError> {
    let response = client.get(url).send().unwrap();
    let body = response.text().unwrap();
    let document = scraper::Html::parse_document(&body);
    parse_events(document)
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

fn parse_events(document: scraper::Html) -> Result<HashSet<Event>, ParseError> {
    // Use same selectors for each event
    let event_selector = scraper::Selector::parse(EVENT_SELECTOR).unwrap();
    let main_info_selector = scraper::Selector::parse(MAIN_INFO_SELECTOR).unwrap();
    let class_info_selector = scraper::Selector::parse(CLASS_INFO_SELECTOR).unwrap();

    document
        .select(&event_selector)
        .map(|row| parse_event(row, &main_info_selector, &class_info_selector))
        .collect()
}

fn parse_event(
    row: scraper::ElementRef,
    main_info_selector: &scraper::Selector,
    class_info_selector: &scraper::Selector,
) -> Result<Event, ParseError> {
    let mut event: Event = Default::default();

    parse_main_info(row, main_info_selector, &mut event)?;
    parse_class_info(row, class_info_selector, &mut event);

    Ok(event)
}

fn parse_main_info(
    row: scraper::ElementRef,
    selector: &scraper::Selector,
    event: &mut Event,
) -> Result<(), ParseError> {
    for line in row.select(&selector) {
        let text = parse_text(line);

        match text.len() {
            3 => {
                event.title = String::from(text[0]);
                event.date = String::from(text[1]);
                event.time = String::from(text[2]);
            }
            5 => {
                event.title = String::from(text[0]);
                event.date = String::from(text[3]);
                event.time = String::from(text[4]);
            }
            _ => {
                return Err(format!(
                    "Expected event main info to have 3 or 5 lines, found {:?}",
                    text
                ))?
            }
        }
    }

    Ok(())
}

fn parse_class_info(row: scraper::ElementRef, selector: &scraper::Selector, event: &mut Event) {
    for line in row.select(&selector) {
        let text = parse_text(line);
        event.class_info = text.iter().map(|t| t.to_string()).collect();
    }
}

fn parse_text(line: scraper::ElementRef) -> Vec<&str> {
    line.text()
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .collect()
}
