use std::{
    future::ready,
    ops::ControlFlow,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use anyhow::{Context, Result, ensure};
use async_lsp::{
    ErrorCode, LanguageServer, MainLoop, ResponseError, ServerSocket,
    lsp_types::{
        request::{RegisterCapability, WorkspaceConfiguration, WorkspaceFoldersRequest},
        *,
    },
    router::Router,
};
use tokio::{process::Child, sync::oneshot, task::JoinHandle, time::timeout};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::{
    documents::{Documents, file_uri},
    language::Language,
};

// rust-analyzer answers initialize before its Cargo workspace is ready for navigation.
enum RustAnalyzerStatus {}

#[derive(serde::Serialize, serde::Deserialize)]
struct Status {
    quiescent: bool,
    health: String,
    message: Option<String>,
}

impl notification::Notification for RustAnalyzerStatus {
    const METHOD: &'static str = "experimental/serverStatus";
    type Params = Status;
}

pub(crate) struct Session {
    pub root: PathBuf,
    pub server: ServerSocket,
    pub documents: Documents,
    child: Child,
    transport: JoinHandle<async_lsp::Result<()>>,
}

impl Session {
    pub(crate) async fn start(language: Language, root: PathBuf) -> Result<Self> {
        let mut command = language.command();
        let mut child = command
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
            .with_context(|| {
                format!(
                    "starting {:?} in {}",
                    command.as_std().get_program(),
                    root.display()
                )
            })?;
        let input = child
            .stdout
            .take()
            .context("missing language server stdout")?;
        let output = child
            .stdin
            .take()
            .context("missing language server stdin")?;
        let folder = WorkspaceFolder {
            uri: file_uri(&root)?,
            name: root
                .file_name()
                .unwrap_or(root.as_os_str())
                .to_string_lossy()
                .into_owned(),
        };
        let (loaded_tx, loaded) = oneshot::channel();
        let (mainloop, server) = MainLoop::new_client(|_| {
            let mut router = Router::new((folder.clone(), Some(loaded_tx)));
            if language == Language::Rust {
                router.notification::<RustAnalyzerStatus>(|(_, loaded_tx), status| {
                    if status.quiescent
                        && let Some(tx) = loaded_tx.take()
                    {
                        let _ = tx.send(status);
                    }
                    ControlFlow::Continue(())
                });
            }
            router
                .request::<RegisterCapability, _>(|_, params| {
                    // The bridge has fixed editor settings, so no settings changes will fire.
                    ready(
                        if params
                            .registrations
                            .iter()
                            .all(|r| r.method == "workspace/didChangeConfiguration")
                        {
                            Ok(())
                        } else {
                            Err(ResponseError::new(
                                ErrorCode::INVALID_PARAMS,
                                "unsupported dynamic registration",
                            ))
                        },
                    )
                })
                .request::<WorkspaceConfiguration, _>(|_, params| {
                    ready(Ok(vec![serde_json::Value::Null; params.items.len()]))
                })
                .request::<WorkspaceFoldersRequest, _>(|(folder, _), ()| {
                    ready(Ok(Some(vec![folder.clone()])))
                })
                .unhandled_notification(|_, _| ControlFlow::Continue(()));
            router
        });
        let transport = tokio::spawn(async move {
            mainloop
                .run_buffered(input.compat(), output.compat_write())
                .await
        });
        let mut session = Self {
            root,
            server,
            child,
            transport,
            documents: Documents::default(),
        };
        let init = session
            .server
            .initialize(InitializeParams {
                process_id: Some(std::process::id()),
                workspace_folders: Some(vec![folder]),
                initialization_options: (language == Language::Rust).then(|| {
                    serde_json::json!({
                        "checkOnSave": false,
                        "files": {"watcher": "server"}
                    })
                }),
                capabilities: ClientCapabilities {
                    experimental: (language == Language::Rust)
                        .then(|| serde_json::json!({"serverStatusNotification": true})),
                    workspace: Some(WorkspaceClientCapabilities {
                        workspace_folders: Some(true),
                        configuration: Some(language == Language::TypeScript),
                        ..Default::default()
                    }),
                    general: Some(GeneralClientCapabilities {
                        position_encodings: Some(vec![PositionEncodingKind::UTF16]),
                        ..Default::default()
                    }),
                    text_document: Some(TextDocumentClientCapabilities {
                        hover: Some(HoverClientCapabilities {
                            content_format: Some(vec![MarkupKind::Markdown, MarkupKind::PlainText]),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await
            .with_context(|| format!("initializing {:?}", command.as_std().get_program()))?;
        ensure!(
            init.capabilities
                .position_encoding
                .is_none_or(|encoding| encoding == PositionEncodingKind::UTF16),
            "language server selected an unsupported position encoding"
        );
        session.server.initialized(InitializedParams {})?;
        if language == Language::Rust {
            let status = loaded
                .await
                .context("rust-analyzer exited while loading the workspace")?;
            ensure!(
                status.health != "error",
                "rust-analyzer workspace error: {}",
                status.message.as_deref().unwrap_or("unknown error")
            );
        }
        Ok(session)
    }

    pub(crate) fn is_live_for(&mut self, root: &Path) -> Result<bool> {
        Ok(self.root == root && !self.transport.is_finished() && self.child.try_wait()?.is_none())
    }

    pub(crate) async fn stop(mut self) {
        let graceful = async {
            self.server.shutdown(()).await?;
            self.server.exit(())?;
            self.child.wait().await?;
            Ok::<_, anyhow::Error>(())
        };
        if !matches!(timeout(Duration::from_secs(2), graceful).await, Ok(Ok(())))
            && let Err(error) = self.child.kill().await
        {
            eprintln!("stopping language server: {error}");
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // Also release both pipes on cancellation during initialization.
        self.transport.abort();
    }
}
