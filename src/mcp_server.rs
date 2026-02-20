//! MCP server: exposes a scrape_url tool for AI clients.

use std::future::Future;

use crate::scraper::{format_for_display, scrape_url};
use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::{ErrorCode, ErrorData as McpError, *},
    schemars, tool, tool_handler, tool_router, ServerHandler,
};
use serde::Deserialize;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct ScraperMcpService {
    tool_router: ToolRouter<ScraperMcpService>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScrapeUrlRequest {
    #[schemars(description = "The URL of the web page to scrape (e.g. https://example.com)")]
    pub url: String,
    #[schemars(description = "If true, use headless Chrome/Chromium (for JS-heavy or bot-protected sites like x.com). Requires build with --features browser.")]
    #[serde(default)]
    pub use_browser: bool,
}

#[tool_router]
impl ScraperMcpService {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Scrape a web page: fetch the URL and return the visible text plus all links. Set use_browser to true for JS-heavy or bot-protected sites (e.g. x.com).")]
    async fn scrape_url_tool(
        &self,
        Parameters(request): Parameters<ScrapeUrlRequest>,
    ) -> Result<CallToolResult, McpError> {
        let page = scrape_url_impl(&request.url, request.use_browser).await.map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: Cow::from(format!("Scrape failed: {}", e)),
            data: None,
        })?;
        let text = format_for_display(&page);
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }
}

async fn scrape_url_impl(url: &str, use_browser: bool) -> Result<crate::scraper::ScrapedPage, anyhow::Error> {
    if use_browser {
        #[cfg(feature = "browser")]
        return crate::browser::scrape_url_with_browser(url).await;
        #[cfg(not(feature = "browser"))]
        anyhow::bail!("use_browser is true but this build does not have the 'browser' feature")
    }
    scrape_url(url).await
}

#[tool_handler]
impl ServerHandler for ScraperMcpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "Web scraper service. Use the scrape_url_tool with a URL to fetch the page and get the visible text a user would see, plus all links listed at the bottom.".to_string(),
            ),
        }
    }
}
