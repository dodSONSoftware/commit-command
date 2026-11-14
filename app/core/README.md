# GitFleet Core — read-only workspaces, controlled writes and application services

Core remains one presentation-independent Rust crate. Default features expose
workspace persistence and read-only Git services. `experimental-writes` still
only enables the disposable commit lab; it is not production approval.

M3 adds optional `application-services`: presentation-independent orchestration,
monitoring, persisted navigation and schema generation over the existing services.
Core remains one crate with no Tauri/React dependency. The separate desktop adapter
is implemented but native acceptance remains incomplete; ordinary desktop writes
remain disabled. Later user-observed native results are in the
[November 2 checkpoint](../../docs/M3_NATIVE_ACCEPTANCE_HANDOFF_2026-11-02.md). See [M3 contracts](../../docs/architecture/M3_DESKTOP_AND_IPC.md) and
[current evidence](../../docs/validation/m3/README.md).

Requires Rust 1.99+, Git 2.56.0+, Python 3, and a C toolchain for bundled SQLite.
Dependencies are locked; tests create their own temporary Git repositories and
application data directories. No live remotes, credentials, AI or desktop are needed.

From this directory:

```sh
GITFLEET_TEST_GPG=/absolute/path/to/gpg sh scripts/validate.sh
cargo run --bin gitfleet-cli -- status /path/to/worktree
cargo run --bin gitfleet-cli -- scan /path/to/workspace
cargo run --bin gitfleet-cli -- history /path/to/worktree 100
cargo run --bin gitfleet-cli -- history /path/to/worktree 100 refs/heads/main
cargo run --bin gitfleet-cli -- commit /path/to/worktree FULL_COMMIT_OID
cargo run --bin gitfleet-cli -- diff /path/to/worktree staged
cargo run --bin gitfleet-cli -- diff /path/to/worktree unstaged 'literal/path.txt'
cargo run --bin gitfleet-cli -- diff /path/to/worktree commit FULL_COMMIT_OID FULL_PARENT_OID
cargo run --bin gitfleet-cli -- preview /path/to/worktree 'literal/path.txt'
```

Root-commit diffs omit the parent argument. Non-root diffs require a true parent.
History accepts an optional serialized `next` cursor as its final argument, after
page size and full ref. Use the library DTOs for lossless non-UTF-8 paths; CLI paths
are native OS arguments. JSON byte fields are arrays, never lossy execution paths.

Workspace configuration requires an explicit data directory outside managed repos:

```sh
cargo run --bin gitfleet-cli -- --data-dir /outside/projects/gitfleet-state workspace create Project /path/to/workspace
cargo run --bin gitfleet-cli -- --data-dir /outside/projects/gitfleet-state workspace list
cargo run --bin gitfleet-cli -- --data-dir /outside/projects/gitfleet-state workspace activate WORKSPACE_ID
cargo run --bin gitfleet-cli -- --data-dir /outside/projects/gitfleet-state workspace scan WORKSPACE_ID
cargo run --bin gitfleet-cli -- --data-dir /outside/projects/gitfleet-state workspace inventory WORKSPACE_ID
cargo run --bin gitfleet-cli -- --data-dir /outside/projects/gitfleet-state workspace last-scan WORKSPACE_ID
cargo run --bin gitfleet-cli -- --data-dir /outside/projects/gitfleet-state workspace refresh WORKSPACE_ID WORKTREE_ID
```

Additional verbs: `active`, `rename ID NAME`, `root ID ROOT`, `remove ID`, and
`exclusions ID JSON`. Exclusions JSON contains `excluded_names` and
`excluded_patterns`. Pattern matching is case-sensitive, workspace-relative, with
literal separators; `.git` traversal is always forbidden. `discover` retains the
M0 compatibility result/error behavior; `scan` exposes partial inventories.
Standalone scans and workspace scans share the same traversal and remaining-time
budget, including metadata inspection and directory enumeration. Timeout returns
an incomplete inventory with a timeout issue; cancellation returns `cancelled`.
Only workspace scans retain in-memory continuation. Standalone results omit
continuation progress, and `discover` keeps its error for incomplete inventories.

Status includes structured changes/counts, HEAD/branch, local-only upstream
comparison, operation markers, last commit, generation and observation time.
The old `status` text remains a compatibility diagnostic; structured fields are
authoritative, including staged gitlinks recovered without scanning children.

[The M1 contracts](../../docs/architecture/M1_READ_ONLY_WORKSPACES.md) define limits,
identity/persistence behavior, unsafe configurations and platform restrictions.
[Status](../../docs/IMPLEMENTATION_STATUS.md) records exact validation and skips;
[handoff](../../docs/IMPLEMENTATION_HANDOFF.md) records the next authorization boundary.

The original suite is preserved:

```sh
cargo test --workspace --all-targets --features experimental-writes
cargo run --features experimental-writes --bin gitfleet-lab -- --output target/lab-results.json
# Explicit positive GPG test requires successful creation and verification.
cargo test --features experimental-writes --test acceptance successful_gpg_signing -- --ignored --exact
```

## Explicit M2 controlled writes

`controlled-writes` enables a separate service and interactive CLI for the
[documented macOS/APFS support envelope](../../docs/architecture/M2_CONTROLLED_WRITES.md).
It does not enable the experimental APIs. Existing workspace/worktree IDs must be
obtained with the read-only workspace CLI above; data must remain outside checkouts.
These commands are examples, not authorization to target a repository:

```sh
cargo run --features controlled-writes --bin gitfleet-write -- /outside/projects/gitfleet-state trust WORKSPACE_ID WORKTREE_ID
cargo run --features controlled-writes --bin gitfleet-write -- /outside/projects/gitfleet-state propose WORKSPACE_ID WORKTREE_ID '{"kind":"commit","selected":[{"path":{"encoding":"utf8","value":"file.txt"},"source":"working_tree"}],"message":"Reviewed message","hooks":"execute","signing":"configured"}'
cargo run --features controlled-writes --bin gitfleet-write -- /outside/projects/gitfleet-state history
cargo run --features controlled-writes --bin gitfleet-write -- /outside/projects/gitfleet-state recover OPERATION_UUID
cargo run --features controlled-writes --bin gitfleet-write -- /outside/projects/gitfleet-state revoke WORKTREE_ID
```

Trust, proposal execution and recovery each require an interactive literal
`APPROVE`. A proposal never writes managed Git objects/index/refs. The approved
execution does; trust in hooks/signers permits unsandboxed user-code execution.
There is no generic Git command, unattended approval or force-lock cleanup endpoint.
Use library NativePath DTOs for native byte paths; this small write CLI accepts
UTF-8 arguments. It is not a desktop UX or bulk executor.

