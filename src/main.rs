//! xscraper: web scraper with CLI and MCP server.
//!
//! CLI: scrape a URL and print visible text + links, or run the MCP server.

mod mcp_server;
mod scraper;

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
    },
    /// Run the MCP server on stdio (for Cursor, Claude, etc.).
    Serve,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Scrape { url } => {
            let page = scrape_url(&url).await?;
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
