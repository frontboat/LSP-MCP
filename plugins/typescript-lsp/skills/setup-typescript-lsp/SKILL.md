---
name: setup-typescript-lsp
description: Verify a TypeScript LSP installation or diagnose a failed first lookup in a TypeScript or JavaScript project.
---

1. Use the user's saved project if they supplied a symbol. For an installation check, copy this skill's `example/` directory to a temporary directory. It contains a TypeScript project with no package dependencies.
2. In the example, call `go_to_definition` on `double` in `main.ts`, line 2, character 22, with the absolute path to the copied workspace. The definition is in `math.ts`, line 2. Positions start at 1 and count UTF-16 code units.
3. Call `hover` on the definition and report the inferred return type, documentation, and file location. For a user-supplied symbol, report the location or hover text returned for that symbol.
4. If launch fails, check the bridge's error and the executable selected by `LSP_MCP_TSC` or `tsc` on PATH. The compiler must support `--lsp --stdio`.
