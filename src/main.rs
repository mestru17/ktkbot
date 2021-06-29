extern crate reqwest;
extern crate scraper;
extern crate serde;

use std::collections::HashSet;
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
        let events = fetch_all_events(&client);
        println!("Fetched events.");

        if !file_path.exists() {
            println!("Found no local list of events.");
            println!("Saving fetched events to {}...", EVENTS_FILE);
            serialize_events(&events, &file_path);
        } else {
            println!("Loading local list of events from {}...", EVENTS_FILE);
            let stored_events = deserialize_events(&file_path);
            println!("Loaded local list of events from {}.", EVENTS_FILE);

            let diff: HashSet<_> = events.difference(&stored_events).collect();

            if !diff.is_empty() {
                println!("New events: {:#?}", diff);
                println!("Updating list of events...");
                serialize_events(&events, &file_path);
            } else {
                println!("No new events");
            }
        }

        println!("Fetching again in 2 minutes...\n");
        thread::sleep(fetch_interval);
    }
}

fn fetch_all_events(client: &blocking::Client) -> HashSet<Event> {
    let mut events: HashSet<Event> = HashSet::new();

    let mut i = 0;
    loop {
        // Fetch and parse event page as HTML
        let url = events_url(i);
        let new_events = fetch_events(&url[..], &client);
        if new_events.is_subset(&events) {
            // No new events, so stop
            break;
        }
        events.extend(new_events);

        i += 1;
    }

    events
}

fn events_url(index: u32) -> String {
    if index == 0 {
        String::from("https://ktk-tennis.halbooking.dk/newlook/proc_liste.asp?pid=01")
    } else {
        format!("https://ktk-tennis.halbooking.dk/newlook/proc_liste.asp?liste=liste1&forrigetype=203&seson=0&scroll={}&pid=01", index - 1)
    }
}

fn fetch_events(url: &str, client: &blocking::Client) -> HashSet<Event> {
    let response = client.get(url).send().unwrap();
    let body = response.text().unwrap();
    let document = scraper::Html::parse_document(&body);
    parse_events(document)
}

fn serialize_events(events: &HashSet<Event>, path: &Path) {
    let display = path.display();

    let mut file = match File::create(&path) {
        Err(why) => panic!("Could not create {}: {}", display, why),
        Ok(file) => file,
    };

    match file.write_all(serde_json::to_string_pretty(&events).unwrap().as_bytes()) {
        Err(why) => panic!("Could not write to {}: {}", display, why),
        Ok(_) => println!("Successfully wrote events to {}", display),
    };
}

fn deserialize_events(path: &Path) -> HashSet<Event> {
    let file = match File::open(&path) {
        Err(why) => panic!("Could not open {}: {}", path.display(), why),
        Ok(file) => file,
    };

    serde_json::from_reader(file).unwrap()
}

fn parse_events(document: scraper::Html) -> HashSet<Event> {
    // Use same selectors for each event
    let event_selector = scraper::Selector::parse(EVENT_SELECTOR).unwrap();
    let main_info_selector = scraper::Selector::parse(MAIN_INFO_SELECTOR).unwrap();
    let class_info_selector = scraper::Selector::parse(CLASS_INFO_SELECTOR).unwrap();

    document
        .select(&event_selector)
        .map(|row| parse_event(row, &main_info_selector, &class_info_selector).unwrap())
        .collect()
}

fn parse_event(
    row: scraper::ElementRef,
    main_info_selector: &scraper::Selector,
    class_info_selector: &scraper::Selector,
) -> Option<Event> {
    let mut event: Event = Default::default();

    parse_main_info(row, main_info_selector, &mut event);
    parse_class_info(row, class_info_selector, &mut event);

    Some(event)
}

fn parse_main_info(row: scraper::ElementRef, selector: &scraper::Selector, event: &mut Event) {
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
            _ => panic!("Unknown number of lines in main info"),
        }
    }
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
