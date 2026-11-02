# GitFleet — Decision Register and Open Items

**Version:** 1.0 · **Date:** November 1, 2026

This register separates **explicitly agreed product behavior** from **proposed implementation defaults** and unresolved architectural validation. It should be updated whenever a requirement is changed.

## Approved decisions

| ID | Area | Decision |
|---|---|---|
| D-001 | Name | GitFleet |
| D-002 | Architecture | Rust Core is independent of all presentation technology; Tauri/React is an adapter |
| D-003 | Workspaces | Multiple saved, one active, one root per workspace |
| D-004 | Discovery | Recursively discover nested repositories; classify independent/submodule/worktree relationships |
| D-005 | Overview | Flat table default + hierarchical tree; shared selection/filter model |
| D-006 | Bulk selection | Parent selection never recursively selects children |
| D-007 | Tabs | Permanent Overview and closeable repository tabs; state persisted |
| D-008 | Repository view | Adaptive Changes when dirty, History when clean; manual choice persists |
| D-009 | Staging | File-level first, hunk-level later |
| D-010 | Commit review | Mandatory for normal individual commits and bulk batches |
| D-011 | Bulk staging | Review all eligible staged/unstaged/untracked content before newly staging |
| D-012 | Commit dependencies | Commit selected submodules before selected parents; re-review changed gitlink |
| D-013 | Push | Separate confirmation; dependency-aware; guided missing-upstream setup |
| D-014 | Monitoring | Active workspace only, while process running; close last window quits |
| D-015 | Discovery changes | Auto detect new/deleted/moved/renamed repos while active |
| D-016 | Activity | Timeline + activity lanes + contribution heatmap; repo/workspace filters |
| D-017 | Activity scope | All local branches; timeline/lanes default 7d, heatmap 12mo |
| D-018 | History | Persistent Operation History and read-only Git-derived Workspace Activity |
| D-019 | Storage | Hybrid human-readable TOML + SQLite |
| D-020 | AI support | Configurable local connection/server/model/profile with editable system message |
| D-021 | AI profiles | Profile-owned system message; hierarchical app/workspace/repo profile assignment |
| D-022 | AI connection | Discover installed models from configured IP/hostname and dropdown selection |
| D-023 | AI diagnostics | Connection test + model inference test |
| D-024 | AI concurrency | Configurable global and per-connection limits |
| D-025 | AI failure | Retry/select another profile/cancel; no silent fallback; auto commit blocked |
| D-026 | AI templates | Structured editor; app/workspace/repo inheritance; persistent + one-off instructions |
| D-027 | AI preview | Live static preview + optional read-only real-context AI test |
| D-028 | AI context | Exact diff + relevant context + Git logs; hierarchical analysis for large diffs |
| D-029 | AI transparency | View actual supplied AI context and analysis coverage |
| D-030 | AI candidates | Up to three independently reviewed candidate commit messages |
| D-031 | AI automation | Assisted and user-initiated bounded automatic commits; never watcher-triggered |
| D-032 | GitFleet Actions | Deterministic typed pre-commit automation, first SemVer file update |
| D-033 | GitFleet Actions authorization | AI suggests; user approves; Rust executes; no unrestricted AI writes |
| D-034 | AI config portability | Import/export profiles, prompts, templates, instructions (no credentials) |
| D-035 | Backup | Manual backups plus change-aware automatic rotating local backups |
| D-036 | Trust | Each discovered repository individually trusted before writes; no inheritance |
| D-037 | Hooks | Hook inspection and bypass-all policy v1; selective controls later |
| D-038 | Hooks policy | Per-repo remembered execute/bypass preference, always presented at review |
| D-039 | Approved content | Final commit tree and message are immutable authorization boundaries |

## Proposed defaults / not explicitly finalized

