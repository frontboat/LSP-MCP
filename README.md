# Rust and TypeScript LSP for Codex

Ask Codex to find a symbol’s definition, list its references, or show its type and docs. Install **Rust LSP** for Rust projects or **TypeScript LSP** for TypeScript and JavaScript. Each desktop plugin starts its own local language server on the first lookup and reuses it for the same workspace.

Local LSP Tools maintains both plugins. [Support](https://github.com/frontboat/LSP-MCP/issues) · [Privacy](PRIVACY.md) · [Terms](LICENSE)

## Use the tools

| Tool | Result |
| --- | --- |
| `go_to_definition` | Definition locations, including across imports |
| `find_references` | Reference locations. Set `include_declaration` to `false` to omit declarations. |
| `hover` | The symbol’s type and documentation, or `null` in the `text` field when there is no hover |

For example, call `find_references` with these arguments.

```json
{
  "workspace": "/path/to/project",
  "file": "src/main.ts",
  "line": 12,
  "character": 8,
  "include_declaration": true
}
```

Lines and characters start at **1**. Characters count UTF-16 code units, so an emoji such as 😀 occupies two positions. Locations include an absolute `file` path, `line`, `character`, `end_line`, and `end_character`. The end position is exclusive. Successful lookups with no matches return an empty `locations` array.

Use an absolute file path or pass an absolute `workspace` with a relative file path. For a monorepo, pass the same workspace directory across calls to keep the language server running.

- **Rust LSP** accepts saved `.rs` files. It looks up the nearest `Cargo.toml` or `rust-project.json` and waits for rust-analyzer to load the workspace before the first lookup.
- **TypeScript LSP** accepts saved TypeScript and JavaScript files, including TSX, JSX, and module-specific extensions. It looks up the nearest `tsconfig.json`, `jsconfig.json`, or `.git`, falling back to the source file’s directory.

Before each lookup the bridge rereads documents it has opened, sends versioned changes for edits, and closes deleted documents. Edits to files the bridge has never opened depend on the language server’s filesystem watcher and can take a moment to appear.

## Install locally

Make the language server available on Codex’s `PATH`. For Rust, use `rust-analyzer` (tested with 1.94.1). With rustup, `rustup component add rust-analyzer rust-src` installs the server and standard library sources. For TypeScript, use a native compiler that supports `tsc --lsp --stdio` (tested with 7.0.2). To select an executable explicitly, set `LSP_MCP_RUST_ANALYZER` or `LSP_MCP_TSC` in the server environment.

Clone or download this repository, then run these commands from the project root to build the bridge for your machine and install the plugin.

```sh
./scripts/package-plugin.sh rust-lsp
codex plugin marketplace add "$PWD"
codex plugin add rust-lsp@lsp-mcp-local
```

For TypeScript, use `typescript-lsp` in both plugin commands. Register the marketplace once. Open a fresh local Codex chat in your project, select **Rust LSP** or **TypeScript LSP**, and ask “Find the definition of the function called on this line and list its other callers.” The package script also writes a platform-specific archive under `dist/`.

### Direct stdio registration

For a standalone MCP registration, build the executable and register an absolute workspace directory. Relative tool paths will resolve against that directory.

```sh
cargo build --release --locked
codex mcp add rust-lsp -- "$PWD/target/release/lsp-mcp" --language rust --workspace /path/to/project
codex mcp add typescript-lsp -- "$PWD/target/release/lsp-mcp" --workspace /path/to/project
```

Use either the plugin installation or direct registration. The plugin already registers the server.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
```

The integration tests use the official Rust MCP client, temporary Cargo and TypeScript projects, and both language servers. They exercise MCP 2026-07-28 discovery, cross-file navigation, saved edits, and UTF-16 positions. The bridge also accepts the legacy MCP initialization handshake. MCP traffic uses stdout. Diagnostics use stderr.

The MCP handlers use the official Rust SDK. The LSP client uses async-lsp and Tokio. [Rust SDK examples](https://github.com/modelcontextprotocol/rust-sdk/tree/main/examples) · [MCP specification](https://modelcontextprotocol.io/specification/) · [Desktop plugin packaging](https://developers.openai.com/plugins/deploy/desktop-only)
