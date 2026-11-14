//! Read-only services. Arbitrary Git arguments are private to the implementation.
use crate::domain::{digest, invalid, now_ms, NativePath, ObjectFormat, ObjectId};
pub use crate::process::{Cancellation, ReadOptions};
use crate::{process, Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::Instant;

pub(crate) fn strings(args: &[&str]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}
pub(crate) fn data_error(message: &str) -> Error {
    Error::with_kind(ErrorKind::InvalidData, message)
}
pub(crate) fn output_text(mut bytes: Vec<u8>) -> Result<String> {
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
    }
    String::from_utf8(bytes).map_err(|_| data_error("Expected UTF-8 Git metadata"))
}
pub(crate) fn path_output(mut bytes: Vec<u8>) -> Result<PathBuf> {
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
    }
    NativePath::from_git(&bytes)?.to_path()
}
pub(crate) fn file_bytes(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.is_symlink() {
        return Err(invalid("Expected a regular non-symlink file"));
    }
    if metadata.len() > limit as u64 {
        return Err(Error::with_kind(
            ErrorKind::OutputLimit,
            "File exceeds preview limit",
        ));
    }
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = options.open(path)?;
    if !file.metadata()?.is_file() {
        return Err(invalid("File type changed during inspection"));
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(Error::with_kind(
            ErrorKind::OutputLimit,
            "File exceeds preview limit",
        ));
    }
    Ok(bytes)
}
pub(crate) fn fs_identity(path: &Path) -> Result<String> {
    let meta = fs::metadata(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(format!(
            "{}:{}:{:?}",
            meta.dev(),
            meta.ino(),
            meta.created().ok()
        ))
    }
    #[cfg(not(unix))]
    {
        Ok(format!(
            "{:?}:{:?}",
            fs::canonicalize(path)?,
            meta.created().ok()
        ))
    }
}

