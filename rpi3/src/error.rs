use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct LpuError {
    details: String
}

impl LpuError {
    pub fn new(msg: &str) -> LpuError {
        LpuError{details: msg.to_string()}
    }
}

impl fmt::Display for LpuError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl Error for LpuError {
    fn description(&self) -> &str {
        &self.details
    }
}