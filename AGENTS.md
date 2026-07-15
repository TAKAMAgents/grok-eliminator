# grok-eliminator agent guide

## Goal

Provide a small, auditable Rust CLI that removes the Grok CLI and credentials
from the current user's macOS, Linux, or Windows machine without damaging
unrelated projects, terminal history, or signed application bundles.

## Boundaries

- The CLI owns local cleanup actions only: known executable links, the Grok
  config directory, global npm package artifacts, shell credential exports,
  platform environment variables, and the cmux shell-path guard on macOS.
- It must not contact xAI, revoke external accounts, print secret values, edit
  `/Applications/cmux.app`, or delete files solely because their contents
  mention Grok.
- `audit` is read-only. Destructive work requires `remove --apply`.

## Engineering rules

- Use typed domain values and explicit boundary conversions.
- Do not use mocks or fake success paths.
- Never log secret values; reports contain presence and path metadata only.
- Use `thiserror` for errors and avoid fallible `unwrap`/`expect` in production.
- Keep cmux support outside the signed app bundle and compile cleanly on all
  supported platforms.

## Validation

```text
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```
