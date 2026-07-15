# grok-eliminator

`grok-eliminator` audits and removes the local Grok CLI installation from the
current user's macOS, Linux, or Windows machine. It is deliberately local-only: it does not contact xAI,
inspect API-key values, modify the signed cmux application, or delete source
code and terminal history that merely mention Grok.

## Usage

```text
cargo install --path . --locked --root "$HOME/.local"
grok-eliminator audit
grok-eliminator remove
grok-eliminator remove --apply
```

`audit` is read-only. `remove` previews actions unless `--apply` is supplied.
Reports contain paths and status only; API-key values are never read into the
report. Existing shells need `exec zsh` after an applied cleanup.

The cleanup covers known Grok CLI paths, npm's actual global package root, the
`@vibe-kit/grok-cli` package, `~/.grok`, `GROK_API_KEY`/`XAI_API_KEY` exports,
macOS launchd variables, Windows user environment values, and cmux's
macOS-only shell reachability. On platforms without a persistent environment
store that the tool can safely edit, the report says `unavailable` and tells
the user to restart after removing the profile export. Signed applications and
unrelated source or terminal-history files are intentionally preserved.
External xAI credentials must be rotated separately.

## Development

```text
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```
