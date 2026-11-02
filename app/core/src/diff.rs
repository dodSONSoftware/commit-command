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
        let files = raw(s, &s.git_os(&metadata_args)?)?;
        let current = if matches!(mode, DiffMode::Unstaged) {
            files
                .iter()
                .filter(|file| file.old_mode != "160000" && file.new_mode != "160000")
                .map(|file| preview_in(s, &file.path))
                .collect::<Result<Vec<_>>>()?
        } else {
            Vec::new()
        };
        let patch = s.git_os(&patch_args)?;
        for before in current {
            if before.fingerprint != preview_in(s, &before.path)?.fingerprint {
                return Err(Error::with_kind(
                    ErrorKind::ConcurrentChange,
                    "Working file changed during diff",
                ));
            }
        }
        if raw(s, &s.git_os(&metadata_args)?)? != files {
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
