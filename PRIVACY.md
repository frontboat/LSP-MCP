# Local LSP Tools privacy

Rust LSP and TypeScript LSP read saved source files on your computer and send their contents to the selected local language server. A lookup can return source paths, line and character positions, type signatures, and documentation to the MCP client. A client such as Codex can include those results in the conversation under its own data settings.

The bridge communicates over local standard input and output. Its source contains no analytics, remote API endpoint, or publisher-operated data store. It keeps open document contents in process memory while a language-server session runs. The language server and its toolchain can read project configuration, load dependencies, and use their own caches or network features. Client logs, chat history, and toolchain caches follow the settings of those products.

Support issues on GitHub are public. Share a minimal example and remove private source, local paths, and credentials before posting. Questions about the plugins can be raised with [Local LSP Tools on GitHub](https://github.com/frontboat/LSP-MCP/issues).
