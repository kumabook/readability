use crate::error::Error;
use crate::extractor::{extract, ReadableHtmlPage};
use std::time::Duration;
use url::Url;

/// Scrape the given url and return a [`ReadableHtmlPage`]
pub async fn scrape(url: &str) -> Result<ReadableHtmlPage, Error> {
    let client = reqwest::Client::builder()
        .timeout(Duration::new(30, 0))
        .user_agent(super::APP_USER_AGENT)
        .build()?;

    let res = client.get(url).send().await?;

    if res.status().is_success() {
        let url = Url::parse(url)?;
        let read = res.text().await?;
        extract(&mut read.as_bytes(), &url)
    } else {
        Err(Error::HttpError(res.status()))
    }
}