Full-file stage/unstage request JSON uses `kind: "stage"`/`"unstage"`, `paths` with
NativePath values and the explicit `hooks` policy. `bypass_all` is complete and
visible; `unsigned` is an explicitly reviewed signing preference. The Rust API
also provides saved hook policy and separate read-only recovery inspection.

M2 validation requires actual positive OpenPGP and SSH results. Set
`GITFLEET_TEST_GPG` to an absolute installed GPG executable; keyrings/keys are created
only in disposable temporary locations and agents are cleaned up. Missing GPG
fails the gate, rather than silently skipping. The all-feature matrix includes a
fixture-only SIGKILL driver (`write-test-support`); production adapters should use
only `controlled-writes`. Non-macOS validation runs default and experimental checks
and explicitly labels M2 unsupported. No release/platform support follows from
unexecuted CI. See [M2 evidence](../../docs/validation/m2/README.md) for exact results.

## Historical v0.1.1 package README and source listings

Everything below is preserved historical material. Paths, commands, source
listings and authoring-time verification statements describe the imported package,
not the assembled M0 tree. The actual source files and current documentation above
are authoritative. The old listings are intentionally not synchronized.

# GitFleet — Rust Core Proof of Concept

A **headless Rust POC** for GitFleet's controlled selective-commit engine, based on the established GitFleet Hook Lab v1–v4 experiments. Its core has **no Tauri, React, browser or AI dependencies**. `gitfleet-cli` provides read-only repository inspection, and `gitfleet-lab` performs write tests **only against disposable repositories created under the OS temporary directory**.

The 24-case acceptance matrix covers selective commits, partial staging, untracked files, deletion/rename, changed hook output, unexpected index/HEAD edits, real `SIGKILL` windows, recovery, and optional ephemeral GPG signing. Tests use the actual installed Git CLI, not a mocked implementation. Further core-level discovery/path-safety tests are included, including canonical-path and linked-worktree regression coverage.

> **Status (v0.1.1):** The original v0.1.0 project was compiled on macOS with Git 2.56.0: 24/24 acceptance tests passed, while repository discovery failed 1 of 4 core tests because its test compared a non-canonical temporary path against a canonical path. v0.1.1 fixes that assertion, adds two discovery regression tests, and removes an unused import. **v0.1.1 has not been compiled or tested with Cargo in the authoring environment** (Rust/Cargo unavailable). Run the tests on your Mac before treating this revision as verified. This is a research lab, not a production-safe commit engine.

## Debian-based CLI — install, build, test, run (end-to-end)

```bash
sudo apt-get update
sudo apt-get install -y rustc cargo git python3 gnupg
cd gitfleet-rust-core-poc
python3 scripts/verify-scaffold.py
cargo build --workspace --all-targets
cargo test --workspace --all-targets -- --nocapture
cargo run --bin gitfleet-lab -- --output results.json
cargo run --bin gitfleet-cli -- discover /tmp
cargo run --bin gitfleet-cli -- status .  # only from within an existing Git repository
```

To run on macOS, install the Rust toolchain using [rustup](https://rustup.rs/) or Homebrew and ensure `git` is available, then use the same `cargo build`, `cargo test`, and `cargo run` commands. GPG-positive signing tests skip when `gpg` is unavailable. The core diagnostic runner never performs Git writes in existing projects.

## Running diagnostics

```bash
cargo test --test acceptance -- --nocapture
cargo test --test core_behaviour -- --nocapture
cargo run --bin gitfleet-lab -- --output results.json
cargo run --bin gitfleet-lab -- --case partial_staging_commits_index_only
```

The tests automatically create disposable Git repositories and delete them. Crash tests deliberately terminate a **child** worker process with `SIGKILL`, then validate journal recovery in the parent. On SIGKILL the worker's temporary `.git/index.lock` is intentionally left behind to test recovery. If a test fails unexpectedly, inspect `stderr` and `results.json`.

## Architectural boundaries

`gitfleet_core` owns Git semantics, approval snapshots, journal, recovery and discovery. The desktop/Tauri shell is **not** included, and future UI adapters must call typed Rust services rather than implementing Git operations themselves. The POC does not implement production-quality authorization tokens, a hook sandbox, a trust store, or network push. See `docs/LIMITATIONS.md` before attempting any real integration.

## Files and full source listings

Each file below is included in the archive. Listing all source files in this README is intentional for portable review. The README itself is excluded from its own source listing to avoid infinite recursive embedding.

### `Cargo.toml`

Cargo manifest, dependencies and binary targets.

```toml
[package]
name = "gitfleet-core"
version = "0.1.1"
edition = "2021"
license = "MIT"
description = "Headless, experimental GitFleet controlled commit engine and disposable Git lab"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[[bin]]
name = "gitfleet-cli"
path = "src/bin/gitfleet_cli.rs"

[[bin]]
name = "gitfleet-lab"
path = "src/bin/gitfleet_lab.rs"
```

### `.gitignore`

Combined Rust, TypeScript/Node and Python ignore patterns.

```text
# Rust
target/
**/*.rs.bk
# TypeScript / Node
node_modules/
dist/
coverage/
# Python
.venv/
__pycache__/
*.pyc
.pytest_cache/
# Secrets and local outputs
.env
.env.*
!.env.example
results.json
*.transaction.json
*.scratch/
.DS_Store
```

### `LICENSE`

MIT licensing terms for this experimental POC.

```text
MIT License

Copyright (c) 2026 GitFleet contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### `src/lib.rs`

UI-independent Rust library entry point and modules.

```rust
//! GitFleet Core proof of concept. All write tests use disposable Git repositories.
//! Not a production-safe Git commit engine; see docs/LIMITATIONS.md.

pub mod commit;
pub mod discovery;
pub mod error;
pub mod git;
pub mod journal;
pub mod lab;

pub use error::{Error, Result};
```

### `src/error.rs`

Typed error and result aliases.

```rust
use std::fmt::{self, Display};

#[derive(Debug)]
pub struct Error(pub String);

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self { Self(e.to_string()) }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self { Self(e.to_string()) }
}

pub type Result<T> = std::result::Result<T, Error>;
```

### `src/git.rs`

Safe-argv Git CLI adapter, index parsing and status.

```rust
//! Narrow, argument-array-based Git CLI adapter. Never invoke a shell for Git operations.
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
        if key.to_string_lossy().starts_with("GIT_") { cmd.env_remove(key); }
    }
    cmd.env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", if cfg!(windows) { "NUL" } else { "/dev/null" })
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_NO_REPLACE_OBJECTS", "1");
    cmd
}

pub fn command(repo: &Path, args: &[&str], index: Option<&Path>, input: Option<&[u8]>) -> Result<Output> {
    let mut cmd = clean_command(OsStr::new("git"));
    cmd.current_dir(repo).args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(i) = index { cmd.env("GIT_INDEX_FILE", i); }
    if input.is_some() { cmd.stdin(Stdio::piped()); }
    let mut child = cmd.spawn()?;
    if let Some(data) = input { child.stdin.take().ok_or_else(|| Error::new("missing stdin"))?.write_all(data)?; }
    child.wait_with_output().map_err(Error::from)
}

