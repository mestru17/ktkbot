use lazy_static::lazy_static;
use regex::Regex;

pub fn length(min: usize, max: usize) -> impl Fn(String) -> Result<(), String> {
    move |s| {
        if s.len() < min || s.len() > max {
            return Err(format!(
                "Invalid length - must be between {} and {} (inclusive) characters long",
                min, max
            ));
        }
        Ok(())
    }
}

pub fn pushover_key(s: String) -> Result<(), String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[a-z0-9]{30}$").unwrap();
    }

    if !RE.is_match(s.as_str()) {
        return Err(String::from(
            "Invalid Pushover key - must consist of 30 alphanumeric characters",
        ));
    }
    Ok(())
}

pub fn uint(s: String) -> Result<(), String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[0-9]{1, 19}$").unwrap();
    }

    if !RE.is_match(s.as_str()) {
        return Err(String::from("Invalid uint - must consist of 1-19 digits"));
    }
    Ok(())
}
