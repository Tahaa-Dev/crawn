# Contributing to fiux

Thanks for your interest in contributing!

---

## Quick Start

1. Fork the repository
2. Create a branch: `git checkout -b feature/your-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Run clippy: `cargo clippy -- -D warnings`
6. Format code: `cargo fmt`
7. Commit: `git commit -m "Add feature X"`
8. Push: `git push origin feature/your-feature`
9. Open a Pull Request

## Guidelines

### Code Style

- Follow Rust standard style (enforced by `cargo fmt` with config in the project's root dir)
- Run `cargo clippy` and fix any warning
- Add tests for new features
- Document public APIs with `///` comments

---

### Commit Messages

- Be descriptive: "Docs: Add better docs for `--verbose`/`-v` flag" not "Update code"
- Reference issues: "Fix #123: Fix missing NDJSON escaping rules"

---

### Pull Requests

- Keep PRs focused (one feature/fix per PR)
- Include tests for new functionality
- Update README if adding user-facing features
- Describe what changed and why
- More tests are always welcome for merging

---

### Reporting Issues

- Check existing issues first
- Provide minimal reproduction case
- Use markdown code blocks for code/errors

---

### Questions?

Open an issue or a discussion. We're happy to help!

---

### License

By contributing, you agree your contributions will be licensed under **MIT**.
