#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
plugin=${1:-typescript-lsp}
case "$plugin" in
    typescript-lsp|rust-lsp) ;;
    *) echo "Usage: $0 [typescript-lsp|rust-lsp]" >&2; exit 2 ;;
esac
package="plugins/$plugin"
mkdir -p "$package/bin" dist
platform="$(uname -s)-$(uname -m)"
# Keep publisher-specific filesystem paths out of the distributed executable.
separator=$(printf '\037')
export CARGO_ENCODED_RUSTFLAGS="--remap-path-prefix=$HOME=/build/home${separator}--remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}=/cargo${separator}--remap-path-prefix=$(pwd -P)=/src/lsp-mcp"
export CARGO_PROFILE_RELEASE_STRIP=symbols

if [ "$(uname -s)" = Darwin ]; then
    set -- aarch64-apple-darwin x86_64-apple-darwin
    cargo build --release --locked --target-dir target --target "$1" --target "$2"
    binary="target/lsp-mcp-macos-universal"
    lipo -create "target/$1/release/lsp-mcp" "target/$2/release/lsp-mcp" -output "$binary"
    codesign --force --sign - --identifier lsp-mcp --timestamp=none "$binary"
    codesign --verify --strict --all-architectures "$binary"
    platform=Darwin-universal
else
    set -- "$(rustc --print host-tuple)"
    cargo build --release --locked --target-dir target
    binary=target/release/lsp-mcp
fi

python3 scripts/dependency-notices.py "$package/bin" "$@"
install -m 755 "$binary" "$package/bin/lsp-mcp"
archive="dist/$plugin-$platform.tar.gz"
# Store portable permissions and omit the packager's ownership and extended attributes.
case "$(tar --version)" in
    *bsdtar*) set -- --uid 0 --gid 0 --uname root --gname root ;;
    *) set -- --owner=0 --group=0 ;;
esac
COPYFILE_DISABLE=1 tar --format=ustar "$@" --exclude=.DS_Store -czf "$archive" -C "$package" plugin.json mcp.json assets skills \
    bin/lsp-mcp bin/THIRD-PARTY-NOTICES.txt bin/RUST-LIBRARY-LICENSES.html
printf '%s\n' "$archive"
