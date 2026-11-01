//! Disposable fixture generation and behavioral acceptance suite.
//! Creates and deletes only directories with the `gitfleet-rust-lab-` temp prefix.
use crate::commit::{self, CrashAt, Outcome, PlanRequest, Source};
use crate::git;
use crate::journal;
use crate::{Error, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SEQ: AtomicU64 = AtomicU64::new(0);

pub const CASES: &[&str] = &[
    "selected_staged_preserves_other_staging", "selected_worktree_preserves_other_staging",
    "partial_staging_commits_index_only", "partial_staging_commits_worktree",
    "mixed_selected_sources", "untracked_selected", "selected_unstaged_deletion",
    "selected_rename_pair", "space_in_path", "hook_modifies_only_worktree",
    "hook_stages_unapproved", "hook_rewrites_message", "post_commit_hook",
    "hook_changes_selected_worktree", "prepare_hook_rewrites_message",
    "selected_worktree_changed_after_review", "external_index_changed",
    "stale_head_before_cas", "real_sigkill_before_ref", "real_sigkill_after_ref",
    "real_sigkill_after_reconcile", "crash_then_external_index_edit",
    "signing_missing_key", "successful_gpg_signing",
];

pub struct Fixture { pub root: PathBuf, pub repo: PathBuf }
impl Fixture {
    pub fn new() -> Result<Self> {
        let t = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| Error::new(e.to_string()))?.as_nanos();
        let root = std::env::temp_dir().join(format!("gitfleet-rust-lab-{}-{}-{}", std::process::id(), t, SEQ.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&root)?;
        let repo = root.join("repo");
        fs::create_dir(&repo)?;
        fs::write(repo.join(".gitfleet-lab-fixture"), b"Disposable repository created by GitFleet Rust Lab\n")?;
        git::run(&repo, &["init", "-q", "-b", "main"], None)?;
        for (key, val) in [("user.name", "GitFleet Lab"), ("user.email", "lab@example.invalid"), ("commit.gpgsign", "false")] {
            git::run(&repo, &["config", "--local", key, val], None)?;
        }
        for (path, data) in [
            ("full.txt", "original\n"), ("partial.txt", "first old\nsecond old\n"),
            ("other.txt", "other original\n"), ("delete.txt", "delete me\n"),
            ("rename.txt", "rename content\n"), ("unrelated.txt", "unrelated original\n"),
            ("space file.txt", "original with spaces\n"),
        ] { fs::write(repo.join(path), data)?; }
        git::run(&repo, &["add", "-A"], None)?;
        git::run(&repo, &["commit", "-qm", "baseline"], None)?;
        Ok(Self { root, repo })
    }
    pub fn journal(&self) -> PathBuf { self.root.join("transaction.json") }
    pub fn stage(&self, file: &str, data: &str) -> Result<()> {
        fs::write(self.repo.join(file), data)?;
        git::run(&self.repo, &["add", "--", file], None)?;
        Ok(())
    }
    pub fn hook(&self, name: &str, script: &str) -> Result<()> {
        let d = git::hook_dir(&self.repo)?;
        fs::create_dir_all(&d)?;
        let p = d.join(name);
        fs::write(&p, format!("#!/bin/sh\nset -eu\n{script}\n"))?;
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&p, fs::Permissions::from_mode(0o755))?;
        }
        Ok(())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if self.root.file_name().and_then(|s| s.to_str()).map(|s| s.starts_with("gitfleet-rust-lab-")).unwrap_or(false) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
fn request(chosen: &[(&str, Source)], sign: bool) -> PlanRequest {
    PlanRequest { selected: chosen.iter().map(|(p,s)| ((*p).to_owned(), *s)).collect(),
                  message: "approved selective commit\n".into(), hooks: true, sign }
}
fn setup_mixed(f: &Fixture) -> Result<()> {
    f.stage("full.txt", "reviewed\n")?;
    f.stage("other.txt", "keep staged\n")?;
    fs::write(f.repo.join("unrelated.txt"), "keep unstaged\n")?;
    fs::write(f.repo.join("new.txt"), "untracked\n")?;
    Ok(())
}
fn setup_partial(f: &Fixture) -> Result<()> {
    f.stage("partial.txt", "first staged\nsecond old\n")?;
    fs::write(f.repo.join("partial.txt"), "first staged\nsecond unstaged\n")?;
    Ok(())
}
fn validate_outcome(name: &str, output: &Outcome, f: &Fixture) -> Result<()> {
    let expected = if name.starts_with("hook_stages_") || name == "hook_rewrites_message" || name == "prepare_hook_rewrites_message" || name == "hook_changes_selected_worktree" {
        "blocked_before_commit"
    } else if name == "selected_worktree_changed_after_review" {
        "blocked_selected_worktree_change"
    } else if name == "external_index_changed" {
        "blocked_external_index_change"
    } else if name == "stale_head_before_cas" { "blocked_stale_head" }
    else if name == "signing_missing_key" { "blocked_commit_tree_failure" }
    else if name == "real_sigkill_before_ref" { "aborted_before_ref" }
    else if name == "crash_then_external_index_edit" { "needs_manual_reconciliation" }
    else if name == "real_sigkill_after_reconcile" { "already_complete" }
    else { "committed" };
    if output.status != expected { return Err(Error::new(format!("{name}: expected {expected}, got {:?}", output))); }
    if (expected == "committed" || expected == "already_complete") && git::run(&f.repo, &["rev-parse", "HEAD"], None)? != output.new_head.clone().unwrap_or_default() {
        return Err(Error::new(format!("{name}: wrong HEAD after commit")));
    }
    Ok(())
}

fn worker_kill(f: &Fixture, crash: CrashAt) -> Result<()> {
    let binary = match std::env::var_os("GITFLEET_LAB_BIN") {
        Some(path) => PathBuf::from(path),
        None => std::env::current_exe()?,
    };
    let crasharg = match crash { CrashAt::BeforeRef => "before_ref", CrashAt::AfterRef => "after_ref", CrashAt::AfterIndex => "after_index" };
    let output = Command::new(binary).args(["--worker", f.repo.to_str().unwrap(), f.journal().to_str().unwrap(), crasharg])
        .stdout(Stdio::piped()).stderr(Stdio::piped()).output()?;
    #[cfg(unix)] {
        use std::os::unix::process::ExitStatusExt;
        if output.status.signal() != Some(9) { return Err(Error::new(format!("Worker not killed by SIGKILL: {:?} {}", output.status, String::from_utf8_lossy(&output.stderr)))); }
    }
    Ok(())
}

