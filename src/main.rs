extern crate reqwest;
extern crate scraper;

use reqwest::blocking;

#[derive(Debug)]
struct Event {
    title: String,
    date: String,
    time: String,
}

impl Event {
    fn from_vec(v: &Vec<&str>) -> Option<Event> {
        match v.len() {
            3 => Some(Event {
                title: String::from(v[0]),
                date: String::from(v[1]),
                time: String::from(v[2]),
            }),
            5 => Some(Event {
                title: String::from(v[0]),
                date: String::from(v[3]),
                time: String::from(v[4]),
            }),
            _ => None,
        }
    }
}

fn main() {
    let response =
        blocking::get("https://ktk-tennis.halbooking.dk/newlook/proc_liste.asp?pid=01").unwrap();
    //println!("{:#?}", response);

    let body = response.text().unwrap();

    let document = scraper::Html::parse_document(&body);

    let events =
        scraper::Selector::parse("tr[class=\"infinite-item\"] > td[class=\"liste_wide min992\"]")
            .unwrap();

    for event in document.select(&events) {
        let text: Vec<_> = event.text().collect();
        let text: Vec<_> = text[1..].iter().map(|t| t.trim()).collect();
        let event = Event::from_vec(&text).unwrap();
        println!("{:?}", event);
    }
}
