# grok-eliminator

Remove local Grok CLI files and local Grok API-key exports.

This tool works on macOS, Linux, and Windows. It is local-only. It does not
contact xAI.

## Fast Path

If you came here after the posts below about reported malware-like code upload
behavior, use this tool to remove local Grok CLI files from your computer.

```sh
git clone https://github.com/TAKAMAgents/grok-eliminator.git
cd grok-eliminator
cargo run -- audit
cargo run -- remove
cargo run -- remove --apply
```

Then restart your terminal and rotate external xAI credentials separately.

## Install

```sh
git clone https://github.com/TAKAMAgents/grok-eliminator.git
cd grok-eliminator
cargo install --path . --locked
```

Or run it without installing:

```sh
cargo run -- audit
```

## Audit Only

This only reports what was found. It does not remove anything.

```sh
grok-eliminator audit
```

## Preview Removal

This shows what would be removed. It does not remove anything.

```sh
grok-eliminator remove
```

## Remove Grok Locally

This applies the cleanup.

```sh
grok-eliminator remove --apply
```

## JSON Output

```sh
grok-eliminator --json audit
```

```sh
grok-eliminator --json remove --apply
```

## After Cleanup

Restart your terminal.

On macOS or Linux shells, you can use:

```sh
exec zsh
```

On Windows, close the terminal and open a new one.

## What It Removes

- Known `grok` executable links.
- `~/.grok`.
- Global npm files for `@vibe-kit/grok-cli`.
- `GROK_API_KEY` and `XAI_API_KEY` exports from common shell profiles.
- macOS launchd variables when available.
- Windows user environment values when available.
- Grok access from cmux shells on macOS, without editing `/Applications/cmux.app`.

## What It Does Not Do

- It does not contact xAI.
- It does not revoke or rotate remote xAI credentials.
- It does not print API-key values.
- It does not edit signed application bundles.
- It does not delete source code or shell history only because they mention Grok.

Rotate external xAI credentials separately if they may have been exposed.

## Background

- Hari Krishnan, July 13, 2026:
  <https://x.com/hrkrshnn/status/2076716354754015368>
- Peter Dedene, July 12, 2026:
  <https://x.com/dedene/status/2076394152779301305>
- The Verge, July 14, 2026:
  <https://www.theverge.com/ai-artificial-intelligence/965600/spacexai-grok-build-repository-upload>

## Developer Checks

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

## More

- [Design notes](docs/design.md)
- [Open source checklist](docs/OPEN_SOURCE_CHECKLIST.md)
- [Contributing](CONTRIBUTING.md)
- [Security](SECURITY.md)
- [License](LICENSE)
