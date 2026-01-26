<h1 align="center">crawn</h1>

[<img alt="crates.io" src="https://img.shields.io/crates/v/crawn.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/crawn)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-crawn-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/crawn)
[![CI](https://github.com/Tahaa-Dev/crawn/workflows/CI/badge.svg)](https://github.com/Tahaa-Dev/crawn/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**A utility for web crawling and scraping**

---

## Features

- **Blazing fast** – Built with Rust & tokio for async I/O
- **Smart filtering** – URL-based keyword matching (no content fetching required)
- **NDJSON output** – One JSON object per line for easy streaming
- **BFS crawling** – Depth-first traversal with configurable depth limits
- **Rate limiting** – Configurable request rate (default: 2 - 5 req/sec)
- **Error recovery** – Gracefully handles network errors and broken links
- **Rich logging** – Colored, timestamped logs with context chains

---

## Installation

Run this command (requires cargo):
```sh
cargo install crawn
```

- Or build from source (requires cargo):
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

- With `--include-text`:
```json
{"url":"https://example.com","title":"Example Domain","depth":0,"text":"Example Domain\nThis domain is..."}
```

- With `--include-content`:
```json
{"url":"https://example.com","title":"Example Domain","depth":0,"content":"<!DOCTYPE html>\n<html>..."}
```

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

---

## Notes

- crawn is licensed under the <a href="LICENSE">MIT license</a>.
- For specifics about contributing to crawn, see <a href="CONTRIBUTING.md">CONTRIBUTING.md</a>.
- For new changes to crawn, see <a href="CHANGELOG.md">CHANGELOG.md</a>.
