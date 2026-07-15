#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cargo install --path "$repo_root" --locked --force --root "$HOME/.local"