| Topic | Proposal | Condition |
|---|---|---|
| Theme | System theme, dark used in design mockups | User may change |
| AI request concurrency | One globally and one per connection | User can configure |
| AI recent current-branch logs | 20 commits | Configurable; context-bounded |
| AI recent relevant file logs | Up to 10 commits | Configurable; context-bounded |
| AI sampling defaults | Conservative temperature around 0.2 | Profile-specific, capability-gated |
| Commit title maximum | 72 chars for Conventional Commits example | Template-configurable |
| Operation history retention | 90 days | Configurable |
| Automatic backups retained | Last 10 | Configurable |
| Initial GitFleet Action | Update SemVer file | Only approved action invoked |
| Bulk execution order | Sequential, stop on first failure | Suitable v1 default |

## Open technical validations before enabling production writes

1. Controlled hook invocation compatibility (including hook managers, config locations, context/stdin/args, Git reference-transaction hooks).
2. Correct selective index reconciliation with split/sparse/unmerged indexes and with linked worktrees.
3. Crash consistency and actual durability of journal state across operating system and filesystem failure modes.
4. Signing behavior using configured GPG and SSH signing, with positive verification on macOS.
5. Safe handling of Git SHA-256 object IDs and arbitrary filenames.
6. Trust identity invalidation for moved/replaced git directories, symlink and TOCTOU attacks.
7. Snapshot authorization and normal hook lifecycle integration without unsafe commit publication.
8. Submodule dependency graph semantics, commit/ref availability checks, and partial batch recovery.
9. Provider capabilities, endpoint authorization and sensitive-content filtering of AI requests.
10. SemVer modification/retry idempotence after crash and external edits.

## Product matters to confirm during implementation planning

- Product owner/organization ID, signing identity, license and distribution approach.
- Exact UX shortcuts, accessibility targets and desktop release OS minimums.
- Default SemVer bump strategy in templates that enable the Action.
- Which advanced Git operations (branch create/checkout, pull modes) are in initial delivery vs subsequent releases.
- Supported minimum Git version and supported hook frameworks for v1.

## Change-control rule

No new product capability is implicitly authorized by a mockup, imported template, AI output, or a development convenience. Add new behavior as a numbered decision with corresponding specification amendment and acceptance tests. The main spec prevails over this register if they temporarily diverge until reconciliation.


## M0 implementation decisions and findings — 2026-11-01

The user authorized M0 only after review. These are safety/engineering decisions,
not new product capabilities:

| ID | Decision | Evidence / consequence |
|---|---|---|
| M0-D001 | Preserve one Rust Core crate and existing docs/app layout | No Tauri/React dependencies or desktop scaffolding |
| M0-D002 | Experimental writes require non-default `experimental-writes` | Default CLI/library contain inspection/discovery only; feature opt-in is not trust/approval |
| M0-D003 | Fixed-query untrusted inspection policy | No write-tree; fsmonitor/hooks/maintenance/lazy fetch/transports disabled; configured content filters explicitly rejected |
| M0-D004 | Preserve POC evidence and explicit skip accounting | Historical core README/source listings and results remain; positive GPG Cargo test is explicitly ignored by default, lab reports actual missing-GPG skip |
| M0-D005 | Distinct requirement namespaces | INV-* release invariants; SEC-* matrix scenarios; crosswalk in acceptance matrix |
| M0-D006 | Reject stale symbolic HEAD, use literal worktree selections | Focused lab corrections only; production writes remain blocked |
| M0-D007 | Fail inspection/discovery visibly on errors | No raw repository-controlled stderr; discovery fails rather than returning incomplete success; detailed unavailable inventory deferred to M1 |
| M0-D008 | Minimum hook inspection and persistent operation results precede production writes | M2 prerequisite; richer manager/audit UI can remain later |

Scope restrictions are detailed in [M0 safety contracts](architecture/M0_SAFETY_BOUNDARIES.md).
Git 2.56.0/macOS is validated; no minimum Git/Rust or Linux/Windows support is claimed.
The ignored non-UTF-8 filename test reflects host fixture EPERM, not proof of Git
support or rejection. Newline filenames are separately tested.

