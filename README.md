# Rust and TypeScript LSP for Codex

Find definitions, references, and inferred types in saved Rust, TypeScript, and JavaScript files using your local language server.

## Install from the GitHub marketplace

On macOS (Apple Silicon or Intel), add the marketplace once.

```sh
codex plugin marketplace add frontboat/LSP-MCP
```

Install the plugin you want, then run its setup skill in a new local Codex chat.

| Plugin | Run in your terminal | Tell Codex |
| --- | --- | --- |
| Rust | `codex plugin add rust-lsp@lsp-mcp-local` | Use the `setup-rust-lsp` skill. |
| TypeScript / JavaScript | `codex plugin add typescript-lsp@lsp-mcp-local` | Use the `setup-typescript-lsp` skill. |

Codex needs `rust-analyzer` on its `PATH` for Rust (`rustup component add rust-analyzer rust-src`), or a native `tsc` that supports `tsc --lsp --stdio` for TypeScript and JavaScript.

To update, run `codex plugin marketplace upgrade lsp-mcp-local` and repeat your install command.

<details>
<summary>MCP tools</summary>

- `go_to_definition` returns file paths and source ranges for matching definitions.
- `find_references` lists uses. Pass `include_declaration: false` to skip the declaration.
- `hover` returns the type or signature and any documentation.

Pass an absolute `file` path or a path relative to an absolute `workspace`. Lines and characters start at 1. Characters count UTF-16 code units.

</details>

<details>
<summary>Build from source or register a standalone MCP server</summary>

For a Linux build or local development, clone this repo. Install Rust, Cargo, Python 3, and the `rust-docs` component. On macOS, install the Xcode command line tools and run `rustup target add aarch64-apple-darwin x86_64-apple-darwin`.

Run these commands from the repo root. For TypeScript, substitute `typescript-lsp` for `rust-lsp`.

```sh
./scripts/package-plugin.sh rust-lsp
codex plugin marketplace add "$PWD"
codex plugin add rust-lsp@lsp-mcp-local
```

To register the bridge as a standalone MCP server, supply your project’s absolute path. For TypeScript, use `typescript-lsp` and `--language typescript`.

```sh
cargo build --release --locked
codex mcp add rust-lsp -- "$PWD/target/release/lsp-mcp" --language rust --workspace /path/to/project
```

</details>

Local LSP Tools · [Downloads and checksums](https://github.com/frontboat/LSP-MCP/releases) · [Support](https://github.com/frontboat/LSP-MCP/issues) · [Privacy](PRIVACY.md) · [Terms](LICENSE)
