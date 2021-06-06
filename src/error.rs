use serde::Deserialize;
use serde_json;
use std::error::Error as StdError;
use std::fmt;
#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::HTTP(_) => {
                write!(f, "http error")
            }
            ErrorKind::Status(err) => {
                write!(f, "status code: {}, message: {}", err.code, err.message)
            }
            ErrorKind::JSON(_) => {
                write!(f, "json error")
            }
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self {
            kind: ErrorKind::HTTP(e),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self {
            kind: ErrorKind::JSON(e),
        }
    }
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    HTTP(reqwest::Error),
    Status(StatusError),
    JSON(serde_json::Error),
}

#[derive(Debug)]
pub struct StatusError {
    pub code: u16,
    pub message: String,
}

impl StatusError {
    pub fn new(code: u16, message: String) -> Self {
        Self { code, message }
    }
}
#[derive(Deserialize)]
pub struct ErrorMessage {
    pub message: String
}