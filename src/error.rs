use std::fmt::{Debug, Display};

use csv;

#[derive(Debug)]
pub enum Error {
    CsvError(csv::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::CsvError(ref err) => write!(f, "{}", err),
        }
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Error {
        Error::CsvError(err)
    }
}
