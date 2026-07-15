# Contributing

Thanks for helping improve `grok-eliminator`. This project is intentionally
small and conservative because it edits local developer machines.

## Ground rules

- Keep cleanup local-only. Do not contact xAI, revoke remote accounts, or read
  API-key values.
- Preserve unrelated files, source trees, shell history, and signed app bundles.
- Make destructive behavior available only through `remove --apply`.
- Use typed domain values and explicit boundary conversions for new behavior.
- Do not add mocks or fake success paths. Tests should exercise real parsing,
  filesystem, process, or platform-boundary behavior where practical.
- Never commit secrets, local credentials, API keys, tokens, private keys,
  session files, or personal data.

## Development setup

Install a stable Rust toolchain that supports Rust 2024 edition. The crate's
minimum supported Rust version is declared in `Cargo.toml`.

```text
cargo test
```

## Required checks

Run the same checks used by CI before opening a pull request:

```text
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

## Pull request checklist

- The change stays inside the local cleanup boundary.
- Reports expose paths and statuses only, never secret values.
- New destructive actions have a dry-run report path.
- Tests cover the changed behavior with realistic inputs or local resources.
- Public-facing documentation is updated when behavior changes.
