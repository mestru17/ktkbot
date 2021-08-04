use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Formatter},
};

use chrono::{FixedOffset, TimeZone};
use scraper::{ElementRef, Html, Selector};

use super::Event;

const EVENT_SELECTOR: &str = "tr[class=\"infinite-item\"]";
const MAIN_INFO_SELECTOR: &str = "td[class=\"liste_wide min992\"]";
const CLASS_INFO_SELECTOR: &str = "td[class=\"liste_wide min992 holdinfo\"]";

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

            // Parse day, month, and year
            let mut date_iter = date.split_whitespace();

            date_iter.next(); // skip name of day

            let day = date_iter.next().ok_or(ParseError::from(format!(
                "Failed to parse day: No more values in date iterator"
            )))?;
            let day: u32 = self.parse_day(&day[..(day.len() - 1)])?;

            let month = date_iter.next().ok_or(ParseError::from(format!(
                "Failed to parse month: No more values in date iterator"
            )))?;
            let month: u32 = self.parse_month(month)?;

            let year = date_iter.next().ok_or(ParseError::from(format!(
                "Failed to parse year: No more values in date iterator"
            )))?;
            let year: i32 = self.parse_year(year)?;

            // Parse time
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
