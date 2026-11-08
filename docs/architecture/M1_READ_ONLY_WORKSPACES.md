# M1 — Read-only workspace services

Date: 2026-11-01. Authorized after M0; implemented in the existing Core crate.
M2 and later remain outside scope. See current implementation status for evidence.

## Implemented contracts and module ownership

| Module | Owns |
|---|---|
| `domain` | Workspace/repository/worktree IDs, tagged lossless paths, object-format-aware OIDs, issues |
| `process` (private) | Restricted service transport, clean environment, four-process limit, output/time/cancellation bounds |
| `inspection` | Metadata binding, per-worktree serialization, consistency fingerprints, structured status/ref DTOs |
| `discovery` | Explicit rescans, exclusions, partial inventories and relationship classification |
| `workspace` | TOML definitions, SQLite identities/membership/cache, active workspace, startup reconciliation |
| `history` | Immutable commit metadata, actual parent edges, refs, bounded paging, shallow boundaries |
| `diff` | Staged/unstaged/historical comparisons, literal path scope, blobs and special-file previews |
| CLI | Native argument parsing and JSON transport only |

Default builds retain no experimental commit/journal/low-level Git/lab module.
`experimental-writes` remains non-default and disposable-only. No Tauri/React,
watcher, desktop shell, trust grant, approval executor, remote operation, graph
layout, activity UI, AI, Action, or production write recovery was implemented.

Core remains a single crate. New direct dependencies serve actual needs: TOML
serialization, bundled SQLite persistence, OS-random UUIDs, exclusion globs,
SHA-256 fingerprints, and Unix process/file safety through libc. Cargo.lock records
exact resolved versions. Rust 1.99+ is declared; local Rust/Cargo 1.99.1 was tested.
Git below 2.56.0 is rejected; newer versions are allowed but are not automatically
validated by the version floor. No package version/release change was made.

## Identity and availability

`RepositoryId` identifies one logical shared Git repository. Each checkout receives
a separate `WorktreeId`. Canonical common/git/worktree directories and their observed
filesystem bindings identify membership. On validated Unix hosts the binding uses
device/inode and available creation-time evidence. IDs live only in SQLite, never
in managed project marker files. No identity grants trust: every stored entry reports
`untrusted: true` and no trust-grant operation exists.

Aliases canonicalize; friendly supplied workspace roots and relative display paths
remain separate from identity. A verified same-filesystem move retains IDs if all
required bindings still match. Copies/replacements get new bindings; old entries
remain missing/unavailable. After successful local inspection of a different identity
at the exact same canonical path, the workspace records `superseded_by` for the old
identity. Historical IDs, state and audit records are retained, but the desktop
excludes them from active inventory and monitoring. Failed inspection never proves
a replacement. Restoring the original filesystem identity can make it current again
on rescan; no path-only rebinding or transfer of trust, approvals or drafts occurs. Matching is conservative evidence, not a cryptographic
identity guarantee against inode reuse or a malicious local actor.

Metadata classifies shared worktrees, independently nested repos, and submodule
edges separately. Submodule declarations are compared with stage-0 index gitlinks;
missing/uninitialized/inconsistent states remain explicit. Child HEAD and dirt are
reported from separately inspected child entries, with gitlink/HEAD comparison.
External worktree paths are reported as relationships but never auto-inspected.

A complete rescan can mark unobserved entries `missing` (including excluded or moved
entries); this is an observation, never proof of deletion and never an instruction
to remove Git files. Incomplete scans mark unobserved memberships `unavailable`.
Root/subtree/inspection errors are retained. Targeted refresh never recursively
rediscovers; failures persist an unavailable issue. Changing root/exclusions
invalidates cached scans and marks old observations unavailable. Old worktree IDs
outside an updated root cannot be refreshed until resolved through a rescan.

`discovery::discover` preserves M0's all-or-error compatibility behavior. New `scan`
returns repositories, relationships, issues and `complete`. Discovery does not
follow descendant symlinks and never traverses `.git`. Exclusions use exact basename
lists plus case-sensitive workspace-relative globs with literal separators; no
implicit `.gitignore` inheritance affects discovery.

