//! Bounded Git-derived activity queries; no audit events or network activity are invented.
use crate::{
    domain::ObjectId,
    history::{self, HistoryScope},
    inspection::{self, ReadOptions},
    Error, ErrorKind, Result,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Commit {
    pub oid: ObjectId,
    pub subject: Vec<u8>,
    pub author: Vec<u8>,
    pub committer_time: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Page {
    pub commits: Vec<Commit>,
    pub complete: bool,
    pub generation: String,
}
pub fn query(
    repo: &Path,
    scope: HistoryScope,
    since: u64,
    until: u64,
    limit: usize,
    options: &ReadOptions,
) -> Result<Page> {
    if since > until
        || until > i64::MAX as u64
        || until - since > 400 * 86400
        || !(1..=10_000).contains(&limit)
    {
        return Err(Error::with_kind(
            ErrorKind::InvalidRequest,
            "Activity requires a valid range of at most 400 days and 1..10000 commits",
        ));
    }
    inspection::read(repo, options, |s| {
        let refs = s.refs()?;
        let mut tips = match &scope {
            HistoryScope::LocalBranches => refs
                .iter()
                .filter(|r| r.name.starts_with("refs/heads/"))
                .map(|r| r.oid.clone())
                .collect::<Vec<_>>(),
            HistoryScope::Reference(name) => {
                let r = refs.iter().find(|r| &r.name == name).ok_or_else(|| {
                    Error::with_kind(ErrorKind::InvalidRequest, "Unknown full reference name")
                })?;
                vec![if r.object_type == "commit" {
                    r.oid.clone()
                } else {
                    r.peeled.clone().ok_or_else(|| {
                        Error::with_kind(ErrorKind::InvalidRequest, "Reference is not a commit")
                    })?
                }]
            }
        };
        tips.sort_by(|a, b| a.hex.cmp(&b.hex));
        tips.dedup();
        if tips.is_empty() {
            return Ok(Page {
                commits: Vec::new(),
                complete: true,
                generation: s.generation.clone(),
            });
        }
        let mut args = vec![
            "rev-list".to_owned(),
            "--topo-order".into(),
            format!("--since-as-filter=@{since}"),
            format!("--until=@{until}"),
            format!("--max-count={}", limit + 1),
        ];
        args.extend(tips.iter().map(|id| id.hex.clone()));
        let refs: Vec<_> = args.iter().map(String::as_str).collect();
        let output = s.git(&refs)?;
        let ids: Vec<_> = output
            .split(|b| *b == b'\n')
            .filter(|s| !s.is_empty())
            .collect();
        let complete = ids.len() <= limit;
        let commits = ids
            .into_iter()
            .take(limit)
            .map(|id| {
                let id = ObjectId::parse(
                    std::str::from_utf8(id)
                        .map_err(|_| inspection::data_error("Invalid activity OID"))?,
                    &s.format,
                )?;
                let c = history::commit_in(s, &id)?;
                Ok(Commit {
                    oid: c.oid,
                    subject: c
                        .message
                        .split(|b| *b == b'\n')
                        .next()
                        .unwrap_or_default()
                        .iter()
                        .copied()
                        .take(512)
                        .collect(),
                    author: c.author.into_iter().take(1024).collect(),
                    committer_time: c.committer_time,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Page {
            commits,
            complete,
            generation: s.generation.clone(),
        })
    })
}
