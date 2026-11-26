# Rust and TypeScript LSP for Codex

Find definitions, references, and inferred types in saved Rust, TypeScript, and JavaScript files using your local language server.

## Install
>macOS only (Apple Silicon or Intel)

```sh
codex plugin marketplace add frontboat/LSP-MCP
```

Install the plugins, run setup skills

```sh
codex plugin add rust-lsp@lsp-mcp-local
codex plugin add typescript-lsp@lsp-mcp-local
```
```txt
Use the `setup-rust-lsp` and `setup-typescript-lsp` skills to setup our LSP MCPs.
```
>*Codex needs `rust-analyzer` on its `PATH` for Rust (`rustup component add rust-analyzer rust-src`), or a native `tsc` that supports `tsc --lsp --stdio` for TypeScript and JavaScript.*

## Update

```sh
codex plugin marketplace upgrade lsp-mcp-local
```

Then rerun `codex plugin add` for each plugin you installed.

## Remove

```sh
codex plugin remove rust-lsp@lsp-mcp-local
codex plugin remove typescript-lsp@lsp-mcp-local
codex plugin marketplace remove lsp-mcp-local
```

<details>
<summary>Tools</summary>

- `go_to_definition` returns file paths and source ranges for matching definitions.
- `find_references` lists uses. Pass `include_declaration: false` to skip the declaration.
- `hover` returns the type or signature and any documentation.

Pass an absolute `file` path or a path relative to an absolute `workspace`. Lines and characters start at 1. Characters count UTF-16 code units.

</details>

<details>
<summary>Build from source or solo MCP</summary>

`rustup target add aarch64-apple-darwin x86_64-apple-darwin`.

From source
>*For TypeScript, substitute `typescript-lsp` for `rust-lsp`.*

```sh
./scripts/package-plugin.sh rust-lsp
codex plugin marketplace add "$PWD"
codex plugin add rust-lsp@lsp-mcp-local
```

Solo MCP server, supply your project’s absolute path
>*For TypeScript, use `typescript-lsp` and `--language typescript`.*

```sh
cargo build --release --locked
codex mcp add rust-lsp -- "$PWD/target/release/lsp-mcp" --language rust --workspace /path/to/project
```

</details>

---

[Downloads and checksums](https://github.com/frontboat/LSP-MCP/releases) · [Support](https://github.com/frontboat/LSP-MCP/issues) · [Privacy](PRIVACY.md) · [Terms](LICENSE)
