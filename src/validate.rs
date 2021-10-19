use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct Error {
    kind: String,
    message: String,
}

impl Error {
    pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Error {
            kind: kind.into(),
            message: message.into(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid {}: {}", self.kind, self.message)
    }
}
