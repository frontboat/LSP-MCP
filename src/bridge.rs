use crate::{
    language::Language,
    lookup::{Cursor, HoverResult, Locations, Lookup, References},
    session::Session,
};
use anyhow::{Context, Result, bail, ensure};
use async_lsp::lsp_types::Position;
use rmcp::{
    ServerHandler,
    handler::server::{tool::schema_for_type, wrapper::Parameters},
    model::{CallToolResult, Implementation, ServerCapabilities, ServerConfig},
    tool, tool_handler, tool_router,
};
use serde_json::Value;
use std::{
    env,
    path::{Path, PathBuf},
};
use tokio::{
    fs,
    sync::{mpsc, oneshot},
    task::JoinHandle,
    time::timeout,
};
use tokio_util::sync::CancellationToken;

struct Job {
    lookup: Lookup,
    cursor: Cursor,
    reply: oneshot::Sender<Result<Value>>,
}

#[derive(Clone)]
pub(crate) struct Bridge {
    jobs: mpsc::Sender<Job>,
    info: ServerConfig,
}

fn server_config(language: Language) -> Result<ServerConfig> {
    let manifest: Value = serde_json::from_str(language.manifest())
        .context("reading the embedded plugin manifest")?;
    let field = |path: &str| {
        manifest
            .pointer(path)
            .and_then(Value::as_str)
            .with_context(|| format!("expected a string at {path} in the plugin manifest"))
    };
    let name = field("/extensions/com.openai/interface/displayName")?;
    let identity = Implementation::new(name, field("/version")?)
        .with_title(name)
        .with_description(field("/description")?);
    Ok(ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
        .with_server_info(identity)
        .with_instructions(format!(
            "In saved {} files, use go_to_definition to follow a symbol, find_references to locate its uses, \
             and hover to read its type or docs. Pass an absolute file path or a workspace. \
             Lines and UTF-16 character offsets start at 1.", language.sources()
        )))
}

pub(crate) async fn start(
    language: Language,
    pinned: Option<PathBuf>,
    stop: CancellationToken,
) -> Result<(Bridge, JoinHandle<()>)> {
    let pinned = match pinned {
        Some(path) => Some(directory(&path).await?),
        None => None,
    };
    let (jobs, incoming) = mpsc::channel(16);
    let worker = Worker {
        language,
        pinned,
        cwd: env::current_dir()?,
        session: None,
    };
    let bridge = Bridge {
        jobs,
        info: server_config(language)?,
    };
    Ok((bridge, tokio::spawn(worker.run(incoming, stop))))
}

#[tool_handler]
impl ServerHandler for Bridge {
    fn get_info(&self) -> ServerConfig {
        self.info.clone()
    }
}

#[tool_router]
impl Bridge {
    #[tool(description = "Find references to the symbol at the given position. Returns absolute paths and one-based UTF-16 ranges with exclusive ends.", annotations(read_only_hint = true), output_schema = schema_for_type::<Locations>())]
    async fn find_references(
        &self,
        Parameters(args): Parameters<References>,
        cancel: CancellationToken,
    ) -> Result<CallToolResult, String> {
        self.query(
            Lookup::References(args.include_declaration.unwrap_or(true)),
            args.cursor,
            cancel,
        )
        .await
    }

    #[tool(description = "Go to the definition of the symbol at the given position. Returns absolute paths and one-based UTF-16 ranges with exclusive ends.", annotations(read_only_hint = true), output_schema = schema_for_type::<Locations>())]
    async fn go_to_definition(
        &self,
        Parameters(cursor): Parameters<Cursor>,
        cancel: CancellationToken,
    ) -> Result<CallToolResult, String> {
        self.query(Lookup::Definition, cursor, cancel).await
    }

    #[tool(description = "Read the symbol's type signature and documentation at the given position.", annotations(read_only_hint = true), output_schema = schema_for_type::<HoverResult>())]
    async fn hover(
        &self,
        Parameters(cursor): Parameters<Cursor>,
        cancel: CancellationToken,
    ) -> Result<CallToolResult, String> {
        self.query(Lookup::Hover, cursor, cancel).await
    }
}

