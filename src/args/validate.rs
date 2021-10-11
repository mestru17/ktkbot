use lazy_static::lazy_static;
use regex::Regex;

pub fn length(min: usize, max: usize) -> Option<impl Fn(String) -> Result<(), String>> {
    if max < min {
        return None;
    }

    Some(move |s: String| {
        if s.len() < min || s.len() > max {
            return Err(format!(
                "Invalid length - must be between {} and {} (inclusive) characters long",
                min, max
            ));
        }
        Ok(())
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_range() {
        assert!(length(0, 2).is_some());
        assert!(length(1, 2).is_some());
        assert!(length(2, 2).is_some());
        assert!(length(3, 2).is_none());
        assert!(length(4, 2).is_none());
    }

    #[test]
    fn length_normal() {
        let f = length(2, 5).unwrap();
        assert!(f(String::from("1")).is_err());
        assert!(f(String::from("12")).is_ok());
        assert!(f(String::from("123")).is_ok());
        assert!(f(String::from("1234")).is_ok());
        assert!(f(String::from("12345")).is_ok());
        assert!(f(String::from("123456")).is_err());
    }

    #[test]
    fn pushover_key_length() {
        assert!(pushover_key(String::from("tooshortopasdfghjklzxcvbnm012")).is_err());
        assert!(pushover_key(String::from("validlengthsdfghjklzxcvbnm0123")).is_ok());
        assert!(pushover_key(String::from("toolongggthsdfghjklzxcvbnm01234")).is_err());
    }

    #[test]
    fn pushover_key_characters() {
        assert!(pushover_key(String::from("invalidiop!sdfghjklzxcvbnm0123")).is_err());
        assert!(pushover_key(String::from("invalidiopasdfghj&lzxcvbnm0123")).is_err());
        assert!(pushover_key(String::from("invalid#¤%&/()=?@£$€xcvbnm0123")).is_err());
        assert!(pushover_key(String::from("qwertyuiopasdfghjklzxcvbnm0123")).is_ok());
    }

    #[test]
    fn uint_length() {
        assert!(uint(String::from("")).is_err());
        assert!(uint(String::from("1")).is_ok());
        assert!(uint(String::from("12")).is_ok());
        assert!(uint(String::from("1234")).is_ok());
        assert!(uint(String::from("12345678")).is_ok());
        assert!(uint(String::from("1234567890123456789")).is_ok());
        assert!(uint(String::from("12345678901234567890")).is_err());
    }

    #[test]
    fn uint_characters() {
        assert!(uint(String::from("invalid")).is_err());
        assert!(uint(String::from("01234!6789")).is_err());
        assert!(uint(String::from("012#456789")).is_err());
        assert!(uint(String::from("0123456789")).is_ok());
    }
}
