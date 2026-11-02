# GitFleet — Product and Technical Specification

**Document version:** 1.0 — initial consolidated specification  
**Date:** November 1, 2026  
**Status:** Requirements baseline for review; not a declaration of production readiness  
**Initial target:** macOS desktop; portable Rust Core and cross-platform architecture  
**Intended implementation:** milestone-based Codex work, after specification review

> **Normative language:** **MUST** = required, **SHOULD** = preferred with documented exceptions, **MAY** = optional. Requirements marked **Confirmed** reflect explicit product decisions. Technical implementation proposals are **Proposed** until proven against the stated acceptance criteria. Mockups are aesthetic references, not authoritative behavior specifications.

---

## 1. Executive summary

GitFleet is a **multi-repository Git workspace manager** for developers who organize projects as multiple independent Git repositories rather than one monorepo. The user selects a root directory, and GitFleet discovers and monitors all Git repositories beneath it, including nested repositories. It presents one coherent workspace for inspecting status, viewing history and diffs, staging selected changes, committing individually or in bulk, and pushing through separately approved workflows.

A **UI-independent Rust Core** owns Git and domain logic. Tauri 2 and React/TypeScript provide the first replaceable presentation adapter. GitFleet includes a GitKraken-inspired commit graph, workspace activity visualizations, persistent operation history, controlled Git hook handling, and optional **local AI-assisted and explicitly authorized AI-automatic commits**. AI-generated content never constitutes authorization to modify files or run Git operations.

### 1.1 Product goals

- Show which repositories need attention without opening each one individually.
- Make individual and bulk staging, commits, and pushes efficient and understandable.
- Preserve all user work, including partial staging, unrelated index entries, and concurrent edits.
- Ensure the exact approved commit tree and message are the only contents authorized for commit.
- Provide polished desktop UX while keeping application logic independent of the presentation technology.
- Support self-hosted AI that produces useful commit messages from selected diffs, relevant context, and Git logs.
- Operate entirely on the user's machine except for **explicitly approved** Git remote and AI endpoint access.

### 1.2 Non-goals for initial production release

- A hosting service, Git server, or GitHub/GitLab replacement.
- A monorepo orchestrator or build-system replacement.
- Unattended background commits or pushes triggered by filesystem events.
- An unrestricted AI agent with filesystem/shell access.
- Arbitrary user-script GitFleet Actions.
- Full-featured rebase/reset/cherry-pick/merge conflict resolution GUI at launch.
- Full individual Git-hook toggles in initial milestone (planned later).
- Multiple simultaneous active workspaces or multi-root workspaces.
- A remote web server or cloud synchronization service.

## 2. Scope and decision status

### 2.1 Confirmed product decisions

| Area | Decision |
|---|---|
| Name | **GitFleet** |
| Architecture | UI-independent Rust Core; Tauri/React as replaceable adapter |
| Workspace | Many saved workspaces, **one active** at a time, **one root** per workspace |
| Discovery | Recursive, includes nested independent repos, submodules, and linked worktrees |
| Presentation | Flat sortable repository table (default) plus hierarchical tree |
| Tabs | Permanent Workspace Overview tab; closable repository tabs |
| Repository detail | Adaptive Changes-first when dirty; History/graph-first when clean; manual selection persists while tab is open |
| Monitoring | Watch active workspace only while application runs; auto discover/remove/move; last window close quits |
| Bulk selection | Explicit per-repository selection, never implicit descendant selection |
| Commits | Review mandatory for normal individual and bulk commits |
| Bulk staging | Review all staged, unstaged, and untracked eligible changes before staging |
| Bulk dependencies | Submodules before selected parents, with changed parent gitlink re-review |
| Push | Separate review; dependency-aware; guided missing-upstream configuration; no automatic force pushes |
| History | Persistent Operation History plus read-only Workspace Activity |
| Activity | Timeline and lane views default seven days; contribution heatmap defaults twelve months |
| AI | Configurable local AI profiles, server model discovery, templates, instructions, three candidate messages |
| AI automation | Only user-initiated, bounded, explicitly authorized automatic commits; never unattended |
| AI context | Approved diffs, relevant source/docs, Git logs; hierarchical analysis for large changes; inspect exact context |
| Templates | Structured template editor, app/workspace/repo inheritance, static preview and read-only AI test |
| AI profiles | Named connections, profile-owned system message, hierarchical assignments, one-off override |
| AI requests | Configurable global and per-connection concurrency; no silent model fallback |
| Actions | Deterministic GitFleet Actions; initially supported SemVer file update |
| Hooks | Inspect hooks, per-repository remembered execute/bypass-all policy; review effective policy; individual toggles later |
| Trust | Explicit trust per repository; bulk trust selected available; not inherited |
| Commit authorization | Actual committed tree and message MUST match approved snapshot |
| Storage | TOML settings + SQLite structured state/history |
| Portability | AI configuration import/export; full manual and local automatic backups |

### 2.2 Matters not yet finalized

These are **design tasks**, not excuses to invent requirements:

- Product/legal brand identifier, code-signing identity, distribution channel, and final license.
- Exact final UI keyboard shortcut bindings and accessibility acceptance details.
- Supported Git and Rust minimum versions, with platform-specific integration testing.
- Hook-manager compatibility matrix, signature verification coverage, linked-worktree special cases.
- SemVer increment default (patch/minor/major/AI-suggested is configurable); exact prerelease policy.
- Per-provider supported model parameters and detailed secret-screening policy.
- Runtime limits, benchmark targets, and backup frequency beyond agreed change-aware behavior.
- Whether a GitFleet Actions registry beyond SemVer should ship after the first release.

Any unresolved setting MUST have a documented conservative default or be blocked from execution until decided.

## 3. Personas and user journeys

### 3.1 Primary user

An experienced developer working on a large logical project comprised of numerous repositories, frequently edited by an IDE, terminal, code agents, and other tools. The user wants a holistic daily status, efficient review, purposeful commit messages, and safe bulk publication.

### 3.2 Daily workflow

