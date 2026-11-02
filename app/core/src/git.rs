//! Experimental lab Git adapter. Not the untrusted inspection service.
use crate::{Error, Result};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub mode: String,
    pub oid: String,
}
pub type Entries = BTreeMap<String, Entry>;

pub fn clean_command(program: &OsStr) -> Command {
    let mut cmd = Command::new(program);
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(key);
        }
    }
    cmd.env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_NO_REPLACE_OBJECTS", "1");
    cmd
}

pub fn command(
    repo: &Path,
    args: &[&str],
    index: Option<&Path>,
    input: Option<&[u8]>,
) -> Result<Output> {
    let mut cmd = clean_command(OsStr::new("git"));
    cmd.current_dir(repo)
        .args(["-c", "maintenance.auto=false", "-c", "gc.auto=0"])
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(i) = index {
        cmd.env("GIT_INDEX_FILE", i);
    }
    if input.is_some() {
        cmd.stdin(Stdio::piped());
    }
    let mut child = cmd.spawn()?;
    if let Some(data) = input {
        child
            .stdin
            .take()
            .ok_or_else(|| Error::new("missing stdin"))?
            .write_all(data)?;
    }
    child.wait_with_output().map_err(Error::from)
}

pub fn run(repo: &Path, args: &[&str], index: Option<&Path>) -> Result<String> {
    run_input(repo, args, index, None)
}
pub fn run_input(
    repo: &Path,
    args: &[&str],
    index: Option<&Path>,
    input: Option<&[u8]>,
) -> Result<String> {
    let out = command(repo, args, index, input)?;
    if !out.status.success() {
        return Err(Error::new(format!(
            "git {:?} exited {:?}: {}",
            args,
            out.status.code(),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .trim_end_matches(['\r', '\n'])
        .to_owned())
}

pub fn index_path(repo: &Path) -> Result<PathBuf> {
    let path = PathBuf::from(run(repo, &["rev-parse", "--git-path", "index"], None)?);
    Ok(if path.is_absolute() {
        path
    } else {
        repo.join(path)
    })
}

pub fn current_ref(repo: &Path) -> Result<String> {
    run(repo, &["symbolic-ref", "-q", "HEAD"], None)
        .map_err(|_| Error::new("Detached HEAD is unsupported by the POC"))
}

pub fn entries(repo: &Path, index: Option<&Path>) -> Result<Entries> {
    let out = command(repo, &["ls-files", "--stage", "-z"], index, None)?;
    if !out.status.success() {
        return Err(Error::new(format!(
            "ls-files failed: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    let mut result = BTreeMap::new();
    for record in out.stdout.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let tab = record
            .iter()
            .position(|b| *b == b'\t')
            .ok_or_else(|| Error::new("Invalid index record"))?;
        let meta = std::str::from_utf8(&record[..tab])
            .map_err(|_| Error::new("Non-ASCII index metadata"))?;
        let path = std::str::from_utf8(&record[tab + 1..])
            .map_err(|_| Error::new("Non-UTF8 paths unsupported by POC"))?;
        let mut parts = meta.split(' ');
        let mode = parts.next().ok_or_else(|| Error::new("missing mode"))?;
        let oid = parts.next().ok_or_else(|| Error::new("missing oid"))?;
        let stage = parts.next().ok_or_else(|| Error::new("missing stage"))?;
        if stage != "0" {
            return Err(Error::new("Unmerged index entries unsupported"));
        }
        result.insert(
            path.to_owned(),
            Entry {
                mode: mode.to_owned(),
                oid: oid.to_owned(),
            },
        );
    }
    Ok(result)
}

pub fn validate_path(repo: &Path, path: &str) -> Result<()> {
    if path.is_empty() || path.contains('\0') || path.contains('\\') {
        return Err(Error::new("Invalid repository-relative path"));
    }
    let parsed = Path::new(path);
    if parsed
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
        || path
            .split('/')
            .any(|x| x == ".git" || x == ".." || x == "." || x.is_empty())
    {
        return Err(Error::new(format!(
            "Invalid repository-relative path: {path}"
        )));
    }
    // Avoid symlinked ancestor directories escaping the repository. The leaf may itself be a symlink.
    let mut dir = repo.to_path_buf();
    let segments: Vec<_> = parsed.components().collect();
    for component in segments.iter().take(segments.len().saturating_sub(1)) {
        dir.push(component.as_os_str());
        if let Ok(m) = fs::symlink_metadata(&dir) {
            if m.file_type().is_symlink() {
                return Err(Error::new("Symlinked ancestor path is unsupported"));
            }
        }
    }
    Ok(())
}

pub fn apply_entry(repo: &Path, index: &Path, path: &str, entry: Option<&Entry>) -> Result<()> {
    validate_path(repo, path)?;
    match entry {
        Some(e) => {
            run(
                repo,
                &[
                    "update-index",
                    "--add",
                    "--cacheinfo",
                    &format!("{},{},{}", e.mode, e.oid, path),
                ],
                Some(index),
            )?;
        }
        None => {
            run(
                repo,
                &["update-index", "--force-remove", "--", path],
                Some(index),
            )?;
        }
    }
    Ok(())
}

pub fn hook_dir(repo: &Path) -> Result<PathBuf> {
    let override_hook = command(
        repo,
        &["config", "--get", "--path", "core.hooksPath"],
        None,
        None,
    )?;
    let path = if override_hook.status.success() {
        PathBuf::from(String::from_utf8_lossy(&override_hook.stdout).trim())
    } else {
        PathBuf::from(run(repo, &["rev-parse", "--git-path", "hooks"], None)?)
    };
    Ok(if path.is_absolute() {
        path
    } else {
        repo.join(path)
    })
}

pub fn status(repo: &Path) -> Result<String> {
    run(
        repo,
        &["status", "--porcelain=v1", "--untracked-files=all"],
        None,
    )
}

pub fn canonical(repo: &Path) -> Result<PathBuf> {
    Ok(fs::canonicalize(repo)?)
}

pub fn hash_bytes(repo: &Path, bytes: &[u8]) -> Result<String> {
    run_input(repo, &["hash-object", "--stdin"], None, Some(bytes))
}

/// Write-producing lab diagnostic: `write-tree` is intentional in disposable fixtures.
pub fn log_state(repo: &Path) -> Result<serde_json::Value> {
    let ent = entries(repo, None)?;
    Ok(serde_json::json!({
        "head": run(repo, &["rev-parse", "HEAD"], None)?,
        "head_tree": run(repo, &["rev-parse", "HEAD^{tree}"], None)?,
        "index_tree": run(repo, &["write-tree"], None)?,
        "entries": ent.iter().map(|(name,e)| format!("{name}:{}:{}",e.mode,e.oid)).collect::<Vec<_>>(),
        "status": status(repo)?,
    }))
}