fn worker(f: &Fixture, crash: CrashAt) -> Result<Outcome> {
    worker_kill(f, crash)?;
    commit::recover(&f.repo, &f.journal())
}

pub fn ensure_lab_fixture(repo: &Path) -> Result<()> {
    let canonical = fs::canonicalize(repo)?;
    let parent = canonical.parent().ok_or_else(|| Error::new("Missing fixture parent"))?;
    let prefix = parent.file_name().and_then(|x| x.to_str()).unwrap_or("");
    if !prefix.starts_with("gitfleet-rust-lab-") || canonical.file_name().and_then(|x| x.to_str()) != Some("repo") || !canonical.join(".gitfleet-lab-fixture").is_file() {
        return Err(Error::new("Worker restricted to disposable GitFleet fixture"));
    }
    Ok(())
}

pub fn run_case(name: &str) -> Result<Value> {
    if !CASES.contains(&name) { return Err(Error::new(format!("Unknown scenario: {name}"))); }
    let f = Fixture::new()?;
    use Source::{Index as I, Worktree as W};
    let mut selections = vec![("full.txt", I)];
    match name {
        "selected_staged_preserves_other_staging" | "selected_worktree_preserves_other_staging" => { setup_mixed(&f)?; if name.contains("worktree") { selections[0].1 = W; } }
        "partial_staging_commits_index_only" | "partial_staging_commits_worktree" => { setup_partial(&f)?; selections = vec![("partial.txt", if name.contains("worktree") {W} else {I})]; }
        "mixed_selected_sources" => { f.stage("full.txt", "selected staged\n")?; fs::write(f.repo.join("partial.txt"), "first new\nsecond old\n")?; f.stage("other.txt", "unselected staged\n")?; selections = vec![("full.txt", I), ("partial.txt", W)]; }
        "untracked_selected" => { fs::write(f.repo.join("new.txt"), "created\n")?; f.stage("other.txt", "other staged\n")?; selections = vec![("new.txt", W)]; }
        "selected_unstaged_deletion" => { fs::remove_file(f.repo.join("delete.txt"))?; f.stage("other.txt", "other staged\n")?; selections = vec![("delete.txt", W)]; }
        "selected_rename_pair" => { fs::rename(f.repo.join("rename.txt"), f.repo.join("renamed.txt"))?; f.stage("other.txt", "other staged\n")?; selections = vec![("rename.txt", W), ("renamed.txt", W)]; }
        "space_in_path" => { f.stage("space file.txt", "with spaces\n")?; f.stage("other.txt", "other staged\n")?; selections = vec![("space file.txt", I)]; }
        "hook_stages_unapproved" => { f.stage("full.txt", "reviewed\n")?; f.hook("pre-commit", "printf 'extra\\n' > unrelated.txt\ngit add -- unrelated.txt")?; }
        "hook_rewrites_message" => { f.stage("full.txt", "reviewed\n")?; f.hook("commit-msg", "printf 'unapproved message\\n' > \"$1\"")?; }
        "prepare_hook_rewrites_message" => { f.stage("full.txt", "reviewed\n")?; f.hook("prepare-commit-msg", "printf 'unapproved message\\n' > \"$1\"")?; }
        "hook_modifies_only_worktree" => { f.stage("full.txt", "reviewed\n")?; f.hook("pre-commit", "printf 'hook-only\\n' > unrelated.txt")?; }
        "hook_changes_selected_worktree" => { f.stage("full.txt", "reviewed\n")?; f.hook("pre-commit", "printf 'hook change\\n' > full.txt")?; selections[0].1 = W; }
        "post_commit_hook" => { f.stage("full.txt", "reviewed\n")?; f.hook("post-commit", "printf 'ran\\n' > post-commit-marker.txt")?; }
        "selected_worktree_changed_after_review" => { f.stage("full.txt", "reviewed\n")?; selections[0].1 = W; }
        "external_index_changed" | "stale_head_before_cas" | "real_sigkill_before_ref" | "real_sigkill_after_ref" | "real_sigkill_after_reconcile" | "crash_then_external_index_edit" | "signing_missing_key" => { f.stage("full.txt", "reviewed\n")?; }
        "successful_gpg_signing" => { f.stage("full.txt", "reviewed\n")?; }
        _ => { f.stage("full.txt", "reviewed\n")?; }
    }
    let before = git::log_state(&f.repo)?;
    if name == "successful_gpg_signing" {
        let out = Command::new("gpg").arg("--version").output();
        if out.is_err() { return Ok(json!({"status":"skipped","reason":"gpg not installed","before":before})); }
        return signing_case(&f, before, &selections);
    }
    let request = request(&selections, name == "signing_missing_key");
    let prepared = commit::prepare(&f.repo, &f.journal(), &request);
    let outcome = match prepared {
        Err(err) => Outcome::blocked("blocked_before_commit", err.to_string()),
        Ok(_) => {
            match name {
                "selected_worktree_changed_after_review" => {
                    fs::write(f.repo.join("full.txt"), "changed since approval\n")?;
                    commit::execute(&f.repo, &f.journal(), None)?
                }
                "external_index_changed" => { f.stage("unrelated.txt", "new external staged\n")?; commit::execute(&f.repo, &f.journal(), None)? }
                "stale_head_before_cas" => {
                    let head = git::run(&f.repo, &["rev-parse", "HEAD"], None)?;
                    let tree = git::run(&f.repo, &["rev-parse", "HEAD^{tree}"], None)?;
                    let oid = git::run_input(&f.repo, &["commit-tree", &tree, "-p", &head], None, Some(b"external branch change\n"))?;
                    git::run(&f.repo, &["update-ref", "refs/heads/main", &oid, &head], None)?;
                    commit::execute(&f.repo, &f.journal(), None)?
                }
                "real_sigkill_before_ref" => worker(&f, CrashAt::BeforeRef)?,
                "real_sigkill_after_ref" => worker(&f, CrashAt::AfterRef)?,
                "real_sigkill_after_reconcile" => worker(&f, CrashAt::AfterIndex)?,
                "crash_then_external_index_edit" => {
                    // Kill after branch update; simulate an operator clearing the stale lock before external Git writes.
                    worker_kill(&f, CrashAt::AfterRef)?;
                    // Test harness alone clears its own abandoned index lock, then makes a real external Git edit.
                    let lock = f.repo.join(".git/index.lock");
                    if lock.exists() { fs::remove_file(&lock)?; }
                    f.stage("other.txt", "external index edit\n")?;
                    commit::recover(&f.repo, &f.journal())?
                }
                "signing_missing_key" => {
                    git::run(&f.repo, &["config", "--local", "user.signingkey", "GITFLEET_MISSING_SIGNING_KEY"], None)?;
                    git::run(&f.repo, &["config", "--local", "gpg.program", "gitfleet-no-gpg-executable"], None)?;
                    commit::execute(&f.repo, &f.journal(), None)?
                }
                _ => commit::execute(&f.repo, &f.journal(), None)?,
            }
        }
    };
    validate_outcome(name, &outcome, &f)?;
    if outcome.status == "committed" || outcome.status == "already_complete" {
        let j = journal::load(&f.journal())?;
        let head = git::run(&f.repo, &["rev-parse", "HEAD"], None)?;
        let tree = git::run(&f.repo, &["rev-parse", "HEAD^{tree}"], None)?;
        let msg = git::run(&f.repo, &["show", "-s", "--format=%B", "HEAD"], None)?;
        if j.new_head.as_deref() != Some(head.as_str()) || tree != j.tree || msg.trim() != j.message.trim() {
            return Err(Error::new(format!("{name}: actual commit differs from authorized tree/message")));
        }
    }
    let after = git::log_state(&f.repo)?;
    validate_preservation(name, &before, &after, &outcome)?;
    Ok(json!({"status":outcome.status,"result":outcome,"before":before,"after":after}))
}

