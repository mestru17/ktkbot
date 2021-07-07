mod event;
mod log;
mod notification;

use log_extern::{error, info, warn};
use notification::Notification;
use reqwest::blocking::Response;
use std::{fmt::Display, path::Path, thread, time::Duration};

use event::{
    fetch::{EventError, EventFetcher},
    Event,
};

const LOG_SPEC: &str = "info";
const LOG_DIRECTORY: &str = "logs";

const PUSHOVER_API_KEY: &str = "***REMOVED***"; // FIXME: Don't store api key in program
const PUSHOVER_GROUP_KEY: &str = "***REMOVED***"; // FIXME: Don't store user key in program

const EVENTS_FILE: &str = "events.json";
const FETCH_INTERVAL_SECONDS: u64 = 120;

fn main() {
    let _logger_handle = log::init_logger(LOG_SPEC, LOG_DIRECTORY)
        .unwrap_or_else(|error| panic!("Failed to initialize logger: {}", error));

    let events_file_path = Path::new(EVENTS_FILE);
    let fetch_interval = Duration::from_secs(FETCH_INTERVAL_SECONDS);

    info!("Creating EventFetcher...");

    let fetcher = EventFetcher::new().unwrap_or_else(crash(|error| {
        format!("Failed to create EventFetcher: {}", error)
    }));

    info!(
        "Created EventFetcher. Loading local list of events from {}...",
        EVENTS_FILE
    );

    // Try to load known events from local file. If it fails, then fetch the events and write them
    // to the file. If writing fails, then continue with in-memory list `stored_events`.
    let mut stored_events = event::deserialize_events(events_file_path).unwrap_or_else(|error| {
        warn!("Failed to load local list of events: {}", error);
        warn!(
            "Fetching events and creating new local list at {} instead...",
            EVENTS_FILE
        );

        let events = fetcher
            .fetch_all()
            .unwrap_or_else(crash(|error| format!("Failed to fetch events: {}", error)));

        info!("Fetched events. Writing them to {}...", EVENTS_FILE);

        if let Err(error) = event::serialize_events(&events, &events_file_path) {
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

        info!("Fetching events...");

        let events = match fetcher.fetch_all() {
            Ok(events) => events,
            Err(EventError::Request(error)) => {
                warn!("Failed to fetch events: {}", error);
                continue;
            }
            Err(EventError::Parse(error)) => {
                exit(format!("Failed to fetch events: {}", error).as_str())
            }
        };

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

        send_push_notification(&diff).unwrap_or_else(crash(|error| {
            format!("Failed to send push notification: {}", error)
        }));

        info!("Sent push notification. Updating local list of events...");

        stored_events = events;
        if let Err(why) = event::serialize_events(&stored_events, &events_file_path) {
            error!("Failed to serialize events: {}", why);
            panic!();
        }

        info!("Updated local list of events.");
    }
}

fn exit(message: &str) -> ! {
    error!("{}", message);
    panic!("{}", message)
}

fn crash<F, E, T>(f: F) -> impl Fn(E) -> T
where
    F: Fn(E) -> String,
    E: Display,
{
    move |error| {
        let message = f(error);
        error!("{}", message);
        panic!("{}", message);
    }
}

fn send_push_notification(events: &Vec<&Event>) -> Result<Response, reqwest::Error> {
    let mut message = String::from("<u>Der er blevet lagt nye tider op</u>:");
    for event in events {
        message.push_str(
            format!(
                "\n- <b>{}</b>: {}",
                event.title,
                event.date_time.format("%a %e %b %Y"),
            )
            .as_str(),
        );
    }
    Notification::builder(PUSHOVER_API_KEY, PUSHOVER_GROUP_KEY, &message[..])
        .title("Nye tider lagt op!")
        .html(true)
        .build()
        .send()
}
