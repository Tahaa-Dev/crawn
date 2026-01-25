# crawn

**Fast async web crawler with smart keyword filtering**

[![CI](https://github.com/Tahaa-Dev/crawn/workflows/CI/badge.svg)](https://github.com/Tahaa-Dev/crawn/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## Features

- **Blazing fast** – Built with Rust + tokio for async I/O
- **Smart filtering** – URL-based keyword matching (no content fetching required)
- **NDJSON output** – One JSON object per line for easy streaming
- **BFS crawling** – Depth-first traversal with configurable depth limits
- **Rate limiting** – Configurable request rate (default: 10 req/sec)
- **Error recovery** – Gracefully handles network errors and broken links
- **Rich logging** – Colored, timestamped logs with context chains

---

## Installation

- Build from source:
```sh
git clone https://github.com/Tahaa-Dev/crawn.git
cd crawn
cargo build --release
```

---

## Usage

- Basic Crawling:
```sh
crawn -o output.ndjson https://example.com
```

- With Logging:
```sh
crawn -o output.ndjson -l crawler.log https://example.com
```

- Verbose Mode (Log All Requests):
```sh
crawn -o output.ndjson -v https://example.com
```

- Custom Depth Limit:
```sh
crawn -o output.ndjson -m 3 https://example.com
```

- Full HTML:
```sh
crawn -o output.ndjson --include-content https://example.com
```

- Extracted text only:
```sh
crawn -o output.ndjson --include-text https://example.com
```

---

## Output Format

Results are written as NDJSON (newline-delimited JSON):
```json
{"url":"https://example.com","title":"Example Domain","depth":0}
{"url":"https://example.com/about","title":"About Us","depth":1}
{"url":"https://example.com/contact","title":"Contact","depth":1}
```

- With --include-text:
```json
{"url":"https://example.com","title":"Example Domain","depth":0,"text":"Example Domain\nThis domain is..."}
```

- With --include-content:
```json
{"url":"https://example.com","title":"Example Domain","depth":0,"content":"<!DOCTYPE html>\n<html>..."}
```

---

## How It Works

1. BFS Crawling:
    - Starts at the seed URL (depth 0)
    - Discovers links on each page
    - Processes links level-by-level (breadth-first)
    - Stops at max_depth (default: 4)

2. Keyword Filtering:
    - Extracts "keywords" from URL paths (sanitized, lowercased)
    - Splits by /, -, _ (e.g., /rust-tutorials/async → ["rust", "tutorials", "async"])
    - Filters stop words, numbers, short words (<3 chars)
    - Matches candidate URLs against base keywords
    - Result: Only crawls relevant pages, skips off-topic content

3. Rate Limiting:
	- Default: 10 requests/second (100ms sleep between requests)
	- Prevents server overload and IP bans
	- Configurable via code (not exposed as CLI flag yet)

4. Error Handling:
	- Network errors: Logged as warnings, crawling continues
	- HTTP 404/500: Skipped, logged as warnings
	- Parse failures: Logged, returns empty JSON
	- Fatal errors: Printed to stdout with full context chain

---

## Logging

#### Log Levels:

- **INFO** (verbose mode only): Request logs
- **WARN** (always): Recoverable errors (404, network timeouts)
- **FATAL** (always): Unrecoverable errors (invalid URL, disk full)

#### Log Format:

```text
2026-01-24 02:37:40.351 [INFO]:
Sent request to URL: https://example.com

2026-01-24 02:37:41.123 [WARN]:
Failed to fetch URL: https://example.com/broken-link
Caused by: HTTP 404 Not Found
```

---

## Examples

- Crawl Documentation Site:
```sh
crawn -o rust-docs.ndjson https://doc.rust-lang.org/book/
```

- Crawl with Logging:
```sh
crawn -o output.ndjson -l crawler.log -v https://example.com
```

- Limit to 2 Levels Deep:
```sh
crawn -o shallow.ndjson -m 2 https://example.com
```

---

## Limitations

- Same-domain only (no external links, by design)
- No JavaScript rendering (static HTML only)
- No authentication (public pages only)
