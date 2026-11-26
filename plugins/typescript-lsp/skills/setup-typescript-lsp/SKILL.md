---
name: setup-typescript-lsp
description: Set up the native TypeScript language server for TypeScript LSP, replace an incompatible compiler for the bridge, and verify a lookup.
---

Run setup on the machine that runs Codex. Install the compiler as a user tool. Preserve the project's TypeScript dependency and lockfile.

1. Check the executable the bridge uses. Run `"${LSP_MCP_TSC:-tsc}" --version`, then `"${LSP_MCP_TSC:-tsc}" --lsp --stdio </dev/null` with a short timeout. Native TypeScript 7 accepts these flags and exits on EOF. An older compiler can print a valid version and still reject `--lsp`. If the probe succeeds, skip installation.
2. Install a compatible compiler using an available package manager:
   - **Homebrew:** Run `brew install typescript`. If Homebrew already owns an outdated copy, run `brew upgrade typescript`. Probe `"$(brew --prefix typescript)/bin/tsc"`.
   - **npm:** Run `npm install --global typescript`, then probe `"$(npm prefix --global)/bin/tsc"`. To keep an existing global compiler or use a writable prefix, run `npm install --global --prefix "$HOME/.local/share/lsp-mcp/typescript" typescript`. Its executable is `$HOME/.local/share/lsp-mcp/typescript/bin/tsc`.
   - **Neither installed:** Install Node.js LTS with the user's Node version manager or the official Node.js distribution, then follow the npm step. [Node.js downloads](https://nodejs.org/en/download)
3. Repeat both probes with the selected executable. If Codex finds a different compiler, put the selected bin directory first on the `PATH` used to launch Codex, or set `LSP_MCP_TSC` to its absolute executable path in the bridge's launch environment. Restart the bridge after changing that environment. A one-off shell export affects only that shell and its children.
4. Verify through the **TypeScript LSP MCP tools**. Copy this skill's `example/` to a temporary directory. Call `go_to_definition` with that directory as the absolute `workspace`, `file="main.ts"`, `line=2`, and `character=22`. Expect `math.ts`, line 2. Call `hover` at the returned location and check for the `number` return type and `Doubles a count.`. If the user supplied a symbol, run the lookup in their project as well. Report the compiler version and result, or the command/tool error that blocked setup.
