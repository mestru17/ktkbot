mod pushover_key;

use reqwest::blocking::{Client, Response};
use serde::Serialize;

pub use self::pushover_key::{PushoverKey, PushoverKeyError};

const PUSHOVER_API_URL: &str = "https://api.pushover.net/1/messages.json";

#[derive(Serialize)]
pub struct Notification<'a> {
    token: &'a str,
    user: &'a str,
    title: Option<&'a str>,
    message: &'a str,
    html: Option<u32>,      // Set to 1 to enable html in message
    monospace: Option<u32>, // Set to 1 to enable monospace font in message
}

impl<'a> Notification<'a> {
    pub fn new(api_key: &'a PushoverKey, group_key: &'a PushoverKey, message: &'a str) -> Self {
        Notification {
            token: api_key.get(),
            user: group_key.get(),
            title: None,
            message,
            html: None,
            monospace: None,
        }
    }

    pub fn title(&mut self, title: &'a str) -> &mut Self {
        self.title = Some(title);
        self
    }

    pub fn html(&mut self, html: bool) -> &mut Self {
        self.html = if html { Some(1) } else { None };
        self
    }

    pub fn monospace(&mut self, monospace: bool) -> &mut Self {
        self.monospace = if monospace { Some(1) } else { None };
        self
    }

    pub fn send(&self) -> Result<Response, reqwest::Error> {
        Client::new()
            .post(PUSHOVER_API_URL)
            .form(self)
            .send()?
            .error_for_status()
    }
}
