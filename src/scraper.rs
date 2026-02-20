//! Core web scraping: fetch a URL, extract visible text and links.

use anyhow::{Context, Result};
use scraper::{ElementRef, Html, Node};
use std::collections::HashSet;
use url::Url;

const HIDDEN_TAGS: &[&str] = &["script", "style", "noscript", "head", "svg", "iframe"];

/// Result of scraping a page: visible text and list of link URLs.
#[derive(Debug, Clone, Default)]
pub struct ScrapedPage {
    /// Text that would typically be visible to a user (no script/style).
    pub text: String,
    /// Absolute URLs of links on the page (deduplicated).
    pub links: Vec<String>,
}

/// Fetches `url`, parses the HTML, and returns visible text plus links.
pub async fn scrape_url(url: &str) -> Result<ScrapedPage> {
    let url = Url::parse(url).context("invalid URL")?;
    let client = reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .build()?;
    let res = client
        .get(url.as_str())
        .send()
        .await
        .context("HTTP request failed")?;
    let status = res.status();
    anyhow::ensure!(
        status.is_success(),
        "HTTP {} for {}",
        status,
        res.url()
    );
    let html = res.text().await.context("reading response body")?;
    parse_page(&html, &url)
}

/// Parses HTML string with base URL for link resolution; returns visible text and links.
pub fn parse_page(html: &str, base_url: &Url) -> Result<ScrapedPage> {
    let document = Html::parse_document(html);
    let text = extract_visible_text(&document);
    let links = extract_links(&document, base_url);
    Ok(ScrapedPage { text, links })
}

/// Collects text from the document body, skipping script, style, and other non-visible elements.
fn extract_visible_text(document: &Html) -> String {
    let body = document
        .select(&scraper::Selector::parse("body").unwrap())
        .next();
    let body = match body {
        Some(b) => b,
        None => return String::new(),
    };
    let raw = collect_text_skip_hidden(body);
    normalize_whitespace(&raw)
}

fn collect_text_skip_hidden(element: ElementRef<'_>) -> String {
    let mut out = String::new();
    for child in element.children() {
        match child.value() {
            Node::Text(t) => {
                let s = t.trim();
                if !s.is_empty() {
                    if !out.is_empty() {
                        out.push(' ');
                    }
                    out.push_str(s);
                }
            }
            Node::Element(el) => {
                if HIDDEN_TAGS.contains(&el.name()) {
                    continue; // skip this element and its descendants
                }
                if let Some(child_el) = ElementRef::wrap(child) {
                    let sub = collect_text_skip_hidden(child_el);
                    if !sub.is_empty() {
                        if !out.is_empty() {
                            out.push(' ');
                        }
                        out.push_str(&sub);
                    }
                }
            }
            _ => {}
        }
    }
    out
}

fn normalize_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut space = false;
    for line in s.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !space {
                out.push('\n');
                space = true;
            }
        } else {
            space = false;
            if !out.is_empty() && !out.ends_with('\n') {
                out.push(' ');
            }
            out.push_str(trimmed);
            out.push('\n');
        }
    }
    out.trim().to_string()
}

fn extract_links(document: &Html, base_url: &Url) -> Vec<String> {
    let selector = scraper::Selector::parse("a[href]").unwrap();
    let mut seen = HashSet::new();
    let mut links = Vec::new();
    for el in document.select(&selector) {
        let href = el.value().attr("href").unwrap_or("").trim();
        if href.is_empty() || href.starts_with('#') || href.starts_with("javascript:") {
            continue;
        }
        let absolute = match base_url.join(href) {
            Ok(u) => u.to_string(),
            Err(_) => continue,
        };
        if seen.insert(absolute.clone()) {
            links.push(absolute);
        }
    }
    links
}

/// Formats a scraped page for display: text first, then a "Links" section.
pub fn format_for_display(page: &ScrapedPage) -> String {
    let mut out = page.text.clone();
    if !page.links.is_empty() {
        out.push_str("\n\n---\nLinks:\n");
        for link in &page.links {
            out.push_str(link);
            out.push('\n');
        }
    }
    out
}