Before M1: decide identity/common-directory/worktree/move semantics, structured
status and filename representation, configuration/TOCTOU policy, persistent workspace
schema, and supported Git versions. M0 inspection assumes stable metadata and does
not sandbox hostile concurrent local actors, config includes or object alternates.

Before M2: resolve all existing open write validations, including authenticated
approval ownership, journal/scratch durability, complete hook bypass, config/signing
compatibility, advanced index preflight and shared-worktree races. D-039 remains
unchanged: the legacy engine's trimmed message comparison is insufficient production
proof. M0's raw-message fixture does not close that gap.

Tab-view persistence after closing a tab, SemVer policies, performance budgets,
product identifiers/license/distribution and accessibility targets remain open.
CI is defined locally; no workflow execution, commit or push was authorized.


## M1 approved implementation decisions — 2026-11-01

The user approved M1 as proposed, including the following decisions/gates. M1 is
implemented for local macOS review. M2 and later remain unauthorized.

| ID | Approved decision | Implementation |
|---|---|---|
| M1-D001 | Keep one Core crate and focused internal modules | domain/process/inspection/discovery/workspace/history/diff; no presentation dependency |
| M1-D002 | Repository/common-directory bindings distinct from worktree IDs | UUIDs persisted in app-owned SQLite; common bindings share repository IDs |
| M1-D003 | Lossless native paths; conservative move/replacement continuity | Canonical metadata plus observed filesystem bindings, separate display paths; clone/replacement fixtures |
| M1-D004 | Identity never grants trust | Every stored entry is untrusted; no trust-grant or production write service |
| M1-D005 | TOML definitions, SQLite observations, Git data authority | Atomic TOML and transactional DB; revision-based startup reconciliation; schema 1 |
| M1-D006 | Machine-oriented byte-safe formats and typed results | Porcelain v2/raw NUL metadata, native path encodings, SHA-1/SHA-256 OIDs, real commit parents |
| M1-D007 | Preserve/extend M0 helper restrictions | Bounded private transport; no external diff/textconv, short gitlinks, no child status recursion |
| M1-D008 | Bound concurrency/output/time and detect ordinary drift | Four subprocesses, one read/worktree, deadlines/cancellation, before/after generations, stale cursors |
| M1-D009 | Conservative validated baseline | macOS arm64/Git 2.56.0/Rust 1.99.1 executed; Git 2.56.0 floor and Rust 1.99 MSRV declared; Linux/Windows unclaimed |
| M1-D010 | Transport-neutral services, no desktop implementation | Thin native-argument JSON CLI; Tauri/React placeholders untouched |

Exact budgets, configuration restrictions and file-format semantics are in
[M1 architecture](architecture/M1_READ_ONLY_WORKSPACES.md). No silent filter or
trusted-inspection relaxation was introduced. Global/system Git settings remain
suppressed under the safe policy; this can differ from unrestricted shell Git.
Sparse-index configuration remains unsupported. Non-UTF-8 byte parsing is tested;
filesystem coverage on the current macOS host is explicitly ignored after EPERM.

The validated move rule uses device/inode/available creation-time evidence and
metadata relationships; it cannot prove identity against malicious copying,
inode reuse or concurrent local manipulation. Trust binding and hostile TOCTOU
containment remain M2 security work, not implied by persisted UUIDs.

Still unresolved before production writes: all ten original technical validations,
full hook and signing configuration policy, write-state preflight, shared worktree
concurrency, opaque approvals and durable journal recovery. Product identifiers,
distribution/license, accessibility/OS minimums, UI tab-close persistence, advanced
Git write operations, SemVer policy and AI/provider decisions remain open.
CI remains a local definition, not executed platform evidence. Runner Git/Rust
provisioning must satisfy the declared floors before claiming support.

