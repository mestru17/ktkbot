extern crate reqwest;
extern crate scraper;

use reqwest::blocking;

#[derive(Debug, Default)]
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
    let events = parse_events(document);
    println!("{:#?}", events);
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
