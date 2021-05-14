use std::fmt;
use thiserror::Error;

#[derive(Debug)]
pub struct ADRError {
    pub error: String,
}

impl fmt::Display for ADRError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.error)
    }
}

impl std::error::Error for ADRError {}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Oh thats bad: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Unknown error")]
    Unknown,
}
