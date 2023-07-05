use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ZoneError {
    details: String,
}
impl ZoneError {
    pub fn new(msg: &str) -> Self {
        Self {
            details: msg.to_string(),
        }
    }
}
impl fmt::Display for ZoneError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}
impl Error for ZoneError {
    fn description(&self) -> &str {
        &self.details
    }
}

#[derive(Debug)]
pub struct WateringError {
    details: String,
}
impl WateringError {
    pub fn new(msg: &str) -> Self {
        Self {
            details: msg.to_string(),
        }
    }
}
impl fmt::Display for WateringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}
impl Error for WateringError {
    fn description(&self) -> &str {
        &self.details
    }
}
