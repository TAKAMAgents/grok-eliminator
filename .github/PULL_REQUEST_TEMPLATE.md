## Summary

-

## Validation

- [ ] `cargo fmt --check`
- [ ] `cargo test`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`

## Safety checklist

- [ ] `audit` remains read-only.
- [ ] Destructive behavior still requires `remove --apply`.
- [ ] Reports do not print secret values.
- [ ] Signed app bundles and unrelated files are preserved.
