use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::HTTP(_) => {
                write!(f, "http error")
            },
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {

        Self { kind: ErrorKind::HTTP(e) }
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
}
