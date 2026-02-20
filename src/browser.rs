//! Headless Chrome/Chromium path for scraping JS-heavy or bot-protected sites.
//!
//! Enabled with the `browser` feature. Requires Chrome or Chromium on the system.

#![cfg(feature = "browser")]

use anyhow::{Context, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use crate::scraper::{parse_page, ScrapedPage};
use futures_util::stream::StreamExt;
use url::Url;

/// Scrapes `url` using a real headless Chrome/Chromium: loads the page (with JS),
/// then extracts visible text and links from the rendered HTML.
///
/// Use this for sites that block plain HTTP clients (e.g. x.com) or need JavaScript to render.
pub async fn scrape_url_with_browser(url: &str) -> Result<ScrapedPage> {
    let _ = Url::parse(url).context("invalid URL")?;

    let config = BrowserConfig::builder()
        .build()
        .map_err(|e| anyhow::anyhow!("browser config: {}", e))?;

    let (browser, mut handler) = Browser::launch(config).await.context("launch browser")?;

    // Handler must be polled continuously so CDP responses (e.g. for new_page) get delivered.
    // Do not exit on first error or the oneshot for new_page() will be canceled.
    let _handler_handle = tokio::spawn(async move {
        while let Some(_) = handler.next().await {}
    });

    // Give the handler task a chance to start so it can process new_page's response.
    tokio::task::yield_now().await;

    let page = browser
        .new_page(url)
        .await
        .context("open page")?;

    // Reduce bot detection (e.g. navigator.webdriver, user agent tweaks).
    let _ = page.enable_stealth_mode().await;

    // Allow time for JS to run; optional short wait for dynamic content.
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let html = page.content().await.context("get page HTML")?;
    let base_str = page
        .url()
        .await
        .context("get page URL")?
        .unwrap_or_else(|| url.to_string());
    let base_url = Url::parse(&base_str).unwrap_or_else(|_| Url::parse("about:blank").unwrap());

    let result = parse_page(&html, &base_url)?;
    Ok(result)
}