1. Launch GitFleet and restore the last active workspace and its navigation state.
2. Reconcile repository inventory and Git status. Show active watcher status.
3. Inspect the Overview table; filter for modified, conflicted, staged, ahead, or behind repositories.
4. Open a repository tab to Changes or History based on dirty state; navigate and stage if needed.
5. Select one or more repositories in Overview; selection persists while navigating tabs.
6. Open Bulk Commit, review exact included files/diffs and applicable Actions/hooks, enter or generate messages.
7. Explicitly approve a bounded batch; execute dependency-aware commits; display actual results.
8. Open separate Bulk Push plan, review remote destinations and outgoing commits, resolve upstreams, and approve.
9. Inspect persistent Operation History and recent Workspace Activity.

## 4. Workspace and repository model

### 4.1 Workspaces

- Each workspace has a stable internal ID, user-editable name, exactly one root path, discovery options, presentation preferences, and independent navigation state.
- Unlimited saved workspaces; only the active workspace has active filesystem watchers and periodic status reconciliation.
- Switching workspace deactivates old watchers, persists selection/UI state, activates new workspace, reconciles inventory, and restores tabs.
- Closing the last application window **quits the application** and stops watchers, timers, and AI jobs safely. No menu-bar daemon or background process remains.
- All app-specific configuration is outside managed project directories; no workspace manifest is required.
- Paths should be canonicalized for identity, while the UI may retain user-friendly display paths. On macOS `/var` and `/private/var` can represent the same path.

### 4.2 Discovery

- Recursively discover worktree roots marked by `.git` directories or valid `.git` files.
- Discover nested independent repositories even when a parent directory is already itself a repository.
- Classify independent nested repository, actual Git submodule, linked worktree, and regular repository using Git metadata and relationships—not mere directory nesting.
- Do not recurse into internal `.git` object storage; do not follow symbolic links by default.
- Apply configurable directory exclusions; default examples: `node_modules`, `.venv`, `target`, `dist`, `build`.
- Support automatic detection of newly cloned, initialized, deleted, renamed, moved, and inaccessible repositories while the app runs.
- Newly cloned repositories appear only after sufficient metadata is available; distinguish **unavailable** from **deleted**.
- Maintain stable repository identity; paths are not names and display names need not be unique. A moved repository retains identity only when confidently matched.
- Preserve missing-repository tabs and bulk selection entries with blocked status until user resolves or removes them.
- Regular status refresh MUST NOT require full recursive workspace rediscovery.

### 4.3 Repository state

State is multidimensional, not a single enum. Include repository ID/path, current branch or detached HEAD, upstream, ahead/behind with freshness, staged/unstaged/untracked counts, conflicts, in-progress Git operations, last local commit, and errors.

**Separate repository identity from Git common directory and worktree identity.** Linked worktrees may share refs/object storage and have different indexes and working trees. Counts and activity deduplication must account for this.

## 5. User interface specification

The approved visual references are in `mockups/`. They establish the **direction**: GitKraken-inspired dark graphite UI, crisp blue accent, readable dense tables, colored ancestry lanes, restrained badges, and polished macOS chrome. The behavioral requirements in this document take precedence over errors or speculative features depicted in generated mockups.

### 5.1 Application shell

- Top: app/workspace selector, persistent navigation and open tabs, contextual Git actions.
- Left: resizable/collapsible workspace navigation and repository explorer.
- Center: Overview or selected repository/workflow screen.
- Optional right: contextual inspector; bottom: operation/status bar.
- Primary windows/panels resizable, widths remembered per workspace; system/light/dark theme and adjustable information density.
- Persistent **Overview** tab cannot close. Opening a repo activates or creates a closable tab. Switching tabs never clears bulk selections or commit drafts.
- Repository tabs retain selected view, commit, file, filters, and scroll location while open; restored tabs preserve saved view across restart. A newly opened tab defaults to Changes if dirty, History otherwise.
- Keyboard navigation, accessible labeling, focus handling, screen-reader names, and error feedback are required. Exact shortcuts are to be finalized during UI specification/testing.

### 5.2 Overview

- Default **flat table** with sortable columns: repository and relative path; branch/HEAD; staged, unstaged, untracked, conflicts; ahead/behind; last commit; trust; error/status.
- Alternate **hierarchical directory tree** with expandable filesystem folders and explicit repository nodes.
- Both views share filters/search and one workspace-level bulk selection model. Collapsing a tree node does not deselect descendants; selecting parent *repository* does not select children.
- Show attention counts and freshness of local status/remote-tracking data; avoid calling a repository "fully synchronized" merely because local tracking refs are current with the last fetch.
- Context inspector may show selected repository details; action controls must respect trust/eligibility.

### 5.3 Repository Changes

- Show staged and unstaged file lists; an individual path may appear in both lists when partially staged.
- Distinguish tracked modifications, additions, deletions, renames, untracked, type changes, conflicts, submodule gitlinks, and binary files.
- Stage/unstage individual files and selected groups. Initially full-file staging; hunk staging is next milestone and must fit the existing diff/selection UI.
- Selected diff mode clearly labels `HEAD↔index` vs `index↔working tree`; syntax highlighting, path, hunk context and binary summaries.
- Persistent per-repository commit composer with subject and optional body, optional AI assistance, staged count, and mandatory **Review Commit** action.
- Refresh from external file changes without losing selection or silently applying an outdated diff; stale review requires recapture.
- Discard/reset are **not** implied by the mockups and remain out of initial destructive-operation scope.

### 5.4 Repository History

- Three-pane graph-centric layout: branches/tags/remotes/worktrees, true commit DAG, and selected commit/file inspector.
- Graph renders actual parent/merge edges and distinct branch lanes, selected commit details and diff; history navigation works from Activity views.
- Fetch, Pull, Push, Branch actions remain distinct from local Refresh and gated by trust/review where write or network activity occurs.
- History is per repository; no artificial shared DAG across unrelated repositories.

### 5.5 Bulk Commit

- Separate workspace workflow: **Select → Prepare/Review → Authorize → Execute → Results**.
- Default selection review discovers all eligible staged, unstaged, and untracked changes; user can exclude paths before staging.
- Existing partial staging must be preserved unless explicitly included; stage-only vs review-all policy is specified per repository when needed.
- Allow shared message, per-repo override, AI-generated messages and one-off instructions.
- Show full per-repository proposed tree/files, hook policy, GitFleet Actions, warnings, commit messages, and dependent gitlink changes.
- Explicit batch authorization covers only selected repositories. Ineligible selected repos remain visible as blocked/skipped.
- Execute dependency-aware selected submodules before selected parent commits; if a parent gitlink changes from approved state, re-review that parent before commit.
- Sequential execution initially; stop on failure by default, preserve successful commits, never pretend the batch is atomic. After failure offer reviewed retry of outstanding work.

