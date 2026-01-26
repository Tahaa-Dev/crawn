<h1 align="center">Changelog</h1>

## [v0.1.1] 2026-01-26

### Summary

- Refactored logging without a file to log to `tokio::io::stdout` instead of `std::io::stderr`
- Added module level docs to main.rs

### Details

- Refactored logging to use `tokio::io::stdout` instead of `std::io::stderr` for:
    - Unix piping to other tools (e.g. `crawn -o output.ndjson https://example.com -v | grep "domain" | cat > domain_urls.log`)
    - Fully async logging with race condition guards using `tokio::sync::Mutex` for easier switching to concurrent tokio workers
    - More control over logging for better complex logging support in the future
- Added module level docs to main.rs for Docs.rs docs
