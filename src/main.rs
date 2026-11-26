mod bridge;
mod documents;
mod language;
mod lookup;
mod session;

use std::{env, path::PathBuf};

use anyhow::{Context, Result, bail};
use language::Language;
use rmcp::{ServiceExt, transport::stdio};
use tokio_util::sync::CancellationToken;

fn main() -> Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let result = runtime.block_on(run());
    // Tokio's blocking stdin read can outlive SIGTERM while the client keeps its pipe open.
    // All LSP cleanup has already finished by the time run returns.
    runtime.shutdown_background();
    result
}

async fn run() -> Result<()> {
    let mut args = env::args_os().skip(1);
    let mut language = Language::TypeScript;
    let mut workspace = None;
    const USAGE: &str = "Usage: lsp-mcp [--language typescript|rust] [--workspace DIRECTORY]";
    while let Some(arg) = args.next() {
        if arg == "--language" {
            language = match args.next().as_deref().and_then(|s| s.to_str()) {
                Some("typescript") => Language::TypeScript,
                Some("rust") => Language::Rust,
                _ => bail!("--language requires typescript or rust"),
            };
        } else if arg == "--workspace" && workspace.is_none() {
            workspace = Some(PathBuf::from(
                args.next().context("--workspace needs a directory")?,
            ));
        } else if arg == "--help" || arg == "-h" {
            eprintln!(
                "{USAGE}\nSet LSP_MCP_TSC or LSP_MCP_RUST_ANALYZER to select the language server executable."
            );
            return Ok(());
        } else {
            bail!("{USAGE}");
        }
    }

    let stop = CancellationToken::new();
    let (server, worker) = bridge::start(language, workspace, stop.clone()).await?;
    let serving = async {
        let service = server.serve_with_ct(stdio(), stop.clone()).await?;
        service.waiting().await?;
        Ok::<_, anyhow::Error>(())
    };
    let result = tokio::select! {
        result = serving => result,
        result = shutdown_signal() => result,
    };
    stop.cancel();
    worker.await?;
    result
}

async fn shutdown_signal() -> Result<()> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut terminate = signal(SignalKind::terminate())?;
        tokio::select! {
            _ = terminate.recv() => {},
            result = tokio::signal::ctrl_c() => result?,
        }
    }
    #[cfg(not(unix))]
    tokio::signal::ctrl_c().await?;
    Ok(())
}