### 5.6 Bulk Push

- Entirely separate from commit; no automatic push following commit or AI automatic commit.
- Present selected repo, source branch, remote, destination ref, outgoing commit list, ahead/behind with freshness, dependency graph, and readiness.
- Guided upstream setup proposes known remotes/current branch names; user chooses and approves new tracking setup.
- Refresh remote knowledge with explicit network activity before final validation; distinguish actual remote availability from stale cached refs.
- Push selected dependent submodules first and verify required commit availability before publishing parents. Do not implicitly select unselected submodules.
- Block force pushing in bulk v1, unexpected divergence, missing destination/credentials, detached HEAD normal push, or dependency failure.
- Failures stop pending work by default. Successful pushes cannot be rolled back automatically; retry requires revalidation.

### 5.7 Workspace Activity and Operation History

**Workspace Activity (read-only Git-derived):** chronological timeline and activity lanes default to last seven days; GitHub-style contribution heatmap defaults to last twelve months; all three filter workspace-wide or to a single repository. Consider all local branches, deduplicate commits per repository and shared linked-worktree Git identity, filter by author/branch/date, and navigate from commit to repo History. Lanes reflect timestamps, **not cross-repo ancestry**. Date attribution defaults to committer time, labeled accurately.

**Operation History (GitFleet-derived):** persistent SQLite audit for stage/unstage, commit, fetch, pull, push, AI generations/actions, and bulk operations; filters for time/repo/type/result, per-repo phases, logs and recovery. Only GitFleet-executed operations appear; external Git operations affect status/activity but do not become fabricated audit events. Default retention: 90 days, configurable.

### 5.8 Hooks Manager, trust and recovery UI

- Hooks Manager enumerates effective local client-side hooks and resolved `core.hooksPath`; show filename, source, executable state, location, read-only content preview, and GitFleet policy.
- Per-repository hook default: **execute** or **bypass all**. Effective selection always shown during commit/push authorization; changing it for one operation does not silently change saved preference. Optional “remember preference” action.
- V1: inspect + normal hook execution + explicit bypass-all *per GitFleet operation*. Do not modify installed hook files. Individual hook toggles via controlled execution mechanism are planned **later**, pending compatibility validation.
- Trust Manager shows untrusted/trusted/revoked; individual trust grants, bulk Trust Selected, source path and identity. New repos inherit no trust. Trust is separate from approval for a particular operation.
- Recovery screen shows journal phase, actual refs/index state, completed/ambiguous actions, possible guarded recovery steps, and exact reasons for manual intervention.

## 6. Rust Core architecture and boundaries

### 6.1 Architectural rule

**Tauri is presentation infrastructure, not the application core.** All domain models, state transitions, execution policy, Git operations, AI orchestration, storage contracts, authorization, and recovery live in Rust modules with no Tauri/React dependency. React is presentation-only, aside from UI state and view composition.

Proposed initial architecture (do not over-engineer into numerous crates before needed):

```text
GitFleet
├── Rust Core (no UI dependencies)
│   ├── domain + typed application services
│   ├── discovery + workspace monitoring
│   ├── Git CLI adapter + status/history/diff
│   ├── controlled commit planner/executor + journal
│   ├── trust + authorization + hooks policy
│   ├── dependency-aware bulk orchestration
│   ├── AI connections, profiles, context, templates
│   ├── deterministic GitFleet Actions
│   └── TOML/SQLite persistence
├── Adapters
│   ├── Tauri IPC/event bridge (initial)
│   ├── CLI/diagnostic runner (tests and development)
│   └── future HTTP API / alternative presentation (not initially active)
└── React + TypeScript + UI components
```

Prefer a small Rust workspace using `gitfleet-core` (domain/application), `gitfleet-git` (Git process adapter), and lightweight `gitfleet-cli`/`gitfleet-desktop` adapters, splitting AI/storage/actions into crates **only when justified** by dependency boundaries and complexity.

### 6.2 Boundary responsibilities

| Boundary | Owns | MUST NOT own |
|---|---|---|
| React | Rendering, view state, user input | Raw Git/shell execution, credentials, trust decisions |
| Tauri adapter | Typed command/event transport, native dialog integration | Business rules or commit semantics |
| Core application services | Invariants, workflow state machine, authorization, orchestration | Tauri-specific types |
| Git adapter | Carefully constructed argv Git calls, parser/metadata, hook invocation adapter | User-facing approval policy |
| AI adapter | Provider protocol, JSON parsing, capability negotiation | Git writes or tool execution from model text |
| Persistence | Schema/migrations/atomic writes/durable journal | Source of truth for actual Git history/status |

No generic `run_command` or arbitrary shell API shall be exposed to the frontend. Rust services MUST reauthorize every sensitive request independently of frontend validation. Tauri capabilities are defense-in-depth, not substitutes for secure Rust handlers.

### 6.3 Contracts (conceptual)

```rust
struct WorkspaceId(Uuid);
struct RepositoryId(Uuid);
struct OperationId(Uuid);

enum SelectedContentSource { Index, WorkingTree }
struct SelectedPath { path: RepositoryRelativePath, source: SelectedContentSource }

struct CommitPlan {
    operation_id: OperationId,
    repository_id: RepositoryId,
    worktree_id: WorktreeId,
    branch_ref: FullRefName,
    expected_head: ObjectId,
    approved_tree: ObjectId,
    approved_message_hash: Digest,
    selected_paths: Vec<SelectedPath>,
    index_snapshot: IndexFingerprint,
    selected_worktree_snapshot: Vec<ContentFingerprint>,
    hook_policy: HookPolicy,
    approved_actions: Vec<ApprovedActionInvocation>,
}
```

This is a **contract sketch**, not compile-ready code. Concrete types must account for initial commits, detached HEAD, non-UTF-8 paths, Git SHA-1/SHA-256 repositories, submodules, and per-worktree metadata. An opaque core-issued authorization must bind to the final reviewed plan; editable JSON is not itself sufficient authorization.

