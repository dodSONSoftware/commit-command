# G09 guarded SemVer invocation — November 8, 2026

Typed version-file Actions now have independent exact native approval, durable
intent, guarded file replacement, separately reviewed staging/commit and read-only
observed-state recovery. They work without AI. Every invocation is new and explicit;
an approved definition/invocation record is not a reusable execution capability.

The initial format and explicit increment/prerelease policy remain those accepted
by the [preview checkpoint](G09_SEMVER_PREVIEW_2026-11-08.md). Core binds current
workspace/worktree identity, exact path, file and directory identities, bytes/mode,
index, HEAD/branch, effective configuration and trust. Changes stop execution.
Only an owned, single-link regular file of at most 1024 bytes with no special mode
bits is eligible; metadata paths, nested repositories, symlinks and hard links are
refused. No hook, signer, AI request, transport, index/ref change or commit runs as
part of file replacement.

Replacement uses macOS atomic exchange between the pinned target directory and a
private application recovery directory on the **same filesystem**. There is no
overwrite/copy fallback on unsupported filesystems or platforms. Candidate data is
synced and replacement intent persisted before exchange. The displaced file is
retained privately; it is never automatically deleted or rolled back. The original
inode, mtime, mode and exact bytes are checked after exchange (exchange changes
ctime). This catches a writer that races the final precheck while preserving that
writer's bytes for reconciliation. A real filesystem regression demonstrates the
unexpected edit is retained instead of destroyed. SDK `RENAME_SWAP` and
`RENAME_NOFOLLOW_ANY` behavior was checked against the installed macOS headers and
[Apple's swap-renaming capability documentation](https://developer.apple.com/documentation/foundation/urlresourcekey/volumesupportsswaprenamingkey).

This is an explicitly bounded macOS Action envelope. A repository and application
state on different filesystems are refused before mutation; ordinary Git operations
do not acquire a new cross-volume support claim from this feature. Other version
file formats and general scripts remain unsupported.

Cancellation/failure after intent or process loss is unconfirmed, not automatically
retried. The shared recovery barrier blocks new and already approved Git writes,
including controlled-only builds without application services. Workspace removal
or scope changes cannot discard an unresolved Action. Recovery compares the frozen
before/after bytes and the retained original, rechecks state at confirmation, and
records not-applied/applied/conflict without mutating files or incrementing again.
An observed matching result does not invent a causal execution receipt. Conflicting
content and retained recovery data stay intact. Definitions interrupted before
approval restore without authority.

Staging is a **separate fresh exact native review** of the still-matching Action
result. Committing is the existing independent immutable tree/message review.
History, paged repository results and recovery notices expose actual outcomes;
ordinary builds can inspect them but cannot invoke mutations. Preview results are
scoped to the source revision so stale asynchronous responses are not presented as
current. AI remains optional throughout.

Selected automated evidence includes SHA-1 and SHA-256 Action → stage → one-file
commit, exact old/new content/mode/index/HEAD and unrelated staged/dirty preservation;
decline, pre-cancel, index/file/trust drift, hard-link/symlink refusal; source/approval
field rejection; restart and stale staging; actual owned SIGKILL before/after
exchange, storage failure, shared recovery barriers, fresh recovery after external
edits, no double increment and no restored approval. The ignored crash-worker entry
is explicitly invoked by its parent test. Full Core runner exited 0: 66/101/91/270/319 passed across default, experimental,
controlled, application-services and all-features, with 1/2/1/4/11 explicit ignored
entries, zero failures and lab 24/24. Format, builds and strict Clippy passed in each
configuration. Full desktop exited 0: 108 renderer / 14 ordinary-host / 17 all-feature
host tests, types/lint/contracts/format/builds/strict Clippy passed. The recovery
barrier guard was added while the matrix ran and appears in application/all-feature
logs; a separate controlled-only targeted run verifies it without application code.
`matrix-summary.json` records exact counts. Development failures are retained in
separate earlier logs; final named runners and native artifacts passed.

Native acceptance reused the already trusted leaf only, with no new trust grant,
credential or local-transport policy. One untracked `version` file (`2.0.0\n`, mode
0750) was explicit fixture preparation. Native independent reviews then:

1. Replaced it with `2.0.1\n`. A checkpoint proves every other managed byte/mode,
   index and HEAD was unchanged, and the original was retained privately.
2. Staged only `version`, preserving HEAD and the dirty unselected file.
3. Committed only that staged version with the exact manually composed message,
   parent/tree/blob and reconciled index. Other worktree HEADs, old objects/config/
   symbolic HEAD/other refs and all unrelated bytes/modes stayed exact. Only the
   selected index/main ref/reflogs and new objects changed.

Unified history showed the Action, stage and commit separately. Ordinary restart
showed the same records with mutation buttons disabled; files and all execution
journals stayed exact, with no replay. Both native exits were 0; old journals and
trust/revocation, source and both artifact hashes, SQLite integrity and writer-lock
reopen passed. The top-level parent remained revoked and the control untrusted.
No push occurred. Evidence and repeatable final oracle:
[`g09-semver-action-2026-11-08`](../validation/g09-semver-action-2026-11-08/).

G09 remains open: user-friendly retained-file inspection, broader native interruption
and accessibility coverage, AI configuration/providers/context/candidates, and
separately bounded automatic execution remain. Retained Action recovery data is
protected pending G10 backup/retention policy; this step does not implement archive
portability, automatic cleanup, ordinary write enablement or release readiness.
