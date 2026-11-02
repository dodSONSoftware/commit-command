# M0 service and safety boundaries

Date: 2026-11-01. Historical M0 boundary. M1 was subsequently authorized and
implemented; see [M1 contracts](M1_READ_ONLY_WORKSPACES.md) for current inspection
capabilities and limits. The proposed production-write contracts here remain unimplemented.

## Implemented entry points

`inspection::snapshot(worktree_root)` and `discovery::discover(root)` are the
normal services. `gitfleet-cli status|discover` calls only those services. The
snapshot is deliberately diagnostic: optional HEAD OID, optional branch name and
Git-quoted porcelain v1 text. It is not the M1 repository model. The CLI no longer
returns the lab-only `entries`, `head_tree` or `index_tree` fields. In particular,
normal inspection never invokes `write-tree`, `update-index`, `add`, commit or
reference-update commands.

A private query enum restricts the process adapter to effective config reading,
root/git-directory resolution, branch/HEAD lookup and status. No arbitrary argv,
shell or executable path is exposed by this service. Default Cargo features are
empty. `git`, `commit`, `journal`, `lab`, the lab binary, and legacy write tests
require `experimental-writes`. This opt-in preserves the experiment, not a
production write capability. No additional crates or UI dependencies were added.

## Untrusted inspection process policy

- Remove inherited `GIT_*` variables before applying the inspection environment.
- Disable system/global Git config, prompting, optional locks, replacement objects,
  lazy object fetch, automatic maintenance/GC, and every Git transport protocol.
- Override fsmonitor (external command or builtin daemon) and untracked cache off;
  disable hooks through empty `core.hooksPath`. Do not launch a pager.
- Ignore submodule detail/recursion. No nested Git process is needed to inspect a
  parent's status; discovered child repositories are inspected separately.
- Read effective local, included and per-worktree configuration. Reject every
  configured `filter.<driver>.clean`, `.smudge` or `.process`, even an empty or
  currently unused one, before running status. Status can hash files through clean
  filters; “read-only Git command” alone is not an execution security boundary.
- External diff/textconv, SSH, credential, signing and other hooks are not needed
  by the allowed commands. Sentinel fixtures configure these alongside fsmonitor
  and verify they do not run. Future history/diff services need their own policy.
- Do not forward repository-controlled Git stderr to inspection clients. Typed
  failures identify invalid roots, unsupported config, inspection failure, invalid
  data or host I/O. Diagnostics must not dump effective config values.

Inspection requires a worktree root, allowing canonical root aliases. It supports
unborn and detached HEAD and unmerged status without requiring a writable or fully
merged index. Porcelain filename quoting is preserved: callers must not split it
into a domain model using whitespace or newlines. Non-UTF-8 metadata/root paths
are rejected. Newline filenames are tested; the macOS fixture filesystem rejects
non-UTF-8 filename creation, so that test is explicitly ignored there.

Repository snapshots in tests include all worktree/.git file bytes, directory
entries, modes and symlink targets, including objects, index, refs and locks.
They intentionally exclude access timestamps. Missing promised objects fail
inspection without invoking an external transport helper or creating objects.
Global/inherited overrides cannot redirect the CLI to a second fixture.

## Supported envelope and explicit limitations

The validated environment is recorded in implementation status: macOS arm64 with
Git 2.56.0. The Linux/macOS workflow is defined locally but has not run remotely.
Other Git versions and Windows are unvalidated; no minimum version is claimed.
Git itself, PATH, host environment, operating system and filesystem are trusted.
This is not an OS sandbox for malicious Git binaries or native Git vulnerabilities.

Content-filter repos are explicitly unsupported for untrusted inspection. Detailed
submodule state, bare repositories, non-UTF-8 root/metadata paths, alternate index
views, and full status/history/diff domain contracts are outside M0. Split/sparse
indexes, SHA-256, unusual filesystems and platform combinations are untested rather
than implicitly supported. Partial clones with missing required objects can fail;
inspection will not fetch them.

Local config includes and `.git` indirection are read, not sandboxed. Concurrent
hostile replacement of config, attributes, git directories or symlink ancestors
between checks/commands is unsupported: config rejection and execution are not
atomic. Do not claim this policy defeats a malicious concurrent local actor.
Stable repository metadata/configuration is a prerequisite; identity/TOCTOU design
remains a gate before broad production use. Object alternates/config includes may
read outside the selected directory; M0 is not a filesystem confidentiality sandbox.

The process API is synchronous with captured output; timeout/output limits and
cancellation remain future service work. FIFO/device metadata, hostile permission
changes and unreadable filesystem trees can fail or block; Git errors are surfaced.
Discovery now propagates read/entry/inspection failures rather than returning a
partial apparently successful inventory. Persisted unavailable identities and
per-repository error inventory are M1 work.

## Proposed production contracts (not implemented)

| Service | Inputs and authority | Required result/invariant | Gate |
|---|---|---|---|
| Identity/inspection | Chosen workspace/root; no trust implied | Separate repository/common-directory/worktree identities; canonical identity plus display paths; explicit unavailable/unsupported state | M1 |
| Trust | Explicit selected identities, current ownership/metadata binding | Revocable per-repo/worktree grants; no child inheritance; moves/replacements require conservative re-evaluation | M1 design / M2 enforcement |
| Proposal | Trusted identity, literal paths/content sources, message, hooks/signing/Actions policy | Readable exact tree/blob proposal with snapshot fingerprints; capture itself never executes hooks or Actions | M2 |
| Approval | Explicit review of a core-owned proposal | Opaque backend-issued operation authorization bound to identity, symbolic HEAD, expected OID, exact tree/message bytes, selected sources, index flags/fingerprints and policies; frontend JSON grants no authority | M2 |
| Execution | Core-owned authorization; fresh identity/state validation | Reject stale/expired/revoked or unsupported plans before side effects; persist phases; controlled hooks; CAS publication; conflict-safe reconciliation; accurate final/post-hook outcome | M2 |
| Recovery | Backend-owned operation ID plus actual Git state | Classify unpublished/published/reconciled/ambiguous; never replay uncertain writes; guarded lock cleanup; manual reconciliation for conflicts; no caller-directed arbitrary journal paths | M2 |

Approval lifetime, crash durability, supported Git/index/hook configurations and
signing policy must be decided/tested before these contracts become executable.
Core must own all checks independently of CLI/Tauri/React. No generic command IPC
or UI-owned trust rule is proposed.

## Remaining experimental write blockers

Same-OID branch switching and literal-path expansion are corrected in the lab.
Nonzero hook rejection has a typed category and exact state checks. Journal phases
are typed with unchanged v0.1.1 serialized names; unknown phases fail closed.
These changes do not resolve the mutable-journal authority problem, scratch index
integrity before publication, durable directory sync/cleanup, index extensions,
unsupported-state preflight, reference-transaction hooks and complete bypass,
shared-worktree concurrency, configured signing, full message-byte enforcement,
post-hook revalidation or power-loss recovery. See the preserved POC limitations.

## Evidence and automation

`sh app/core/scripts/validate.sh` runs default and opt-in builds/tests/Clippy,
format checks, static layout checks and the original diagnostic scenarios. Default
output goes to ignored `app/core/target/m0-evidence/`. An optional first argument
selects an evidence directory relative to `app/core/` (or an absolute directory).
The script never points write fixtures at a user-selected repository.

The GitHub workflow defines macOS/Linux jobs and uploads fixture-only diagnostic
metadata if executed. It has not been pushed or run. Its stable Rust/runner labels
are validation targets, not a pinned support contract; actual versions are saved
in each run. Git/Rust minimums and release-platform promises remain open.
