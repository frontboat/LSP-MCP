---
name: setup-rust-lsp
description: Set up rust-analyzer for Rust LSP, install missing toolchain components, and verify a lookup in a saved Rust project.
---

Run setup on the machine that runs Codex. Execute the install commands for the missing dependencies. Start in the user's Cargo workspace so rustup honors its toolchain pin.

1. Check `cargo --version`, `rustc --version`, and `"${LSP_MCP_RUST_ANALYZER:-rust-analyzer}" --version`. A rustup shim can exist even when the component is missing. Reuse working executables.
2. Install what is missing using the toolchain that owns the workspace:
   - **rustup:** Check `rustup show active-toolchain` and `rustup component list --installed`. If there is no default or project pin, run `rustup default stable`. From the workspace, run `rustup component add rust-analyzer rust-src` for missing components. Keep the project's pin. If that channel lacks rust-analyzer, use an available standalone server and verify it against the pinned workspace.
   - **Homebrew-managed Rust:** Run `brew install rust-analyzer` if the server is missing. Use `brew install rust` if Cargo or rustc is missing.
   - **No Rust toolchain:** If rustup exists in `${CARGO_HOME:-$HOME/.cargo}/bin`, add that directory to `PATH` and use it. Otherwise, run the following in one shell, then follow the rustup step. [Rust installer](https://rustup.rs/)

     ```sh
     installer=$(mktemp)
     curl --proto '=https' --tlsv1.2 -fsSLo "$installer" https://sh.rustup.rs &&
       sh "$installer" -y --profile minimal &&
       . "${CARGO_HOME:-$HOME/.cargo}/env"
     ```

3. Recheck the executables from step 1. If Codex cannot spawn the server, check the bridge's `LSP_MCP_RUST_ANALYZER` override and its `PATH`. For rustup, include `${CARGO_HOME:-$HOME/.cargo}/bin` so the bridge can also run Cargo and rustc. Retry the lookup after correcting the launch environment. A running bridge needs a restart to inherit a changed environment.
4. Verify through the **Rust LSP MCP tools**. For the bundled check, copy this skill's `example/` to a temporary directory. Call `go_to_definition` with that directory as the absolute `workspace`, `file="src/lib.rs"`, `line=2`, and `character=31`. Expect `src/math.rs`, line 2. Call `hover` at the returned location and check for `double` and `Doubles a count.`. If the user supplied a symbol, run the lookup in their workspace as well. Report the server version and result, or the command/tool error that blocked setup.