### 6.4 Process and filesystem boundaries

- Spawn Git with argument arrays, never interpolated shell strings. Treat filenames/refnames and Git output as hostile input.
- Validate canonical repository roots and normalized relative paths; reject traversal/symlink escape, untrusted worktree paths and unexpected Git metadata locations.
- Use narrowly scoped process environment; do not inadvertently discard the user's required hook/signing configuration. Deny unsafe environment inheritance where appropriate.
- Handle binary/non-UTF-8 filenames and output with NUL-delimited Git modes where applicable; never parse porcelain status as naive newline-delimited text.
- Trust grants do not sandbox hooks. Trusted hooks can execute arbitrary code as the user; explain that in authorization UI.
- Long-running operations report structured progress, support cancellation at defined safe points, and persist indeterminate state for recovery.

## 7. Controlled commit transaction — security-critical

### 7.1 Fundamental invariant

**A commit created/published by GitFleet MUST have the exact approved tree and approved message.** The selected repo, worktree, branch, Git actions, hook policy and expected HEAD are also part of the authorization boundary.

Reviewing file *names* is insufficient; GitFleet must authorize actual blob/tree content and detect unexpected changes. No AI output or hook output broadens authorization.

### 7.2 Proposed transaction states

`Draft → SnapshotCaptured → Reviewed/Authorized → Preparing → HooksRunning → Revalidated → CommitObjectCreated → RefUpdatePending → RefUpdated → IndexReconciled → PostCommitProcessed → Complete`

Terminal/exception states: `Blocked`, `Failed`, `CancelledSafely`, `NeedsReauthorization`, `NeedsManualReconciliation`, `UncertainOutcome`.

### 7.3 Proposed execution procedure

1. Resolve and validate trusted repository/worktree/branch identity and supported Git state.
2. Capture the precise staged index entries, selected working-tree contents, expected HEAD, and selected GitFleet Action inputs.
3. Construct approved proposed tree in an isolated index, preserving all unselected staged/unstaged work. Show diff and message.
4. Acquire user authorization (for normal commit, mandatory review; for explicit AI Automatic, bounded approval before autonomous steps).
5. Revalidate relevant Git state and authorized content; apply only approved deterministic Actions with bounded, guarded file modifications; capture/rebuild the proposal when Actions change included content.
6. Execute supported hook lifecycle in a controlled preparatory phase and validate tree/message again. If a hook changes approved contents, block **before branch ref publication** and ask for renewed review. Preserve any hook side effects for inspection.
7. Create the exact commit object, with supported configured signing and correct parent/identity, using validated plumbing. Journal commit OID before ref publication.
8. Update expected branch ref with compare-and-swap semantics, rejecting stale HEAD/ref races. Journal the result durably.
9. Reconcile the real index to preserve unselected staged contents and unstaged/partially staged changes; never overwrite conflicting external modifications. On ambiguity, stop for manual reconciliation.
10. Run supported post-commit processing separately, capture side effects, verify the actual repository state, and record final status.

**Caveat:** Ordinary `git commit` can permit hooks to change the index/message before commit creation. `commit-tree` + `update-ref` affords stronger control in the tested subset but is **not** automatically hook-equivalent to native `git commit`. The implementation MUST declare supported hook configurations and fail closed for unsupported ones. A post-creation check alone does not enforce authorization.

### 7.4 Safety and crash recovery

- Durable operation journal stores operation ID, old/new HEAD OIDs, approved tree, message hash, index fingerprint, phase, results, and recovery prerequisites; avoids source-content dumps and secrets.
- Journal state transitions must be designed for durability on the target filesystem; interruption at every boundary requires test coverage.
- Recovery must determine whether ref publication occurred, whether reconciliation is already complete, and whether external state has changed. No automatic replay of an uncertain commit.
- A created-but-unreferenced commit object is a recoverable artifact, not evidence of published success. Capture its OID.
- Index locks and per-process mutexes alone cannot prevent unrelated external Git clients from changing state. Use validation and explicit recovery.
- For selected staged-only operations, preserve untouched index entries and worktree. For selected working-tree changes, update only targeted index entries when safe.
- Git hooks may modify the working tree while being allowed to leave the approved commit tree unchanged; preserve and report these side effects.
- A successful `post-commit` hook may generate new changes. Report them; do not pretend they were part of the approved commit.

### 7.5 POC evidence and production gate

**Evidence:** Hook Labs v1–v4 plus Rust Core POC v0.1.1. On macOS with Git 2.56.0, v0.1.1 compiled successfully; 24/24 acceptance tests and 6/6 core behavior tests passed; diagnostic runner reported 23 pass / 1 skip (positive GPG signing skipped because GPG unavailable). These tests cover the **experimental subset**, not universal Git semantics.

**Required additional production validation:** hook frameworks/Husky/Lefthook, signing (GPG and SSH), split/sparse/unmerged indexes, submodule gitlinks, linked worktrees/shared refs, SHA-256 repos, non-UTF-8 paths, symlink adversarial cases, overlapping Git processes, and platform portability. If a configuration cannot satisfy invariants, do not execute the write.

## 8. Repository trust, hooks and approvals

### 8.1 Trust

- All discovered repos are untrusted for write operations until explicitly approved. Minimal safe read-only inspection remains possible.
- Trust approval is per repository identity/worktree context; newly discovered nested children, submodules or worktrees do not automatically inherit a parent grant.
- Bulk trust grants must list exactly which repositories will be trusted.
- Trust revocation disables future GitFleet writes, Actions, and hook execution, without deleting Git data.
- Trust binding must be re-evaluated after metadata/path replacement, suspicious moves, or ownership changes.

### 8.2 Hook policy

- Default execute installed applicable local client-side hooks; optional remembered per-repository preference to bypass all for GitFleet operations.
- Always display effective hook policy and any bypass warning in the relevant operation review; changes per operation do not change stored preference unless user requests it.
- Inspection includes `core.hooksPath`, per-hook executable state and read-only content. Be explicit about scope: GitFleet cannot disable server-side hooks.
- Bypass-all must use an explicit supported per-command strategy and be audited. Do not silently bypass `prepare-commit-msg` by assuming `--no-verify` covers it.
- Individual hook selectivity and bulk hook toggles are **future** capabilities, pending tests of arguments, stdin, environment, cwd, exit codes, and hook frameworks.
- Hook code is not sandboxed by this design; trust grants signal acceptance of that risk.

