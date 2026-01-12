use std::fmt::Display;

#[derive(Debug)]
pub struct WobjError(String);

impl<I, E: Display> From<winnow::error::ParseError<I, E>> for WobjError {
    fn from(error: winnow::error::ParseError<I, E>) -> Self {
        Self(error.inner().to_string())
    }
}

impl std::fmt::Display for WobjError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for WobjError {}