pub fn run(repo: &Path, args: &[&str], index: Option<&Path>) -> Result<String> {
    run_input(repo, args, index, None)
}
pub fn run_input(repo: &Path, args: &[&str], index: Option<&Path>, input: Option<&[u8]>) -> Result<String> {
    let out = command(repo, args, index, input)?;
    if !out.status.success() {
        return Err(Error::new(format!("git {:?} exited {:?}: {}", args, out.status.code(), String::from_utf8_lossy(&out.stderr).trim())));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim_end_matches(|c| c == '\r' || c == '\n').to_owned())
}

pub fn index_path(repo: &Path) -> Result<PathBuf> {
    let path = PathBuf::from(run(repo, &["rev-parse", "--git-path", "index"], None)?);
    Ok(if path.is_absolute() { path } else { repo.join(path) })
}

pub fn current_ref(repo: &Path) -> Result<String> {
    run(repo, &["symbolic-ref", "-q", "HEAD"], None)
        .map_err(|_| Error::new("Detached HEAD is unsupported by the POC"))
}

pub fn entries(repo: &Path, index: Option<&Path>) -> Result<Entries> {
    let out = command(repo, &["ls-files", "--stage", "-z"], index, None)?;
    if !out.status.success() { return Err(Error::new(format!("ls-files failed: {}", String::from_utf8_lossy(&out.stderr)))); }
    let mut result = BTreeMap::new();
    for record in out.stdout.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let tab = record.iter().position(|b| *b == b'\t').ok_or_else(|| Error::new("Invalid index record"))?;
        let meta = std::str::from_utf8(&record[..tab]).map_err(|_| Error::new("Non-ASCII index metadata"))?;
        let path = std::str::from_utf8(&record[tab + 1..]).map_err(|_| Error::new("Non-UTF8 paths unsupported by POC"))?;
        let mut parts = meta.split(' ');
        let mode = parts.next().ok_or_else(|| Error::new("missing mode"))?;
        let oid = parts.next().ok_or_else(|| Error::new("missing oid"))?;
        let stage = parts.next().ok_or_else(|| Error::new("missing stage"))?;
        if stage != "0" { return Err(Error::new("Unmerged index entries unsupported")); }
        result.insert(path.to_owned(), Entry { mode: mode.to_owned(), oid: oid.to_owned() });
    }
    Ok(result)
}

pub fn validate_path(repo: &Path, path: &str) -> Result<()> {
    if path.is_empty() || path.contains('\0') || path.contains('\\') { return Err(Error::new("Invalid repository-relative path")); }
    let parsed = Path::new(path);
    if parsed.components().any(|c| !matches!(c, Component::Normal(_))) || path.split('/').any(|x| x == ".git" || x == ".." || x == "." || x.is_empty()) {
        return Err(Error::new(format!("Invalid repository-relative path: {path}")));
    }
    // Avoid symlinked ancestor directories escaping the repository. The leaf may itself be a symlink.
    let mut dir = repo.to_path_buf();
    let segments: Vec<_> = parsed.components().collect();
    for component in segments.iter().take(segments.len().saturating_sub(1)) {
        dir.push(component.as_os_str());
        if let Ok(m) = fs::symlink_metadata(&dir) {
            if m.file_type().is_symlink() { return Err(Error::new("Symlinked ancestor path is unsupported")); }
        }
    }
    Ok(())
}

pub fn apply_entry(repo: &Path, index: &Path, path: &str, entry: Option<&Entry>) -> Result<()> {
    validate_path(repo, path)?;
    match entry {
        Some(e) => { run(repo, &["update-index", "--add", "--cacheinfo", &format!("{},{},{}", e.mode, e.oid, path)], Some(index))?; }
        None => { run(repo, &["update-index", "--force-remove", "--", path], Some(index))?; }
    }
    Ok(())
}

pub fn hook_dir(repo: &Path) -> Result<PathBuf> {
    let override_hook = command(repo, &["config", "--get", "--path", "core.hooksPath"], None, None)?;
    let path = if override_hook.status.success() {
        PathBuf::from(String::from_utf8_lossy(&override_hook.stdout).trim())
    } else {
        PathBuf::from(run(repo, &["rev-parse", "--git-path", "hooks"], None)?)
    };
    Ok(if path.is_absolute() { path } else { repo.join(path) })
}

pub fn status(repo: &Path) -> Result<String> { run(repo, &["status", "--porcelain=v1", "--untracked-files=all"], None) }

pub fn canonical(repo: &Path) -> Result<PathBuf> { Ok(fs::canonicalize(repo)?) }

pub fn hash_bytes(repo: &Path, bytes: &[u8]) -> Result<String> { run_input(repo, &["hash-object", "--stdin"], None, Some(bytes)) }

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
```

### `src/discovery.rs`

Read-only nested Git repository discovery.

```rust
//! Read-only recursive repository discovery. Never follows directory symlinks.
use crate::git;
use crate::Result;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct Repository {
    pub path: PathBuf,
    pub git_dir: String,
    pub branch: Option<String>,
    pub status: String,
}

pub fn discover(root: &Path) -> Result<Vec<Repository>> {
    let root = fs::canonicalize(root)?;
    let mut found = Vec::new();
    let mut pending = vec![root];
    while let Some(dir) = pending.pop() {
        let marker = dir.join(".git");
        if marker.is_dir() || marker.is_file() {
            if let Ok(git_dir) = git::run(&dir, &["rev-parse", "--absolute-git-dir"], None) {
                found.push(Repository {
                    path: dir.clone(), git_dir,
                    branch: git::run(&dir, &["symbolic-ref", "-q", "--short", "HEAD"], None).ok(),
                    status: git::status(&dir).unwrap_or_else(|e| format!("status unavailable: {e}")),
                });
            }
        }
        let rd = match fs::read_dir(&dir) { Ok(rd) => rd, Err(_) => continue };
        for entry in rd.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if [".git", "node_modules", "target", ".venv", ".gitfleet-rust-lab"].contains(&name.as_ref()) { continue; }
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() && !ft.is_symlink() { pending.push(entry.path()); }
            }
        }
    }
    found.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(found)
}
```

### `src/journal.rs`

Versioned JSON transaction journal with atomic replacement.

```rust
//! Minimal durable journal. This is a research implementation, not a transactional database.
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    pub format_version: u32,
    pub state: String,
    pub repo: PathBuf,
    pub git_dir: PathBuf,
    pub git_ref: String,
    pub old_head: String,
    pub new_head: Option<String>,
    pub tree: String,
    pub message: String,
    pub selection: BTreeMap<String, String>,
    pub worktree_fingerprints: BTreeMap<String, String>,
    pub original_index_entries: BTreeMap<String, (String, String)>,
    pub reconciled_index_hash: String,
    pub scratch: PathBuf,
    pub hooks_executed: Vec<String>,
    pub hooks_enabled: bool,
    pub sign: bool,
    pub post_hook_invoked: bool,
    pub post_hook_result: Option<String>,
}

