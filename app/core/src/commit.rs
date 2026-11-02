//! Controlled selective commit experiment. This is NOT an arbitrary-hook sandbox.
//! Supported: local branch, ordinary index (stage-0), UTF-8 paths, selected index/worktree content.
use crate::git::{self, Entries};
use crate::journal::{self, Journal, JournalState};
use crate::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Index,
    Worktree,
}

#[derive(Debug, Clone)]
pub struct PlanRequest {
    pub selected: BTreeMap<String, Source>,
    pub message: String,
    pub hooks: bool,
    pub sign: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrashAt {
    BeforeRef,
    AfterRef,
    AfterIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_head: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_hook: Option<String>,
}
impl Outcome {
    pub fn from_status(s: &str) -> Self {
        Self {
            status: s.to_owned(),
            reason: None,
            new_head: None,
            post_hook: None,
        }
    }
    pub fn blocked(s: &str, reason: impl Into<String>) -> Self {
        Self {
            status: s.to_owned(),
            reason: Some(reason.into()),
            new_head: None,
            post_hook: None,
        }
    }
}

fn get_git_dir(repo: &Path) -> Result<PathBuf> {
    let d = PathBuf::from(git::run(repo, &["rev-parse", "--absolute-git-dir"], None)?);
    Ok(fs::canonicalize(d)?)
}
fn validate_identity(repo: &Path, j: &Journal) -> Result<()> {
    if git::canonical(repo)? != j.repo || get_git_dir(repo)? != j.git_dir {
        return Err(Error::new("Repository identity differs from journal"));
    }
    Ok(())
}
fn fingerprint(repo: &Path, path: &str) -> Result<String> {
    git::validate_path(repo, path)?;
    let target = repo.join(path);
    match fs::symlink_metadata(&target) {
        Ok(meta) if meta.file_type().is_symlink() => {
            let link = fs::read_link(&target)?;
            Ok(format!(
                "symlink:{}",
                git::hash_bytes(repo, link.as_os_str().as_encoded_bytes())?
            ))
        }
        Ok(meta) if meta.is_file() => {
            let data = fs::read(target)?;
            #[cfg(unix)]
            let executable = {
                use std::os::unix::fs::PermissionsExt;
                meta.permissions().mode() & 0o111 != 0
            };
            #[cfg(not(unix))]
            let executable = false;
            Ok(format!(
                "file:executable={}:{}",
                executable,
                git::hash_bytes(repo, &data)?
            ))
        }
        Ok(_) => Err(Error::new(format!(
            "Unsupported selected filesystem type: {path}"
        ))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok("missing".into()),
        Err(e) => Err(e.into()),
    }
}
fn entry_snapshot(e: &Entries) -> BTreeMap<String, (String, String)> {
    e.iter()
        .map(|(name, entry)| (name.clone(), (entry.mode.clone(), entry.oid.clone())))
        .collect()
}
fn snapshot_fingerprints(repo: &Path, j: &Journal) -> Result<bool> {
    for (path, expected) in &j.worktree_fingerprints {
        if fingerprint(repo, path)? != *expected {
            return Ok(false);
        }
    }
    Ok(true)
}
fn journal_index(scratch: &Path, name: &str) -> PathBuf {
    scratch.join(name)
}

fn execute_hook(repo: &Path, index: &Path, script: &Path, args: &[&str]) -> Result<()> {
    let mut cmd = git::clean_command(script.as_os_str());
    cmd.current_dir(repo)
        .args(args)
        .env("GIT_INDEX_FILE", index)
        .env("GIT_EDITOR", ":")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(Error::with_kind(
            ErrorKind::HookRejected,
            format!(
                "Hook {} exited {:?}: {}",
                script.display(),
                output.status.code(),
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    Ok(())
}
fn run_hooks(repo: &Path, index: &Path, message: &Path, enabled: bool) -> Result<Vec<String>> {
    let mut invoked = Vec::new();
    if !enabled {
        return Ok(invoked);
    }
    let hookdir = git::hook_dir(repo)?;
    for name in ["pre-commit", "prepare-commit-msg", "commit-msg"] {
        let script = hookdir.join(name);
        if !script.is_file() || !is_executable(&script) {
            continue;
        }
        let path = script.canonicalize()?;
        let message_str = message
            .to_str()
            .ok_or_else(|| Error::new("Non-UTF8 message file path"))?;
        let arguments: Vec<&str> = match name {
            "prepare-commit-msg" => vec![message_str, "message"],
            "commit-msg" => vec![message_str],
            _ => Vec::new(),
        };
        execute_hook(repo, index, &path, &arguments)?;
        invoked.push(name.to_owned());
    }
    Ok(invoked)
}
#[cfg(unix)]
fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(p)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}
#[cfg(not(unix))]
fn is_executable(p: &Path) -> bool {
    p.is_file()
}

pub fn prepare(repo: &Path, journal_path: &Path, request: &PlanRequest) -> Result<Journal> {
    if journal_path.exists() {
        return Err(Error::new("Unresolved existing journal"));
    }
    if request.selected.is_empty() || request.message.trim().is_empty() {
        return Err(Error::new("Empty selection or message"));
    }
    for path in request.selected.keys() {
        git::validate_path(repo, path)?;
    }
    let canonical = git::canonical(repo)?;
    let git_dir = get_git_dir(repo)?;
    let git_ref = git::current_ref(repo)?;
    let old_head = git::run(repo, &["rev-parse", "HEAD"], None)?;
    let old_tree = git::run(repo, &["rev-parse", "HEAD^{tree}"], None)?;
    let real_index_path = git::index_path(repo)?;
    let original = git::entries(repo, None)?;
    let scratch = journal_path.with_extension("scratch");
    fs::create_dir(&scratch)?;
    let selected_index = journal_index(&scratch, "selected.index");
    let reconciled_index = journal_index(&scratch, "reconciled.index");
    git::run(repo, &["read-tree", "HEAD"], Some(&selected_index))?;
    fs::copy(&real_index_path, &reconciled_index)?;
    let mut worktree_fingerprints = BTreeMap::new();
    for (path, source) in &request.selected {
        match source {
            Source::Index => {
                if !original.contains_key(path)
                    && !git::command(
                        repo,
                        &["cat-file", "-e", &format!("HEAD:{path}")],
                        None,
                        None,
                    )?
                    .status
                    .success()
                {
                    return Err(Error::new(format!("Neither staged nor tracked: {path}")));
                }
                git::apply_entry(repo, &selected_index, path, original.get(path))?;
            }
            Source::Worktree => {
                worktree_fingerprints.insert(path.clone(), fingerprint(repo, path)?);
                git::run(
                    repo,
                    &["--literal-pathspecs", "add", "-A", "--", path],
                    Some(&selected_index),
                )?;
            }
        }
    }
    let tree = git::run(repo, &["write-tree"], Some(&selected_index))?;
    if tree == old_tree {
        return Err(Error::new("Selection produces empty commit"));
    }
    let message_file = scratch.join("message.txt");
    fs::write(&message_file, request.message.as_bytes())?;
    let executed_hooks = run_hooks(repo, &selected_index, &message_file, request.hooks)?;
    if tree != git::run(repo, &["write-tree"], Some(&selected_index))?
        || fs::read(&message_file)? != request.message.as_bytes()
    {
        return Err(Error::new(
            "Hook changed approved tree or commit message; review required",
        ));
    }
    for (p, before) in &worktree_fingerprints {
        if fingerprint(repo, p)? != *before {
            return Err(Error::new(
                "Hook changed selected working-tree content; review required",
            ));
        }
    }
    let selected_entries = git::entries(repo, Some(&selected_index))?;
    for path in request.selected.keys() {
        git::apply_entry(repo, &reconciled_index, path, selected_entries.get(path))?;
    }
    let merged = git::entries(repo, Some(&reconciled_index))?;
    for (name, value) in &original {
        if !request.selected.contains_key(name) && merged.get(name) != Some(value) {
            return Err(Error::new("Reconciliation modified unselected index entry"));
        }
    }
    let reconciled_hash = git::hash_bytes(repo, &fs::read(&reconciled_index)?)?;
    let journal = Journal {
        format_version: 1,
        state: JournalState::Prepared,
        repo: canonical,
        git_dir,
        git_ref,
        old_head,
        new_head: None,
        tree,
        message: request.message.clone(),
        selection: request
            .selected
            .iter()
            .map(|(p, s)| {
                (
                    p.clone(),
                    match s {
                        Source::Index => "index",
                        Source::Worktree => "worktree",
                    }
                    .to_owned(),
                )
            })
            .collect(),
        worktree_fingerprints,
        original_index_entries: entry_snapshot(&original),
        reconciled_index_hash: reconciled_hash,
        scratch,
        hooks_executed: executed_hooks,
        hooks_enabled: request.hooks,
        sign: request.sign,
        post_hook_invoked: false,
        post_hook_result: None,
    };
    journal::save(journal_path, &journal)?;
    Ok(journal)
}

fn current_entries(repo: &Path) -> Result<BTreeMap<String, (String, String)>> {
    Ok(entry_snapshot(&git::entries(repo, None)?))
}
fn unchanged(repo: &Path, j: &Journal) -> Result<Option<Outcome>> {
    if git::current_ref(repo).ok().as_deref() != Some(j.git_ref.as_str()) {
        return Ok(Some(Outcome::blocked(
            "blocked_stale_head",
            "Checked-out branch differs from reviewed state",
        )));
    }
    if git::run(repo, &["rev-parse", &j.git_ref], None)? != j.old_head {
        return Ok(Some(Outcome::blocked(
            "blocked_stale_head",
            "HEAD differs from reviewed state",
        )));
    }
    if current_entries(repo)? != j.original_index_entries {
        return Ok(Some(Outcome::blocked(
            "blocked_external_index_change",
            "Git index changed after review",
        )));
    }
    if !snapshot_fingerprints(repo, j)? {
        return Ok(Some(Outcome::blocked(
            "blocked_selected_worktree_change",
            "Selected working-tree content changed",
        )));
    }
    Ok(None)
}
fn owned_lock(repo: &Path, j: &Journal, lock: &Path) -> Result<bool> {
    if !lock.exists() {
        return Ok(false);
    }
    Ok(git::hash_bytes(repo, &fs::read(lock)?)? == j.reconciled_index_hash)
}
fn lock_path(repo: &Path) -> Result<PathBuf> {
    let index = git::index_path(repo)?;
    Ok(index.with_file_name(format!(
        "{}.lock",
        index
            .file_name()
            .unwrap_or(OsStr::new("index"))
            .to_string_lossy()
    )))
}
fn finish(repo: &Path, journal_path: &Path, j: &mut Journal) -> Result<Outcome> {
    let index_path = git::index_path(repo)?;
    let lock = lock_path(repo)?;
    if !owned_lock(repo, j, &lock)? || current_entries(repo)? != j.original_index_entries {
        return Ok(Outcome::blocked(
            "needs_manual_reconciliation",
            "Index lock missing/foreign or current index differs",
        ));
    }
    fs::rename(&lock, &index_path)?;
    journal::sync_directory(index_path.parent().unwrap());
    j.state = JournalState::IndexPromoted;
    journal::save(journal_path, j)?;
    finalize(repo, journal_path, j)
}
fn finalize(repo: &Path, path: &Path, j: &mut Journal) -> Result<Outcome> {
    let new = j
        .new_head
        .clone()
        .ok_or_else(|| Error::new("missing new commit"))?;
    if git::run(repo, &["rev-parse", &j.git_ref], None)? != new {
        return Ok(Outcome::blocked(
            "needs_manual_reconciliation",
            "Branch moved after commit",
        ));
    }
    if git::entries(repo, None)? != git::entries(repo, Some(&j.scratch.join("reconciled.index")))? {
        return Ok(Outcome::blocked(
            "needs_manual_reconciliation",
            "Index differs from approved reconciled index",
        ));
    }
    if git::run(repo, &["rev-parse", &format!("{new}^{{tree}}")], None)? != j.tree {
        return Ok(Outcome::blocked(
            "needs_manual_reconciliation",
            "Committed tree differs from approved tree",
        ));
    }
    if git::run(repo, &["show", "-s", "--format=%B", &new], None)?.trim() != j.message.trim() {
        return Ok(Outcome::blocked(
            "needs_manual_reconciliation",
            "Committed message differs from approved message",
        ));
    }
    let mut post = None;
    if j.hooks_enabled && !j.post_hook_invoked {
        let script = git::hook_dir(repo)?.join("post-commit");
        if script.is_file() && is_executable(&script) {
            j.post_hook_invoked = true;
            journal::save(path, j)?;
            let out = git::clean_command(script.canonicalize()?.as_os_str())
                .current_dir(repo)
                .output()?;
            post = Some(format!(
                "exit={:?}, stderr={}",
                out.status.code(),
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
    }
    j.post_hook_result = post.clone();
    j.state = JournalState::Complete;
    journal::save(path, j)?;
    Ok(Outcome {
        status: "committed".into(),
        reason: None,
        new_head: Some(new),
        post_hook: post,
    })
}

pub fn execute(repo: &Path, journal_path: &Path, crash: Option<CrashAt>) -> Result<Outcome> {
    let mut j = journal::load(journal_path)?;
    validate_identity(repo, &j)?;
    if j.state != JournalState::Prepared {
        return Err(Error::new(format!("Journal not prepared: {:?}", j.state)));
    }
    if let Some(out) = unchanged(repo, &j)? {
        return Ok(out);
    }
    let mut args = vec!["commit-tree", &j.tree, "-p", &j.old_head];
    if j.sign {
        args.push("-S");
    }
    let out = git::command(repo, &args, None, Some(j.message.as_bytes()))?;
    if !out.status.success() {
        return Ok(Outcome::blocked(
            "blocked_commit_tree_failure",
            String::from_utf8_lossy(&out.stderr).trim().to_owned(),
        ));
    }
    let new_oid = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    j.new_head = Some(new_oid.clone());
    j.state = JournalState::CommitObjectCreated;
    journal::save(journal_path, &j)?;
    let lock = lock_path(repo)?;
    let mut lock_file = match OpenOptions::new().write(true).create_new(true).open(&lock) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Ok(Outcome::blocked(
                "blocked_index_locked",
                "Git index is locked",
            ))
        }
        Err(e) => return Err(e.into()),
    };
    lock_file.write_all(&fs::read(j.scratch.join("reconciled.index"))?)?;
    lock_file.sync_all()?;
    drop(lock_file);
    if let Some(out) = unchanged(repo, &j)? {
        let _ = fs::remove_file(lock);
        return Ok(out);
    }
    j.state = JournalState::RefUpdatePending;
    journal::save(journal_path, &j)?;
    if crash == Some(CrashAt::BeforeRef) {
        kill_self();
    }
    let update = git::command(
        repo,
        &[
            "update-ref",
            "-m",
            "GitFleet controlled POC",
            &j.git_ref,
            &new_oid,
            &j.old_head,
        ],
        None,
        None,
    )?;
    if !update.status.success() {
        let _ = fs::remove_file(lock);
        j.state = JournalState::RefRejected;
        journal::save(journal_path, &j)?;
        return Ok(Outcome::blocked(
            "blocked_stale_head",
            String::from_utf8_lossy(&update.stderr).to_string(),
        ));
    }
    if crash == Some(CrashAt::AfterRef) {
        kill_self();
    }
    j.state = JournalState::RefUpdatedPendingIndex;
    journal::save(journal_path, &j)?;
    let out = finish(repo, journal_path, &mut j)?;
    if crash == Some(CrashAt::AfterIndex) {
        kill_self();
    }
    Ok(out)
}

pub fn recover(repo: &Path, journal_path: &Path) -> Result<Outcome> {
    let mut j = journal::load(journal_path)?;
    validate_identity(repo, &j)?;
    if j.state == JournalState::Complete {
        return Ok(Outcome {
            status: "already_complete".into(),
            reason: None,
            new_head: j.new_head,
            post_hook: j.post_hook_result,
        });
    }
    if j.state == JournalState::Prepared {
        return Ok(Outcome::from_status("not_started"));
    }
    let actual = git::run(repo, &["rev-parse", &j.git_ref], None)?;
    let lock = lock_path(repo)?;
    if actual == j.old_head {
        if lock.exists() {
            if !owned_lock(repo, &j, &lock)? {
                return Ok(Outcome::blocked(
                    "needs_manual_reconciliation",
                    "Foreign Git index lock",
                ));
            }
            fs::remove_file(lock)?;
        }
        j.state = JournalState::AbortedBeforeRef;
        journal::save(journal_path, &j)?;
        return Ok(Outcome::from_status("aborted_before_ref"));
    }
    if j.new_head.as_deref() != Some(actual.as_str()) {
        return Ok(Outcome::blocked(
            "needs_manual_reconciliation",
            "Branch advanced to an unexpected commit",
        ));
    }
    if j.state == JournalState::IndexPromoted {
        return finalize(repo, journal_path, &mut j);
    }
    if lock.exists() {
        return finish(repo, journal_path, &mut j);
    }
    if git::entries(repo, None)? == git::entries(repo, Some(&j.scratch.join("reconciled.index")))? {
        j.state = JournalState::IndexPromoted;
        journal::save(journal_path, &j)?;
        return finalize(repo, journal_path, &mut j);
    }
    Ok(Outcome::blocked(
        "needs_manual_reconciliation",
        "Index lock missing and index differs; manual review required",
    ))
}

#[cfg(unix)]
fn kill_self() {
    unsafe extern "C" {
        fn kill(pid: i32, sig: i32) -> i32;
        fn getpid() -> i32;
    }
    unsafe {
        kill(getpid(), 9);
    }
    std::process::exit(137);
}
#[cfg(not(unix))]
fn kill_self() {
    std::process::exit(137);
}
