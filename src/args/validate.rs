use lazy_static::lazy_static;
use regex::Regex;

/// Creates a closure for validating the length of given strings.
///
/// Because this function panics, it is the caller's job to ensure that `min <= max`.
///
/// # Panics
///
/// Panics if the [`min`] value is greater than the [`max`] value.
///
/// # Examples
///
/// ```ignore
/// let validate = length(2, 4);
/// assert!(validate(String::from("1")).is_err());
/// assert!(validate(String::from("12")).is_ok());
/// assert!(validate(String::from("123")).is_ok());
/// assert!(validate(String::from("1234")).is_ok());
/// assert!(validate(String::from("12345")).is_err());
/// ```
pub fn length(min: usize, max: usize) -> impl Fn(String) -> Result<(), String> {
    if min > max {
        panic!("length max must be greater than or equal to min.");
    }

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

/// Checks that a given string is a valid Pushover key.
///
/// # Examples
///
/// ```ignore
/// let too_short = String::from("29charactersdfghjklzxcvbnm012");
/// assert!(validate::pushover_key(too_short).is_err());
///
/// let too_long = String::from("31charactersdfghjklzxcvbnm01234");
/// assert!(validate::pushover_key(too_long).is_err());
///
/// let invalid_character = String::from("invalidiop!sdfghjklzxcvbnm0123");
/// assert!(validate::pushover_key(invalid_character).is_err());
///
/// let valid = String::from("qwertyuiopasdfghjklzxcvbnm0123");
/// assert!(validate::pushover_key(valid).is_ok());
/// ```
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

/// Checks that a given string is a valid unsigned integer.
///
/// # Examples
///
/// ```ignore
/// let too_short = String::from("");
/// assert!(validate::uint(too_short).is_err());
///
/// let too_long = String::from("12345678901234567890");
/// assert!(validate::uint(too_long).is_err());
///
/// let invalid_character = String::from("-5435");
/// assert!(validate::uint(invalid_character).is_err());
///
/// let valid = String::from("1234567890123456789");
/// assert!(validate::uint(valid).is_ok());
/// ```
pub fn uint(s: String) -> Result<(), String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[0-9]{1, 19}$").unwrap();
    }

    if !RE.is_match(s.as_str()) {
        return Err(String::from("Invalid uint - must consist of 1-19 digits"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn length_invalid() {
        let _validate = length(4, 3);
    }

    #[test]
    fn length_valid() {
        let validate = length(2, 4);
        assert!(validate(String::from("1")).is_err());
        assert!(validate(String::from("12")).is_ok());
        assert!(validate(String::from("123")).is_ok());
        assert!(validate(String::from("1234")).is_ok());
        assert!(validate(String::from("12345")).is_err());
    }

    #[test]
    fn pushover_key_test() {
        let too_short = String::from("29charactersdfghjklzxcvbnm012");
        assert!(pushover_key(too_short).is_err());

        let too_long = String::from("31charactersdfghjklzxcvbnm01234");
        assert!(pushover_key(too_long).is_err());

        let invalid_character = String::from("invalidiop!sdfghjklzxcvbnm0123");
        assert!(pushover_key(invalid_character).is_err());

        let valid = String::from("qwertyuiopasdfghjklzxcvbnm0123");
        assert!(pushover_key(valid).is_ok());
    }

    #[test]
    fn uint_test() {
        let too_short = String::from("");
        assert!(uint(too_short).is_err());

        let too_long = String::from("12345678901234567890");
        assert!(uint(too_long).is_err());

        let invalid_character = String::from("-5435");
        assert!(uint(invalid_character).is_err());

        let valid = String::from("1234567890123456789");
        assert!(uint(valid).is_ok());
    }
}