fn validate_preservation(name: &str, before: &Value, after: &Value, out: &Outcome) -> Result<()> {
    let s = after["status"].as_str().unwrap_or("");
    match name {
        "selected_staged_preserves_other_staging" | "selected_worktree_preserves_other_staging" => {
            for needle in ["M  other.txt", " M unrelated.txt", "?? new.txt"] {
                if !s.contains(needle) { return Err(Error::new(format!("{name}: missing preserved {needle}: {s}"))); }
            }
        }
        "partial_staging_commits_index_only" => if !s.contains(" M partial.txt") { return Err(Error::new(format!("Missing unstaged partial work: {s}"))); },
        "mixed_selected_sources" | "untracked_selected" | "selected_rename_pair" | "selected_unstaged_deletion" | "space_in_path" => {
            if !s.contains("M  other.txt") { return Err(Error::new(format!("Unselected staged change lost: {s}"))); }
        }
        "post_commit_hook" => if !s.contains("?? post-commit-marker.txt") { return Err(Error::new(format!("Post hook did not execute: {s}"))); },
        "hook_modifies_only_worktree" => if !s.contains(" M unrelated.txt") { return Err(Error::new(format!("Worktree hook effect not preserved: {s}"))); },
        "hook_stages_unapproved" | "hook_rewrites_message" | "prepare_hook_rewrites_message" | "hook_changes_selected_worktree" | "external_index_changed" | "selected_worktree_changed_after_review" | "signing_missing_key" | "real_sigkill_before_ref" => {
            if before["head"] != after["head"] { return Err(Error::new(format!("Unexpected branch update in {name}"))); }
        }
        _ => {}
    }
    if out.status == "committed" && before["head"] == after["head"] { return Err(Error::new(format!("Expected branch change for {name}"))); }
    Ok(())
}

