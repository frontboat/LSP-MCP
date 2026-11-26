use std::{
    collections::HashMap,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, ensure};
use async_lsp::{LanguageServer, ServerSocket, lsp_types::*};
use tokio::fs;

#[derive(Default)]
pub(crate) struct Documents(HashMap<PathBuf, OpenDocument>);

struct OpenDocument {
    uri: Url,
    version: i32,
    text: String,
}

pub(crate) fn file_uri(path: &Path) -> Result<Url> {
    Url::from_file_path(path).map_err(|()| anyhow!("invalid file path {}", path.display()))
}

// Unlike str::lines, LSP includes the empty final line and recognizes a lone CR.
fn lines(text: &str) -> impl Iterator<Item = &str> {
    let mut rest = Some(text);
    std::iter::from_fn(move || {
        let line = rest.take()?;
        if let Some(i) = line.find(['\r', '\n']) {
            let tail = &line[i..];
            rest = Some(&tail[if tail.starts_with("\r\n") { 2 } else { 1 }..]);
            Some(&line[..i])
        } else {
            Some(line)
        }
    })
}

fn end_position(text: &str) -> Result<Position> {
    let (index, line) = lines(text)
        .enumerate()
        .last()
        .context("missing document line")?;
    Ok(Position::new(
        index.try_into()?,
        line.encode_utf16().count().try_into()?,
    ))
}

fn validate_position(text: &str, pos: Position) -> Result<()> {
    let line = lines(text)
        .nth(pos.line as usize)
        .context("line is past the end of the document")?;
    let mut remaining = pos.character;
    for ch in line.chars() {
        if remaining == 0 {
            return Ok(());
        }
        remaining = remaining
            .checked_sub(ch.len_utf16() as u32)
            .context("character splits a UTF-16 surrogate pair")?;
    }
    ensure!(remaining == 0, "character is past the end of the line");
    Ok(())
}

impl Documents {
    pub(crate) async fn sync(
        &mut self,
        server: &mut ServerSocket,
        file: &Path,
        language: &str,
        position: Position,
    ) -> Result<TextDocumentPositionParams> {
        let mut closed = Vec::new();
        for (path, doc) in &mut self.0 {
            let text = match fs::read_to_string(path).await {
                Ok(text) => text,
                Err(e) if e.kind() == ErrorKind::NotFound => {
                    closed.push((path.clone(), doc.uri.clone()));
                    continue;
                }
                Err(e) => return Err(e).with_context(|| format!("reading {}", path.display())),
            };
            if text != doc.text {
                let version = doc
                    .version
                    .checked_add(1)
                    .context("document version exhausted")?;
                server.did_change(DidChangeTextDocumentParams {
                    text_document: VersionedTextDocumentIdentifier::new(doc.uri.clone(), version),
                    content_changes: vec![TextDocumentContentChangeEvent {
                        range: Some(Range::new(Position::default(), end_position(&doc.text)?)),
                        range_length: None,
                        text: text.clone(),
                    }],
                })?;
                doc.text = text;
                doc.version = version;
            }
        }
        for (path, uri) in closed {
            server.did_close(DidCloseTextDocumentParams {
                text_document: TextDocumentIdentifier::new(uri),
            })?;
            self.0.remove(&path);
        }

        let doc = if let Some(doc) = self.0.get(file) {
            doc
        } else {
            let text = fs::read_to_string(file)
                .await
                .with_context(|| format!("reading {}", file.display()))?;
            let uri = file_uri(file)?;
            server.did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem::new(
                    uri.clone(),
                    language.to_owned(),
                    1,
                    text.clone(),
                ),
            })?;
            self.0.entry(file.to_owned()).or_insert(OpenDocument {
                uri,
                version: 1,
                text,
            })
        };
        validate_position(&doc.text, position)?;
        Ok(TextDocumentPositionParams::new(
            TextDocumentIdentifier::new(doc.uri.clone()),
            position,
        ))
    }
}
