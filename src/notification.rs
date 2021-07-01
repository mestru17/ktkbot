use reqwest::blocking::{Client, Response};
use serde::Serialize;

const PUSHOVER_API_URL: &str = "https://api.pushover.net/1/messages.json";

#[derive(Serialize)]
pub struct Notification {
    token: String,
    user: String,
    title: Option<String>,
    message: String,
    html: Option<u32>,      // Set to 1 to enable html in message
    monospace: Option<u32>, // Set to 1 to enable monospace font in message
}

impl Notification {
    pub fn builder(token: &str, user: &str, message: &str) -> NotificationBuilder {
        NotificationBuilder::new(token, user, message)
    }

    pub fn send(self) -> Result<Response, reqwest::Error> {
        Client::new().post(PUSHOVER_API_URL).form(&self).send()
    }
}

pub struct NotificationBuilder {
    token: String,
    user: String,
    title: Option<String>,
    message: String,
    html: Option<u32>,      // Set to 1 to enable html in message
    monospace: Option<u32>, // Set to 1 to enable monospace font in message
}

impl NotificationBuilder {
    pub fn new(token: &str, user: &str, message: &str) -> NotificationBuilder {
        NotificationBuilder {
            token: String::from(token),
            user: String::from(user),
            title: None,
            message: String::from(message),
            html: None,
            monospace: None,
        }
    }

    pub fn title(mut self, title: &str) -> NotificationBuilder {
        self.title = Some(String::from(title));
        self
    }

    pub fn html(mut self, html: bool) -> NotificationBuilder {
        self.html = if html { Some(1) } else { None };
        self
    }

    pub fn monospace(mut self, monospace: bool) -> NotificationBuilder {
        self.monospace = if monospace { Some(1) } else { None };
        self
    }

    pub fn build(self) -> Notification {
        Notification {
            token: self.token,
            user: self.user,
            title: self.title,
            message: self.message,
            html: self.html,
            monospace: self.monospace,
        }
    }
}
