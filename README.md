# Rust and TypeScript LSP for Codex

Find definitions, references, and inferred types in saved Rust, TypeScript, and JavaScript files using your local language server.

## Install from the GitHub marketplace

Both plugins include a prebuilt bridge for Apple Silicon and Intel Macs. Add the marketplace once, then run the install command for each language you use.

```sh
codex plugin marketplace add frontboat/LSP-MCP

codex plugin add rust-lsp@lsp-mcp-local
codex plugin add typescript-lsp@lsp-mcp-local
```

Put your language server on Codex’s `PATH`.

- For Rust, install `rust-analyzer` and the standard library sources with `rustup component add rust-analyzer rust-src`.
- For TypeScript and JavaScript, use a native `tsc` with support for `tsc --lsp --stdio` (tested with 7.0.2).

Start a new local Codex chat. Select **@Rust LSP** or **@TypeScript LSP** and try the bundled example.

> Find the definition of `double` in your example project and show me its callers.

To update, run `codex plugin marketplace upgrade lsp-mcp-local`, then repeat the install command for each plugin.

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
