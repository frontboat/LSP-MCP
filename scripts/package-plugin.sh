#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
plugin=${1:-typescript-lsp}
case "$plugin" in
    typescript-lsp|rust-lsp) ;;
    *) echo "Usage: $0 [typescript-lsp|rust-lsp]" >&2; exit 2 ;;
esac
cargo build --release --locked --target-dir target
package="plugins/$plugin"
mkdir -p "$package/bin" dist
install -m 755 target/release/lsp-mcp "$package/bin/lsp-mcp"
archive="dist/$plugin-$(uname -s)-$(uname -m).tar.gz"
COPYFILE_DISABLE=1 tar -czf "$archive" -C "$package" plugin.json mcp.json assets skills bin/lsp-mcp
printf '%s\n' "$archive"