pub fn save(path: &Path, value: &Journal) -> Result<()> {
    fs::create_dir_all(path.parent().ok_or_else(|| Error::new("journal has no parent"))?)?;
    let tmp = path.with_extension("tmp");
    let bytes = serde_json::to_vec_pretty(value)?;
    let mut file = OpenOptions::new().create_new(true).write(true).open(&tmp)?;
    if let Err(e) = (|| -> std::io::Result<()> { file.write_all(&bytes)?; file.write_all(b"\n")?; file.sync_all() })() {
        let _ = fs::remove_file(&tmp);
        return Err(e.into());
    }
    drop(file);
    fs::rename(&tmp, path)?;
    sync_directory(path.parent().unwrap());
    Ok(())
}

pub fn load(path: &Path) -> Result<Journal> {
    let value: Journal = serde_json::from_slice(&fs::read(path)?)?;
    if value.format_version != 1 { return Err(Error::new("Unsupported journal version")); }
    Ok(value)
}

pub fn sync_directory(path: &Path) {
    if let Ok(f) = File::open(path) { let _ = f.sync_all(); }
}
```

### `src/commit.rs`

Approved selective tree, controlled hook execution, CAS ref update and recovery.

```rust
//! Controlled selective commit experiment. This is NOT an arbitrary-hook sandbox.
//! Supported: local branch, ordinary index (stage-0), UTF-8 paths, selected index/worktree content.
use crate::git::{self, Entries};
use crate::journal::{self, Journal};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Source { Index, Worktree }

#[derive(Debug, Clone)]
pub struct PlanRequest {
    pub selected: BTreeMap<String, Source>,
    pub message: String,
    pub hooks: bool,
    pub sign: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrashAt { BeforeRef, AfterRef, AfterIndex }

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
    pub fn from_status(s: &str) -> Self { Self { status: s.to_owned(), reason: None, new_head: None, post_hook: None } }
    pub fn blocked(s: &str, reason: impl Into<String>) -> Self {
        Self { status: s.to_owned(), reason: Some(reason.into()), new_head: None, post_hook: None }
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
            Ok(format!("symlink:{}", git::hash_bytes(repo, link.as_os_str().as_encoded_bytes())?))
        }
        Ok(meta) if meta.is_file() => {
            let data = fs::read(target)?;
            #[cfg(unix)]
            let executable = { use std::os::unix::fs::PermissionsExt; meta.permissions().mode() & 0o111 != 0 };
            #[cfg(not(unix))]
            let executable = false;
            Ok(format!("file:executable={}:{}", executable, git::hash_bytes(repo, &data)?))
        }
        Ok(_) => Err(Error::new(format!("Unsupported selected filesystem type: {path}"))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok("missing".into()),
        Err(e) => Err(e.into()),
    }
}
fn entry_snapshot(e: &Entries) -> BTreeMap<String, (String, String)> {
    e.iter().map(|(name, entry)| (name.clone(), (entry.mode.clone(), entry.oid.clone()))).collect()
}
fn snapshot_fingerprints(repo: &Path, j: &Journal) -> Result<bool> {
    for (path, expected) in &j.worktree_fingerprints {
        if fingerprint(repo, path)? != *expected { return Ok(false); }
    }
    Ok(true)
}
fn journal_index(scratch: &Path, name: &str) -> PathBuf { scratch.join(name) }

fn execute_hook(repo: &Path, index: &Path, script: &Path, args: &[&str]) -> Result<()> {
    let mut cmd = git::clean_command(script.as_os_str());
    cmd.current_dir(repo).args(args).env("GIT_INDEX_FILE", index).env("GIT_EDITOR", ":")
       .stdout(Stdio::piped()).stderr(Stdio::piped());
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(Error::new(format!("Hook {} exited {:?}: {}", script.display(), output.status.code(), String::from_utf8_lossy(&output.stderr).trim())));
    }
    Ok(())
}
fn run_hooks(repo: &Path, index: &Path, message: &Path, enabled: bool) -> Result<Vec<String>> {
    let mut invoked = Vec::new();
    if !enabled { return Ok(invoked); }
    let hookdir = git::hook_dir(repo)?;
    for name in ["pre-commit", "prepare-commit-msg", "commit-msg"] {
        let script = hookdir.join(name);
        if !script.is_file() || !is_executable(&script) { continue; }
        let path = script.canonicalize()?;
        let message_str = message.to_str().ok_or_else(|| Error::new("Non-UTF8 message file path"))?;
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
    fs::metadata(p).map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
}
#[cfg(not(unix))]
fn is_executable(p: &Path) -> bool { p.is_file() }

pub fn prepare(repo: &Path, journal_path: &Path, request: &PlanRequest) -> Result<Journal> {
    if journal_path.exists() { return Err(Error::new("Unresolved existing journal")); }
    if request.selected.is_empty() || request.message.trim().is_empty() { return Err(Error::new("Empty selection or message")); }
    for path in request.selected.keys() { git::validate_path(repo, path)?; }
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
                if !original.contains_key(path) && git::command(repo, &["cat-file", "-e", &format!("HEAD:{path}")], None, None)?.status.success() == false {
                    return Err(Error::new(format!("Neither staged nor tracked: {path}")));
                }
                git::apply_entry(repo, &selected_index, path, original.get(path))?;
            }
            Source::Worktree => {
                worktree_fingerprints.insert(path.clone(), fingerprint(repo, path)?);
                git::run(repo, &["add", "-A", "--", path], Some(&selected_index))?;
            }
        }
    }
    let tree = git::run(repo, &["write-tree"], Some(&selected_index))?;
    if tree == old_tree { return Err(Error::new("Selection produces empty commit")); }
    let message_file = scratch.join("message.txt");
    fs::write(&message_file, request.message.as_bytes())?;
    let executed_hooks = run_hooks(repo, &selected_index, &message_file, request.hooks)?;
    if tree != git::run(repo, &["write-tree"], Some(&selected_index))? || fs::read(&message_file)? != request.message.as_bytes() {
        return Err(Error::new("Hook changed approved tree or commit message; review required"));
    }
    for (p, before) in &worktree_fingerprints {
        if fingerprint(repo, p)? != *before { return Err(Error::new("Hook changed selected working-tree content; review required")); }
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
        format_version: 1, state: "prepared".into(), repo: canonical,
        git_dir, git_ref, old_head, new_head: None, tree,
        message: request.message.clone(),
        selection: request.selected.iter().map(|(p, s)| (p.clone(), match s {Source::Index => "index", Source::Worktree => "worktree"}.to_owned())).collect(),
        worktree_fingerprints, original_index_entries: entry_snapshot(&original),
        reconciled_index_hash: reconciled_hash, scratch,
        hooks_executed: executed_hooks, hooks_enabled: request.hooks, sign: request.sign,
        post_hook_invoked: false, post_hook_result: None,
    };
    journal::save(journal_path, &journal)?;
    Ok(journal)
}

