---
name: setup-rust-lsp
description: Verify a Rust LSP installation or diagnose a failed first lookup in a Cargo workspace.
---

1. Use the user's saved Rust workspace if they supplied a symbol. For an installation check, copy this skill's `example/` directory to a temporary directory. It contains a dependency-free Cargo project.
2. In the example, call `go_to_definition` on `double` in `src/lib.rs`, line 2, character 31, with the absolute path to the copied workspace. The definition is in `src/math.rs`, line 2. Positions start at 1 and count UTF-16 code units.
3. Call `hover` on the definition and report the signature, its documentation, and the file location. For a user-supplied symbol, report the location or hover text returned for that symbol.
4. If launch fails, check the bridge's error, the executable selected by `LSP_MCP_RUST_ANALYZER` or `rust-analyzer` on PATH, and whether the workspace has a `Cargo.toml` or `rust-project.json`.
