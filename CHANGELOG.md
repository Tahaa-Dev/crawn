<h1 align="center">Changelog</h1>

## [v0.3.0] 2026-03-14

### Summary

- Made CLI fully integrated with the Unix ecosystem

### Details

- CLI now defaults to logging to ./crawn.log instead of Stdout when log-file is unspecified
- Base URL is now not required as it can be extracted from Stdin if it is not provided via CLI args
- Output is now to Stdout for piping to other Unix tools, example:

```bash
crawn --include-text https://example.com | grep 'rust' | sed -i 's/[^\r]\n/\r\n/g' | jq -s '.' | cat > output.json
```

---

## [v0.2.0] 2026-02-03

### Summary

- Added concurrency to crawn with concurrent `tokio::task`s
- Improved logging and error messages
- Added crawling summary at the end of each successful crawl

---

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