fn current_entries(repo: &Path) -> Result<BTreeMap<String, (String, String)>> { Ok(entry_snapshot(&git::entries(repo, None)?)) }
fn unchanged(repo: &Path, j: &Journal) -> Result<Option<Outcome>> {
    if git::run(repo, &["rev-parse", &j.git_ref], None)? != j.old_head {
        return Ok(Some(Outcome::blocked("blocked_stale_head", "HEAD differs from reviewed state")));
    }
    if current_entries(repo)? != j.original_index_entries {
        return Ok(Some(Outcome::blocked("blocked_external_index_change", "Git index changed after review")));
    }
    if !snapshot_fingerprints(repo, j)? {
        return Ok(Some(Outcome::blocked("blocked_selected_worktree_change", "Selected working-tree content changed")));
    }
    Ok(None)
}
fn owned_lock(repo: &Path, j: &Journal, lock: &Path) -> Result<bool> {
    if !lock.exists() { return Ok(false); }
    Ok(git::hash_bytes(repo, &fs::read(lock)?)? == j.reconciled_index_hash)
}
fn lock_path(repo: &Path) -> Result<PathBuf> {
    let index = git::index_path(repo)?;
    Ok(index.with_file_name(format!("{}.lock", index.file_name().unwrap_or(OsStr::new("index")).to_string_lossy())))
}
fn finish(repo: &Path, journal_path: &Path, j: &mut Journal) -> Result<Outcome> {
    let index_path = git::index_path(repo)?;
    let lock = lock_path(repo)?;
    if !owned_lock(repo, j, &lock)? || current_entries(repo)? != j.original_index_entries {
        return Ok(Outcome::blocked("needs_manual_reconciliation", "Index lock missing/foreign or current index differs"));
    }
    fs::rename(&lock, &index_path)?;
    journal::sync_directory(index_path.parent().unwrap());
    j.state = "index_promoted".into();
    journal::save(journal_path, j)?;
    finalize(repo, journal_path, j)
}
fn finalize(repo: &Path, path: &Path, j: &mut Journal) -> Result<Outcome> {
    let new = j.new_head.clone().ok_or_else(|| Error::new("missing new commit"))?;
    if git::run(repo, &["rev-parse", &j.git_ref], None)? != new {
        return Ok(Outcome::blocked("needs_manual_reconciliation", "Branch moved after commit"));
    }
    if git::entries(repo, None)? != git::entries(repo, Some(&j.scratch.join("reconciled.index")))? {
        return Ok(Outcome::blocked("needs_manual_reconciliation", "Index differs from approved reconciled index"));
    }
    if git::run(repo, &["rev-parse", &format!("{new}^{{tree}}")], None)? != j.tree {
        return Ok(Outcome::blocked("needs_manual_reconciliation", "Committed tree differs from approved tree"));
    }
    if git::run(repo, &["show", "-s", "--format=%B", &new], None)?.trim() != j.message.trim() {
        return Ok(Outcome::blocked("needs_manual_reconciliation", "Committed message differs from approved message"));
    }
    let mut post = None;
    if j.hooks_enabled && !j.post_hook_invoked {
        let script = git::hook_dir(repo)?.join("post-commit");
        if script.is_file() && is_executable(&script) {
            j.post_hook_invoked = true;
            journal::save(path, j)?;
            let out = git::clean_command(script.canonicalize()?.as_os_str()).current_dir(repo).output()?;
            post = Some(format!("exit={:?}, stderr={}", out.status.code(), String::from_utf8_lossy(&out.stderr).trim()));
        }
    }
    j.post_hook_result = post.clone();
    j.state = "complete".into();
    journal::save(path, j)?;
    Ok(Outcome { status: "committed".into(), reason: None, new_head: Some(new), post_hook: post })
}

pub fn execute(repo: &Path, journal_path: &Path, crash: Option<CrashAt>) -> Result<Outcome> {
    let mut j = journal::load(journal_path)?;
    validate_identity(repo, &j)?;
    if j.state != "prepared" { return Err(Error::new(format!("Journal not prepared: {}", j.state))); }
    if let Some(out) = unchanged(repo, &j)? { return Ok(out); }
    let mut args = vec!["commit-tree", &j.tree, "-p", &j.old_head];
    if j.sign { args.push("-S"); }
    let out = git::command(repo, &args, None, Some(j.message.as_bytes()))?;
    if !out.status.success() {
        return Ok(Outcome::blocked("blocked_commit_tree_failure", String::from_utf8_lossy(&out.stderr).trim().to_owned()));
    }
    let new_oid = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    j.new_head = Some(new_oid.clone());
    j.state = "commit_object_created".into();
    journal::save(journal_path, &j)?;
    let lock = lock_path(repo)?;
    let mut lock_file = match OpenOptions::new().write(true).create_new(true).open(&lock) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => return Ok(Outcome::blocked("blocked_index_locked", "Git index is locked")),
        Err(e) => return Err(e.into()),
    };
    lock_file.write_all(&fs::read(j.scratch.join("reconciled.index"))?)?;
    lock_file.sync_all()?;
    drop(lock_file);
    if let Some(out) = unchanged(repo, &j)? {
        let _ = fs::remove_file(lock);
        return Ok(out);
    }
    j.state = "ref_update_pending".into();
    journal::save(journal_path, &j)?;
    if crash == Some(CrashAt::BeforeRef) { kill_self(); }
    let update = git::command(repo, &["update-ref", "-m", "GitFleet controlled POC", &j.git_ref, &new_oid, &j.old_head], None, None)?;
    if !update.status.success() {
        let _ = fs::remove_file(lock);
        j.state = "ref_rejected".into();
        journal::save(journal_path, &j)?;
        return Ok(Outcome::blocked("blocked_stale_head", String::from_utf8_lossy(&update.stderr).to_string()));
    }
    if crash == Some(CrashAt::AfterRef) { kill_self(); }
    j.state = "ref_updated_pending_index".into();
    journal::save(journal_path, &j)?;
    let out = finish(repo, journal_path, &mut j)?;
    if crash == Some(CrashAt::AfterIndex) { kill_self(); }
    Ok(out)
}

