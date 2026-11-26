use std::{env, path::Path, time::Duration};

use anyhow::{Result, bail};
use tokio::process::Command;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Language {
    TypeScript,
    Rust,
}

impl Language {
    pub(crate) fn manifest(self) -> &'static str {
        match self {
            Self::TypeScript => include_str!("../plugins/typescript-lsp/plugin.json"),
            Self::Rust => include_str!("../plugins/rust-lsp/plugin.json"),
        }
    }

    pub(crate) fn sources(self) -> &'static str {
        match self {
            Self::TypeScript => "TS/JS",
            Self::Rust => "Rust",
        }
    }

    pub(crate) fn language_id(self, path: &Path) -> Result<&'static str> {
        match (self, path.extension().and_then(|s| s.to_str())) {
            (Self::TypeScript, Some("ts" | "mts" | "cts")) => Ok("typescript"),
            (Self::TypeScript, Some("tsx")) => Ok("typescriptreact"),
            (Self::TypeScript, Some("js" | "mjs" | "cjs")) => Ok("javascript"),
            (Self::TypeScript, Some("jsx")) => Ok("javascriptreact"),
            (Self::Rust, Some("rs")) => Ok("rust"),
            _ => bail!(
                "expected a {} source file: {}",
                self.sources(),
                path.display()
            ),
        }
    }

    pub(crate) fn project_markers(self) -> &'static [&'static str] {
        match self {
            Self::TypeScript => &["tsconfig.json", "jsconfig.json", ".git"],
            Self::Rust => &["Cargo.toml", "rust-project.json"],
        }
    }

    pub(crate) fn command(self) -> Command {
        match self {
            Self::TypeScript => {
                let mut cmd =
                    Command::new(env::var_os("LSP_MCP_TSC").unwrap_or_else(|| "tsc".into()));
                cmd.args(["--lsp", "--stdio"]);
                cmd
            }
            Self::Rust => Command::new(
                env::var_os("LSP_MCP_RUST_ANALYZER").unwrap_or_else(|| "rust-analyzer".into()),
            ),
        }
    }

    pub(crate) fn lookup_timeout(self) -> Duration {
        Duration::from_secs(match self {
            Self::TypeScript => 45,
            Self::Rust => 120,
        })
    }
}