## Persistence

`config.toml` version 1 owns workspace IDs/names/roots/exclusions. `state.db` schema 1
owns global identity bindings, workspace memberships, availability, last active ID,
last scan and derived status. Git remains authoritative for actual repository data.
Raw Git config values are not persisted; fingerprints use hashes. Cached commit
metadata is private application state, not operation-history or source logging.

An explicit data directory is required for CLI persistence. It is rejected inside
an existing checkout, including canonical aliases. Definitions cannot contain the
state directory. Config/state/lock files reject symlink/special-file targets;
files created by the service are private on Unix. Native kernel I/O and the chosen
local filesystem are trusted; this is not a filesystem sandbox.

One OS file lock owns the data directory across the service lifetime; SQLite closes
before that lock is released. A second writer receives Busy. Definitions are written
to a uniquely created temp file, synchronized, renamed and directory-synchronized.
If interrupted, the destination is old or new valid TOML; an orphan temp file grants
no authority and is ignored. TOML formatting/comments are not preserved across edits;
invalid/unknown fields and unsupported versions are rejected rather than discarded.

SQLite uses rollback journaling, full synchronization and transactions. Initial
schema creation is transactional; an existing known-empty v0 database gets a safety
copy before schema initialization. Unknown unversioned tables and newer schemas are
rejected. No destructive schema upgrade is currently defined. Future upgrades must
add transaction/failure tests and safe snapshots before being accepted.

A successful TOML rename followed by DB failure is reconciled on restart. SQLite
records the definition revision; root/discovery changes invalidate old observations.
No attempt is made to atomically commit TOML and SQLite together. Invalid configuration
is preserved without fallback defaults. Neither store contains production Git
approval records; M0's mutable lab journal remains experimental.

## Structured status and history

Porcelain v2 with NUL-delimited path records supplies staged/unstaged/conflicted/
untracked changes, rename source/destination, modes and submodule markers. A partially
staged file counts in both staged and unstaged categories; conflicts have their own
count. HEAD is absent for unborn branches; branch is absent for detached HEAD.

Parent status suppresses recursive submodule inspection. A cached tree/index diff
recovers staged gitlinks; child status/HEAD information is reported on inventory
relationships. Counts do not recursively add child changes. The legacy quoted v1
status field is diagnostic only and may differ from structured gitlink reporting.

Upstream OID/ahead/behind compare local tracking refs only. Fetch freshness is null;
missing upstream objects/refs are explicit. No network refresh occurs. Merge/rebase/
cherry-pick/revert/sequencer markers and last commit metadata are reported.

History pages default to 100 commits and accept 1..500. Scope is all local branches
or an existing full ref. Each commit retains raw author/committer/message bytes,
parsed author/committer times, encoding header, tree and actual parent OIDs. Ref
names are currently UTF-8; non-UTF-8 refs fail typed inspection rather than becoming
lossy executable arguments. Shallow OIDs are explicit; missing objects never fetch.

Cursors bind scope, offset and the current generation (including index/config/ref
state). Drift rejects StaleCursor. They are paging data, never authority. Offset is
bounded to 1,000,000. Unborn history is empty. No DAG edges are synthesized from row
order and no graph UI/layout exists.

## Diffs, paths and helper restrictions

Staged diffs compare HEAD/index; unstaged diffs compare index/worktree. Historical
diffs require a real explicit parent for non-root commits. Root diffs use Git's
root comparison without writing an empty-tree object. Raw NUL metadata is parsed
separately from patch bytes. Binary changes, executable modes, deletions, rename
pairs and gitlink OIDs remain explicit. Patch text is not parsed for filenames.

`NativePath` separates UTF-8, Unix bytes and Windows wide serialization. Native
arguments and lossless bytes are used for execution; escaped display text never
reconstructs a path. Byte parser/serialization coverage runs here even though
macOS rejects the filesystem fixture for invalid UTF-8 names.