pub fn recover(repo: &Path, journal_path: &Path) -> Result<Outcome> {
    let mut j = journal::load(journal_path)?;
    validate_identity(repo, &j)?;
    if j.state == "complete" { return Ok(Outcome { status: "already_complete".into(), reason: None, new_head: j.new_head, post_hook: j.post_hook_result }); }
    if j.state == "prepared" { return Ok(Outcome::from_status("not_started")); }
    let actual = git::run(repo, &["rev-parse", &j.git_ref], None)?;
    let lock = lock_path(repo)?;
    if actual == j.old_head {
        if lock.exists() {
            if !owned_lock(repo, &j, &lock)? { return Ok(Outcome::blocked("needs_manual_reconciliation", "Foreign Git index lock")); }
            fs::remove_file(lock)?;
        }
        j.state = "aborted_before_ref".into();
        journal::save(journal_path, &j)?;
        return Ok(Outcome::from_status("aborted_before_ref"));
    }
    if j.new_head.as_deref() != Some(actual.as_str()) {
        return Ok(Outcome::blocked("needs_manual_reconciliation", "Branch advanced to an unexpected commit"));
    }
    if j.state == "index_promoted" { return finalize(repo, journal_path, &mut j); }
    if lock.exists() { return finish(repo, journal_path, &mut j); }
    if git::entries(repo, None)? == git::entries(repo, Some(&j.scratch.join("reconciled.index")))? {
        j.state = "index_promoted".into();
        journal::save(journal_path, &j)?;
        return finalize(repo, journal_path, &mut j);
    }
    Ok(Outcome::blocked("needs_manual_reconciliation", "Index lock missing and index differs; manual review required"))
}

#[cfg(unix)]
fn kill_self() {
    unsafe extern "C" { fn kill(pid: i32, sig: i32) -> i32; fn getpid() -> i32; }
    unsafe { kill(getpid(), 9); }
    std::process::exit(137);
}
#[cfg(not(unix))]
fn kill_self() { std::process::exit(137); }
```

### `src/lab.rs`

Disposable Git fixtures and 24 automated v4-derived scenarios.

```rust
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
```

### `src/bin/gitfleet_cli.rs`

Read-only discovery and status CLI.

```rust
//! Read-only diagnostic CLI; intentionally does not expose commit execution.
use gitfleet_core::{discovery, git, Error, Result};
use std::path::Path;
fn main() {
    if let Err(e) = run() { eprintln!("gitfleet-cli: {e}"); std::process::exit(1); }
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (verb, location) = match args.as_slice() {
        [verb, location] => (verb.as_str(), location.as_str()),
        _ => return Err(Error::new("Usage: gitfleet-cli discover <root> | status <repo>")),
    };
    let path = Path::new(location);
    let result = match verb {
        "discover" => serde_json::to_value(discovery::discover(path)?)?,
        "status" => git::log_state(path)?,
        _ => return Err(Error::new("Unsupported command; use discover or status")),
    };
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
```

### `src/bin/gitfleet_lab.rs`

Disposable lab diagnostics, SIGKILL worker and JSON reporting.

```rust
//! GitFleet disposable Git integration lab; no writes to user-supplied repositories.
use gitfleet_core::{commit::{self, CrashAt}, lab, Error, Result};
use std::path::Path;
fn main() {
    if let Err(e) = run() { eprintln!("gitfleet-lab: {e}"); std::process::exit(1); }
}
fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [cmd, case] if cmd == "--case" => {
            let result = lab::run_case(case)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        [cmd, repo, journal, phase] if cmd == "--worker" => {
            let repo = Path::new(repo);
            lab::ensure_lab_fixture(repo)?;
            let journal_path = Path::new(journal);
            if journal_path.parent() != repo.parent() { return Err(Error::new("Journal must be in lab fixture root")); }
            let crash = match phase.as_str() {
                "before_ref" => CrashAt::BeforeRef, "after_ref" => CrashAt::AfterRef,
                "after_index" => CrashAt::AfterIndex,
                _ => return Err(Error::new("Unknown crash phase")),
            };
            let out = commit::execute(repo, journal_path, Some(crash))?;
            println!("{}",serde_json::to_string_pretty(&out)?);
        }
        [cmd, output] if cmd == "--output" => {
            let report = lab::run_suite()?;
            let bytes = serde_json::to_vec_pretty(&report)?;
            std::fs::write(output, &bytes)?;
            println!("{}", String::from_utf8_lossy(&bytes));
        }
        [] => { println!("{}", serde_json::to_string_pretty(&lab::run_suite()?)?); }
        _ => return Err(Error::new("Usage: gitfleet-lab [--output results.json] | --case <name>")),
    }
    Ok(())
}
```

### `tests/acceptance.rs`

24 process-isolated acceptance tests.

```rust
//! One subprocess per scenario keeps test configurations and signing keys isolated.
use std::process::Command;

fn scenario(name: &str) {
    let result = Command::new(env!("CARGO_BIN_EXE_gitfleet-lab"))
        .arg("--case").arg(name).output().expect("run disposable lab");
    assert!(result.status.success(), "{name}: {}", String::from_utf8_lossy(&result.stderr));
    let response: serde_json::Value = serde_json::from_slice(&result.stdout).expect("valid JSON results");
    assert!(response["status"].is_string(), "{name}: missing status");
}

macro_rules! test_case {
    ($($name:ident),+ $(,)?) => { $(#[test] fn $name() { scenario(stringify!($name)); })+ }
}

test_case!(
    selected_staged_preserves_other_staging,
    selected_worktree_preserves_other_staging,
    partial_staging_commits_index_only,
    partial_staging_commits_worktree,
    mixed_selected_sources,
    untracked_selected,
    selected_unstaged_deletion,
    selected_rename_pair,
    space_in_path,
    hook_modifies_only_worktree,
    hook_stages_unapproved,
    hook_rewrites_message,
    post_commit_hook,
    hook_changes_selected_worktree,
    prepare_hook_rewrites_message,
    selected_worktree_changed_after_review,
    external_index_changed,
    stale_head_before_cas,
    real_sigkill_before_ref,
    real_sigkill_after_ref,
    real_sigkill_after_reconcile,
    crash_then_external_index_edit,
    signing_missing_key,
    successful_gpg_signing,
);
```

### `tests/core_behaviour.rs`

Additional read-only discovery and path-validation tests.

```rust
use gitfleet_core::{discovery, git, lab::Fixture};
use std::fs;

#[test]
fn discover_nested_repositories_and_gitfile_marker() {
    let f = Fixture::new().unwrap();
    let nested = f.repo.join("nested");
    fs::create_dir_all(&nested).unwrap();
    git::run(&nested, &["init", "-q"], None).unwrap();
    let found = discovery::discover(&f.repo).unwrap();
    assert_eq!(found.len(), 2);
    // Discovery returns canonical paths. On macOS /var typically resolves to /private/var.
    let expected_root = fs::canonicalize(&f.repo).unwrap();
    let expected_nested = fs::canonicalize(&nested).unwrap();
    assert!(found.iter().any(|r| r.path == expected_root));
    assert!(found.iter().any(|r| r.path == expected_nested));
}

#[cfg(unix)]
#[test]
fn discovery_canonicalizes_symlinked_root() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new().unwrap();
    let alias = f.root.join("repository-alias");
    symlink(&f.repo, &alias).unwrap();
    let found = discovery::discover(&alias).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, fs::canonicalize(&f.repo).unwrap());
}

