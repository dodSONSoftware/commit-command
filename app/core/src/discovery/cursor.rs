//! In-memory continuation only: persisted inventories never restore execution/cursor authority.
use super::*;
use crate::domain::digest;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct ScanProgress {
    pub inspected_repositories: usize,
    pub pending_directories: usize,
    pub classified_repositories: usize,
    pub phase: String,
}

#[derive(Clone)]
pub(crate) struct Cursor {
    root: PathBuf,
    binding: Option<String>,
    options_digest: String,
    pending: Vec<PathBuf>,
    timed_out: Option<PathBuf>,
    visited: usize,
    classified: usize,
    nested: usize,
    report: Inventory,
}
impl Cursor {
    pub fn new(root: &Path, options: &DiscoveryOptions) -> Result<Self> {
        options.validate()?;
        let mut report = Inventory {
            root: NativePath::from_path(root),
            complete: false,
            repositories: Vec::new(),
            relationships: Vec::new(),
            issues: Vec::new(),
            skipped_caches: Vec::new(),
            progress: None,
        };
        let (root, binding, pending) = match fs::canonicalize(root) {
            Ok(path) if path.is_dir() => {
                report.root = NativePath::from_path(&path);
                let binding = inspection::fs_identity(&path)?;
                (path.clone(), Some(binding), vec![path])
            }
            _ => {
                report.issues.push(ScanIssue {
                    path: report.root.clone(),
                    error: Error::with_kind(
                        ErrorKind::Unavailable,
                        "Workspace root is unavailable",
                    )
                    .into(),
                });
                (root.to_path_buf(), None, Vec::new())
            }
        };
        Ok(Self {
            root,
            binding,
            pending,
            timed_out: None,
            options_digest: digest(&serde_json::to_vec(options)?),
            visited: 0,
            classified: 0,
            nested: 0,
            report,
        })
    }
    pub fn matches(&self, root: &Path, options: &DiscoveryOptions) -> bool {
        fs::canonicalize(root).ok().as_ref() == Some(&self.root)
            && inspection::fs_identity(&self.root).ok() == self.binding
            && serde_json::to_vec(options).is_ok_and(|v| digest(&v) == self.options_digest)
    }
    fn remaining(options: &ReadOptions, start: Instant) -> Result<ReadOptions> {
        options.check(start)?;
        let mut remaining = options.clone();
        remaining.timeout = options.timeout.saturating_sub(start.elapsed());
        Ok(remaining)
    }
    fn directory(
        &mut self,
        path: &Path,
        options: &DiscoveryOptions,
        read: &ReadOptions,
        start: Instant,
    ) -> Result<()> {
        if self.visited >= 100_000 {
            return Err(Error::with_kind(
                ErrorKind::OutputLimit,
                "Discovery directory limit exceeded",
            ));
        }
        // A queued directory may have disappeared or become a symlink between slices.
        // Never follow the replacement into another tree.
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_dir() && !metadata.is_symlink() => {}
            result => {
                let error = match result {
                    Err(error) => Error::from(error),
                    Ok(_) => Error::with_kind(
                        ErrorKind::InvalidPath,
                        "Queued discovery directory was replaced",
                    ),
                };
                self.report.issues.push(ScanIssue {
                    path: NativePath::from_path(path),
                    error: error.into(),
                });
                self.visited += 1;
                return Ok(());
            }
        }
        if path != self.root && tagged_cache(path) {
            self.report.skipped_caches.push(NativePath::from_path(path));
            self.visited += 1;
            return Ok(());
        }
        // Stage this unit before publishing it so an exhausted read can be retried once.
        let mut entry = None;
        let mut issues = Vec::new();
        if fs::symlink_metadata(path.join(".git")).is_ok() {
            let remaining = Self::remaining(read, start)?;
            match inspection::repository_metadata_with_options(path, &remaining) {
                Ok(metadata) => {
                    let remaining = Self::remaining(read, start)?;
                    let (status, issue) = match inspection::snapshot_with_options(path, &remaining)
                    {
                        Ok(status) => (Some(status), None),
                        Err(error)
                            if matches!(error.kind, ErrorKind::Timeout | ErrorKind::Cancelled) =>
                        {
                            return Err(error)
                        }
                        Err(error) => (None, Some(error.into())),
                    };
                    entry = Some(Discovered {
                        metadata,
                        status,
                        issue,
                    });
                }
                Err(error) if matches!(error.kind, ErrorKind::Timeout | ErrorKind::Cancelled) => {
                    return Err(error)
                }
                Err(error) => issues.push(ScanIssue {
                    path: NativePath::from_path(path),
                    error: error.into(),
                }),
            }
        }
        let mut children = Vec::new();
        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries {
                    Self::remaining(read, start)?;
                    match entry.and_then(|entry| Ok((entry.path(), entry.file_type()?))) {
                        Ok((child, kind))
                            if kind.is_dir()
                                && !kind.is_symlink()
                                && !options.excluded(&self.root, &child) =>
                        {
                            children.push(child)
                        }
                        Ok(_) => {}
                        Err(error) => issues.push(ScanIssue {
                            path: NativePath::from_path(path),
                            error: Error::from(error).into(),
                        }),
                    }
                    if children.len() + self.pending.len() > 100_000 {
                        return Err(Error::with_kind(
                            ErrorKind::OutputLimit,
                            "Discovery directory limit exceeded",
                        ));
                    }
                }
            }
            Err(error) => issues.push(ScanIssue {
                path: NativePath::from_path(path),
                error: Error::from(error).into(),
            }),
        }
        children.sort();
        self.pending.extend(children.into_iter().rev());
        self.report.issues.extend(issues);
        if let Some(entry) = entry {
            self.report.repositories.push(entry);
        }
        self.visited += 1;
        Ok(())
    }
    pub fn advance(&mut self, options: &DiscoveryOptions, read: &ReadOptions) -> Result<Inventory> {
        let start = Instant::now();
        self.timed_out = None;
        let result = self.advance_units(options, read, start);
        if let Err(error) = &result {
            if error.kind != ErrorKind::Timeout {
                return result.map(|_| self.report.clone());
            }
        }
        let mut report = self.report.clone();
        report
            .repositories
            .sort_by_key(|entry| entry.metadata.root.to_path().ok());
        report
            .skipped_caches
            .sort_by_key(|path| path.to_path().ok());
        if let Err(error) = result {
            report.issues.push(ScanIssue {
                path: NativePath::from_path(
                    self.timed_out
                        .as_ref()
                        .or(self.pending.last())
                        .unwrap_or(&self.root),
                ),
                error: error.into(),
            });
            report.progress = Some(ScanProgress {
                inspected_repositories: report.repositories.len(),
                pending_directories: self.pending.len(),
                classified_repositories: self.classified.min(self.nested),
                phase: if !self.pending.is_empty() {
                    "traversal"
                } else if self.classified < report.repositories.len() {
                    "git_relationships"
                } else {
                    "nested_relationships"
                }
                .into(),
            });
        }
        report.complete = report.progress.is_none()
            && report.issues.is_empty()
            && report
                .repositories
                .iter()
                .all(|entry| entry.issue.is_none());
        Ok(report)
    }
    fn advance_units(
        &mut self,
        options: &DiscoveryOptions,
        read: &ReadOptions,
        start: Instant,
    ) -> Result<()> {
        while let Some(path) = self.pending.last().cloned() {
            Self::remaining(read, start)?;
            // Remove the current unit before adding its children; restore on failure.
            self.pending.pop();
            if let Err(error) = self.directory(&path, options, read, start) {
                if error.kind == ErrorKind::Timeout {
                    self.timed_out = Some(path.clone());
                    // Retry this unit after queued siblings have had their next slice.
                    self.pending.insert(0, path);
                } else {
                    self.pending.push(path);
                }
                return Err(error);
            }
        }
        self.report
            .repositories
            .sort_by_key(|entry| entry.metadata.root.to_path().ok());
        while self.classified < self.report.repositories.len() {
            let remaining = Self::remaining(read, start)?;
            classify_links(
                &mut self.report,
                &remaining,
                self.classified,
                self.classified + 1,
            )?;
            self.classified += 1;
        }
        while self.nested < self.report.repositories.len() {
            Self::remaining(read, start)?;
            classify_nesting(&mut self.report, self.nested, self.nested + 1)?;
            self.nested += 1;
        }
        Ok(())
    }
}