fn signing_case(f: &Fixture, before: Value, selections: &[(&str, Source)]) -> Result<Value> {
    let gnupg = f.root.join("gnupg");
    fs::create_dir(&gnupg)?;
    #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; fs::set_permissions(&gnupg, fs::Permissions::from_mode(0o700))?; }
    let generated = Command::new("gpg").env("GNUPGHOME", &gnupg)
        .args(["--batch","--yes","--pinentry-mode","loopback","--passphrase","", "--quick-generate-key", "GitFleet Lab <lab@example.invalid>", "ed25519", "sign", "0"])
        .output()?;
    if !generated.status.success() { return Ok(json!({"status":"skipped","reason":String::from_utf8_lossy(&generated.stderr).to_string()})); }
    let keys = Command::new("gpg").env("GNUPGHOME", &gnupg).args(["--with-colons", "--list-secret-keys"]).output()?;
    let listing = String::from_utf8_lossy(&keys.stdout);
    let fingerprint = listing.lines().filter(|s|s.starts_with("fpr:")).find_map(|s| s.split(':').nth(9)).ok_or_else(|| Error::new("No GPG key fingerprint"))?;
    git::run(&f.repo, &["config", "--local", "user.signingkey", fingerprint], None)?;
    let req = request(selections, true);
    commit::prepare(&f.repo, &f.journal(), &req)?;
    // GitFleet controlled signing invokes Git, which inherits GPG agent environment.
    // An ephemeral GNUPGHOME needs to be set for this scoped lab execution.
    let old = std::env::var_os("GNUPGHOME");
    std::env::set_var("GNUPGHOME", &gnupg);
    let outcome = commit::execute(&f.repo, &f.journal(), None);
    if let Some(x) = old { std::env::set_var("GNUPGHOME", x); } else { std::env::remove_var("GNUPGHOME"); }
    let outcome = outcome?;
    if outcome.status != "committed" { return Err(Error::new(format!("GPG signing failed: {:?}",outcome))); }
    let new = outcome.new_head.clone().unwrap();
    let commit = git::run(&f.repo, &["cat-file", "-p", &new], None)?;
    if !commit.contains("gpgsig ") { return Err(Error::new("Signed commit has no gpgsig")); }
    let verify = Command::new("git").current_dir(&f.repo).env("GNUPGHOME", &gnupg).args(["verify-commit", &new]).output()?;
    if !verify.status.success() { return Err(Error::new("GPG signature verification failed")); }
    Ok(json!({"status":"committed","signed":true,"signature_verified":true,"before":before,"after":git::log_state(&f.repo)?}))
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub git_version: String,
    pub cases: BTreeMap<String, Value>,
    pub passed: usize,
    pub skipped: usize,
    pub disclaimer: &'static str,
}
pub fn run_suite() -> Result<Report> {
    let mut cases = BTreeMap::new();
    let mut passed = 0;
    let mut skipped = 0;
    for case in CASES {
        let result = run_case(case)?;
        if result["status"] == "skipped" { skipped += 1; } else { passed += 1; }
        cases.insert((*case).into(), result);
    }
    Ok(Report {
        git_version: git::run(Path::new("."), &["--version"], None)?, cases,
        passed, skipped,
        disclaimer: "Research-only; manual hook subset, local branches, ordinary indexes. Not production-safe.",
    })
}
