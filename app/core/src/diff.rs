//! Bounded read-only content comparisons; no external diff/textconv or index writes.
use crate::domain::{digest, now_ms, NativePath, ObjectId};
use crate::inspection::{self, data_error, file_bytes, strings, ReadOptions, Session};
use crate::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DiffMode {
    Staged,
    Unstaged,
    Historical {
        commit: ObjectId,
        parent: Option<ObjectId>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct FileChange {
    pub path: NativePath,
    pub original_path: Option<NativePath>,
    pub change: String,
    pub old_mode: String,
    pub new_mode: String,
    pub old_oid: Option<ObjectId>,
    pub new_oid: Option<ObjectId>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Diff {
    pub mode: DiffMode,
    pub files: Vec<FileChange>,
    pub patch: Vec<u8>,
    pub complete: bool,
    pub generation: String,
    pub observed_at_ms: u64,
}
fn raw(s: &Session<'_>, bytes: &[u8]) -> Result<Vec<FileChange>> {
    let mut parts = bytes.split(|b| *b == 0).filter(|v| !v.is_empty());
    let mut files = Vec::new();
    while let Some(meta) = parts.next() {
        let meta =
            std::str::from_utf8(meta).map_err(|_| data_error("Invalid raw diff metadata"))?;
        let fields: Vec<_> = meta.trim_start_matches(':').split(' ').collect();
        if fields.len() != 5 {
            return Err(data_error("Invalid raw diff fields"));
        }
        let object = |text: &str| {
            if text.bytes().all(|b| b == b'0') {
                Ok(None)
            } else {
                ObjectId::parse(text, &s.format).map(Some)
            }
        };
        let first = NativePath::from_git(
            parts
                .next()
                .ok_or_else(|| data_error("Missing diff path"))?,
        )?;
        let rename = fields[4].starts_with('R') || fields[4].starts_with('C');
        let (path, original_path) = if rename {
            (
                NativePath::from_git(
                    parts
                        .next()
                        .ok_or_else(|| data_error("Missing rename path"))?,
                )?,
                Some(first),
            )
        } else {
            (first, None)
        };
        files.push(FileChange {
            path,
            original_path,
            change: fields[4].into(),
            old_mode: fields[0].into(),
            new_mode: fields[1].into(),
            old_oid: object(fields[2])?,
            new_oid: object(fields[3])?,
        });
    }
    Ok(files)
}
fn numstat_paths(bytes: &[u8]) -> Result<std::collections::BTreeSet<Vec<u8>>> {
    let mut parts = bytes.split(|byte| *byte == 0);
    let mut paths = std::collections::BTreeSet::new();
    while let Some(record) = parts.next() {
        if record.is_empty() {
            continue;
        }
        let mut fields = record.splitn(3, |byte| *byte == b'\t');
        for _ in 0..2 {
            let count = fields
                .next()
                .ok_or_else(|| data_error("Missing diff count"))?;
            if count != b"-" && (count.is_empty() || !count.iter().all(u8::is_ascii_digit)) {
                return Err(data_error("Invalid diff count"));
            }
        }
        let path = fields
            .next()
            .ok_or_else(|| data_error("Missing diff path"))?;
        let path = if path.is_empty() {
            let old = parts
                .next()
                .ok_or_else(|| data_error("Missing rename source"))?;
            let new = parts
                .next()
                .ok_or_else(|| data_error("Missing rename destination"))?;
            if old.is_empty() || new.is_empty() {
                return Err(data_error("Empty rename path"));
            }
            new
        } else {
            path
        };
        paths.insert(path.to_vec());
    }
    Ok(paths)
}
fn args(s: &Session<'_>, mode: &DiffMode, patch: bool) -> Result<Vec<std::ffi::OsString>> {
    let mut args = strings(&[
        "diff",
        "--no-ext-diff",
        "--no-textconv",
        "--no-color",
        "--submodule=short",
        "--no-abbrev",
        "--find-renames=50%",
    ]);
    args.push(
        if matches!(mode, DiffMode::Unstaged) {
            "--ignore-submodules=all"
        } else {
            "--ignore-submodules=none"
        }
        .into(),
    );
    if patch {
        args.extend(strings(&["--patch"]));
    } else {
        args.extend(strings(&["--raw", "-z"]));
    }
    match mode {
        DiffMode::Staged => {
            args.push("--cached".into());
        }
        DiffMode::Unstaged => {}
        DiffMode::Historical { commit, parent } => {
            let detail = crate::history::commit_in(s, commit)?;
            if detail.parents.is_empty() {
                if parent.is_some() {
                    return Err(Error::with_kind(
                        ErrorKind::InvalidRequest,
                        "Root commit has no parent",
                    ));
                }
                args[0] = "diff-tree".into();
                args.extend(strings(&["--root", "--no-commit-id", "-r"]));
                args.push(commit.hex.clone().into());
            } else {
                let parent = parent.as_ref().ok_or_else(|| {
                    Error::with_kind(
                        ErrorKind::InvalidRequest,
                        "Historical diff requires an explicit parent",
                    )
                })?;
                if !detail.parents.contains(parent) {
                    return Err(Error::with_kind(
                        ErrorKind::InvalidRequest,
                        "Selected parent is not a commit parent",
                    ));
                }
                args.push(parent.hex.clone().into());
                args.push(commit.hex.clone().into());
            }
        }
    }
    Ok(args)
}
pub(crate) fn staged_files(s: &Session<'_>) -> Result<Vec<FileChange>> {
    let mut command = args(s, &DiffMode::Staged, false)?;
    command.push("--".into());
    raw(s, &s.git_os(&command)?)
}
pub fn inspect(repo: &Path, mode: DiffMode, path: Option<&NativePath>) -> Result<Diff> {
    inspect_with_options(
        repo,
        mode,
        path,
        &ReadOptions {
            timeout: Duration::from_secs(30),
            ..ReadOptions::default()
        },
    )
}
pub fn inspect_with_options(
    repo: &Path,
    mode: DiffMode,
    path: Option<&NativePath>,
    options: &ReadOptions,
) -> Result<Diff> {
    inspection::read(repo, options, |s| {
        let relative = path.map(NativePath::relative).transpose()?;
        let mut metadata_args = args(s, &mode, false)?;
        let mut patch_args = args(s, &mode, true)?;
        for args in [&mut metadata_args, &mut patch_args] {
            args.push("--".into());
            if let Some(relative) = &relative {
                args.push(relative.as_os_str().to_owned());
            }
        }
        let raw_files = raw(s, &s.git_os(&metadata_args)?)?;
        let files = if matches!(mode, DiffMode::Unstaged) && !raw_files.is_empty() {
            let mut counts_args = metadata_args.clone();
            let raw_flag = counts_args
                .iter_mut()
                .find(|arg| *arg == "--raw")
                .ok_or_else(|| data_error("Missing raw diff argument"))?;
            *raw_flag = "--numstat".into();
            let changed = numstat_paths(&s.git_os(&counts_args)?)?;
            let mut files = Vec::new();
            for file in &raw_files {
                // Raw diff can report stat-cache misses even when blob content is equal.
                let path = file.path.to_path()?;
                if file.change != "M"
                    || file.old_mode != file.new_mode
                    || changed.contains(path.as_os_str().as_encoded_bytes())
                {
                    files.push(file.clone());
                }
            }
            files
        } else {
            raw_files.clone()
        };
        let current = if matches!(mode, DiffMode::Unstaged) {
            raw_files
                .iter()
                .filter(|file| file.old_mode != "160000" && file.new_mode != "160000")
                .map(|file| Ok((&file.path, fingerprint_in(s, &file.path)?)))
                .collect::<Result<Vec<_>>>()?
        } else {
            Vec::new()
        };
        let patch = s.git_os(&patch_args)?;
        for (path, before) in current {
            if before != fingerprint_in(s, path)? {
                return Err(Error::with_kind(
                    ErrorKind::ConcurrentChange,
                    "Working file changed during diff",
                ));
            }
        }
        if raw(s, &s.git_os(&metadata_args)?)? != raw_files {
            return Err(Error::with_kind(
                ErrorKind::ConcurrentChange,
                "Diff metadata changed during inspection",
            ));
        }
        Ok(Diff {
            mode,
            files,
            patch,
            complete: true,
            generation: s.generation.clone(),
            observed_at_ms: now_ms(),
        })
    })
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
pub struct Preview {
    pub path: NativePath,
    pub kind: String,
    pub bytes: Vec<u8>,
    pub binary: bool,
    pub fingerprint: String,
    pub complete: bool,
}
fn fingerprint_in(s: &Session<'_>, path: &NativePath) -> Result<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    s.check()?;
    let target = s.safe_path(path)?;
    let metadata = match std::fs::symlink_metadata(&target) {
        Ok(metadata) => metadata,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(digest(b"missing")),
        Err(e) => return Err(e.into()),
    };
    if metadata.is_symlink() {
        let value = std::fs::read_link(&target)?;
        let mut hash = Sha256::new();
        hash.update(b"symlink");
        hash.update(value.as_os_str().as_encoded_bytes());
        return Ok(format!("{:x}", hash.finalize()));
    }
    if !metadata.is_file() {
        return Err(Error::with_kind(
            ErrorKind::InvalidPath,
            "Unsupported preview file type",
        ));
    }
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = options.open(&target)?;
    if !file.metadata()?.is_file() {
        return Err(Error::with_kind(
            ErrorKind::InvalidPath,
            "File type changed during inspection",
        ));
    }
    if file.metadata()?.len() > 2 * 1024 * 1024 {
        return Err(Error::with_kind(
            ErrorKind::OutputLimit,
            "File exceeds preview limit",
        ));
    }
    let mut hash = Sha256::new();
    hash.update(b"regular");
    let mut size = 0;
    let mut buffer = [0; 65536];
    loop {
        s.check()?;
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        size += n;
        if size > 2 * 1024 * 1024 {
            return Err(Error::with_kind(
                ErrorKind::OutputLimit,
                "File exceeds preview limit",
            ));
        }
        hash.update(&buffer[..n]);
        #[cfg(test)]
        if fingerprint_tests::CANCEL_ON_CHUNK.replace(false) {
            s.options.cancellation.cancel();
        }
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn preview_in(s: &Session<'_>, path: &NativePath) -> Result<Preview> {
    let target = s.safe_path(path)?;
    let metadata = std::fs::symlink_metadata(&target);
    let (kind, bytes) = match metadata {
        Ok(metadata) if metadata.is_symlink() => (
            "symlink",
            NativePath::from_path(&std::fs::read_link(target)?)
                .to_path()?
                .as_os_str()
                .as_encoded_bytes()
                .to_vec(),
        ),
        Ok(metadata) if metadata.is_file() => ("regular", file_bytes(&target, 2 * 1024 * 1024)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ("missing", Vec::new()),
        Err(error) => return Err(error.into()),
        _ => {
            return Err(Error::with_kind(
                ErrorKind::InvalidPath,
                "Unsupported preview file type",
            ))
        }
    };
    let fingerprint = digest(&[kind.as_bytes(), &bytes].concat());
    Ok(Preview {
        path: path.clone(),
        kind: kind.into(),
        binary: bytes.contains(&0) || std::str::from_utf8(&bytes).is_err(),
        bytes,
        fingerprint,
        complete: true,
    })
}
pub fn preview(repo: &Path, path: &NativePath) -> Result<Preview> {
    preview_with_options(repo, path, &ReadOptions::default())
}
pub fn preview_with_options(
    repo: &Path,
    path: &NativePath,
    options: &ReadOptions,
) -> Result<Preview> {
    inspection::read(repo, options, |s| preview_in(s, path))
}
/// Read an immutable blob without invoking content conversion helpers.
pub fn blob(repo: &Path, oid: &ObjectId) -> Result<Vec<u8>> {
    inspection::read(
        repo,
        &ReadOptions {
            max_output: 2 * 1024 * 1024,
            ..ReadOptions::default()
        },
        |s| s.blob(oid),
    )
}

#[cfg(test)]
mod fingerprint_tests {
    use super::*;
    use std::fs;
    thread_local! { pub(super) static CANCEL_ON_CHUNK: std::cell::Cell<bool> = const {std::cell::Cell::new(false)}; }
    #[test]
    fn streaming_fingerprints_match_preview_contract_and_limits() {
        let root = std::env::temp_dir().join(format!("diff-fingerprint-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        assert!(std::process::Command::new("git")
            .args(["init", "-q"])
            .arg(&root)
            .status()
            .unwrap()
            .success());
        for bytes in [vec![], b"text\n".to_vec(), vec![0; 2 * 1024 * 1024]] {
            fs::write(root.join("file"), bytes).unwrap();
            inspection::read(&root, &Default::default(), |s| {
                let path = NativePath::Utf8("file".into());
                assert_eq!(fingerprint_in(s, &path)?, preview_in(s, &path)?.fingerprint);
                Ok(())
            })
            .unwrap();
        }
        fs::remove_file(root.join("file")).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("target", root.join("link")).unwrap();
        inspection::read(&root, &Default::default(), |s| {
            for name in ["file", "link"] {
                let path = NativePath::Utf8(name.into());
                assert_eq!(fingerprint_in(s, &path)?, preview_in(s, &path)?.fingerprint);
            }
            fs::write(root.join("file"), vec![0; 2 * 1024 * 1024 + 1])?;
            assert_eq!(
                fingerprint_in(s, &NativePath::Utf8("file".into()))
                    .unwrap_err()
                    .kind,
                ErrorKind::OutputLimit
            );
            Ok(())
        })
        .unwrap();
        fs::write(root.join("file"), vec![0; 131072]).unwrap();
        let options = ReadOptions::default();
        let error = inspection::read(&root, &options, |s| {
            CANCEL_ON_CHUNK.set(true);
            fingerprint_in(s, &NativePath::Utf8("file".into()))
        })
        .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Cancelled);
        assert!(options.cancellation.is_cancelled());
        let options = ReadOptions {
            timeout: Duration::from_millis(500),
            ..Default::default()
        };
        let error = inspection::read(&root, &options, |s| {
            std::thread::sleep(Duration::from_millis(500));
            fingerprint_in(s, &NativePath::Utf8("file".into()))
        })
        .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Timeout);
        fs::remove_dir_all(root).unwrap();
    }
}