### 8.3 Authorization policy

| Operation | Authorization |
|---|---|
| Stage/unstage | Explicit user action and trusted repository |
| Individual commit | Mandatory final review/confirm |
| Bulk commit | One mandatory batch review/confirm for explicit selected set |
| AI-assisted commit | Generated proposals reviewed/confirmed |
| AI automatic commit | Explicit *bounded* operation authorization, no per-repo repeated prompt unless scope changes |
| Push | Separate review/approval per individual or batch |
| Hook bypass | Highlighted effective policy as part of authorization |
| New Actions | Persisted approved definitions plus execution scope |

## 9. Bulk dependency-aware execution

- Represent submodule/parent relationships as first-class dependency edges, not mere path relations.
- For selected commits, topologically order chosen submodules before parents; never auto-select missing dependencies. If selected child changes parent gitlink candidate, parent must be re-reviewed.
- For pushes, verify referenced submodule commits exist on intended reachable remote before publishing parent; do not push unrelated/unselected submodules automatically.
- Stop on first failure by default; completed repositories remain committed/pushed; pending units require fresh validation to retry.
- Protect linked worktrees that share refs/common object storage; serialize conflicting write operations across shared identities.
- Always record per-repository status: `succeeded`, `failed`, `blocked`, `skipped`, `pending`, or `uncertain`; show precise dependency blockers.
- A batch is **not atomic**. Never advertise cross-repository transactional rollback.

## 10. GitFleet Actions

### 10.1 Separation of responsibilities

1. **AI instructions**: text guiding analysis and commit-message generation.
2. **Action definitions**: typed, approved, constrained operations that may prepare content.
3. **Rust executor**: deterministic validation, preview, execution, staging, recovery.

The AI may suggest a supported Action definition, but **cannot create executable permissions merely by writing text**. User approval converts a suggestion into a stored approved definition. Imported actions require local reauthorization.

### 10.2 Initial supported action: Update SemVer File

- Repository-relative target (`version` by example), strictly validated within repository.
- Read a SemVer value; calculate configured major/minor/patch increment or bounded AI suggestion subject to deterministic validation.
- Show old/new value in **Preview**. During execution, ensure expected old content still matches.
- Write atomically where feasible, stage only approved result, expose verified `version` template variable, include it in final commit review/AI context.
- On failure after mutation, record exactly what changed and guard against unintended double increment on retry. Do not blindly roll back when external edits may exist.
- No arbitrary AI shell commands, arbitrary scripts, or unrestricted write access in initial release.

### 10.3 Lifecycle

`Inspect → Preview → Approve Definition/Invocation → Execute → Validate → Stage → Commit → Audit/Recover`.

## 11. Local AI subsystem

### 11.1 AI connection and profile entities

**AI Connection:** provider kind, HTTP(S) endpoint/IP/hostname/port, optional auth via OS secure storage, timeouts and connection capabilities. Ollama primary; OpenAI-compatible endpoint adapter for LM Studio and similar providers. No mandatory cloud provider.

**AI Profile:** stable named profile linked to a connection, selected installed model, provider-supported generation parameters, **profile-owned editable system message**, and context/generation defaults.

**Assignment:** application default → optional workspace override → optional repository override → optional one-off operation override (most specific wins). In bulk, each repo may resolve to a different profile. No silent fallback on endpoint/model errors.

### 11.2 Connection/profile editor

- Enter provider and server IP/hostname/port (or full URL) in application Settings.
- **Test Connection & Discover Models:** query the provider for installed models, fill searchable dropdown, optionally inspect model size/context/capabilities when reported.
- `Refresh Models`, manual ID fallback for limited providers, and unavailable-model indicator without losing stored configuration.
- `Test Inference` uses synthetic diff input and the **current unsaved editor state**, returns response/error, latency and token metadata when available, without repo writes.
- Editable system message with restore default, duplicate profile, and generation parameter controls. Unset parameters mean provider defaults; do not force unsupported flags.
- Candidate parameter set: temperature, top-p, top-k, context size, output token limit, seed, min-p, repetition penalty, stop sequences, keepalive, thinking options and timeout—capability-gated.
- Concurrency defaults to **one request globally and one per connection**, configurable; limits count individual hierarchical-analysis requests, not just repositories.
- Model failure: report and offer Retry / Select Another Profile / Cancel. Automatic commit for affected repo blocks; no fallback without explicit choice and reanalysis.

### 11.3 AI template and instruction management

- **Structured Template Editor**: subject/body format, allowed types, scope, summary max length (example 72), rules, example commits and validation.
- Additive persistent instructions at application/workspace/repository levels; optional one-off instructions per individual or batch/repo operation.
- One selected effective output template with explicit overrides and **View Effective Configuration**.
- Static preview renders deterministic sample values without AI. Separate **Test with AI** uses read-only selected repository snapshot and current unsaved editor settings; simulates supported action previews but never writes.
- Profiles and templates are separate entities. Editable profile system messages do not override Rust-enforced security boundaries.

### 11.4 AI analysis pipeline

1. Capture exact approved/proposed changes, accounting for staged, unstaged and untracked selection.
2. Gather **bounded relevant context**: nearby code, relevant project docs, recent current-branch Git logs (~20 initial candidate default), and affected-file log history (~10 initial candidate default). These are configurable suggestions.
3. Apply safe content filtering, secret screening, explicit endpoint approval and context budget.
4. For large changes, hierarchically analyze logical groups; preserve file/hunk provenance, full coverage accounting, and omitted/truncated warnings.
5. Generate **up to three meaningfully distinct structured candidates per repository** with message, rationale, summary, concerns, coverage and rule validation.
6. Assisted mode: user chooses, edits or regenerates. Automatic mode: choose highest-ranked *valid* candidate; no invented confidence percentages; block if analysis coverage or result is unacceptable.
7. Expose **View AI Context**: diff, source/docs, Git history, stage-level summaries, actual filtered request payload, provider/model/version, omissions, and warnings.
8. Revalidate snapshot before GitFleet Action or Git write. AI can never add paths, command execution or new Actions to the authorized scope.

