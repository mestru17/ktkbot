use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

const KEY_LENGTH: usize = 30;

/// An immutable Pushover key that is guaranteed to be formatted correctly.
///
/// The Pushover API requires so-called *API* and *group* keys. These consist of exactly 30
/// ASCII-alphanumeric characters. This invariant is guaranteed by the implementation of this type
/// by returning an error when trying to create an instance using an invalidly formatted string.
///
/// # Examples
///
/// You can create a `PushoverKey` from any type that implements [`Into<String>`] with
/// [`PushoverKey::new`]:
///
/// ```ignore
/// let valid_key = PushoverKey::new("somevalidkeydfghjklzxcvbnm0123").unwrap();
/// ```
///
/// Of course, use of [`unwrap`][`Result::unwrap`] is discouraged. Potential errors should be
/// handled:
///
/// ```ignore
/// match PushoverKey::new("somevalidkeydfghjklzxcvbnm0123") {
///     Ok(key) => {}, // use key...
///     Err(PushoverKeyError::InvalidLength(l)) => {}, // handle invalid length error...
///     Err(PushoverKeyError::NotAlphanumeric) => {}, // handle not alphanumeric error...
/// };
///
/// assert!(PushoverKey::new("tooshort").is_err());
/// assert!(PushoverKey::new("toolongdfghjklzxcvbnm0123dfghjklzxcvbnm0123").is_err());
/// assert!(PushoverKey::new("non_alpha_numeric!#¤%/(nm01234").is_err());
/// assert!(PushoverKey::new("valid30AlphaNumericCharacterss").is_ok());
/// ```
///
/// You can get the internal string value, which is guaranteed to formatted correctly, with the
/// [`get`][`PushoverKey::get`] method:
///
/// ```ignore
/// let valid_key_str = "somevalidkeydfghjklzxcvbnm0123";
///
/// let valid_key = PushoverKey::new(valid_key_str).unwrap();
///
/// assert_eq!(valid_key.get(), valid_key_str);
/// ```
///
/// Note that any uppercase ASCII letters are converted into lowercase during instantiation of a
/// `PushoverKey` as part of a sanitization step that is performed before validation:
///
/// ```ignore
/// let valid_key_str = "somevalidkeyWiThUppErCasem0123";
///
/// let valid_key = PushoverKey::new(valid_key_str).unwrap();
///
/// assert_ne!(valid_key.get(), valid_key_str);
/// assert_eq!(valid_key.get(), valid_key_str.to_ascii_lowercase());
/// ```
#[derive(Debug)]
pub struct PushoverKey {
    value: String,
}

impl PushoverKey {
    /// Creates a [`PushoverKey`] from any type that implements [`Into<String>`].
    ///
    /// # Errors
    ///
    /// If the string representation of the given type `raw` is not fromatted correctly - i.e. does
    /// not consist of exactly 30 ASCII-alphanumeric characters - then an error variant is
    /// returned.
    ///
    /// # Examples
    ///
    /// Create a key while handling any errors:
    ///
    /// ```
    /// # use ktkbot::notification::{PushoverKey, PushoverKeyError};
    /// let key = match PushoverKey::new("valid30AlphaNumericCharacters") {
    ///     Ok(key) => key,
    ///     Err(PushoverKeyError::InvalidLength(l)) => {
    ///         # PushoverKey::new("valid30AlphaNumericCharacterss").unwrap()
    ///         // handle invalid length error...
    ///     },
    ///     Err(PushoverKeyError::NotAlphanumeric) => {
    ///         # PushoverKey::new("valid30AlphaNumericCharacterss").unwrap()
    ///         // handle not alphanumeric error...
    ///     },
    /// };
    ///
    /// assert!(PushoverKey::new("tooshort").is_err());
    /// assert!(PushoverKey::new("toolongdfghjklzxcvbnm0123dfghjklzxcvbnm0123").is_err());
    /// assert!(PushoverKey::new("non_alpha_numeric!#¤%/(nm01234").is_err());
    /// assert!(PushoverKey::new("valid30AlphaNumericCharacterss").is_ok());
    /// ```
    pub fn new(raw: impl Into<String>) -> Result<Self, PushoverKeyError> {
        let mut key: String = raw.into();
        Self::sanitize(&mut key);
        Self::validate(&key)?;
        Ok(Self { value: key })
    }

    /// Gets the contained string representation of the key, which is guaranteed to be formatted
    /// correctly.
    ///
    /// # Examples
    ///
    /// Get the string representation:
    ///
    /// ```
    /// # use ktkbot::notification::{PushoverKey};
    /// let valid_key_str = "somevalidkeydfghjklzxcvbnm0123";
    ///
    /// let valid_key = PushoverKey::new(valid_key_str).unwrap();
    ///
    /// assert_eq!(valid_key.get(), valid_key_str);
    /// ```
    ///
    /// Note that any uppercase ASCII letters are converted into lowercase during instantiation of a
    /// `PushoverKey` as part of a sanitization step that is performed before validation:
    ///
    /// ```
    /// # use ktkbot::notification::{PushoverKey};
    /// let valid_key_str = "somevalidkeyWiThUppErCasem0123";
    ///
    /// let valid_key = PushoverKey::new(valid_key_str).unwrap();
    ///
    /// assert_ne!(valid_key.get(), valid_key_str);
    /// assert_eq!(valid_key.get(), valid_key_str.to_ascii_lowercase());
    /// ```
    pub fn get(&self) -> &str {
        &self.value
    }

    fn sanitize(raw: &mut String) {
        raw.make_ascii_lowercase();
    }

    fn validate(key: &str) -> Result<(), PushoverKeyError> {
        if key.len() != KEY_LENGTH {
            return Err(PushoverKeyError::InvalidLength(key.len()));
        }
        if !key.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(PushoverKeyError::NotAlphanumeric);
        }
        Ok(())
    }
}

impl FromStr for PushoverKey {
    type Err = PushoverKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PushoverKey::new(s)
    }
}

/// An error representing an invalidly formatted [`PushoverKey`].
#[derive(Debug)]
pub enum PushoverKeyError {
    InvalidLength(usize),
    NotAlphanumeric,
}

impl Display for PushoverKeyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength(l) => {
                write!(
                    f,
                    "Invalid pushover key length: {}. Expected {}.",
                    l, KEY_LENGTH
                )
            }
            Self::NotAlphanumeric => {
                write!(f, "Pushover key is not ASCII-alphanumeric.")
            }
        }
    }
}
