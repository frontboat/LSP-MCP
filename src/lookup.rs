use anyhow::{Result, anyhow};
use async_lsp::{LanguageServer, ServerSocket, lsp_types::*};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{num::NonZeroU32, path::PathBuf};

#[derive(Deserialize, JsonSchema)]
pub(crate) struct Cursor {
    /// Absolute source path, or a path relative to an explicit workspace. Without a workspace, relative paths use the bridge's launch directory.
    pub file: PathBuf,
    /// One-based line number.
    pub line: NonZeroU32,
    /// One-based character offset in UTF-16 code units.
    pub character: NonZeroU32,
    /// Absolute project directory. Defaults to the pinned workspace or the nearest project containing the file.
    pub workspace: Option<PathBuf>,
}

#[derive(Deserialize, JsonSchema)]
pub(crate) struct References {
    #[serde(flatten)]
    pub cursor: Cursor,
    /// Include the symbol's declaration. Defaults to true.
    pub include_declaration: Option<bool>,
}

pub(crate) enum Lookup {
    Definition,
    References(bool),
    Hover,
}

#[derive(Serialize, JsonSchema)]
pub(crate) struct Locations {
    locations: Vec<SourceRange>,
}

#[derive(Serialize, JsonSchema, Ord, PartialOrd, Eq, PartialEq)]
struct SourceRange {
    file: PathBuf,
    line: u64,
    character: u64,
    end_line: u64,
    end_character: u64,
}

impl TryFrom<Location> for SourceRange {
    type Error = anyhow::Error;
    fn try_from(loc: Location) -> Result<Self> {
        Ok(Self {
            file: loc
                .uri
                .to_file_path()
                .map_err(|()| anyhow!("expected a local file URI, received {}", loc.uri))?,
            line: u64::from(loc.range.start.line) + 1,
            character: u64::from(loc.range.start.character) + 1,
            end_line: u64::from(loc.range.end.line) + 1,
            end_character: u64::from(loc.range.end.character) + 1,
        })
    }
}

#[derive(Serialize, JsonSchema)]
pub(crate) struct HoverResult {
    /// Type information and documentation, or null if there is no symbol here.
    text: Option<String>,
}

fn marked_text(value: MarkedString) -> String {
    match value {
        MarkedString::String(s) => s,
        MarkedString::LanguageString(code) => code.value,
    }
}

impl Lookup {
    pub(crate) async fn run(
        self,
        server: &mut ServerSocket,
        position: TextDocumentPositionParams,
    ) -> Result<Value> {
        let found = match self {
            Self::Definition => match server
                .definition(GotoDefinitionParams {
                    text_document_position_params: position,
                    work_done_progress_params: Default::default(),
                    partial_result_params: Default::default(),
                })
                .await?
            {
                None => Vec::new(),
                Some(GotoDefinitionResponse::Scalar(loc)) => vec![loc],
                Some(GotoDefinitionResponse::Array(locs)) => locs,
                Some(GotoDefinitionResponse::Link(links)) => links
                    .into_iter()
                    .map(|link| Location::new(link.target_uri, link.target_selection_range))
                    .collect(),
            },
            Self::References(include_declaration) => server
                .references(ReferenceParams {
                    text_document_position: position,
                    context: ReferenceContext {
                        include_declaration,
                    },
                    work_done_progress_params: Default::default(),
                    partial_result_params: Default::default(),
                })
                .await?
                .unwrap_or_default(),
            Self::Hover => {
                let hover = server
                    .hover(HoverParams {
                        text_document_position_params: position,
                        work_done_progress_params: Default::default(),
                    })
                    .await?;
                let text = hover.map(|h| match h.contents {
                    HoverContents::Markup(markup) => markup.value,
                    HoverContents::Scalar(s) => marked_text(s),
                    HoverContents::Array(parts) => parts
                        .into_iter()
                        .map(marked_text)
                        .collect::<Vec<_>>()
                        .join("\n\n"),
                });
                return Ok(serde_json::to_value(HoverResult { text })?);
            }
        };
        let mut locations = found
            .into_iter()
            .map(SourceRange::try_from)
            .collect::<Result<Vec<_>>>()?;
        locations.sort_unstable();
        locations.dedup();
        Ok(serde_json::to_value(Locations { locations })?)
    }
}