Source/diff/commit-history text is **untrusted data** and must never be promoted to executable instructions merely because the LLM emitted them. Prompt injection mitigated by strict boundary separation, scoped context and validation—not solely by prompting.

### 11.5 AI-assisted vs AI-automatic commits

| Mode | Message generation | Authorization | Execution |
|---|---|---|---|
| Assisted | Up to 3 candidates | Standard mandatory review | User confirms commit |
| Explicit Automatic | Up to 3 candidates; top validated candidate | Bounded user authorization before run | Rust may commit selected repos/files without per-repo prompt if nothing material changes |

AI automatic mode **never starts from watcher events**, never auto-pushes, and blocks on incomplete analysis, missing model, failed action or changed authorization snapshot.

### 11.6 Provider safety

- No source transmission until endpoint explicitly approved. HTTP LAN endpoints are unencrypted; warn and support HTTPS, secure auth and trusted certificates.
- No cross-origin redirects when sending source content. Provider and final destination host must be validated.
- Do not log source prompts/diffs/credentials by default; retain only model/profile/template version, input fingerprints, coverage, decisions, and bounded sanitized results.
- `.gitignore` does not ensure secret safety; tracked files may contain secrets. Screening is defense-in-depth; the user can inspect exact context before AI transfer when required.

## 12. Configuration, persistence and recovery

### 12.1 Hybrid storage

- **TOML:** human-readable app settings, saved workspace definitions, discovery/exclusions, monitor settings, AI connection metadata, profile/template references.
- **UTF-8 Markdown/text:** larger editable system messages and instruction bodies outside Git repos.
- **SQLite:** operation history, journal or durable metadata as appropriate, cached repository metadata, per-workspace navigation, bulk selection, activity cache, window layout and other structured state.
- Git itself remains authoritative for refs, index, status and commit history. Caches may be discarded/rebuilt; operation history/journal needs conservative recovery.
- Store auth secrets in OS credential store, not TOML/SQLite exports.

### 12.2 Suggested macOS layout (exact app identifier provisional)

```text
~/Library/Application Support/GitFleet/
├── config.toml
├── profiles/
├── prompts/
├── templates/
├── state.db
├── backups/
└── logs/
```

### 12.3 Backup and portability

- AI configuration ZIP import/export: connections (without secrets), profiles, system messages, templates, persistent instructions, and optionally approved Action definitions (reauthorize on import). Versioned manifest, safe archive extraction, conflict resolution, atomic apply.
- Full manual backup/restore of settings/workspaces/UI state, optional operation history. Repositories/credentials excluded.
- Automatic **change-aware local backups** while running, before migrations, rotate last ten automatic backups by default, validate archive and snapshot SQLite transactionally. No background backup daemon.
- Restore is previewed and validated, safety-backups current state first, never replays incomplete Git operations or blindly restores trust/action privileges.
- Database migrations versioned, backed up, and failure-safe; TOML written atomically.

## 13. Monitoring, performance and lifecycle

- Active workspace watcher detects repository content/metadata changes and new/deleted/moved repos.
- Debounce bursts; prioritize targeted status changes; periodic inventory/status reconciliation repairs missed events.
- Avoid recursively watching huge `.git/objects` or build/dependency directories; watch limits/errors visible rather than silently disabling monitoring.
- Monitor only while application process runs and active workspace is selected. Minimized windows keep monitoring; closing final window quits.
- During quit, stop accepting new operations, cancel queued jobs, attempt safe in-flight shutdown, leave journal-reconcilable state on hard interruption.
- Large workspaces must remain usable via incremental rendering, paging/virtualized tables and prioritized background work. Exact latency budgets to be benchmarked before signoff.

## 14. Error handling and reliability

### 14.1 Core error classes

`InvalidPath`, `UntrustedRepository`, `UnsupportedRepositoryState`, `StaleReview`, `IndexChanged`, `HeadChanged`, `HookRejected`, `HookChangedReviewedContent`, `SigningFailed`, `ActionFailed`, `AIUnavailable`, `AIInvalidOutput`, `RemoteDiverged`, `DependencyBlocked`, `TransactionInterrupted`, `NeedsManualReconciliation`, `StorageMigrationFailed`.

Every failure must have a stable machine-readable code, user-appropriate message, relevant repository/op ID, sanitized diagnostic detail, and permitted recovery actions.

### 14.2 Mandatory failure modes

| Event | Required reaction |
|---|---|
| Root/repo inaccessible | Report and retain identity; no empty-workspace misrepresentation |
| New nested repo | Discover; untrusted until approved |
| External file or index changes after review | Invalidate and recapture authorization |
| Hook adds staged content or rewrites message | Block before published commit; report side effects |
| Hook fails or signing unavailable | Stop and report actual Git state |
| Concurrent branch advance | Fail CAS update; retain object/journal for recovery |
| Crash before/after ref update | Reconcile journal and actual refs/index; no duplicate commit |
| Recovery index mismatch | No overwrite; require manual reconciliation |
| Child submodule fails | Dependent selected parents blocked |
| Bulk partial commit/push | Preserve successes, show incomplete and safe retry |
| AI failure/unavailable model | Block affected automatic commit; no silent fallback |
| Unsupported Git hook/index format | Fail closed for write if authorization invariant cannot be enforced |
| Failed backup/restore | Retain prior valid config; report error |

## 15. Security model and explicit prohibitions

### 15.1 Trust boundaries

- Frontend and untrusted repository data cannot authorize themselves.
- Tauri capability scoping protects frontend access, while Rust validates semantic permissions for every operation.
- Trust grants authorize the *possibility* of write operations and hook execution, not any specific commit/push.
- Hook scripts can execute arbitrary OS commands under current user and are not inherently safe because GitFleet invokes them.
- AI-generated strings and repository contents have no authority to add files, actions, endpoints, or bypass checks.

### 15.2 MUST NOT behaviors

