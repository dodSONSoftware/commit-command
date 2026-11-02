# M2 controlled writes: contracts and support envelope

Implemented 2026-11-01 following explicit M2 authorization. This is the production
controlled-write service for the limited envelope below. Feature selection alone
never grants trust or approves an operation. The default build remains read-only
with respect to managed Git repositories. M3 and later are not implemented.

## Architecture and authority

One Rust crate remains under `app/core/`. No presentation dependency, new crate,
watcher, network operation, branch-management API, AI, Actions, hunk staging or bulk
transaction was added. `writes` is exported only with non-default
`controlled-writes`. It has its own private fixed-command adapter, policy checks,
SQLite records and transaction/recovery code. It does not call the experimental
commit/journal APIs. `experimental-writes` continues to expose only the historical
POC and disposable lab. `write-test-support` additionally exposes fixture-restricted
fault injection; release adapters should never select this feature.

`ControlledWrites` owns the existing `WorkspaceService` data-directory lock. M1
workspace definitions remain TOML version 1. Opening the write service backs up
SQLite schema 1 with SQLite's backup API, syncs the private backup, then migrates
transactionally to schema 2 (trust, operations, phase events). Default M1 services
can reopen schema 2. No operation payload, trust grant or approval comes from a
caller-selected journal path. UUIDs select Core-owned records and artifact paths.

The CLI `gitfleet-write` is a separate feature-gated interactive adapter. Trust,
operation execution and recovery each display their Core-produced review and
require literal `APPROVE`. It has no unattended approval switch or arbitrary Git
argv endpoint. Applications must treat the Rust capability boundary as the source
of authority; a future desktop IPC adapter must not expose raw execution methods
without an explicit user gesture.

## Trust

A grant applies to one observed worktree, never its descendants, linked worktrees,
clones or other workspace members automatically. Its record binds canonical
root/git/common paths, filesystem identities, ownership/modes, object format,
effective configuration hash and origins, hook manifest, configured signing
program/key fingerprint and signer home. Moves, replacement, changed ownership,
configuration, hook or signer invalidate the grant. Trust is revocable. M1's IDs,
scan results and cached status do not grant trust.

Trust inspection reads effective local/worktree/user/system/include configuration;
it does not execute a repository-selected signing program or hook. SSH public-key
fingerprints are inspected with `/usr/bin/ssh-keygen`. Source configuration bytes
are hashed, while only required non-secret policy values and origin paths are
retained. Configuration origins are native bytes. Hook review shows effective
location, source/resolved paths, executable status, fingerprint and recognizable
Husky/Lefthook markers. Includes may read outside the repository; this is not a
filesystem confidentiality sandbox.

Ordinary trust inspection refuses foreign HEAD/index locks. A crashed operation's
locks can be inspected for explicit reauthorization only when Core proves their
recorded identity and complete content hash; it does not delete them. Recovery
still requires a separate, fresh recovery review and approval.

Trust authorizes execution of reviewed local code. Hooks and signers run as the
user and are **not sandboxed**. A malicious hook can ignore environment restrictions,
write arbitrary accessible files, invoke other tools or contact the network. The
publication checks protect GitFleet's approved commit, not the entire machine
against code the user trusted. Script dependencies, interpreter binaries, shared
libraries, GPG-agent/pinentry settings and deliberate same-user attacks are not
transitively fingerprinted or isolated. Do not trust hostile executable helpers.

## Proposal and approval

`propose(workspace, worktree, request)` checks availability, scope, supported state
and trust before creating an isolated bare object database and alternate indexes
in the private application-data directory. It does not write objects, indexes,
refs or lock files in the managed repository. Selected paths are literal native
relative paths; display strings never become execution paths. Duplicate/case-alias
selections and symlinked ancestors are rejected.

Supported requests:

- Full-file stage or unstage for explicit paths, without altering worktree bytes.
- Commit selected index versions, worktree versions, or a mixture. Commit content
  starts from HEAD, while the reconciled real index preserves excluded staging.
- Initial commits, deletions/rename pairs, binary contents, symlinks, executable
  modes and explicit gitlink entries. Parent commits never recursively commit
  children or grant them trust. An initialized selected worktree gitlink is inspected
  independently; an index-selected uninitialized gitlink can retain its exact OID.

The immutable proposal contains the request, workspace/repository/worktree IDs,
branch and expected parent, exact resulting tree, reviewed message bytes, author
name/email/date, hook policy/manifest, configured signer and fingerprint, patch,
creation time and digest. A missing terminal message newline is added **before**
review; no other cleanup or hook message rewrite is accepted. Empty changes and
empty/NUL-containing messages are rejected. The caller approves the exact stored
proposal, not a mutable reconstruction.

