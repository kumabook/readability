use readability::extractor::ReadableHtmlPage;

#[test]
fn should_scrape_blocking() {
    let product: ReadableHtmlPage =
        readability::extractor::scrape("https://blog.rust-lang.org/2023/02/01/Rustup-1.25.2.html")
            .expect("scrape blog entry");

    assert_eq!(product.title, "Announcing Rustup 1.25.2 | Rust Blog");
    assert!(!product.text.is_empty());
    assert!(!product.content.is_empty());
}