#[test]
fn discover_linked_worktree_with_git_file() {
    let f = Fixture::new().unwrap();
    let linked = f.repo.join("linked-worktree");
    let linked_arg = linked.to_str().unwrap();
    git::run(&f.repo, &["worktree", "add", "--detach", linked_arg], None).unwrap();
    assert!(linked.join(".git").is_file());
    let found = discovery::discover(&f.repo).unwrap();
    assert_eq!(found.len(), 2);
    assert!(found.iter().any(|r| r.path == fs::canonicalize(&f.repo).unwrap()));
    assert!(found.iter().any(|r| r.path == fs::canonicalize(&linked).unwrap()));
}

#[test]
fn prevent_parent_path_escape() {
    let f = Fixture::new().unwrap();
    assert!(git::validate_path(&f.repo, "../outside.txt").is_err());
    assert!(git::validate_path(&f.repo, "/tmp/outside.txt").is_err());
    assert!(git::validate_path(&f.repo, ".git/config").is_err());
    assert!(git::validate_path(&f.repo, "src//nested.rs").is_err());
    assert!(git::validate_path(&f.repo, "valid file.rs").is_ok());
}

#[cfg(unix)]
#[test]
fn discovery_does_not_follow_symlinked_directories() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new().unwrap();
    symlink(&f.root, f.repo.join("loop")).unwrap();
    let found = discovery::discover(&f.repo).unwrap();
    assert_eq!(found.len(), 1);
}

#[test]
fn read_only_status_is_available() {
    let f = Fixture::new().unwrap();
    f.stage("full.txt", "staged\n").unwrap();
    let state = git::log_state(&f.repo).unwrap();
    assert!(state["status"].as_str().unwrap().contains("M  full.txt"));
}
```

### `docs/ARCHITECTURE.md`

Core layer boundaries and controlled commit lifecycle.

```markdown
# GitFleet Rust Core POC Architecture

## Layer boundaries

- `gitfleet_core::git`: narrow Git CLI adapter (`Command`, argv only, no shell), index inspection, path validation, read-only repository state.
- `gitfleet_core::commit`: controlled selective commit planning, isolated indexes, hook invocation, signing via Git, compare-and-swap ref updates, guarded index reconciliation.
- `gitfleet_core::journal`: versioned, fsync-backed journal writes and restart reconciliation.
- `gitfleet_core::discovery`: read-only nested repository discovery, skipping symlinks and common high-volume directories.
- `gitfleet_core::lab`: disposable test fixtures and v4-compatible behavioral scenarios.
- `gitfleet-cli`: read-only adapter (`discover`, `status`).
- `gitfleet-lab`: test-only write adapter, restricted to disposable fixture repositories.

The core contains no Tauri, React, browser, or HTTP server dependencies. A future `gitfleet-tauri` adapter can call service interfaces over typed requests. A web API would be a separate trusted boundary.

## Commit pipeline

1. Record current branch OID, logical index entries, and selected working-tree fingerprints.
2. Create a scratch index starting at `HEAD`, overlay precisely selected index/worktree paths.
3. Compute approved tree and message. Invoke supported validation hooks against scratch index.
4. Reject hooks changing the approved tree, message, or selected working-tree content.
5. Build a reconciled index containing approved selected entries plus untouched other staged entries.
6. Persist journal, then on execution revalidate branch/index/selected worktree.
7. `git commit-tree` creates the immutable commit object. This may leave an orphan if later checks fail.
8. Write an owned `index.lock` containing reconciled index, revalidate state, then use `git update-ref` with expected old HEAD.
9. Promote index lock to real index, fsync, verify commit tree and message, attempt post-commit hook, finish journal.
10. Recovery examines current ref, owned lock, actual index and journal; it never replays a successful commit.

## Authorization and threat boundary

This POC tests a reviewed tree/message boundary, but does **not** implement a production approval token or trust store. The test harness represents a trusted actor. Journal JSON is not signed and must not be accepted from a hostile client. Hooks are executable local programs, not sandboxed. See `LIMITATIONS.md`.

## Why not run ordinary `git commit`?

Normal Git commit hooks may mutate the index or message *after* user review. GitFleet's manually controlled pipeline examines hook effects before creating a commit object. It intentionally supports only a subset of Git's native lifecycle until compatibility is demonstrated.

## Rust production evolution

Before exposing write actions to Tauri, implement a typed request/authorization protocol, privileged service checks, durable operation IDs in SQLite, hook compatibility policy, index-format validation, portable path bytes, locking and race stress tests, plus explicit repository trust.
```

### `docs/LIMITATIONS.md`

Explicit unsupported configurations and production release gates.

```markdown
# Known Limitations and Release Gates

**This is a controlled lab, not a production-safe commit engine. Never use it against live projects.**

1. **Cargo execution not verified by artifact authoring environment.** Rust toolchain is not installed there. Source structure and reference Python 24-case suite were checked; Rust build and tests must run on a machine with Cargo.
2. **Manual Git hook subset.** Invokes `pre-commit`, `prepare-commit-msg`, `commit-msg`, and `post-commit`. Does not reproduce all native Git lifecycle context, `pre-merge-commit`, `post-rewrite`, hook manager semantics, or hook invocation from another process.
3. **Executable hooks are not sandboxed.** A malicious or defective hook can modify arbitrary accessible paths, spawn processes, or contact the network. Repo trust and hook policy must gate production writes.
4. **Local branch only.** Detached HEAD and unborn repositories are unsupported for controlled commits. Merge/rebase sequencer states and unmerged index entries are not supported.
5. **Ordinary index only.** Split index, sparse index, intent-to-add, skip-worktree, special extensions, index v4, unusual file mode filters, multiple alternate indexes, and non-UTF8 filenames require dedicated tests or explicit rejection.
6. **Concurrent external Git processes.** Index lock and branch compare-and-swap mitigate but do not eliminate all races, especially processes that ignore Git's locks or act between validation and ref update.
7. **No atomic branch + index update.** Durable journal and recovery cover tested SIGKILL windows but not all filesystem, write-reordering, corruption, or cross-volume failures.
8. **Journal authorization is not authenticated.** Test data are trusted; tampered journals may redirect operations. Production must use opaque, backend-owned approval identities and a durable operation database.
9. **Signing.** GPG positive case runs only when GPG is installed; its generated key is isolated in the disposable fixture. SSH signing is not tested. Key-agent and `gpg.format` behavior remain to be tested.
10. **Post-commit hook.** An at-most-once attempt is journaled. A crash between setting attempted and actual execution may suppress the hook; there is no exactly-once delivery guarantee.
11. **No GitFleet Actions or AI.** Future deterministic SemVer updates, AI templates, profiles, and bulk authorization will be separate application services.
12. **Discovery is intentionally narrow.** Ignores directory symlinks and common directories; doesn't classify linked worktrees/submodules or implement a watcher.
13. **No production write CLI.** `gitfleet-cli` exposes only read-only commands. `gitfleet-lab` is explicitly disposable-only.
14. **Path handling.** Selected paths must be UTF-8 and cannot have symlinked ancestor directories; non-UTF8 paths must be supported or explicitly blocked in production.
15. **No remote operations.** No fetch/pull/push, dependency-aware bulk commit/push, or credential handling in POC.
16. **Recovery stop policy.** If unexpected index/ref state occurs, the lab stops with `needs_manual_reconciliation`; it never attempts destructive cleanup.

