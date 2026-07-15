# Open Source Checklist

Use this before making the repository public.

## Required before publication

- Review the full working tree diff.
- Confirm `LICENSE` has the intended copyright holder.
- Confirm `Cargo.toml` metadata is accurate.
- Run:

```text
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo package --allow-dirty
```

- Scan the current tree and Git history for secrets.
- Push the open-source prep commit.
- Confirm GitHub detects the MIT license after the commit is pushed.
- Set a repository description and topics on GitHub.
- Enable GitHub private vulnerability reporting.
- Confirm the release workflow can publish downloadable binaries from a tag.
- Confirm the install scripts install the latest release binary.
- Make the repository public only after the checks above are complete.

## After publication

- Watch the first public CI run.
- Confirm issue templates and the pull request template render correctly.
- Confirm Dependabot opens dependency update pull requests.
- Create a signed release tag when publishing the first stable build.
- Confirm release assets download and run on each supported platform.
