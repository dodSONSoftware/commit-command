//! Actual Git parent edges and bounded history, without graph layout or networking.
use crate::domain::{now_ms, ObjectId};
use crate::inspection::{self, data_error, ReadOptions, Reference, Session};
use crate::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Commit {
    pub oid: ObjectId,
    pub tree: ObjectId,
    pub parents: Vec<ObjectId>,
    pub author: Vec<u8>,
    pub committer: Vec<u8>,
    pub message: Vec<u8>,
    pub author_time: i64,
    pub committer_time: i64,
    pub encoding: Option<String>,
}
pub(crate) fn commit_in(s: &Session<'_>, oid: &ObjectId) -> Result<Commit> {
    oid.validate()?;
    if oid.format != s.format {
        return Err(data_error("Object format mismatch"));
    }
    let bytes = s.git(&["cat-file", "commit", &oid.hex]).map_err(|error| {
        if error.kind == ErrorKind::InspectionFailed {
            Error::with_kind(
                ErrorKind::MissingObject,
                "Required commit is unavailable; fetching is disabled",
            )
        } else {
            error
        }
    })?;
    parse_commit(s, oid, &bytes)
}
fn parse_commit(s: &Session<'_>, oid: &ObjectId, bytes: &[u8]) -> Result<Commit> {
    let separator = bytes
        .windows(2)
        .position(|pair| pair == b"\n\n")
        .ok_or_else(|| data_error("Invalid commit headers"))?;
    let mut tree = None;
    let mut parents = Vec::new();
    let mut author = None;
    let mut committer = None;
    let mut encoding = None;
    for line in bytes[..separator].split(|b| *b == b'\n') {
        let oid_value = |value: &[u8]| {
            ObjectId::parse(
                std::str::from_utf8(value).map_err(|_| data_error("Invalid commit OID"))?,
                &s.format,
            )
        };
        if let Some(value) = line.strip_prefix(b"tree ") {
            tree = Some(oid_value(value)?);
        }
        if let Some(value) = line.strip_prefix(b"parent ") {
            parents.push(oid_value(value)?);
        }
        if let Some(value) = line.strip_prefix(b"author ") {
            author = Some(value.to_vec());
        }
        if let Some(value) = line.strip_prefix(b"committer ") {
            committer = Some(value.to_vec());
        }
        if let Some(value) = line.strip_prefix(b"encoding ") {
            encoding = Some(String::from_utf8_lossy(value).into_owned());
        }
    }
    let author = author.ok_or_else(|| data_error("Missing author"))?;
    let committer = committer.ok_or_else(|| data_error("Missing committer"))?;
    let timestamp = |identity: &[u8]| -> Result<i64> {
        let value = identity
            .rsplit(|b| *b == b' ')
            .nth(1)
            .ok_or_else(|| data_error("Missing commit timestamp"))?;
        std::str::from_utf8(value)
            .ok()
            .and_then(|value| value.parse().ok())
            .ok_or_else(|| data_error("Invalid commit timestamp"))
    };
    let author_time = timestamp(&author)?;
    let committer_time = timestamp(&committer)?;
    Ok(Commit {
        oid: oid.clone(),
        tree: tree.ok_or_else(|| data_error("Missing commit tree"))?,
        parents,
        author,
        committer,
        author_time,
        committer_time,
        encoding,
        message: bytes[separator + 2..].to_vec(),
    })
}
pub(crate) const COMMIT_BATCH_SIZE: usize = 64;
pub(crate) fn commits_in(s: &Session<'_>, ids: &[ObjectId]) -> Result<Vec<Commit>> {
    if ids.len() > COMMIT_BATCH_SIZE {
        return Err(data_error("Commit batch exceeds bounds"));
    }
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut input = Vec::new();
    for oid in ids {
        oid.validate()?;
        if oid.format != s.format {
            return Err(data_error("Object format mismatch"));
        }
        input.extend_from_slice(oid.hex.as_bytes());
        input.push(b'\n');
    }
    let bytes = s.git_input(&["cat-file", "--batch"], input)?;
    let mut remaining = bytes.as_slice();
    let mut commits = Vec::new();
    for oid in ids {
        s.check()?;
        let end = remaining
            .iter()
            .position(|b| *b == b'\n')
            .ok_or_else(|| data_error("Missing batch header"))?;
        let header = std::str::from_utf8(&remaining[..end])
            .map_err(|_| data_error("Invalid batch header"))?;
        let parts = header.split(' ').collect::<Vec<_>>();
        if parts.first().copied() != Some(oid.hex.as_str()) {
            return Err(data_error("Unexpected batch object ID"));
        }
        if parts.as_slice() == [oid.hex.as_str(), "missing"]
            || parts.get(1).copied() != Some("commit")
        {
            return Err(Error::with_kind(
                ErrorKind::MissingObject,
                "Required commit is unavailable; fetching is disabled",
            ));
        }
        if parts.len() != 3 {
            return Err(data_error("Invalid batch object header"));
        }
        let size = parts[2]
            .parse::<usize>()
            .map_err(|_| data_error("Invalid batch object size"))?;
        remaining = &remaining[end + 1..];
        if size >= remaining.len() || remaining[size] != b'\n' {
            return Err(data_error("Truncated batch object"));
        }
        commits.push(parse_commit(s, oid, &remaining[..size])?);
        remaining = &remaining[size + 1..];
    }
    if !remaining.is_empty() {
        return Err(data_error("Unexpected trailing batch data"));
    }
    Ok(commits)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
#[serde(tag = "kind", content = "reference", rename_all = "snake_case")]
pub enum HistoryScope {
    LocalBranches,
    Reference(String),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Cursor {
    pub generation: String,
    pub scope: HistoryScope,
    pub offset: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct HistoryPage {
    pub commits: Vec<Commit>,
    pub references: Vec<Reference>,
    pub next: Option<Cursor>,
    pub shallow: bool,
    pub shallow_boundaries: Vec<ObjectId>,
    pub generation: String,
    pub observed_at_ms: u64,
}
pub fn page(
    repo: &Path,
    scope: HistoryScope,
    limit: usize,
    cursor: Option<&Cursor>,
) -> Result<HistoryPage> {
    page_with_options(
        repo,
        scope,
        limit,
        cursor,
        &ReadOptions {
            timeout: Duration::from_secs(30),
            ..ReadOptions::default()
        },
    )
}
pub fn page_with_options(
    repo: &Path,
    scope: HistoryScope,
    limit: usize,
    cursor: Option<&Cursor>,
    options: &ReadOptions,
) -> Result<HistoryPage> {
    if !(1..=500).contains(&limit) {
        return Err(Error::with_kind(
            ErrorKind::InvalidRequest,
            "History page size must be 1..500",
        ));
    }
    inspection::read(repo, options, |s| {
        if let Some(cursor) = cursor {
            if cursor.generation != s.generation
                || cursor.scope != scope
                || cursor.offset > 1_000_000
            {
                return Err(Error::with_kind(
                    ErrorKind::StaleCursor,
                    "History cursor is stale or invalid",
                ));
            }
        }
        let references = s.refs()?;
        let mut tips = match &scope {
            HistoryScope::LocalBranches => references
                .iter()
                .filter(|r| r.name.starts_with("refs/heads/"))
                .map(|r| r.oid.clone())
                .collect::<Vec<_>>(),
            HistoryScope::Reference(name) => {
                let reference = references.iter().find(|r| &r.name == name).ok_or_else(|| {
                    Error::with_kind(ErrorKind::InvalidRequest, "Unknown full reference name")
                })?;
                if reference.object_type == "commit" {
                    vec![reference.oid.clone()]
                } else if let Some(peeled) = &reference.peeled {
                    vec![peeled.clone()]
                } else {
                    return Err(data_error("Reference does not identify a commit"));
                }
            }
        };
        tips.sort_by(|a, b| a.hex.cmp(&b.hex));
        tips.dedup();
        let offset = cursor.map_or(0, |cursor| cursor.offset);
        let mut args = vec![
            "rev-list".to_owned(),
            "--topo-order".into(),
            format!("--max-count={}", limit + 1),
            format!("--skip={offset}"),
        ];
        args.extend(tips.iter().map(|id| id.hex.clone()));
        let ids = if tips.is_empty() {
            Vec::new()
        } else {
            let args: Vec<_> = args.iter().map(String::as_str).collect();
            let bytes = s.git(&args)?;
            bytes
                .split(|b| *b == b'\n')
                .filter(|v| !v.is_empty())
                .map(|v| {
                    ObjectId::parse(
                        std::str::from_utf8(v).map_err(|_| data_error("Invalid history OID"))?,
                        &s.format,
                    )
                })
                .collect::<Result<Vec<_>>>()?
        };
        let next = if ids.len() > limit {
            Some(Cursor {
                generation: s.generation.clone(),
                scope: scope.clone(),
                offset: offset + limit,
            })
        } else {
            None
        };
        let mut commits = Vec::new();
        let mut page_bytes = 0usize;
        for batch in ids[..ids.len().min(limit)].chunks(COMMIT_BATCH_SIZE) {
            for commit in commits_in(s, batch)? {
                page_bytes = page_bytes.saturating_add(
                    commit.message.len()
                        + commit.author.len()
                        + commit.committer.len()
                        + commit.parents.len() * 64
                        + 256,
                );
                if page_bytes > options.max_output {
                    return Err(Error::with_kind(
                        ErrorKind::OutputLimit,
                        "History page exceeds response budget",
                    ));
                }
                commits.push(commit);
            }
        }
        let shallow_path = s.common_dir.join("shallow");
        let shallow_boundaries = if shallow_path.exists() {
            inspection::file_bytes(&shallow_path, 2 * 1024 * 1024)?
                .split(|b| *b == b'\n')
                .filter(|line| !line.is_empty())
                .map(|line| {
                    ObjectId::parse(
                        std::str::from_utf8(line).map_err(|_| data_error("Invalid shallow OID"))?,
                        &s.format,
                    )
                })
                .collect::<Result<Vec<_>>>()?
        } else {
            Vec::new()
        };
        Ok(HistoryPage {
            commits,
            references,
            next,
            shallow: !shallow_boundaries.is_empty(),
            shallow_boundaries,
            generation: s.generation.clone(),
            observed_at_ms: now_ms(),
        })
    })
}
pub fn commit(repo: &Path, oid: &ObjectId) -> Result<Commit> {
    commit_with_options(repo, oid, &ReadOptions::default())
}
pub fn commit_with_options(repo: &Path, oid: &ObjectId, options: &ReadOptions) -> Result<Commit> {
    inspection::read(repo, options, |s| commit_in(s, oid))
}

#[cfg(test)]
#[path = "../tests/support/mod.rs"]
mod test_support;
#[cfg(test)]
mod batch_tests {
    use super::*;
    #[test]
    fn batch_matches_single_commit_and_bounds_missing_objects() {
        let f = test_support::Fixture::new();
        f.commit_file();
        f.git(&[
            "commit",
            "--allow-empty",
            "-qm",
            &"large message".repeat(500),
        ]);
        let oid = inspection::repository_metadata(&f.repo).unwrap();
        let format = oid.object_format;
        let head = ObjectId::parse(
            std::str::from_utf8(&f.git(&["rev-parse", "HEAD"]))
                .unwrap()
                .trim(),
            &format,
        )
        .unwrap();
        let before = test_support::tree_bytes(&f.repo);
        inspection::read(&f.repo, &Default::default(), |s| {
            let single = commit_in(s, &head)?;
            let batch = commits_in(s, &[head.clone(), head.clone()])?;
            for value in batch {
                assert_eq!(serde_json::to_value(value)?, serde_json::to_value(&single)?);
            }
            let missing = ObjectId::parse(&"0".repeat(head.hex.len()), &format)?;
            assert_eq!(
                commits_in(s, &[missing]).unwrap_err().kind,
                ErrorKind::MissingObject
            );
            assert_eq!(
                commits_in(s, &[single.tree]).unwrap_err().kind,
                ErrorKind::MissingObject
            );
            Ok(())
        })
        .unwrap();
        let small = ReadOptions {
            max_output: 1024,
            ..Default::default()
        };
        assert_eq!(
            inspection::read(&f.repo, &small, |s| commits_in(
                s,
                std::slice::from_ref(&head)
            ))
            .unwrap_err()
            .kind,
            ErrorKind::OutputLimit
        );
        assert_eq!(before, test_support::tree_bytes(&f.repo));
    }
}
