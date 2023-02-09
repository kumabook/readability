use readability::extractor::ReadableHtmlPage;

#[tokio::test]
async fn should_scrape_blocking() {
    let page: ReadableHtmlPage =
        readability::extractor::scrape("https://blog.rust-lang.org/2023/02/01/Rustup-1.25.2.html")
            .await
            .expect("scrape blog entry");

    assert_eq!(page.title, "Announcing Rustup 1.25.2 | Rust Blog");
    assert!(!page.text.is_empty());
    assert!(!page.content.is_empty());
}

#[tokio::test]
async fn should_scrape_website_checking_user_agent() {
    let page: ReadableHtmlPage = readability::extractor::scrape(
        "https://dev.to/mayashavin/testing-vue-components-the-right-way-2hio",
    )
    .await
    .expect("scrape blog entry");

    assert_eq!(
        page.title,
        "Testing Vue components the right way - DEV Community 👩\u{200d}💻👨\u{200d}💻"
    );
    assert!(!page.text.is_empty());
    assert!(!page.content.is_empty());
}

#[tokio::test]
async fn should_scrape_w() {
    let page: ReadableHtmlPage = readability::extractor::scrape(
        "https://medium.com/@stevenchayes/so-you-messed-up-your-new-years-resolution-a4052e502906",
    )
    .await
    .expect("scrape blog entry");

    assert_eq!(
        page.title,
        "So You Messed Up Your New Year’s Resolution | by Steven C. Hayes | Jan, 2023 | Medium"
    );
    assert_eq!(page.text, "");
    assert_eq!(page.content, "");
}
