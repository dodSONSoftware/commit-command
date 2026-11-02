//! Graph geometry is derived only from actual commit parents, with bounded page continuity.
use crate::{domain::ObjectId, history, inspection::ReadOptions, Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Cursor {
    pub history: history::Cursor,
    pub lanes: Vec<ObjectId>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Row {
    pub commit: history::Commit,
    pub lane: usize,
    pub tip: bool,
    pub before: Vec<ObjectId>,
    pub after: Vec<ObjectId>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Page {
    pub rows: Vec<Row>,
    pub references: Vec<crate::inspection::Reference>,
    pub next: Option<Cursor>,
    pub shallow_boundaries: Vec<ObjectId>,
    pub generation: String,
}
fn layout(commits: Vec<history::Commit>, lanes: &mut Vec<ObjectId>) -> Result<Vec<Row>> {
    commits
        .into_iter()
        .map(|commit| {
            let tip = !lanes.contains(&commit.oid);
            let lane = lanes
                .iter()
                .position(|id| id == &commit.oid)
                .unwrap_or_else(|| {
                    lanes.push(commit.oid.clone());
                    lanes.len() - 1
                });
            if lanes.len() > 256 {
                return Err(Error::with_kind(
                    ErrorKind::OutputLimit,
                    "Graph exceeds 256 active lanes",
                ));
            }
            let before = lanes.clone();
            lanes.remove(lane);
            let mut insertion = lane;
            for parent in &commit.parents {
                if !lanes.contains(parent) {
                    lanes.insert(insertion, parent.clone());
                    insertion += 1;
                    if lanes.len() > 256 {
                        return Err(Error::with_kind(
                            ErrorKind::OutputLimit,
                            "Graph exceeds 256 active lanes",
                        ));
                    }
                }
            }
            Ok(Row {
                commit,
                lane,
                tip,
                before,
                after: lanes.clone(),
            })
        })
        .collect()
}
pub fn page(
    repo: &Path,
    scope: history::HistoryScope,
    limit: usize,
    cursor: Option<&Cursor>,
    options: &ReadOptions,
) -> Result<Page> {
    let mut lanes = cursor.map(|c| c.lanes.clone()).unwrap_or_default();
    if lanes.len() > 256 {
        return Err(Error::with_kind(
            ErrorKind::InvalidRequest,
            "Graph lane limit exceeded",
        ));
    }
    for id in &lanes {
        id.validate()?;
    }
    if lanes
        .iter()
        .enumerate()
        .any(|(i, id)| lanes[..i].contains(id))
    {
        return Err(Error::with_kind(
            ErrorKind::InvalidRequest,
            "Duplicate graph lane",
        ));
    }
    let history =
        history::page_with_options(repo, scope, limit, cursor.map(|c| &c.history), options)?;
    if lanes.iter().any(|id| {
        history
            .commits
            .first()
            .is_some_and(|c| id.format != c.oid.format)
    }) {
        return Err(Error::with_kind(
            ErrorKind::InvalidRequest,
            "Graph object format mismatch",
        ));
    }
    let rows = layout(history.commits, &mut lanes)?;
    Ok(Page {
        rows,
        references: history.references,
        next: history.next.map(|history| Cursor { history, lanes }),
        shallow_boundaries: history.shallow_boundaries,
        generation: history.generation,
    })
}