## Hard release gates

- Compile and run the Rust tests on macOS and Linux.
- Reconcile outcomes with the v4 Python baseline and investigate every discrepancy.
- Add index/worktree/HEAD stress and fuzzer tests; validate all supported index features.
- Implement production repository trust, scoped authorizations, and immutable approved snapshots.
- Test real-world hooks, GPG/SSH signing, linked worktrees, submodules, crash/restart and power-loss-like faults.
- Define branch, lock, journal, and index cleanup contracts before any UI enables live Git writes.
```

### `docs/TEST_MATRIX.md`

Expected scenarios and behavioral invariants.

```markdown
# Behavioral Test Matrix

This lab translates 24 cases from GitFleet Hook Lab v4 into a Rust diagnostic harness. In addition, `tests/core_behaviour.rs` covers discovery and path safety.

| Category | Scenarios |
|---|---|
| Selected staged/worktree | `selected_staged_preserves_other_staging`, `selected_worktree_preserves_other_staging`, `mixed_selected_sources` |
| Partial staging | `partial_staging_commits_index_only`, `partial_staging_commits_worktree` |
| Path kinds | `untracked_selected`, `selected_unstaged_deletion`, `selected_rename_pair`, `space_in_path` |
| Hooks | `hook_modifies_only_worktree`, `hook_stages_unapproved`, `hook_rewrites_message`, `post_commit_hook`, `hook_changes_selected_worktree`, `prepare_hook_rewrites_message` |
| Races | `selected_worktree_changed_after_review`, `external_index_changed`, `stale_head_before_cas` |
| True termination | `real_sigkill_before_ref`, `real_sigkill_after_ref`, `real_sigkill_after_reconcile`, `crash_then_external_index_edit` |
| Signing | `signing_missing_key`, `successful_gpg_signing` (skips if GPG missing) |

Each scenario runs in a fresh OS temporary directory, initializes a Git repo, executes the controlled pipeline, checks expected outcome and core invariants, then deletes the fixture.

`cargo test --workspace` runs one subprocess per case. `gitfleet-lab --output results.json` runs the same suite and emits a structured JSON report suitable for comparing to v4.

## Intended invariants

- A created commit has exactly the approved tree and message.
- A failed pre-ref operation does not advance the branch.
- Unrelated staged/unstaged/untracked contents remain represented correctly.
- Controlled recovery does not silently rerun an already published commit.
- Hook mutations outside approved tree/message cause a block before commit creation.
- External index/ref edits are not silently overwritten.

## Repository discovery regression tests (0.1.1)

- Root and nested paths are compared after `fs::canonicalize`, because macOS may expose temporary folders using `/var/...` while the canonical path is `/private/var/...`.
- A symlinked workspace-root alias returns the canonical repository path.
- A linked worktree with a `.git` **file** is discovered alongside the parent repository.

These tests are read-only except for creating disposable local Git fixtures.
```

### `scripts/verify-scaffold.py`

Static scaffold validation, not a Rust compiler or test runner.

```python
#!/usr/bin/env python3
"""Static verification only; does not compile Rust or assert test execution."""
from pathlib import Path
import re
import sys
try:
    import tomllib
except ImportError:
    print('Python 3 is required for static verification')
    sys.exit(2)

root = Path(__file__).resolve().parents[1]
manifest = tomllib.loads((root/'Cargo.toml').read_text())
assert manifest['package']['name'] == 'gitfleet-core'
required = [
    'Cargo.toml', '.gitignore', 'README.md',
    'src/lib.rs', 'src/error.rs', 'src/git.rs', 'src/discovery.rs',
    'src/journal.rs', 'src/commit.rs', 'src/lab.rs',
    'src/bin/gitfleet_cli.rs', 'src/bin/gitfleet_lab.rs',
    'tests/acceptance.rs', 'tests/core_behaviour.rs',
    'docs/ARCHITECTURE.md', 'docs/LIMITATIONS.md', 'docs/TEST_MATRIX.md'
]
assert all((root/p).is_file() for p in required)
text = (root/'tests/acceptance.rs').read_text()
assert len(re.findall(r'^    [a-z_]+,?$', text, flags=re.MULTILINE)) >= 24
assert not any('tauri' in x.name.lower() for x in root.rglob('Cargo.toml'))
print('Static manifest/scaffold checks passed. Rust build/tests: NOT verified by this script.')
```

## Possible enhancements and improvements

1. **Critical:** Run Cargo on macOS/Linux and correct any compilation errors or behavioral discrepancies against Python Hook Lab v4. Add a CI matrix and check in `Cargo.lock` after validation.
2. **Critical:** Replace journal-as-authorization with backend-owned, immutable approval records and repository trust checks. Prevent untrusted clients from injecting plans.
3. **Critical:** Prove durability and concurrency with fuzzing, lock-injection, fault-injection, and power-loss simulation. Harden crash-safe journal writes.
4. **High:** Expand hook compatibility tests to common managers and Git hook lifecycle variants; explicitly disable unsupported hook semantics.
5. **High:** Test and either support or reject linked worktrees, submodules, split/sparse indexes, merge states, symlink edge cases and non-UTF8 paths.
6. **High:** Implement signed-commit compatibility for GPG and SSH, with noninteractive credential handling and valid signature verification.
7. **Medium:** Extract a stable application-service API and durable SQLite operation history; add a strictly read-only monitoring layer.
8. **Later:** Add the Tauri/React adapter, workspace table/graph, AI profiles, deterministic GitFleet Actions, and dependency-aware bulk operations.

Workspace discovery now offers in-process continuation after a bounded partial scan. `workspace scan-complete WORKSPACE_ID MAX_SLICES` explicitly runs 1–100 bounded slices in one CLI process. Desktop Continue discovery resumes the same in-memory cursor; restart requires fresh Rescan. Saved incomplete inventory never restores a live cursor or authority.
