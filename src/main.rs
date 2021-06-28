extern crate reqwest;
extern crate scraper;
extern crate serde;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use reqwest::blocking;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
struct Event {
    title: String,
    date: String,
    time: String,
    class_info: Vec<String>,
}

const EVENT_SELECTOR: &str = "tr[class=\"infinite-item\"]";
const MAIN_INFO_SELECTOR: &str = "td[class=\"liste_wide min992\"]";
const CLASS_INFO_SELECTOR: &str = "td[class=\"liste_wide min992 holdinfo\"]";

fn main() {
    // Fetch and parse event page as HTML
    let response =
        blocking::get("https://ktk-tennis.halbooking.dk/newlook/proc_liste.asp?pid=01").unwrap();
    let body = response.text().unwrap();
    let document = scraper::Html::parse_document(&body);

    // Parse and print events
    let mut events = parse_events(document);

    serialize_events(&events, &Path::new("events.json"));

    events.clear();

    events = deserialize_events(&Path::new("events.json"));

    println!("{:#?}", events);
}

fn serialize_events(events: &Vec<Event>, path: &Path) {
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

fn deserialize_events(path: &Path) -> Vec<Event> {
    let display = path.display();

    let file = match File::open("events.json") {
        Err(why) => panic!("Could not open {}: {}", display, why),
        Ok(file) => file,
    };

    serde_json::from_reader(file).unwrap()
}

fn parse_events(document: scraper::Html) -> Vec<Event> {
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
