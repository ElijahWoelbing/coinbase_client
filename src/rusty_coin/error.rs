use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind
}

impl StdError for Error {

}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::HttpError => {write!(f,"http error")},
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(_: reqwest::Error) -> Self {
        Self {
            kind: ErrorKind::HttpError
        }
    }
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind
        }
    }
}
#[derive(Debug)]
pub enum ErrorKind {
    HttpError
}


