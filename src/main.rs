mod args;
mod event;
mod log;
mod notification;

use args::PushoverConfig;
use log_extern::{error, info, warn};
use notification::Notification;
use reqwest::blocking::Response;
use std::thread;

use event::{
    fetch::{EventFetcher, FetchError},
    Event,
};

fn main() {
    let config = args::parse_config();

    let _logger_handle = log::init_logger(&config.log)
        .unwrap_or_else(|error| panic!("Failed to initialize logger: {}", error));

    info!("Creating EventFetcher...");

    let fetcher = EventFetcher::new()
        .unwrap_or_else(|error| exit(format!("Failed to create EventFetcher: {}", error).as_str()));

    info!(
        "Created EventFetcher. Loading local list of events from {:?}...",
        config.events_file()
    );

    // Try to load known events from local file. If it fails, then fetch the events and write them
    // to the file. If writing fails, then continue with in-memory list `stored_events`.
    let mut stored_events =
        event::deserialize_events(config.events_file()).unwrap_or_else(|error| {
            warn!("Failed to load local list of events: {}", error);
            warn!(
                "Fetching events and creating new local list at {:?} instead...",
                config.events_file()
            );

            let events = fetcher.fetch_all().unwrap_or_else(|error| {
                exit(format!("Failed to fetch events: {}", error).as_str())
            });

            info!(
                "Fetched events. Writing them to {:?}...",
                config.events_file()
            );

            if let Err(error) = event::serialize_events(&events, config.events_file()) {
                warn!(
                    "Failed to save fetched events to {:?}: {}",
                    config.events_file(),
                    error
                );
                warn!("Continuing without saving events to disk, only storing them in memory.");
            } else {
                info!("Wrote events to {:?}.", config.events_file());
            }

            events
        });

    // Continuously fetch events and compare to local list of events. If there are any new ones,
    // then send a push notification and update local list.
    let mut running = false;
    loop {
        if running {
            info!("Fetching again in 2 minutes.\n");
            thread::sleep(config.fetch_interval);
        } else {
            running = true;
            info!("Now running.");
        }

        info!("Fetching events...");

        let events = match fetcher.fetch_all() {
            Ok(events) => events,
            Err(FetchError::Request(error)) => {
                warn!("Failed to fetch events: {}", error);
                continue;
            }
            Err(FetchError::Parse(error)) => {
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

        send_push_notification(&diff, &config.pushover).unwrap_or_else(|error| {
            exit(format!("Failed to send push notification: {}", error).as_str())
        });

        info!("Sent push notification. Updating local list of events...");

        stored_events = events;
        if let Err(error) = event::serialize_events(&stored_events, config.events_file()) {
            exit(format!("Failed to serialize events: {}", error).as_str());
        }

        info!("Updated local list of events.");
    }
}

fn exit(message: &str) -> ! {
    error!("{}", message);
    panic!("{}", message)
}

fn send_push_notification(
    events: &Vec<&Event>,
    config: &PushoverConfig,
) -> Result<Response, reqwest::Error> {
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
    Notification::new(config.api_key(), config.group_key(), &message[..])
        .title("Nye tider lagt op!")
        .html(true)
        .send()
}
