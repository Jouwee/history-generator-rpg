use std::fmt::Display;

/// A simple in-game error
#[derive(Debug)]
pub struct Error {
    message: String
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_str(&self.message);
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self { message: value.to_string() }
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self { message: value.clone() }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self { message: value.to_string() }
    }
}