`Approval` is private, opaque, in-memory, non-serializable, redacted in Debug output,
session-bound and limited to ten minutes (a shorter service limit is supported).
Only its token hash/session association is persisted. A restart requires a new
proposal/approval; repeated use of a consumed capability returns its recorded
outcome and never reruns hooks or creates another commit. Changed selected
worktree bytes, real index, branch (including a same-OID switch), HEAD, trust,
configuration, hook or signing policy invalidate execution. Cancellation before
publication records a nonpublished outcome. Cancellation/timeouts during publication
can be uncertain and require actual-state recovery.

Bounds include 10,000 selected paths, 100,000 index entries, 16 MiB file/index and
message artifact reads, 512 KiB commit messages, 2 MiB hook inspection, 64 MiB program fingerprinting,
32 MiB individual loose-object promotion, 200,000 writable-metadata entries and
32 metadata directory levels. Hook/signing/Git subprocess defaults are 120 seconds
and 16 MiB stdout; stderr is limited to 64 KiB. The shared process pool permits
four subprocesses; timeout/output/cancellation terminate owned Unix process groups,
including blocked stdin writers. Hostile process detachment and blocked kernel I/O
are outside this guarantee.

## Hooks and signing

`Execute` directly invokes executable `pre-commit`, `prepare-commit-msg` and
`commit-msg` in sequence, with repository cwd, real Git metadata and an alternate
selected index/object database. Managed objects are a read alternate; normal hook
Git commands can inspect selected blobs and keep new preparatory objects private. Message hooks receive the reviewed message artifact; prepare gets
source `message`. GIT_EDITOR is `:`. Nested Git gets hooks/fsmonitor/automatic
maintenance disabled. Each script fingerprint is rechecked at invocation. Each
preparatory hook must leave the approved tree and message unchanged, and selected
worktree/index/HEAD/policy must still match. A hook adding even an excluded path to
the alternate index blocks publication. Excluded worktree side effects are preserved
and reported; no rollback of arbitrary hook writes is attempted.

After index reconciliation, `post-index-change` receives `0 0`; commits then invoke
`post-commit`. Attempts are durably recorded before execution and are never replayed
by recovery. Nonzero exit/timeouts after publication are separate results, not a
claim that the commit failed. Real index/ref changes by post hooks require manual
reconciliation. A crash around a post-hook attempt remains `uncertain_outcome`;
subsequent recovery does not erase that uncertainty.

`BypassAll` suppresses every applicable local hook, including native
reference-transaction and index hooks, without modifying hook files or repository
configuration. The per-worktree remembered preference is explicit and the actual
policy always appears on the final proposal. Execute mode fails closed on executable
reference-transaction hooks and recognizable Husky/Lefthook managers. The finite
manifest is not a general hook-manager compatibility guarantee. Hooks that need a
native `git commit` environment, interactive stdin, dynamic delegation or nested
hook dispatch are unsupported until separately validated.

Configured signing has no unsigned fallback. An explicit unsigned request is
reviewed as such. Git boolean spellings such as `yes` are honored. OpenPGP requires
a full fingerprint and an existing explicit/GNUPGHOME/standard signer home. The
reviewed GPG binary is invoked through an app-owned wrapper with `--no-options`
and `--no-auto-key-retrieve`; repository configuration cannot enable key retrieval.
SSH requires an absolute plain key path plus public key; dynamic key commands,
inline keys and X.509 are unsupported. Core freezes the approved public key and
constructs its own allowed-signers file. A configured executable is resolved and
fingerprinted before trust but only executed after approval.

Before publication, raw commit bytes must match the approved tree, sole expected
parent (or no parent), author/committer identity/date and exact message. Signed
commits must additionally verify against the approved OpenPGP full fingerprint
(or matching primary-key fingerprint) or SSH fingerprint. Missing keys/programs,
wrong-key signatures, signing rejection and timeout block publication. Positive
OpenPGP and SSH gates both execute and independently verify real signatures.

## Publication and durability

Supported writes require local **macOS/APFS**, Git 2.56.0+, the files ref backend,
a local symbolic branch, and an ordinary checksum-valid SHA-1/SHA-256 index v2/v3.
Only the TREE extension and zero-valued v3 extended flags are accepted. Unmerged,
split, sparse, assume-valid, skip-worktree, intent-to-add, index v4 and unknown
extensions fail closed. Conflicting same-branch linked checkouts, detached/bare repositories, in-progress merge/rebase/
cherry-pick/revert/bisect/sequencer states, content conversions/filters/custom attributesFile settings, symlinked
writable metadata and foreign locks are unsupported. Local APFS is checked for
worktree, per-worktree and shared Git metadata. Ordinary M1 read support remains
broader and is not reduced by these write restrictions.