- No arbitrary shell IPC, string-concatenated shell Git commands, or AI-proposed command execution.
- No automatic staging of unrelated files outside reviewed scope.
- No silent forced push, silent hook bypass, or silent AI model/endpoint fallback.
- No automatic remote destination selection for missing upstream without approval.
- No process that continues to monitor after the last window is closed.
- No claim of success until actual Git state and journal reconcile.
- No unbounded recursive symlink traversal or traversal into Git object storage.
- No automatic rollback of worktree files when newer external edits may exist.
- No credential export/logging, and no full source prompt logging by default.

## 16. Testing strategy and acceptance gates

### 16.1 Automated test layers

- Rust unit tests for domain validation, repository identity, authorization hashing, Action semantics, AI template rendering and profile resolution.
- Disposable Git integration fixtures for discovery, status, real commit/plumbing operations, hook mutations, partial staging, signing and concurrency.
- Real child-process crash tests around each durable transaction boundary, with restart/reconciliation.
- Hook-framework fixtures and Git configurations (normal/linked worktree/submodule, packed/split/sparse/unmerged indexes, non-UTF8 filenames, SHA-256 Git repos where supported).
- Tauri adapter contract tests, frontend accessibility/UI tests, end-to-end desktop flows.
- Local AI mock server tests + optional live explicit opt-in integration tests, never requiring a real developer repo or actual remote push in default CI.

### 16.2 Release-blocking invariants

| ID | Requirement / testable assertion |
|---|---|
| INV-001 | Untrusted repository cannot perform GitFleet writes or execute hooks |
| INV-002 | Approved committed tree equals actual published commit tree |
| INV-003 | Approved message equals actual published commit message |
| INV-004 | Any changed selected content/index/HEAD invalidates stale authorization |
| INV-005 | Partial staging, excluded staged entries and untracked files survive correctly |
| INV-006 | Unsupported hooks/configurations fail closed before unsafe publication |
| INV-007 | Crash recovery never silently replays an uncertain operation |
| INV-008 | Hook bypass requires explicit reviewed policy and audit |
| INV-009 | Push is never implicitly triggered by commit or AI auto commit |
| INV-010 | AI text has no direct filesystem/shell/Git execution rights |
| INV-011 | Pre-commit Action input/output modifications are previewed and bounded |
| INV-012 | Imported Actions are disabled until reauthorized |
| INV-013 | Outbound code transfer occurs only to approved AI destination |
| INV-014 | No unsafe cross-repository dependency violation in bulk commit/push |
| INV-015 | Tauri presentation adapter can be replaced without changing core domain logic |

Detailed scenario coverage is in `ACCEPTANCE_MATRIX.md`. M0 renamed this section's
former `SEC-001`–`SEC-015` to `INV-001`–`INV-015` without changing their meaning;
`SEC-*` exclusively identifies acceptance-matrix scenarios. See the crosswalk there.

## 17. Implementation roadmap

Milestones are ordered by prerequisite and must have explicit acceptance tests; a milestone is not complete merely because a mockup resembles the target.

| Milestone | Scope | Exit gate |
|---|---|---|
| **M0 — Baseline & hardening plan** | Bring POC v0.1.1 under main repo, audit experimental limitations, establish CI/test fixtures and Rust API boundaries | Reproduce current 30 Rust tests; documented unsupported Git states |
| **M1 — Core read-only workspace** | Real discovery/status/history/diff, nested relationships, trust identities, persistent workspaces; CLI-backed service | Disposable integration fixtures, path security, stable IDs |
| **M2 — Production controlled writes** | Authorization plans, full-file stage/unstage, safe selective commit, hook policy and minimum effective-hook inspection, signing, durable journal, crash recovery and persistent per-operation results | INV-001–INV-008 production matrix passes in supported environments |
| **M3 — Desktop foundation** | Tauri thin adapter, React shell, workspace Overview Table/Tree, tabs, monitoring, Changes/History base | UI state continuity, no business rules in frontend |
| **M4 — Graph & activity** | Commit DAG, inspector/diffs, timeline, lanes, contribution heatmap, operation audit | Graph correctness and dedup/activity tests |
| **M5 — Bulk operations** | Reviewed bulk commit/push, submodule order, missing upstream, partial results/retry | Dependency and partial-failure tests, no auto push |
| **M6 — AI foundation** | Connections/model discovery, profiles/system messages, parameters/concurrency, templates/instructions, context inspection/test | Provider mock tests, no data leakage, explicit approval |
| **M7 — AI commit + Actions** | Hierarchical analysis, 3 candidates, assisted/explicit automatic commits, deterministic SemVer action | INV-009–INV-013 and idempotence tests |
| **M8 — Polish and release readiness** | Expanded hook inspection, backup/import/export, accessibility, performance, packaging, hardening | All release gates and platform tests |
| **Post-v1** | Hunk-level staging; selective individual hook management; additional GitFleet Actions; multiple windows/other adapters | Separate scoped designs and tests |

**Pragmatic sequencing:** UI may be scaffolded in parallel once M1 contracts are stable; do not enable any real write path until M2 security gates pass. M4/M6 can proceed with mock data while relevant core APIs mature, without promoting mock outputs to production.

## 18. Acceptance examples (behavior-driven)

1. **Multi-repo workspace:** Given a root with independent nested repos, a submodule, and a linked worktree, Overview lists/classifies them without double-counting shared identity; no discovered repo becomes trusted automatically.
2. **Adaptive tab:** Given a dirty repo with no restored explicit view, opening its tab shows Changes; after manual History selection, external file modifications don't forcibly switch back.
3. **Partial staging:** Given a file both staged and subsequently modified, commit of approved staged content leaves the unstaged changes intact.
4. **Review boundary:** Given a hook that stages an extra file or rewrites commit message, the commit does not publish without renewed authorization.
5. **Crash recovery:** Given SIGKILL before or after ref publication, restart reconciles the journal and does not repeat an already published commit.
6. **Bulk selection:** Given a selected parent repo with nested children, only the parent is selected until each child is explicitly selected.
7. **Submodule batch:** Given selected child and parent with a new child commit, child commits first; changed parent gitlink requires renewed approval before parent commit.
8. **Push destination:** Given a local branch without upstream, GitFleet offers guided remote/ref selection and requires final push review; never guesses publication destination.
9. **AI example:** Given `version = 1.4.2` and approved SemVer patch action, preview shows `1.4.3` without changes; actual run modifies/stages version only after bounded authorization and regenerates context before commit.
10. **AI failure:** Given unavailable model mid-batch, the affected repository stops without substituted model; successful other commits remain recorded, and no push occurs.
11. **External edit:** Given Codex modifies a selected file after review, GitFleet invalidates prior approval and requests a new snapshot.
12. **Quit:** Given the last window closes, the application stops monitoring and doesn't leave a service running; restart rescans workspace.

