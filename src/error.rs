use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct APIError {
    details: String,
}

impl APIError {
    pub fn new(msg: &str) -> APIError {
        APIError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for APIError {
    fn description(&self) -> &str {
        &self.details
    }
}