## M2 approved implementation decisions — 2026-11-01

The user approved M2's proposed decisions and acceptance gates, then authorized
implementation. These supersede the corresponding production-write open validations
above only within [the M2 contract](architecture/M2_CONTROLLED_WRITES.md).

| ID | Approved decision / implemented boundary |
|---|---|
| M2-D001 | Keep one presentation-independent Core crate. Separate non-default `controlled-writes` from default inspection and the experimental POC. |
| M2-D002 | Trust binds one worktree's canonical identity/ownership/configuration/hooks/signer; no linked/nested/clone inheritance. Revocation and changed bindings prevent execution. |
| M2-D003 | Core-owned immutable proposals and opaque one-use/session-bound approvals, maximum ten minutes; restart requires a fresh proposal. |
| M2-D004 | Explicit full-file stage/unstage and selected index/worktree commits; preserve excluded index/worktree content. No hunk or bulk writes. |
| M2-D005 | Local macOS/APFS, Git 2.56+, files refs, symbolic local branch, checksum-valid ordinary v2/v3 SHA-1/SHA-256 indexes. Advanced/unknown states fail closed. |
| M2-D006 | Direct preparatory/post hook subset with visible manifest and complete explicit bypass-all. Reference-transaction hooks and recognizable Husky/Lefthook Execute mode are unsupported. Trusted code is not an OS sandbox. |
| M2-D007 | Respect configured OpenPGP/SSH signing, explicitly review unsigned requests, verify the actual signature/key and commit bytes before ref publication; never silently fall back unsigned. |
| M2-D008 | Shared-common-directory writer lock plus native HEAD/index locks. Git 2.56 rejects combined HEAD symref verification/referent update; detached operation context performs branch CAS while real HEAD is held and verified. |
| M2-D009 | Native branch reflog UUID receipt plus guarded HEAD reflog before-image/owned-lock handling, exact index reconciliation, durable phase journal and actual-state recovery. |
| M2-D010 | Private artifacts and atomic durable object promotion; SQLite schema-1 backup before transactional schema-2 migration; M1 can reopen schema 2. |
| M2-D011 | Recovery is separately reviewed and revalidated; no commit/hook replay, no overwriting foreign edits/locks, and persistent post-hook uncertainty. Explicit re-trust may inspect proven crashed-operation locks. |
| M2-D012 | Keep unresolved evidence and private operation history in M2. Configurable 90-day retention remains proposed; automatic pruning must not remove unresolved recovery evidence. |

### Open validation disposition after M2

1. Direct hook subset, effective hook paths, bypass, rejection/tampering and post
   outcomes are tested. General hook managers and reference-transaction Execute
   mode remain unsupported; no native-commit lifecycle equivalence is claimed.
2. Ordinary index reconciliation and linked worktrees pass. Split/sparse/unmerged/
   extended-flag/v4 writes are deliberately rejected; read support stays broader.
3. Thirteen real SIGKILL phase boundaries and storage/corruption injections are
   tested. OS crashes, power loss, hardware/storage-controller behavior and network
   filesystems remain untested and are not release claims.
4. Positive isolated OpenPGP and SSH signing plus independent verification pass on
   macOS; wrong-key and timeout failures are covered. Interactive/hardware-token,
   X.509 and dynamic SSH key workflows remain unsupported.
5. SHA-256 writes and literal/newline/binary/mode/symlink/gitlink cases pass. The
   existing non-UTF-8 macOS filesystem test is still ignored; parser coverage is
   not a substitute for filesystem acceptance.
6. Moves/replacement/permissions, config/hook changes, revocation and independent
   linked-worktree trust are covered. Hostile same-user TOCTOU/hard-link attacks
   and arbitrary trusted executable behavior are outside the supported envelope.
7. Exact approved tree/message/parents/identity are verified before publication.
   Native HEAD locking and branch CAS close ordinary same-OID branch switching.
