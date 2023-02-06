#[cfg(any(feature = "http-async", feature = "http-blocking"))]
use reqwest;
use std::error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use url;

#[derive(Debug)]
pub enum Error {
    #[cfg(any(feature = "http-async", feature = "http-blocking"))]
    NetworkError(reqwest::Error),
    UrlParseError(url::ParseError),
    HttpError(reqwest::StatusCode),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            #[cfg(any(feature = "http-async", feature = "http-blocking"))]
            Error::NetworkError(ref e) => write!(f, "NetworkError:  {e}"),
            Error::UrlParseError(ref e) => write!(f, "UrlParseError:  {e}"),
            Error::HttpError(status_code) => write!(f, "Http error, status: {status_code}"),
        }
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Error {
        Error::UrlParseError(err)
    }
}

#[cfg(any(feature = "http-async", feature = "http-blocking"))]
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::NetworkError(err)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        ""
    }
}
