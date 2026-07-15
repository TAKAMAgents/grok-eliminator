#!/bin/sh
set -eu

repo="TAKAMAgents/grok-eliminator"
install_dir="${GROK_ELIMINATOR_INSTALL_DIR:-$HOME/.local/bin}"
tmp_dir="$(mktemp -d)"

cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT INT TERM

download() {
  url="$1"
  output="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$output"
  elif command -v wget >/dev/null 2>&1; then
    wget -q "$url" -O "$output"
  else
    echo "curl or wget is required" >&2
    exit 1
  fi
}

case "$(uname -s):$(uname -m)" in
  Linux:x86_64)
    asset="grok-eliminator-linux-x86_64.tar.gz"
    ;;
  Darwin:arm64)
    asset="grok-eliminator-macos-aarch64.tar.gz"
    ;;
  Darwin:x86_64)
    asset="grok-eliminator-macos-x86_64.tar.gz"
    ;;
  *)
    echo "unsupported system: $(uname -s) $(uname -m)" >&2
    exit 1
    ;;
esac

url="https://github.com/$repo/releases/latest/download/$asset"
archive="$tmp_dir/$asset"

download "$url" "$archive"
tar -xzf "$archive" -C "$tmp_dir"

mkdir -p "$install_dir"
cp "$tmp_dir/grok-eliminator" "$install_dir/grok-eliminator"
chmod +x "$install_dir/grok-eliminator"

echo "installed: $install_dir/grok-eliminator"
echo "run: $install_dir/grok-eliminator audit"

case ":$PATH:" in
  *":$install_dir:"*) ;;
  *)
    echo "if grok-eliminator is not found, add this to your shell profile:"
    echo "export PATH=\"$install_dir:\$PATH\""
    ;;
esac
