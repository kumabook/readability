use crate::dom;
use crate::error::Error;
use crate::extractor::{extract, ReadableHtmlPage};
use crate::scorer;
use crate::scorer::Candidate;
use html5ever::tendril::stream::TendrilSink;
use html5ever::{parse_document, serialize};
use markup5ever_rcdom::{RcDom, SerializableHandle};
use reqwest;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::default::Default;
use std::io::Read;
use std::path::Path;
use std::time::Duration;
use url::Url;

/// Scrape the given url and return a [`ReadableHtmlPage`]
pub fn scrape(url: &str) -> Result<ReadableHtmlPage, Error> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::new(30, 0))
        .user_agent(super::APP_USER_AGENT)
        .build()?;

    let mut res = client.get(url).send()?;
    if res.status().is_success() {
        let url = Url::parse(url)?;
        extract(&mut res, &url)
    } else {
        Err(Error::HttpError)
    }
}
