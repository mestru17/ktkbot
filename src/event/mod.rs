pub mod fetch;
pub mod parse;

use chrono::{DateTime, FixedOffset, TimeZone};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::HashSet,
    fs::File,
    hash::{Hash, Hasher},
    io::Write,
    path::Path,
};

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
