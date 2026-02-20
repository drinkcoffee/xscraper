# xscraper

Web scraper that fetches a URL and dumps the visible text and links (like the X / Twitter web interface). It can be used from the **command line** or as an **MCP server** for AI clients (Cursor, Claude, etc.).

## Prerequisites

- **Rust** (1.70+). Install from [rustup.rs](https://rustup.rs).
- **Chrome or Chromium** (only if you use the `browser` feature for JS-heavy or bot-protected sites like x.com).

## Building

**Default build** (HTTP-only scraper, no browser):

```bash
cargo build
```

**With headless browser support** (for sites that need a real browser, e.g. x.com):

```bash
cargo build --features browser
```

Release build:

```bash
cargo build --release
cargo build --release --features browser   # with browser
```

## Running

### Scrape a URL (CLI)

**HTTP only** (fast, works for most static sites):

```bash
cargo run -- scrape "https://example.com"
```

**Headless browser** (for JS-heavy or bot-protected sites like x.com; requires `browser` feature and Chrome/Chromium installed):

```bash
cargo run --features browser -- scrape --browser "https://example.com"
```

Output is the visible text, then a `---` section with all links.

### Run the MCP server

Start the MCP server on stdio (for Cursor, Claude Desktop, etc.):

```bash
cargo run -- serve
```

With browser support:

```bash
cargo run --features browser -- serve
```

Configure your MCP client to run the binary, for example in Cursor’s MCP settings (e.g. `~/.cursor/mcp.json`):

```json
{
  "mcpServers": {
    "xscraper": {
      "command": "cargo",
      "args": ["run", "--features", "browser", "--", "serve"]
    }
  }
}
```

Or after installing the binary (see below), use the `xscraper` command directly.

### Install the binary

Install into `~/.cargo/bin` so you can run `xscraper` from the shell:

```bash
cargo install --path .
# with browser support:
cargo install --path . --features browser
```

Then:

```bash
xscraper scrape "https://example.com"
xscraper scrape --browser "https://x.com"
xscraper serve
```

## MCP tool

When the server is running, the tool **scrape_url_tool** is available:

- **url** (required): The page URL to scrape.
- **use_browser** (optional): Set to `true` to use headless Chrome/Chromium for sites like x.com. Only works if the server was built with `--features browser`.

## Features

| Feature   | Description |
|----------|-------------|
| (default) | HTTP client only; no extra dependencies. |
| `browser` | Adds headless Chrome/Chromium via [chromiumoxide](https://crates.io/crates/chromiumoxide). Use for JS-heavy or bot-protected sites. |
