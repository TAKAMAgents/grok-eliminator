# grok-eliminator

Remove local Grok CLI files and local Grok API-key exports.

Works on macOS, Linux, and Windows. Local-only. Does not contact xAI.

It does not revoke remote xAI credentials. Rotate external xAI credentials
separately if they may have been exposed.

## Install Latest Binary

macOS or Linux:

```sh
curl -fsSL https://raw.githubusercontent.com/TAKAMAgents/grok-eliminator/main/scripts/install-release.sh | sh
```

Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/TAKAMAgents/grok-eliminator/main/scripts/install-release.ps1 | iex
```

Manual downloads are on the [latest release page](https://github.com/TAKAMAgents/grok-eliminator/releases/latest).

## Use

First audit and preview:

```sh
grok-eliminator audit
grok-eliminator remove
```

Apply cleanup only when ready:

```sh
grok-eliminator remove --apply
```

Restart your terminal after cleanup.

## Build From Source

Requires Rust and Cargo.

```sh
git clone https://github.com/TAKAMAgents/grok-eliminator.git
cd grok-eliminator
cargo run -- audit
cargo run -- remove
```

Apply cleanup only when ready:

```sh
cargo run -- remove --apply
```

## Commands

```sh
grok-eliminator audit
grok-eliminator remove
grok-eliminator remove --apply
grok-eliminator --json audit
```

- `audit` reports what was found.
- `remove` previews what would be removed.
- `remove --apply` applies cleanup.
- Reports show paths and status only. They do not print API-key values.

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

## Publish a Release

```sh
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions will build release binaries and attach them to the release.

## More

- [Design notes](docs/design.md)
- [Open source checklist](docs/OPEN_SOURCE_CHECKLIST.md)
- [Contributing](CONTRIBUTING.md)
- [Security](SECURITY.md)
- [License](LICENSE)
