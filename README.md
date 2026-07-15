# grok-eliminator

`grok-eliminator` audits and removes the local Grok CLI installation from a
macOS workstation. It is deliberately local-only: it does not contact xAI,
inspect API-key values, modify the signed cmux application, or delete source
code and terminal history that merely mention Grok.

## Status

The repository is bootstrapped for the first implementation milestone.

## Planned usage

```text
cargo run -- audit
cargo run -- remove
cargo run -- remove --apply
```

`audit` is read-only. `remove` previews actions unless `--apply` is supplied.

## Development

```text
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```
