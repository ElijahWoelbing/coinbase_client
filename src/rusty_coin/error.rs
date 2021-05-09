use std::error::Error as StdError;
use std::fmt;

use tokio::fs::write;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::HTTP => {
                write!(f, "http error")
            },
            ErrorKind::Connection => {
                write!(f, "connection error")
            },
            ErrorKind::Decode => {
                write!(f, "decode error")
            },
            ErrorKind::Request => {
                write!(f, "request error")
            },
            ErrorKind::Status => {
                write!(f, "status error")
            },
            ErrorKind::Timeout => {
                write!(f, "timeout error")
            }
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        let mut kind = ErrorKind::HTTP;
        if e.is_decode() {
            kind = ErrorKind::Decode;
        } else if e.is_connect() {
            kind = ErrorKind::Connection;
        } else if e.is_request() {
            kind = ErrorKind::Request;
        } else if e.is_status() {
            kind = ErrorKind::Status;
        } else if e.is_timeout() {
            kind = ErrorKind::Timeout;
        }


        Self { kind }
    }
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }
}
#[derive(Debug)]
pub enum ErrorKind {
    HTTP,
    Connection,
    Decode,
    Request,
    Status,
    Timeout
}
