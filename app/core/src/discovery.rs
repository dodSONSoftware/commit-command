//! Explicit rescans, partial inventories, and metadata-backed relationships.
use crate::domain::{Issue, NativePath};
use crate::inspection::{self, Metadata, ReadOptions};
use crate::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
mod cursor;
pub(crate) use cursor::Cursor;
pub use cursor::ScanProgress;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct DiscoveryOptions {
    pub excluded_names: Vec<String>,
    pub excluded_patterns: Vec<String>,
}
impl Default for DiscoveryOptions {
    fn default() -> Self {
        Self {
            excluded_names: [
                "node_modules",
                ".venv",
                "target",
                "dist",
                "build",
                ".gitfleet-rust-lab",
            ]
            .map(str::to_owned)
            .to_vec(),
            excluded_patterns: Vec::new(),
        }
    }
}
impl DiscoveryOptions {
    pub fn validate(&self) -> Result<()> {
        for pattern in &self.excluded_patterns {
            glob::Pattern::new(pattern).map_err(|_| {
                Error::with_kind(ErrorKind::InvalidRequest, "Invalid exclusion glob")
            })?;
        }
        if self
            .excluded_names
            .iter()
            .any(|name| name.is_empty() || name.contains('/') || name.contains('\\'))
        {
            return Err(Error::with_kind(
                ErrorKind::InvalidRequest,
                "Excluded names must be directory basenames",
            ));
        }
        Ok(())
    }
    fn excluded(&self, root: &Path, path: &Path) -> bool {
        if path.file_name().is_some_and(|name| name == ".git") {
            return true;
        }
        if self
            .excluded_names
            .iter()
            .any(|name| path.file_name().is_some_and(|base| base == name.as_str()))
        {
            return true;
        }
        self.excluded_patterns.iter().any(|pattern| {
            glob::Pattern::new(pattern).is_ok_and(|pattern| {
                pattern.matches_path_with(
                    path.strip_prefix(root).unwrap_or(path),
                    glob::MatchOptions {
                        case_sensitive: true,
                        require_literal_separator: true,
                        require_literal_leading_dot: false,
                    },
                )
            })
        })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct ScanIssue {
    pub path: NativePath,
    pub error: Issue,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Discovered {
    pub metadata: Metadata,
    pub status: Option<inspection::Snapshot>,
    pub issue: Option<Issue>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum RelationshipKind {
    NestedIndependent,
    Submodule,
    LinkedWorktree,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Relationship {
    pub parent: NativePath,
    pub child: NativePath,
    pub kind: RelationshipKind,
    pub state: String,
    pub gitlink_oid: Option<String>,
    pub child_head: Option<crate::domain::ObjectId>,
    pub child_dirty: Option<bool>,
    pub gitlink_matches_head: Option<bool>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Inventory {
    pub root: NativePath,
    pub complete: bool,
    pub repositories: Vec<Discovered>,
    pub relationships: Vec<Relationship>,
    pub issues: Vec<ScanIssue>,
    #[serde(default)]
    pub skipped_caches: Vec<NativePath>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<ScanProgress>,
}

// Match only the standard signature, without following links or reading an
// unbounded marker. Invalid/unreadable tags do not suppress normal discovery.
fn tagged_cache(directory: &Path) -> bool {
    let path = directory.join("CACHEDIR.TAG");
    if !fs::symlink_metadata(&path).is_ok_and(|m| m.is_file() && !m.is_symlink()) {
        return false;
    }
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let Ok(mut file) = options.open(path) else {
        return false;
    };
    if !file.metadata().is_ok_and(|m| m.is_file()) {
        return false;
    }
    let mut signature = [0; 43];
    file.read_exact(&mut signature).is_ok()
        && signature == *b"Signature: 8a477f597d28d172789f06886806bc55"
}

pub fn scan(root: &Path, options: &DiscoveryOptions) -> Result<Inventory> {
    scan_with_options(root, options, &ReadOptions::default())
}
pub fn scan_with_options(
    root: &Path,
    options: &DiscoveryOptions,
    read_options: &ReadOptions,
) -> Result<Inventory> {
    // One bounded traversal implementation serves both adapters. A one-shot
    // caller receives a partial report on timeout, without resumable authority.
    let mut cursor = Cursor::new(root, options)?;
    let mut report = cursor.advance(options, read_options)?;
    report.progress = None;
    Ok(report)
}

fn classify_links(
    report: &mut Inventory,
    options: &ReadOptions,
    start: usize,
    end: usize,
) -> Result<()> {
    for entry in report.repositories.iter().take(end).skip(start) {
        let parent = entry.metadata.root.to_path()?;
        let links = inspection::read(&parent, options, |s| {
            let mut declared = Vec::new();
            let modules = s.root.join(".gitmodules");
            if modules.exists() {
                inspection::file_bytes(&modules, 2 * 1024 * 1024)?;
                // --list succeeds even with no submodule entries; never process includes.
                let bytes = s.git(&[
                    "config",
                    "--null",
                    "--no-includes",
                    "--file",
                    ".gitmodules",
                    "--list",
                ])?;
                for record in bytes.split(|b| *b == 0).filter(|r| !r.is_empty()) {
                    if let Some(separator) = record.iter().position(|b| *b == b'\n') {
                        let key = &record[..separator];
                        if key.starts_with(b"submodule.") && key.ends_with(b".path") {
                            let path = NativePath::from_git(&record[separator + 1..])?;
                            path.relative()?;
                            declared.push(path);
                        }
                    }
                }
            }
            let gitlinks = match entry
                .status
                .as_ref()
                .filter(|status| status.generation == s.generation)
                .and_then(|status| status.index_gitlinks.as_ref())
            {
                Some(links) => links.clone(),
                None => s.index_gitlinks()?,
            }
            .into_iter()
            .map(|link| (link.path, link.oid.hex))
            .collect::<Vec<_>>();
            let worktrees = s.git(&["worktree", "list", "--porcelain", "-z"])?;
            let external = worktrees
                .split(|b| *b == 0)
                .filter_map(|record| record.strip_prefix(b"worktree "))
                .map(NativePath::from_git)
                .collect::<Result<Vec<_>>>()?;
            Ok((declared, gitlinks, external))
        });
        let (declared, links, worktrees) = match links {
            Ok(links) => links,
            Err(error) => {
                if matches!(error.kind, ErrorKind::Timeout | ErrorKind::Cancelled) {
                    return Err(error);
                }
                report.issues.push(ScanIssue {
                    path: NativePath::from_path(&parent),
                    error: error.into(),
                });
                continue;
            }
        };
        for child in worktrees {
            let child_path = child.to_path()?;
            if child_path != parent && !child_path.starts_with(report.root.to_path()?) {
                report.relationships.push(Relationship {
                    parent: entry.metadata.root.clone(),
                    child,
                    kind: RelationshipKind::LinkedWorktree,
                    state: "outside_workspace".into(),
                    gitlink_oid: None,
                    child_head: None,
                    child_dirty: None,
                    gitlink_matches_head: None,
                });
            }
        }
        for path in declared.iter().chain(links.iter().map(|(path, _)| path)) {
            let child = parent.join(path.relative()?);
            if report.relationships.iter().any(|r| {
                r.parent == entry.metadata.root
                    && r.child.to_path().ok().as_ref() == Some(&child)
                    && matches!(r.kind, RelationshipKind::Submodule)
            }) {
                continue;
            }
            let oid = links
                .iter()
                .find(|(p, _)| p == path)
                .map(|(_, oid)| oid.clone());
            let initialized = report
                .repositories
                .iter()
                .any(|r| r.metadata.root.to_path().ok().as_ref() == Some(&child));
            let state = if oid.is_none() {
                "declared_without_gitlink"
            } else if !declared.contains(path) {
                "gitlink_without_declaration"
            } else if initialized {
                "initialized"
            } else if child.exists() {
                "uninitialized_or_excluded"
            } else {
                "missing"
            };
            report.relationships.push(Relationship {
                parent: entry.metadata.root.clone(),
                child: NativePath::from_path(&child),
                kind: RelationshipKind::Submodule,
                state: state.into(),
                child_head: report
                    .repositories
                    .iter()
                    .find(|r| r.metadata.root.to_path().ok().as_ref() == Some(&child))
                    .and_then(|r| r.status.as_ref())
                    .and_then(|status| status.head_oid.clone()),
                child_dirty: report
                    .repositories
                    .iter()
                    .find(|r| r.metadata.root.to_path().ok().as_ref() == Some(&child))
                    .and_then(|r| r.status.as_ref())
                    .map(|status| {
                        status.counts.staged
                            + status.counts.unstaged
                            + status.counts.untracked
                            + status.counts.conflicted
                            > 0
                    }),
                gitlink_matches_head: oid.as_ref().and_then(|oid| {
                    report
                        .repositories
                        .iter()
                        .find(|r| r.metadata.root.to_path().ok().as_ref() == Some(&child))
                        .and_then(|r| r.status.as_ref())
                        .and_then(|status| status.head.as_ref())
                        .map(|head| head == oid)
                }),
                gitlink_oid: oid,
            });
        }
    }
    Ok(())
}
fn classify_nesting(report: &mut Inventory, start: usize, end: usize) -> Result<()> {
    for (i, entry) in report.repositories.iter().enumerate().take(end).skip(start) {
        let child = entry.metadata.root.to_path()?;
        for (j, other) in report.repositories.iter().enumerate().take(i) {
            let _ = j;
            if entry.metadata.common_binding == other.metadata.common_binding {
                report.relationships.push(Relationship {
                    parent: other.metadata.root.clone(),
                    child: entry.metadata.root.clone(),
                    kind: RelationshipKind::LinkedWorktree,
                    state: "shared_common_directory".into(),
                    gitlink_oid: None,
                    child_head: None,
                    child_dirty: None,
                    gitlink_matches_head: None,
                });
            }
        }
        let nearest = report
            .repositories
            .iter()
            .filter_map(|other| {
                let root = other.metadata.root.to_path().ok()?;
                if root != child && child.starts_with(&root) {
                    Some((root.components().count(), other))
                } else {
                    None
                }
            })
            .max_by_key(|(depth, _)| *depth);
        if let Some((_, parent)) = nearest {
            if parent.metadata.common_binding != entry.metadata.common_binding
                && !report.relationships.iter().any(|r| {
                    r.parent == parent.metadata.root
                        && r.child == entry.metadata.root
                        && matches!(r.kind, RelationshipKind::Submodule)
                })
            {
                report.relationships.push(Relationship {
                    parent: parent.metadata.root.clone(),
                    child: entry.metadata.root.clone(),
                    kind: RelationshipKind::NestedIndependent,
                    state: "independent".into(),
                    gitlink_oid: None,
                    child_head: None,
                    child_dirty: None,
                    gitlink_matches_head: None,
                });
            }
        }
    }
    Ok(())
}
/// Compatibility wrapper: old callers still receive an error for incomplete scans.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Repository {
    pub path: PathBuf,
    pub git_dir: String,
    pub branch: Option<String>,
    pub status: String,
}
pub fn discover(root: &Path) -> Result<Vec<Repository>> {
    let report = scan(root, &DiscoveryOptions::default())?;
    if !report.complete {
        return Err(Error::with_kind(
            ErrorKind::InspectionFailed,
            "Inventory is incomplete; use scan for per-path errors",
        ));
    }
    report
        .repositories
        .into_iter()
        .map(|entry| {
            let status = entry
                .status
                .ok_or_else(|| inspection::data_error("Missing status"))?;
            Ok(Repository {
                path: entry.metadata.root.to_path()?,
                git_dir: entry
                    .metadata
                    .git_dir
                    .to_path()?
                    .to_string_lossy()
                    .into_owned(),
                branch: status.branch,
                status: status.status,
            })
        })
        .collect()
}
