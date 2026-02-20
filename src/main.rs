//! xscraper: web scraper with CLI and MCP server.
//!
//! CLI: scrape a URL and print visible text + links, or run the MCP server.

mod mcp_server;
mod scraper;

#[cfg(feature = "browser")]
mod browser;

use anyhow::Result;
use clap::{Parser, Subcommand};
use rmcp::transport::stdio;
use rmcp::ServiceExt;

use crate::scraper::{format_for_display, scrape_url};

#[derive(Parser)]
#[command(name = "xscraper")]
#[command(about = "Web scraper: dump visible text and links from a URL. Use as CLI or MCP server.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scrape a URL and print visible text, then links (default if URL given as positional arg).
    Scrape {
        /// URL to scrape (e.g. https://example.com)
        url: String,
        /// Use headless Chrome/Chromium (for JS-heavy or bot-protected sites like x.com). Requires browser feature and Chrome installed.
        #[arg(long)]
        browser: bool,
    },
    /// Run the MCP server on stdio (for Cursor, Claude, etc.).
    Serve,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Scrape { url, browser: use_browser } => {
            let page = if use_browser {
                #[cfg(feature = "browser")]
                {
                    crate::browser::scrape_url_with_browser(&url).await?
                }
                #[cfg(not(feature = "browser"))]
                {
                    anyhow::bail!("--browser requires building with the 'browser' feature: cargo build --features browser")
                }
            } else {
                scrape_url(&url).await?
            };
            println!("{}", format_for_display(&page));
        }
        Commands::Serve => {
            let service = mcp_server::ScraperMcpService::new();
            let server = service.serve(stdio()).await?;
            server.waiting().await?;
        }
    }
    Ok(())
}