pub(crate) struct Session<'a> {
    pub root: PathBuf,
    pub git_dir: PathBuf,
    pub common_dir: PathBuf,
    pub format: ObjectFormat,
    pub options: &'a ReadOptions,
    pub generation: String,
    started: Instant,
}
impl Session<'_> {
    pub(crate) fn check(&self) -> Result<()> {
        self.options.check(self.started)
    }
    pub(crate) fn git(&self, args: &[&str]) -> Result<Vec<u8>> {
        self.git_os(&strings(args))
    }
    pub(crate) fn git_os(&self, args: &[OsString]) -> Result<Vec<u8>> {
        let output = self.output(args)?;
        if !output.status.success() {
            return Err(Error::with_kind(
                ErrorKind::InspectionFailed,
                "Git inspection failed; data may be unavailable or incomplete",
            ));
        }
        Ok(output.stdout)
    }
    pub(crate) fn output(&self, args: &[OsString]) -> Result<std::process::Output> {
        self.options.check(self.started)?;
        let mut remaining = self.options.clone();
        remaining.timeout = self.options.timeout.saturating_sub(self.started.elapsed());
        process::git(&self.root, args, &remaining)
    }
    pub(crate) fn git_input(&self, args: &[&str], input: Vec<u8>) -> Result<Vec<u8>> {
        self.check()?;
        let mut remaining = self.options.clone();
        remaining.timeout = remaining.timeout.saturating_sub(self.started.elapsed());
        let output = process::git_input(&self.root, &strings(args), &remaining, Some(input))?;
        if !output.status.success() {
            return Err(data_error("Git batch inspection failed"));
        }
        Ok(output.stdout)
    }
    pub(crate) fn head(&self) -> Result<Option<ObjectId>> {
        let output = self.output(&strings(&["rev-parse", "--verify", "--quiet", "HEAD"]))?;
        if output.status.code() == Some(1) {
            return Ok(None);
        }
        if !output.status.success() {
            return Err(data_error("Cannot resolve HEAD"));
        }
        ObjectId::parse(&output_text(output.stdout)?, &self.format).map(Some)
    }
    pub(crate) fn refs(&self) -> Result<Vec<Reference>> {
        let bytes = self.git(&[
            "for-each-ref",
            "--format=%(objectname)%00%(objecttype)%00%(refname)%00%(upstream)%00%(*objectname)",
        ])?;
        let mut result = Vec::new();
        for line in bytes.split(|b| *b == b'\n').filter(|line| !line.is_empty()) {
            let parts: Vec<_> = line.split(|b| *b == 0).collect();
            if parts.len() != 5 {
                return Err(data_error("Invalid reference record"));
            }
            let text = |value: &[u8]| {
                String::from_utf8(value.to_vec()).map_err(|_| data_error("Invalid reference text"))
            };
            result.push(Reference {
                name: text(parts[2])?,
                oid: ObjectId::parse(&text(parts[0])?, &self.format)?,
                object_type: text(parts[1])?,
                upstream: if parts[3].is_empty() {
                    None
                } else {
                    Some(text(parts[3])?)
                },
                peeled: if parts[4].is_empty() {
                    None
                } else {
                    Some(ObjectId::parse(&text(parts[4])?, &self.format)?)
                },
            });
        }
        Ok(result)
    }
    pub(crate) fn index_gitlinks(&self) -> Result<Vec<IndexGitlink>> {
        let bytes = self.git(&["ls-files", "--stage", "-z"])?;
        let mut links = Vec::new();
        for record in bytes.split(|b| *b == 0).filter(|r| !r.is_empty()) {
            let tab = record
                .iter()
                .position(|b| *b == b'\t')
                .ok_or_else(|| data_error("Invalid index record"))?;
            let fields = std::str::from_utf8(&record[..tab])
                .map_err(|_| data_error("Invalid index metadata"))?
                .split(' ')
                .collect::<Vec<_>>();
            if fields.len() != 3 {
                return Err(data_error("Invalid index metadata"));
            }
            if fields[0] == "160000" && fields[2] == "0" {
                let path = NativePath::from_git(&record[tab + 1..])?;
                path.relative()?;
                links.push(IndexGitlink {
                    path,
                    oid: ObjectId::parse(fields[1], &self.format)?,
                });
            }
        }
        Ok(links)
    }
    fn fingerprint(&self) -> Result<String> {
        let config = self.git(&["config", "--null", "--list", "--includes"])?;
        validate_config(&config)?;
        let mut bytes = config;
        for directory in [&self.root, &self.git_dir, &self.common_dir] {
            bytes.extend_from_slice(fs_identity(directory)?.as_bytes());
            bytes.push(0);
        }
        for path in [
            self.git_dir.join("HEAD"),
            self.git_dir.join("index"),
            self.git_dir.join("commondir"),
            self.root.join(".git"),
        ] {
            match fs::symlink_metadata(&path) {
                Ok(meta) if meta.is_dir() => {}
                Ok(_) => {
                    bytes.extend(file_bytes(&path, 16 * 1024 * 1024)?);
                    bytes.push(0);
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    bytes.extend_from_slice(b"absent\0")
                }
                Err(error) => return Err(error.into()),
            }
        }
        bytes.extend(self.git(&["for-each-ref", "--format=%(refname)%00%(objectname)"])?);
        Ok(digest(&bytes))
    }
    pub(crate) fn blob(&self, oid: &ObjectId) -> Result<Vec<u8>> {
        oid.validate()?;
        if oid.format != self.format {
            return Err(data_error("Object format mismatch"));
        }
        self.git(&["cat-file", "blob", &oid.hex]).map_err(|error| {
            if error.kind == ErrorKind::InspectionFailed {
                Error::with_kind(
                    ErrorKind::MissingObject,
                    "Required blob is unavailable; fetching is disabled",
                )
            } else {
                error
            }
        })
    }
    pub(crate) fn safe_path(&self, path: &NativePath) -> Result<PathBuf> {
        let relative = path.relative()?;
        let mut current = self.root.clone();
        let components: Vec<_> = relative.components().collect();
        for component in &components[..components.len() - 1] {
            current.push(component.as_os_str());
            let metadata = fs::symlink_metadata(&current)?;
            if metadata.is_symlink() || !metadata.is_dir() {
                return Err(invalid("Symlinked/non-directory ancestor is unsupported"));
            }
        }
        Ok(self.root.join(relative))
    }
}
fn validate_config(bytes: &[u8]) -> Result<()> {
    for record in bytes.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let key = String::from_utf8_lossy(record.split(|b| *b == b'\n').next().unwrap_or_default())
            .to_ascii_lowercase();
        if (key.starts_with("filter.")
            && [".clean", ".smudge", ".process"]
                .iter()
                .any(|suffix| key.ends_with(suffix)))
            || key == "index.sparse"
        {
            return Err(Error::with_kind(ErrorKind::UnsupportedConfiguration, "Content filters and sparse-index configuration are unsupported for untrusted inspection"));
        }
    }
    Ok(())
}
type WorktreeLocks = Mutex<BTreeMap<PathBuf, Weak<Mutex<()>>>>;
pub(crate) fn read<T>(
    repo: &Path,
    options: &ReadOptions,
    action: impl FnOnce(&Session<'_>) -> Result<T>,
) -> Result<T> {
    static LOCKS: OnceLock<WorktreeLocks> = OnceLock::new();
    let root = fs::canonicalize(repo)
        .map_err(|_| Error::with_kind(ErrorKind::Unavailable, "Worktree root is unavailable"))?;
    let lock = {
        let mut locks = LOCKS
            .get_or_init(|| Mutex::new(BTreeMap::new()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        locks.retain(|_, lock| lock.strong_count() != 0);
        if let Some(lock) = locks.get(&root).and_then(Weak::upgrade) {
            lock
        } else {
            let lock = Arc::new(Mutex::new(()));
            locks.insert(root.clone(), Arc::downgrade(&lock));
            lock
        }
    };
    let start = Instant::now();
    let _guard = loop {
        options.check(start)?;
        match lock.try_lock() {
            Ok(guard) => break guard,
            Err(std::sync::TryLockError::Poisoned(error)) => break error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => {
                std::thread::sleep(std::time::Duration::from_millis(5))
            }
        }
    };
    let run = |args: &[&str]| -> Result<Vec<u8>> {
        options.check(start)?;
        let mut remaining = options.clone();
        remaining.timeout = options.timeout.saturating_sub(start.elapsed());
        let output = process::git(&root, &strings(args), &remaining)?;
        if !output.status.success() {
            return Err(Error::with_kind(
                ErrorKind::InspectionFailed,
                "Cannot inspect repository metadata",
            ));
        }
        Ok(output.stdout)
    };
    validate_config(&run(&["config", "--null", "--list", "--includes"])?)?;
    let resolved = fs::canonicalize(path_output(run(&["rev-parse", "--show-toplevel"])?)?)?;
    if root != resolved {
        return Err(invalid("Inspection requires the worktree root"));
    }
    let git_dir = fs::canonicalize(path_output(run(&["rev-parse", "--absolute-git-dir"])?)?)?;
    let common_dir = fs::canonicalize(path_output(run(&[
        "rev-parse",
        "--path-format=absolute",
        "--git-common-dir",
    ])?)?)?;
    let format = match output_text(run(&["rev-parse", "--show-object-format"])?)?.as_str() {
        "sha1" => ObjectFormat::Sha1,
        "sha256" => ObjectFormat::Sha256,
        _ => return Err(data_error("Unsupported object format")),
    };
    let mut session = Session {
        root,
        git_dir,
        common_dir,
        format,
        options,
        generation: String::new(),
        started: start,
    };
    session.generation = session.fingerprint()?;
    let result = action(&session)?;
    if session.fingerprint()? != session.generation {
        return Err(Error::with_kind(
            ErrorKind::ConcurrentChange,
            "Repository changed during inspection",
        ));
    }
    Ok(result)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Metadata {
    pub root: NativePath,
    pub git_dir: NativePath,
    pub common_dir: NativePath,
    pub root_binding: String,
    pub git_binding: String,
    pub common_binding: String,
    pub object_format: ObjectFormat,
}
pub fn repository_metadata(repo: &Path) -> Result<Metadata> {
    repository_metadata_with_options(repo, &ReadOptions::default())
}
pub fn repository_metadata_with_options(repo: &Path, options: &ReadOptions) -> Result<Metadata> {
    read(repo, options, |s| {
        Ok(Metadata {
            root: NativePath::from_path(&s.root),
            git_dir: NativePath::from_path(&s.git_dir),
            common_dir: NativePath::from_path(&s.common_dir),
            root_binding: fs_identity(&s.root)?,
            git_binding: fs_identity(&s.git_dir)?,
            common_binding: fs_identity(&s.common_dir)?,
            object_format: s.format.clone(),
        })
    })
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Reference {
    pub name: String,
    pub oid: ObjectId,
    pub object_type: String,
    pub upstream: Option<String>,
    pub peeled: Option<ObjectId>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Counts {
    pub staged: usize,
    pub unstaged: usize,
    pub untracked: usize,
    pub conflicted: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Change {
    pub path: NativePath,
    pub original_path: Option<NativePath>,
    pub index: char,
    pub worktree: char,
    pub conflict: bool,
    pub submodule: String,
    pub head_mode: String,
    pub index_mode: String,
    pub worktree_mode: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Tracking {
    pub reference: String,
    pub oid: Option<ObjectId>,
    pub ahead: Option<u64>,
    pub behind: Option<u64>,
    pub comparison: String,
    pub fetch_freshness: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct IndexGitlink {
    pub path: NativePath,
    pub oid: ObjectId,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Snapshot {
    pub head: Option<String>,
    pub branch: Option<String>,
    /// Compatibility diagnostic only. Structured consumers use changes/counts.
    pub status: String,
    pub head_oid: Option<ObjectId>,
    pub unborn: bool,
    pub detached: bool,
    pub counts: Counts,
    pub changes: Vec<Change>,
    /// Own-index metadata only; workspace inventory compares independently inspected child HEADs.
    #[serde(default)]
    pub index_gitlinks: Option<Vec<IndexGitlink>>,
    pub tracking: Option<Tracking>,
    /// None in older persisted snapshots; Some(empty) means no configured remotes.
    #[serde(default)]
    pub remotes: Option<Vec<String>>,
    pub operations: Vec<String>,
    pub last_commit: Option<crate::history::Commit>,
    pub generation: String,
    pub observed_at_ms: u64,
}
/// Parse byte-safe porcelain v2 records; ignore unknown optional headers.
pub fn parse_status(bytes: &[u8]) -> Result<(Counts, Vec<Change>)> {
    let mut counts = Counts::default();
    let mut changes = Vec::new();
    let mut records = bytes.split(|b| *b == 0).filter(|record| !record.is_empty());
    while let Some(record) = records.next() {
        if record.starts_with(b"#") {
            continue;
        }
        let tag = record[0];
        if tag == b'?' {
            counts.untracked += 1;
            changes.push(Change {
                path: NativePath::from_git(
                    record
                        .get(2..)
                        .ok_or_else(|| data_error("Invalid untracked record"))?,
                )?,
                original_path: None,
                index: '?',
                worktree: '?',
                conflict: false,
                submodule: "N...".into(),
                head_mode: "000000".into(),
                index_mode: "000000".into(),
                worktree_mode: "000000".into(),
            });
            continue;
        }
        let fields = match tag {
            b'1' => 9,
            b'2' => 10,
            b'u' => 11,
            _ => return Err(data_error("Unknown status record")),
        };
        let parts: Vec<_> = record.splitn(fields, |b| *b == b' ').collect();
        if parts.len() != fields || parts[1].len() != 2 {
            return Err(data_error("Invalid status record"));
        }
        let conflict = tag == b'u';
        let index = parts[1][0] as char;
        let worktree = parts[1][1] as char;
        if conflict {
            counts.conflicted += 1;
        } else {
            counts.staged += usize::from(index != '.');
            counts.unstaged += usize::from(worktree != '.');
        }
        let ascii = |v: &[u8]| {
            String::from_utf8(v.to_vec()).map_err(|_| data_error("Invalid status metadata"))
        };
        changes.push(Change {
            path: NativePath::from_git(parts[fields - 1])?,
            original_path: if tag == b'2' {
                Some(NativePath::from_git(
                    records
                        .next()
                        .ok_or_else(|| data_error("Missing rename source"))?,
                )?)
            } else {
                None
            },
            index,
            worktree,
            conflict,
            submodule: ascii(parts[2])?,
            head_mode: ascii(parts[3])?,
            index_mode: ascii(parts[4])?,
            worktree_mode: ascii(parts[if conflict { 6 } else { 5 }])?,
        });
    }
    Ok((counts, changes))
}
pub fn snapshot(repo: &Path) -> Result<Snapshot> {
    snapshot_with_options(repo, &ReadOptions::default())
}
pub fn snapshot_with_options(repo: &Path, options: &ReadOptions) -> Result<Snapshot> {
    read(repo, options, |s| {
        let head_oid = s.head()?;
        let branch_output = s.output(&strings(&["symbolic-ref", "-q", "--short", "HEAD"]))?;
        let branch = if branch_output.status.code() == Some(1) {
            None
        } else if branch_output.status.success() {
            Some(output_text(branch_output.stdout)?)
        } else {
            return Err(data_error("Cannot inspect symbolic HEAD"));
        };
        let raw = s.git(&[
            "status",
            "--porcelain=v2",
            "-z",
            "--branch",
            "--untracked-files=all",
            "--ignore-submodules=all",
        ])?;
        let (mut counts, mut changes) = parse_status(&raw)?;
        // Parent status must not recurse into child configurations. Recover staged
        // gitlink changes from the index/tree comparison, which reads no child worktree.
        for file in crate::diff::staged_files(s)?
            .into_iter()
            .filter(|file| file.old_mode == "160000" || file.new_mode == "160000")
        {
            if !changes.iter().any(|change| change.path == file.path) {
                counts.staged += 1;
                changes.push(Change {
                    path: file.path,
                    original_path: file.original_path,
                    index: file.change.chars().next().unwrap_or('M'),
                    worktree: '.',
                    conflict: false,
                    submodule: "S...".into(),
                    head_mode: file.old_mode,
                    index_mode: file.new_mode.clone(),
                    worktree_mode: file.new_mode,
                });
            }
        }
        let remotes = output_text(s.git(&["remote"])?)?
            .lines()
            .map(str::to_owned)
            .collect();
        let refs = s.refs()?;
        let tracking = branch
            .as_ref()
            .and_then(|branch| {
                refs.iter()
                    .find(|r| r.name == format!("refs/heads/{branch}"))
            })
            .and_then(|r| r.upstream.as_ref())
            .map(|name| -> Result<Tracking> {
                let oid = refs.iter().find(|r| &r.name == name).map(|r| r.oid.clone());
                let mut ahead = None;
                let mut behind = None;
                if let (Some(head), Some(upstream)) = (&head_oid, &oid) {
                    let text = output_text(s.git(&[
                        "rev-list",
                        "--left-right",
                        "--count",
                        &format!("{}...{}", head.hex, upstream.hex),
                    ])?)?;
                    let values: Vec<_> = text.split_whitespace().collect();
                    if values.len() == 2 {
                        ahead = values[0].parse().ok();
                        behind = values[1].parse().ok();
                    }
                }
                Ok(Tracking {
                    reference: name.clone(),
                    oid,
                    ahead,
                    behind,
                    comparison: "local_tracking_refs_only".into(),
                    fetch_freshness: None,
                })
            })
            .transpose()?;
        let mut operations = Vec::new();
        for (name, marker) in [
            ("merge", "MERGE_HEAD"),
            ("rebase", "rebase-merge"),
            ("rebase", "rebase-apply"),
            ("cherry_pick", "CHERRY_PICK_HEAD"),
            ("revert", "REVERT_HEAD"),
            ("sequencer", "sequencer"),
        ] {
            if s.git_dir.join(marker).exists() && !operations.iter().any(|v| v == name) {
                operations.push(name.to_owned());
            }
        }
        let last_commit = head_oid
            .as_ref()
            .map(|oid| crate::history::commit_in(s, oid))
            .transpose()?;
        Ok(Snapshot {
            head: head_oid.as_ref().map(|id| id.hex.clone()),
            unborn: head_oid.is_none(),
            detached: branch.is_none(),
            branch,
            status: output_text(s.git(&[
                "status",
                "--porcelain=v1",
                "--untracked-files=all",
                "--ignore-submodules=all",
            ])?)?,
            head_oid,
            counts,
            changes,
            index_gitlinks: Some(s.index_gitlinks()?),
            tracking,
            remotes: Some(remotes),
            operations,
            last_commit,
            generation: s.generation.clone(),
            observed_at_ms: now_ms(),
        })
    })
}

#[cfg(test)]
mod consistency_tests {
    use super::*;
    mod support {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/support/mod.rs"));
    }
    #[test]
    fn ref_and_config_mutation_during_read_are_rejected() {
        let fixture = support::Fixture::new();
        fixture.commit_file();
        assert_eq!(
            read(&fixture.repo, &ReadOptions::default(), |_| {
                fixture.git(&["branch", "external"]);
                Ok(())
            })
            .unwrap_err()
            .kind,
            ErrorKind::ConcurrentChange
        );
        assert_eq!(
            read(&fixture.repo, &ReadOptions::default(), |_| {
                fixture.git(&["config", "filter.injected.clean", "echo ran > helper-ran"]);
                Ok(())
            })
            .unwrap_err()
            .kind,
            ErrorKind::UnsupportedConfiguration
        );
        assert!(!fixture.repo.join("helper-ran").exists());
    }
    #[test]
    fn multi_command_reads_for_one_worktree_are_serialized() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let fixture = support::Fixture::new();
        fixture.commit_file();
        let before = support::tree_bytes(&fixture.root);
        let count = AtomicUsize::new(0);
        let maximum = AtomicUsize::new(0);
        std::thread::scope(|scope| {
            for _ in 0..3 {
                let count = &count;
                let maximum = &maximum;
                let repo = &fixture.repo;
                scope.spawn(move || {
                    read(repo, &ReadOptions::default(), |_| {
                        let active = count.fetch_add(1, Ordering::SeqCst) + 1;
                        maximum.fetch_max(active, Ordering::SeqCst);
                        std::thread::sleep(std::time::Duration::from_millis(20));
                        count.fetch_sub(1, Ordering::SeqCst);
                        Ok(())
                    })
                    .unwrap()
                });
            }
        });
        assert_eq!(maximum.load(Ordering::SeqCst), 1);
        assert_eq!(before, support::tree_bytes(&fixture.root));
    }
}
