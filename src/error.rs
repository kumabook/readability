use std::fmt::{Display, Formatter, Result as FmtResult};
use std::error;
use hyper;
use url;

#[derive(Debug)]
pub enum Error {
    NetworkError(hyper::Error),
    UrlParseError(url::ParseError),
    Unexpected,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Error::NetworkError(ref e)   => write!(f, "NetworkError:  {}", e),
            Error::UrlParseError(ref e)  => write!(f, "UrlParseError:  {}", e),
            Error::Unexpected            => write!(f, "UnexpectedError"),
        }
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Error {
        Error::UrlParseError(err)
    }
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Error {
        Error::NetworkError(err)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str { "" }
}