8. Multi-repository dependency ordering, remote ref availability and partial batch
   recovery belong to later authorized work; M2 does not implement them.
9. AI endpoint authorization/redaction and Actions retry semantics remain unchanged
   open items for their respective future milestones.

License/distribution, desktop accessibility/OS minimums and other product decisions
remain open. The CI definition is local and unexecuted remotely; no new platform
acceptance follows from it.

## M3 authorized decisions and implementation disposition — 2026-11-01

The user approved the M3 proposal and implementation. **This entry does not declare
M3 accepted:** native UI acceptance remains partial. The earlier organization-policy
block is historical; [November 2 user-observed results](M3_NATIVE_ACCEPTANCE_HANDOFF_2026-11-02.md)
supplement it without waiving the remaining gates.

| ID | Decision / disposition |
|---|---|
| M3-D001 | Populate the existing desktop directories with a thin Tauri 2/React/TypeScript adapter; preserve one Core crate and default read-only managed Git services. |
| M3-D002 | One Core application facade owns workspace persistence and the M2 session. A single bounded worker serializes mutations/reads off the UI thread; no second data-directory writer. |
| M3-D003 | Typed version-1 requests/responses, epochs, bounded job polling and generated TypeScript from generic Rust JSON Schema. No privileged frontend event consumer is needed in this initial adapter. |
| M3-D004 | Exact custom-command permissions plus actual caller label/page/origin checks; no broad native plugins, remote capability grants, generic commands/filesystem/SQL or serialized approvals. |
| M3-D005 | Separate minimal authoritative review bundle; Rust-only nonce capability and OS-native confirmation callback before M2 approval/execution. Native end-to-end validation remains open. |
| M3-D006 | Main-renderer reload/context changes invalidate reviews and queued callbacks; cancellation is per operation and cannot promise rollback after publication. |
| M3-D007 | Keep ordinary desktop write exposure disabled while native gates are open. `desktop-writes` is a non-default acceptance build flag, not repository authority or release readiness. |
| M3-D008 | Core notification hints plus periodic reconciliation, active-workspace scope, explicit stale/error states and no watcher-authorized writes. Trusted Rust scheduling options support deterministic tests; renderer cannot alter them. |
| M3-D009 | Persist navigation/drafts in backed-up schema 3; TOML stays version 1. Protect pending recovery context from workspace removal/scope changes. Retain historical operation evidence. |
| M3-D010 | Correct advisory-lock owner destruction explicitly for workspace and common-repository locks; preserve real held-owner Busy semantics and original regression assertions. |
| M3-D011 | Develop/validate on current macOS/APFS only. Use a provisional development identifier and local dev app bundle; no signing/notarization/installer/distribution claim. |
| M3-D012 | Basic History and operation results only. Graph/activity/full audit are M4; bulk Git M5; AI/Actions and release work remain protected later scope. |

The native access rejection was: “Computer Use is blocked from using the app
'dev.gitfleet.desktop' by your organization's policy.” No alternate app identity,
OS automation or other bypass was used. Live UI tests require permitted native
access or a human operator. Later user observations cover directory selection,
review/confirmation, desktop signing and ordinary restart; they do not establish
complete acceptance. Actual running-operation cancellation, guarded recovery,
last-window shutdown, forced interruption, remaining hostile-content/context/expiry
cases, VoiceOver and measured contrast remain open. Workspace-removal and History
inspector defects are patched with renderer regressions; native retest is pending.
Other reported visual/trust-refresh/review-readability defects remain unresolved.

No future milestone or enabling of ordinary desktop writes is implicitly authorized
by passing Core/renderer/mock-runtime tests. See [the M3 ledger](ACCEPTANCE_MATRIX.md#12-m3-acceptance-gates--implementation-with-native-gates-open)
and [manual native steps](../app/desktop/README.md#native-acceptance-workflow--still-required).