Paths are normalized repository-relative names, interpreted literally. Traversal,
metadata components and symlink ancestors are rejected for worktree previews.
Symlink leaves return link content; FIFOs/devices/directories cannot become previews.
Unix regular-file opens use no-follow/nonblocking flags and check the opened type.
Immutable blobs are read without content conversion. Complete patches/previews have
`complete=true`; limits return typed errors instead of silently truncated success.

M0 restrictions remain: remove inherited Git overrides, suppress system/global
config, prompts, pager, optional locks, fsmonitor/untracked cache, hooks, maintenance,
replacement objects, lazy fetch and all transport protocols. Explicit no-ext-diff,
no-textconv and short submodule output apply to diff requests. Configured content
filters (even unused/empty) and sparse-index configuration are rejected. Global
ignore/filter settings are intentionally not inherited, so safe-policy results can
differ from a user's unrestricted shell Git configuration. No filter compatibility
or trusted-inspection relaxation was added.

## Bounds and consistency

- Four Git subprocesses globally; one multi-command read per canonical worktree.
- Default total session/status/commit-detail deadline 15 seconds; history pages and diffs 30 seconds.
- Child stdout default 8 MiB (caller range 1..64 MiB); stderr cap 64 KiB. Readers
  enforce bounds while draining. History also enforces a bounded page payload.
- File/blob previews 2 MiB; configuration/shallow files 2 MiB; index snapshot 16 MiB.
- Default explicit scan deadline 15 seconds, maximum 100,000 directory visits;
  library callers can pass ReadOptions. Persisted inventory JSON cap 64 MiB.
- Cancellation/deadline/output failures kill and reap the owned child/group on Unix.
  Kernel filesystem calls are not asynchronously cancellable; supported use assumes
  responsive local filesystems. Windows process-tree behavior remains unvalidated.

Read generations fingerprint metadata bindings, effective config, HEAD/index and
refs before/after. Working-file diffs additionally compare selected content and raw
metadata around patch generation. Ordinary drift returns ConcurrentChange. Status
is an observation of a changing workspace, not a globally atomic filesystem snapshot;
the retained v1 diagnostic is a separate command. Hostile concurrent metadata/config
replacement remains unsupported: validation does not sandbox executable races. Git,
PATH, host OS and local filesystems are trusted. Config includes/object alternates
may read outside the root; no filesystem confidentiality guarantee is claimed.

## Validation and remaining gates

The acceptance matrix maps M1-A01..A15 to runnable tests. Local macOS/Git 2.56.0
evidence is under `docs/validation/m1/`; M0 evidence is preserved separately.
Ordinary/unmerged/split-index inspection, SHA-256, gitlinks, symlinks, binary/modes,
newline/pathspec-looking names, process cancellation/limits, consistent reads,
storage/restart failures and CLI equivalence are covered. Sparse configurations
fail closed. Positive GPG and non-UTF-8 filesystem integration retain explicit
exclusions. No Linux or Windows support is claimed; the local CI definition has not
run remotely and runners must satisfy the Git/Rust floors before tests can pass.

M2 still needs production trust and approvals, write preflight, complete hook
lifecycle/bypass, user signing configuration, journal/scratch durability, index
reconciliation and shared-worktree concurrency gates. No M1 result closes those.

### Tagged caches and scan-level presentation

Child directories carrying a valid regular non-symlink CACHEDIR.TAG signature are
excluded from recursive discovery and explicitly listed in `skipped_caches`.
An explicitly selected tagged workspace root is still inspected. Invalid or unreadable
tags do not hide malformed Git metadata or inspection errors. Completeness is always
within name/glob/tagged-cache exclusions. Desktop inventory supplies a compact last-scan
summary with both discovery and per-repository errors; unknown/partial states are visible
independently of repository-attention counts. See the [cache fix report](../reports/DISCOVERY_CACHE_SCAN_STATUS_2026-11-07.md).