Execution holds a cross-process advisory lock on the common-directory inode, then
acquires Git's native `index.lock` and `HEAD.lock` exclusively. Ownership receipts
include file identity plus complete content hash, recorded before filling each
lock. Partial/changed locks cannot be automatically adopted or removed. Actual
HEAD is rechecked under its native lock. Tools ignoring locks and hostile filesystem
replacement/hard-link attacks remain outside the support envelope.

Git 2.56 rejects combining `symref-verify HEAD` with a referent update as a duplicate
HEAD update. M2 therefore publishes from an app-owned **detached** Git context whose
`commondir` points to the shared repository. The real HEAD remains protected by
its native lock. Git performs branch old-OID compare-and-swap and writes the branch
reflog with an operation UUID receipt. Core then appends the exact corresponding
HEAD reflog line under the owned native HEAD lock and its recorded before-image
hash. An unexpected or partially written HEAD log requires manual reconciliation.
Linked worktrees use their own HEAD log and shared branch metadata.

New objects are validated with `git fsck --strict` before and after promotion.
Loose objects are written to unique temporary files, synced, then made visible
atomically without replacing existing objects. Incomplete promotions can leave
unreferenced objects or temporary artifacts; publication never relies on a partial
final object. The candidate index hash is checked before copying into its lock and
before rename. Real index reconciliation requires its original bytes, owned lock
and expected published branch. It preserves unrelated staging and worktree bytes.

App records use SQLite transactions, synchronous FULL, macOS fullfsync and a private
schema backup. Artifacts/locks/objects/ref/reflog data and containing directories
are synced in order; files use macOS F_FULLFSYNC as well as fsync. Multi-file Git
state and SQLite are **not one atomic transaction**. The journal exposes that gap
and recovery inspects actual state. SIGKILL/fault injection proves process-crash
behavior, not universal power-loss, storage-controller or OS-crash durability.

## Recovery and audit

Recorded phases cover proposal, approval, preparing, hooks, revalidation, object
ready, owned locks, publication pending, ref updated before publication receipt,
published, HEAD log written, index renamed before lock release, reconciled,
post-processing and completion. Typed outcomes distinguish reauthorization,
blocked/cancelled before publication, published-needs-reconciliation,
post-processing failure, manual reconciliation and uncertainty.

`inspect_recovery` is read-only. Its Core-owned review binds operation, current
identity, branch/OID, index, lock identities/content and reflogs. `recover` requires
that unmodified review, rechecks after acquiring the common lock, and validates
current scope and trust. It may:

- Abort an unpublished operation, deleting only proven owned locks. Objects remain.
- Recognize native ref/reflog evidence even when the process died before persisting
  the published flag; verify the exact commit and reconcile only the owned index.
- Recognize an already-reconciled index/HEAD log without creating a duplicate commit.
- Refuse unexpected refs, indexes, locks, artifact corruption or partial reflogs.

Recovery never creates/retries a commit, runs a hook or overwrites external index
edits. Published records are historical; later external branch changes do not
rewrite a completed operation's result. Revoked trust prevents recovery writes;
explicit re-trust can inspect proven pending locks, then a new recovery review is
required. There is no force-delete-lock or destructive rollback endpoint.

Persistent operations retain proposal/tree/message/paths/signing identity, phases,
OIDs, per-operation results and sanitized failure categories. Raw hook/signer output,
credentials and approval plaintext are not persisted. Patches, source artifacts,
messages, paths and public signing identities are nevertheless sensitive local
content; app data and backups use private permissions. Records/artifacts are retained
without automatic pruning in M2. The proposed configurable 90-day policy remains
an open product default; recovery evidence must never be pruned automatically before
resolution. Disk usage, explicit retention/export and encryption require later work.

## Evidence and remaining boundary

The [M2 acceptance ledger](../ACCEPTANCE_MATRIX.md#11-m2-acceptance-gates--controlled-writes)
and [validation evidence](../validation/m2/README.md) identify executed tests,
ignores and unsupported cases. M0/M1 evidence and the original POC results remain
historical and unchanged. Native non-UTF-8 macOS filename creation is still excluded;
byte-safe parser/serialization coverage does not substitute for that filesystem test.
Network filesystems, other OSes, reftable, interactive signing/hardware tokens,
unknown hook frameworks and OS/power-loss tests have no acceptance claim.