impl Bridge {
    async fn query(
        &self,
        lookup: Lookup,
        cursor: Cursor,
        cancel: CancellationToken,
    ) -> Result<CallToolResult, String> {
        let (reply, result) = oneshot::channel();
        let response = async {
            self.jobs
                .send(Job {
                    lookup,
                    cursor,
                    reply,
                })
                .await
                .map_err(|_| "bridge stopped")?;
            result
                .await
                .map_err(|_| "bridge stopped")?
                .map(CallToolResult::structured)
                .map_err(|err| format!("{err:#}"))
        };
        cancel
            .run_until_cancelled(response)
            .await
            .ok_or("lookup cancelled")?
    }
}

struct Worker {
    language: Language,
    pinned: Option<PathBuf>,
    cwd: PathBuf,
    session: Option<Session>,
}

impl Worker {
    async fn run(mut self, mut jobs: mpsc::Receiver<Job>, stop: CancellationToken) {
        loop {
            let mut job = tokio::select! {
                _ = stop.cancelled() => break,
                job = jobs.recv() => match job {
                    Some(job) => job,
                    None => break,
                },
            };
            if job.reply.is_closed() {
                continue;
            }
            let limit = self.language.lookup_timeout();
            let result = tokio::select! {
                _ = stop.cancelled() => break,
                _ = job.reply.closed() => {
                    // async-lsp does not cancel an on-wire request when its future is dropped.
                    self.stop_session().await;
                    continue;
                },
                value = timeout(limit, self.lookup(job.lookup, job.cursor)) => value,
            };
            let result = match result {
                Ok(result) => result,
                Err(elapsed) => {
                    self.stop_session().await;
                    Err(elapsed).with_context(|| {
                        format!(
                            "{} lookup exceeded {} seconds",
                            self.language.sources(),
                            limit.as_secs()
                        )
                    })
                }
            };
            // A cancelled MCP caller may have dropped its receiver.
            let _ = job.reply.send(result);
        }
        self.stop_session().await;
    }

    async fn stop_session(&mut self) {
        if let Some(session) = self.session.take() {
            session.stop().await
        }
    }

    async fn lookup(&mut self, lookup: Lookup, cursor: Cursor) -> Result<Value> {
        let (root, file) = self.resolve(&cursor).await?;
        let language_id = self.language.language_id(&file)?;
        let live = match &mut self.session {
            Some(session) => session.is_live_for(&root)?,
            None => false,
        };
        if !live {
            self.stop_session().await;
            self.session = Some(Session::start(self.language, root).await?);
        }
        let session = self
            .session
            .as_mut()
            .context("missing language server session")?;
        let position = Position::new(cursor.line.get() - 1, cursor.character.get() - 1);
        let params = session
            .documents
            .sync(&mut session.server, &file, language_id, position)
            .await?;
        lookup.run(&mut session.server, params).await
    }

    async fn resolve(&self, cursor: &Cursor) -> Result<(PathBuf, PathBuf)> {
        let explicit = match &cursor.workspace {
            Some(path) => {
                ensure!(
                    path.is_absolute(),
                    "workspace must be an absolute directory"
                );
                let root = directory(path).await?;
                ensure!(
                    self.pinned.as_ref().is_none_or(|p| *p == root),
                    "workspace differs from the pinned --workspace"
                );
                Some(root)
            }
            None => self.pinned.clone(),
        };
        let source = explicit.as_ref().unwrap_or(&self.cwd).join(&cursor.file);
        let file = fs::canonicalize(&source)
            .await
            .with_context(|| format!("resolving {}", source.display()))?;
        ensure!(
            fs::metadata(&file).await?.is_file(),
            "expected a regular source file"
        );
        let root = match explicit {
            Some(root) => {
                ensure!(
                    file.starts_with(&root),
                    "file is outside workspace {}",
                    root.display()
                );
                root
            }
            None => project_for(self.language, &file).await?,
        };
        Ok((root, file))
    }
}

async fn directory(path: &Path) -> Result<PathBuf> {
    let dir = fs::canonicalize(path)
        .await
        .with_context(|| format!("resolving workspace {}", path.display()))?;
    ensure!(
        fs::metadata(&dir).await?.is_dir(),
        "workspace must be a directory"
    );
    Ok(dir)
}

async fn project_for(language: Language, file: &Path) -> Result<PathBuf> {
    let Some(parent) = file.parent() else {
        bail!("file needs a parent directory")
    };
    for dir in parent.ancestors() {
        for marker in language.project_markers() {
            if fs::try_exists(dir.join(marker)).await? {
                return Ok(dir.to_owned());
            }
        }
    }
    if language == Language::Rust {
        bail!(
            "no Cargo.toml or rust-project.json found above {}",
            file.display()
        );
    }
    Ok(parent.to_owned())
}