## 19. Open verification register / risks

| ID | Topic | Risk | Resolution gate |
|---|---|---|---|
| RISK-001 | Hook lifecycle equivalence to Git native commit | High | Hook-manager integration matrix + explicit unsupported cases before M2 release |
| RISK-002 | Narrow crash window / journal durability | High | SIGKILL at every phase; fsync/recovery tests |
| RISK-003 | External index/worktree races | High | Mutation stress and conflict-safe reconciliation |
| RISK-004 | Worktrees/common ref concurrency | High | Shared Git metadata tests |
| RISK-005 | Large nested scans + watcher OS limits | Medium | Large synthetic workspace benchmarks |
| RISK-006 | AI data exfiltration/secrets | High | Endpoint approval + exact-context viewer + redaction tests |
| RISK-007 | Hook-bypass behavioral incompatibility | High | Review exact hook execution semantics by operation |
| RISK-008 | Malicious imported archives/config | High | Parsing and path traversal adversarial tests |
| RISK-009 | SemVer Action crash/retry double bump | High | Expected-old-value guard + restart/idempotence tests |
| RISK-010 | Remote drift between fetch and push | Medium | Final server rejection handling and retry review |

## 20. References and provenance

### 20.1 Official technical reference

- Git hooks and lifecycle: https://git-scm.com/docs/githooks
- Git commit object creation and signing: https://git-scm.com/docs/git-commit-tree.html
- Git ref compare-and-swap updates: https://git-scm.com/docs/git-update-ref.html
- Tauri v2 capabilities: https://v2.tauri.app/security/capabilities/
- Tauri security: https://tauri.app/security/
- Ollama API: https://docs.ollama.com/api

### 20.2 Project evidence

- Python GitFleet Hook Labs v1, v2, v3, v4.
- `gitfleet-rust-core-poc-v0.1.1.zip` and submitted macOS `cargo build` / `cargo test` / diagnostic results (24 acceptance + 6 core tests passed; GPG-positive diagnostic skipped).
- Approved UI concept boards: `mockups/01-core-workflows.png`, `mockups/02-ai-and-settings.png`, `mockups/03-operations-and-reliability.png`.

### 20.3 Implementation instruction

This document is the **requirements baseline**, not permission to operate on a real repository, run bulk Git writes, or connect to a production AI server. Codex should begin by reviewing this specification, auditing the existing Rust POC against it, and producing a bounded implementation plan. All tests default to disposable repositories and mock AI providers. Any live Git write/network access requires explicit user authorization.


## 21. M1 implementation reconciliation — 2026-11-01

M1 was authorized after M0 and implemented as a headless read-only workspace
milestone. [M1 contracts](architecture/M1_READ_ONLY_WORKSPACES.md) document concrete
identity/path/persistence/process decisions. [Implementation status](IMPLEMENTATION_STATUS.md)
records local validation, exclusions and next authorization boundary. M1-A01–A15
in the acceptance matrix map the approved milestone gates to runnable evidence.

Core remains a single crate; the conceptual multi-crate suggestion in section 6.1
is not a requirement to split it. Experimental writes remain opt-in and unsafe for
real projects. Desktop scaffolding, watchers, graph rendering, production trust/
approval/writes, remote operations, AI, Actions and backup UI remain later work.
Only the core portion of shared M1/M3/M4 requirements is fulfilled.

The initial validated baseline is macOS arm64 with Git 2.56.0 and Rust/Cargo 1.99.1.
The implementation declares Git >=2.56.0 and Rust >=1.99; versions beyond the tested
pair and Linux/Windows are not implicitly validated. Safe inspection suppresses
system/global executable configuration and rejects content filters/sparse-index
configuration. Native byte paths and SHA-256 inspection are modeled/tested within
the documented envelope. The original pre-production write risks remain open.


### M2 implementation reconciliation — 2026-11-01

The user authorized M2 implementation, including the proposed bounded support
policy. [M2 contracts](architecture/M2_CONTROLLED_WRITES.md) and the
[M2 acceptance ledger](ACCEPTANCE_MATRIX.md#11-m2-acceptance-gates--controlled-writes)
now describe the production service's implemented scope. M0/M1 default inspection
and the historical POC remain separate. Core still has one crate and no presentation
dependency. The M2 gate closes only for local macOS/APFS, supported files refs and
ordinary index states; advanced indexes, hook managers/reference-transaction Execute,
non-UTF-8 filesystem creation, other platforms and OS/power-loss validation remain
explicit exclusions. Positive GPG/SSH acceptance is no longer a pending Mac gate.
Thirteen SIGKILL boundaries establish process-crash evidence, not universal storage
durability. Operation records are retained without automatic pruning; configurable
90-day retention remains a proposed default, not an implemented M2 deletion policy.
M3 and later require separate authorization. No desktop, bulk, remote or AI write
capability is implied by M2 service completion.


### M3 implementation reconciliation — updated 2026-11-02

M3 was subsequently authorized. A Tauri/React adapter, transport-neutral Core
application services, scoped IPC, navigation persistence and monitoring are now
implemented. Core remains one crate independent of presentation dependencies.
[The M3 contracts](architecture/M3_DESKTOP_AND_IPC.md) and
[M3 acceptance ledger](ACCEPTANCE_MATRIX.md#12-m3-acceptance-gates--implementation-with-native-gates-open)
identify automated evidence and open native gates. The earlier agent access denial
remains historical; the [November 2 checkpoint](M3_NATIVE_ACCEPTANCE_HANDOFF_2026-11-02.md)
records later user-observed native approval, read/write, signing and restart evidence.
M3 is not complete, and ordinary desktop writes remain disabled in Rust and UI.
Actual cancellation, guarded recovery, close/forced-interruption, remaining security
cases and accessibility acceptance still require permitted testing.
M0–M2 safety boundaries and exclusions remain; M4 and later are not authorized.
