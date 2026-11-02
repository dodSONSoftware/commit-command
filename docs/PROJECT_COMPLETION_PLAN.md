# GitFleet project completion plan

Prepared November 2, 2026. Status: execution authorized by the user in this chat. Implement the remaining milestones, validate them, save milestone reports in `docs/reports/` and make local milestone commits. Do not push. Native app access was successfully verified after the user added the GitFleet app rule and restarted Codex. Release publication and external service credentials remain separate boundaries.

GitFleet should be completed by preserving its existing Rust services, closing M3 native acceptance, then implementing M4 through M8 in bounded packages. Each package below has a deliverable and an observable exit condition. The first release target is macOS; Linux and Windows acceptance must be resolved explicitly against the release requirements before claiming the entire specification complete.

Execution checkpoint: **resumed at the user's request**. Foreground Cancel, close
during a running hook, forced interruption/guarded abort and serialized background
polling now have current native evidence; see the
[latest report](reports/M3_STALE_REVIEW_WINDOW_2026-11-02.md). Continue remaining
native acceptance and unfinished packages. No push or ordinary-write default change.

## Completion scope and governing contracts

The [specification](GITFLEET_SPECIFICATION.md), [decision register](DECISIONS_AND_OPEN_ITEMS.md), [acceptance matrix](ACCEPTANCE_MATRIX.md), [current status](IMPLEMENTATION_STATUS.md), and [November 2 native checkpoint](M3_NATIVE_ACCEPTANCE_HANDOFF_2026-11-02.md) govern execution. The historical M0–M2 contracts continue to govern supported Git behavior. This plan proposes implementation details; it does not change confirmed requirements.

Completion means that the specified workspace, individual Git, graph/activity, bulk commit/push, AI, SemVer Action, backup/portability, accessibility and release workflows work together with evidence for their acceptance criteria. Every requirement must end in one of three explicit dispositions: accepted implementation, documented supported-envelope exclusion with required fail-closed behavior, or an agreed deferral. An untested requirement does not become an exclusion automatically.

Hunk staging, individual hook toggles, arbitrary Actions, destructive reset/discard/amend/rebase workflows, multiple active workspaces, multiple presentation windows and additional adapters remain outside v1 unless the product scope is changed. Fetch is needed for the specified push workflow; the exact Pull and Branch actions remain an unresolved product decision, not implicit permission to add merge or checkout behavior.

The following rules apply throughout:

- Preserve existing tracked and untracked work. The working tree is the implementation baseline; HEAD alone omits much of the product. Do not commit, push or release without a later explicit instruction.
- Keep Core in one crate, independent of Tauri/React. Add cohesive modules when needed; split crates only for a demonstrated dependency or ownership benefit.
- Keep Git execution, authorization, trust, workflow state and recovery in Rust. Extend typed application requests and generated contracts; do not expose shell, arbitrary Git argv, SQL or raw storage endpoints.
- Preserve exact approved trees/messages, unrelated staging and working bytes, independent worktree trust, literal native paths, shared-repository locking and no replay of uncertain operations.
- Use disposable repositories, signing keys, remote fixtures and provider mocks for tests. Existing user repositories, personal keys, live remotes and real AI endpoints require their own authorization.
- Native policy restrictions remain binding. A denied native gate stays blocked until permitted access or a qualified operator supplies the required evidence. Keep independent implementation work moving without pretending the gate passed.

## Original planning baseline (historical)

This section records the initial planning assessment before P0 and later
implementation. Current package states and evidence are in
[execution status](EXECUTION_STATUS.md); the historical counts/findings below must
not be read as current capability limits.

The repository scan covers Core modules and binaries, integration suites, desktop/native source, generated contracts, build and validation scripts, manifests and lockfiles, CI, specifications, historical evidence and mockup references. Dependency/build directories and Git internals are not application source. This is a planning assessment, not an exhaustive security audit of every line or dependency.

| Area | Existing implementation | Remaining completion work |
|---|---|---|
| M0 and M1 | Restricted inspection, native identities/paths, recursive discovery, structured status, local history/diffs, TOML/SQLite workspace persistence | Preserve regressions; complete product UI details and later platform acceptance |
| M2 | Controlled stage/unstage/commit, trust, configured signing, hooks, immutable review, durable phases and guarded recovery | Preserve narrow macOS/APFS support envelope; extend deliberately for future operation kinds |
| M3 | Application facade, monitoring, native host, separate review page, Table/Tree, tabs/drafts and minimal operation results | Remaining native gates, known UI issues, product read/write enablement decision |
| Recent fixes | Workspace removal and Changes/History isolation have two component regressions | Native retest of both patches |
| M4 | Commit parents/refs and minimal operation storage exist | DAG layout, graph UI, activity queries/views and full operation audit |
| M5 | Repository relationships and per-repository write engine exist | Batch orchestration, remote policy/execution, dependency verification and push UI |
| M6 and M7 | `src/ai/` and `src/actions/` contain placeholders | Provider/configuration/context pipeline, candidates, bounded automatic authority and SemVer execution |
| M8 | Atomic config writes, migration backups, private state and local dev bundle scripts exist | General backup/import/export/restore, retention, performance/accessibility hardening and distribution |

The initial follow-up validation in [implementation status](IMPLEMENTATION_STATUS.md#follow-up-validation--2026-11-02) records 16 renderer tests, lint, TypeScript/Vite build, formatting and generated-contract checks passing. The five-configuration Core matrix and native mock/build counts are historical November 1 evidence. Later native outcomes are user-observed and lack exact test-time binary attestation. They must not be erased or promoted into an attested run for current sources.

Initial code findings (historical; subsequent work is tracked in execution status):

- `history.rs` supplies actual parents and bounded paging, but the desktop facade currently requests local-branch history with a fixed page size. Graph browsing needs typed scope/ref requests and stable pagination.
- `application/review.rs` produces long textual reviews from serialized proposals. Readability should improve through Core-produced operation-specific content while authority remains in Rust.
- The native host serializes Core work on one worker, and the bridge polls jobs. Future AI/network jobs must not block cancellation, status, shutdown or timely reconciliation.
- The read process adapter disables transport protocols. Remote operations need a separate explicit policy; weakening safe inspection globally would violate the existing boundary.
- SQLite is at schema 3 for desktop navigation. New migrations must preserve M2 operation payloads, active recovery evidence and old state backups.
- `controlled-writes` rejects non-Unix compilation; its actual write support is narrower, macOS/APFS. The desktop selects application services, which select controlled writes. Cross-platform desktop delivery therefore requires platform work, not just packaging.
- CI uses generic runner toolchains while the project requires Git 2.56+ and Rust 1.99+. Hosted results cannot be assumed; prerequisite provisioning and GPG availability must be deterministic.
- The early requirement tables retain historical POC/M1 status labels even where later milestone ledgers supersede them. A complete requirement crosswalk remains necessary.

## Execution order and package tracking

Follow this primary sequence:

`P0 baseline → M3 closure → M4 graph and audit → M5 local batches → M5 remote and push → M6 AI foundation → M7 Actions and AI commits → M8 integration and release`

Backup primitives should be strengthened before later schema migrations; accessibility and performance checks accompany each UI package. Do not postpone foundational data safety until the end. Independent read-only UI work can proceed during a native access blocker when that work is authorized, while M3 remains open.

Start each package by recording its requirement IDs, precise inputs, affected modules, risks and current authorization. Finish with the changed behavior, actual tests/evidence, unresolved issues and next package. Maintain a small execution record alongside this plan rather than repeatedly rewriting historical handoffs. Use states Planned, In progress, Blocked, Awaiting native acceptance and Accepted. P0 is accepted. M3 has current fixes and selected native passes but remains partial. M4 and M5A have automated and selected native evidence; local remote inspection and separately reviewed branch observation are implemented. Native control is available again; no current Mac-lock blocker is assumed. Fetch, authenticated transport/upstream/push and M6–M8 remain unfinished. See [execution status](EXECUTION_STATUS.md) for the current next work. Existing capabilities in the historical baseline table are not new work.

## P0 Establish the current execution baseline

### P0A Preserve evidence and reconcile requirements

1. Record Git status including untracked files; inventory relevant current source hashes without staging or committing.
2. Inspect the later temporary evidence manifest and preserve only approved non-secret evidence under a new durable validation directory. Exclude private keys, secret keyrings, raw sensitive app state and credential-bearing URLs. Record provenance and hashes. If evidence is missing, retain the checkpoint as historical testimony and identify only the affected retests.
3. Record exact source/build inputs, binary hashes, feature selections, environment and disposable fixture identities for every new run. Do not retroactively attest old binaries.
4. Map every WS/UI/GIT/SEC/BULK/PUSH/AI/ACT/STATE/RELEASE requirement to current owning code, tests and remaining package. Reconcile early historical labels with later milestone evidence without editing raw historical logs.

Exit: an authoritative current requirement crosswalk, preserved evidence inventory and an exact starting point for future checks. No old success is silently reclassified as current acceptance.

### P0B Make validation reproducible

Check installed prerequisites, run the existing Core and desktop validation scripts against disposable fixtures, and record results under a new run directory. Provision exact compatible Git/Rust/GPG tooling in CI or fail with a clear prerequisite error. Preserve all five feature configurations, mandatory isolated signing positives and explicit ignores. Do not install global tools or update dependencies merely to clear unrelated warnings.

Treat the handoff's 10 moderate development advisories as historical. Refresh dependency findings during hardening, then make narrowly justified upgrades with regression checks. Production-only audit results do not replace the development/build dependency review.

Exit: reproducible local commands and a CI definition with deterministic prerequisites. A hosted run is accepted only after it actually runs under separately authorized Git publication/workflow execution.

## M3 Complete the desktop foundation

### M3A Finish the observed UI defects

Affected areas: `app/desktop/src/views/App.tsx`, `src/stores/model.ts`, `src/review.ts`, styles, native window setup, and `app/core/src/application/{review,state}.rs`.

- Natively retest workspace removal and History isolation, including removal with no active replacement, opening another saved workspace, pending state saves and late read responses.
- Restore main window position/size through native state, clamp restored bounds to available displays, and test moved/disconnected displays. Keep native geometry distinct from workspace panel preferences.
- Replace the tab-like trust inspection control with a clear action. Refresh trust/eligibility after relevant native results and observed invalidation, using narrowly scoped notifications or polling as hints to request authoritative Core state.
- Make authoritative reviews operation-specific: repository/worktree/branch, exact paths and sources, before/after content or patch, effective hooks, applicable signing/message, and recovery consequences. Keep byte-exact inspection available without forcing users through raw arrays. Stage/unstage reviews omit irrelevant commit/signing fields. Escape hostile/control/bidi content consistently.
- Refresh or clear submitted file selections only after checking actual execution results and new status; preserve them after cancellation/failure when still valid. Fix stale counts after publication.
- Constrain narrow composer controls, long paths/messages and review scrolling. Preserve the user's deferred theme polish decision; fix usability/accessibility issues without broad cosmetic redesign.
- Extract components from the large App view only when implementing a specific workflow makes the separation useful. Preserve the tested state model and IPC behavior.

Exit: component/Core tests for changed behavior and native reproduction/retest evidence for each reported defect. A prettier review must still bind to the same stored plan and Rust confirmation capability.

### M3B Complete cancellation and recovery acceptance

Use the [existing native acceptance plan](M3_NATIVE_ACCEPTANCE_PLAN.md) and the specific blocked HookRejected operation from the checkpoint.

1. Identify the exact owned process, launch arguments, binary and data directory. Inspect guarded recovery in the ordinary read-only build and prove no Git mutation, hook replay or duplicate operation.
2. Create a fresh disposable slow-hook fixture. Wait for an observed start signal, actually request Cancel while the operation is running, and independently inspect outcome, owned child processes, sentinel count, refs/index/worktree and durable state.
3. Test review cancellation/replacement/close, main-page reload, main-window close during reads and writes, and immediate app-data lock reacquisition after exit.
4. Use a documented procedure to interrupt an exactly identified fixture process before and after publication where practical. Verify restart/recovery against actual refs/index/receipts; distinguish blocked, unpublished, reconciled, published-with-failure and uncertain outcomes. Never replay commits/hooks or remove unproven locks.
5. Complete remaining hostile content, caller/ACL, context switch and review expiration cases. Use trusted test timing facilities where appropriate; do not expose time/approval overrides through IPC.
6. Test keyboard/focus and VoiceOver in main/review/native dialogs, measure contrast, and verify large/long content remains usable. Record defects and fixes separately from passes.

Exit: every applicable M3-A01 through A20 criterion has evidence or an explicitly resolved scope disposition. The slow-hook HookRejected observation is never labeled cancellation or timeout.

### M3C Decide ordinary desktop write availability

After M3 gates pass, prepare the exact feature/default change and supported-envelope text for review. Enabling normal writes is a separate product decision under the existing contracts. Preserve per-worktree trust, independent native approval and fail-closed support checks. Test both read-only and enabled configurations; unsupported hosts remain blocked.

Exit: the approved distribution behavior has end-to-end native evidence. Completing acceptance alone does not silently change build defaults.

## M4 Implement graph activity and operation audit

### M4A Add graph queries and deterministic layout

Extend `history.rs` and typed application contracts for local branches, tags, available tracking refs and explicit refs. Use true parent OIDs, including merges, roots, shallow/missing boundaries and disconnected histories. Add deterministic graph rows/lanes and continuation state across pages in a presentation-independent module; React renders geometry and interaction. Never join unrelated repositories into one DAG or infer ancestry from timestamps.

Tests: merges/octopus merges, divergent branches, tags pointing to the same commit, unborn/shallow repositories, missing objects, equal timestamps, SHA-256, pagination and stale cursors. Local queries must not fetch.

### M4B Build graph and activity views

Add the branches/tags/worktrees navigator, virtualized history rows, graph lanes, exact selected-parent/file inspector and navigation anchors. Add workspace/repository timeline and activity lanes defaulting to seven days, plus a twelve-month contribution heatmap. Aggregate all local branches and deduplicate by logical repository plus OID across linked worktrees. Label committer-time attribution and define calendar/timezone boundaries explicitly.

Tests: independent repos sharing identical OIDs, linked-worktree deduplication, merges counted once, author/branch/date filters, month/year/DST boundaries, navigation into History, and stale/missing object handling. Benchmark paging and interaction with a generated large history.

### M4C Generalize operation audit without rewriting the write journal

Add typed audit records/projections for future batch, remote, AI and Action operations. Keep the existing M2 execution journal authoritative for recovery; do not replace its payloads with UI summaries. Add paged filters by time, workspace/repository, type and result, with phase details and permitted recovery actions. External Git activity is never fabricated as an app operation.

Specify retention before deletion: configurable history retention may remove eligible terminal summaries, but unresolved/uncertain recovery records, required artifacts and active-operation references remain protected. Preserve migration compatibility with schema 3.

Exit for M4: GIT-023, STATE-002/003 and the expanded STATE-004 are demonstrated with independent Git oracles, deterministic layout/aggregation tests and accessible native navigation.

## M5 Implement reviewed bulk and remote workflows

### M5A Plan and execute local commit batches

Add a Core bulk module and durable batch/item records. Build eligibility from explicit selected worktrees, all eligible path sources and per-repository exclusions; preparation does not stage managed files. Support shared messages and per-repository overrides. Build dependency edges only from actual submodule metadata/gitlinks; directory nesting alone imposes no ordering.

Execute selected children before selected parents, serializing shared-common-directory conflicts through existing locks. A new child commit changes the parent's gitlink proposal and therefore requires a fresh parent review; batch approval cannot cover an unknown future parent tree. Never silently select an omitted child. Persist item outcomes and stop pending work on first failure by default. Retry only unfinished items after fresh validation; never recreate completed commits.

Tests: multiple independent nested repos, selected/unselected submodules, multi-level dependencies, invalid/cyclic relationships, partial staging, linked worktrees, child failure, changed parent gitlink, cancellation, crashes between items, revoked trust and safe retry. Deliver the Select → Prepare/Review → Authorize → Execute → Results desktop flow.

### M5B Introduce explicit remote operation policy

Add a dedicated Rust remote service and request/review types for remote inspection, explicit fetch, push planning and guarded upstream configuration. Do not relax the read-only adapter's protocol bans. Resolve and display effective fetch/push URLs, destination refs and configuration origins; account for URL rewriting, multiple push URLs and unsupported protocols/remote helpers. Reject ambiguous destinations rather than guessing.

Choose supported transports and credential interaction, host/certificate verification, proxy/SSH configuration and hook behavior explicitly. Keep credentials in native secure storage or approved helper flows; scrub logs and URL userinfo. Fetch is a mutation of local Git metadata and a network operation: require explicit scope/authorization, journal actual effects and revalidate reviews after changed refs/configuration.

Use local bare remotes and isolated loopback fixtures by default. Cover missing credentials, auth rejection, cancellation, disconnected transport and bounded process output. Do not require public services for CI.

### M5C Add separately reviewed individual and bulk push

Push review binds worktree identity, source ref/OID, effective destination, outgoing commits, hook policy, upstream changes and dependency evidence. Refresh remote knowledge only through explicit network activity. Reject non-fast-forward/divergent, unsupported detached, ambiguous-destination and force push cases. Recheck source state immediately before execution, handle server rejection, and observe actual remote state after uncertain interruption. Never automatically retry an ambiguous publication.

For selected parent pushes, prove required child commits are available on the intended reachable child remote; use reachability evidence rather than a local tracking ref alone. Push selected dependencies first, never unselected children. Present missing upstream as a separate explicit reviewed config change. Implement and test applicable pre-push hook stdin/arguments and explicit bypass semantics; unsupported managers/configurations remain fail closed.

Deliver separate push review/results screens. No commit or AI success path calls push automatically.

Exit for M5: BULK-001 through 007 and PUSH-001 through 006 pass, including server drift, partial publication, safe retry and independent remote-ref verification. Pull and Branch actions are implemented only if their exact non-destructive scope has been resolved separately.

## M6 Implement AI configuration and safe provider access

### M6A Add connections profiles templates and secure storage

Populate `app/core/src/ai/` with bounded configuration, profile resolution and template/instruction modules. Add versioned TOML/text/SQLite storage as appropriate, migration backups and a narrow credential-store interface supplied by the native adapter. Core never depends on Tauri. Connections contain provider/endpoint/capabilities; profiles own model, editable system message and optional generation parameters. Resolve application → workspace → repository → operation overrides explicitly.

Implement independent connection discovery and inference tests, searchable installed models with manual fallback, refresh/unavailable indicators, profile duplication/default restore, capability-gated parameters and structured subject/body template rules. Diagnostics use current unsaved editor input. Static preview is deterministic and cannot invoke AI or Git writes.

### M6B Build transport policy and job scheduling

Start with Ollama and the specified OpenAI-compatible local provider adapter. Verify current official provider contracts at implementation time before selecting HTTP/TLS/credential libraries. Bound connect/read/total time, response size and structured output; validate approved scheme/host/port and effective redirect destination. Explicitly approve destinations before source transfer. Offer HTTPS and approved auth; disclose HTTP transport in the approval UI. Prevent redirects from changing source-transfer authority.

Enforce global and per-connection concurrency, initially proposed as one each. Count every hierarchical analysis request. Separate long-running provider work from the serialized storage owner using immutable captured inputs and scoped results; keep cancellation/status/shutdown responsive. Do not introduce a general background daemon or renderer-owned scheduler.

Tests use provider mocks for missing models, malformed JSON, partial streams where supported, unsupported parameters, redirects, oversized results, cancellation, auth failures and shutdown. Errors offer retry/profile selection/cancel; never silently fall back.

### M6C Implement bounded context transfer and inspection

Capture selected changes and relevant bounded source/docs/history with provenance. Apply secret screening, file/source exclusions and explicit token/byte budgets before transport. Do not assume `.gitignore` provides secrecy. Build the context viewer from the exact filtered payload and resolved effective profile/template/settings used by the provider, with omitted/redacted material and reasons. Source-bearing diagnostic payloads stay ephemeral unless retention is explicitly chosen; persistent audit stores safe metadata/fingerprints.

Exit for M6: AI-001 through 009, AI-012 and AI-017 pass at their applicable foundation boundary, including unsaved-state diagnostics, hierarchy resolution, no unauthorized endpoint traffic, secure secret storage and responsive cancellation. Hierarchical generation coverage closes in M7.

## M7 Implement SemVer Actions and AI commit workflows

### M7A Add deterministic SemVer preview execution and recovery

Populate `app/core/src/actions/` with a typed SemVer-file Action only. Resolve an exact repository-relative target, accepted syntax/prerelease rules, increment strategy and expected old bytes. Show before/after preview without writing. Persist approved definition/version and invocation scope; imported definitions start disabled.

Execute bounded file replacement with expected-old-content and filesystem identity checks. Record intent and before/after hashes before mutation, preserve modes/newline policy, then record the actual result. Crash recovery distinguishes not applied, applied exactly once and conflicting external changes. Never recompute an increment on retry or blindly restore an old file over a newer external edit.

Rebuild the final content proposal and AI context after a successful Action. For ordinary commits, approval binds the final resulting tree/message; do not feed a pre-Action approval into the M2 executor. Stage only the approved paths/result through the existing controlled-write boundary.

Tests: major/minor/patch, prerelease decisions, invalid syntax, binary/unexpected content, traversal/symlinks, changed old value, crash before/after rename, stale index, repeated retry and external edit after mutation. Verify exact committed blobs and no unrelated file changes.

### M7B Add hierarchical analysis and assisted messages

Group large captured changes deterministically, analyze bounded groups, retain file/hunk provenance and coverage, then generate up to three structured distinct candidates. Validate template rules, provider output and omissions in Rust; expose rationale/concerns and real coverage, without invented confidence scores. Missing model, unacceptable coverage or invalid output blocks automatic execution.

Assisted mode lets the user select/edit/regenerate a message, then uses ordinary immutable commit review. Generating messages and testing templates cannot stage or commit.

### M7C Add a separate bounded automatic authorization contract

The existing M2 capability authorizes an exact tree/message. It must not be reused to imply approval for an unknown future AI message. Define a separate user-issued bounded run contract covering selected repos/files/snapshots, effective endpoint/profile/template, allowed Action invocation/output constraints, acceptable candidate validation and expiration/cancellation policy.

Rust may select a valid candidate and issue an internal exact-plan approval only after verifying every run constraint and the final tree/message. Material drift or a changed parent gitlink stops for renewed authorization. Review the exact permitted SemVer change before the run; AI can suggest only supported bounded values. Recovery never resumes an automatic run's Git writes without validating the run lifecycle and necessary fresh authorization.

Compose the batch coordinator, context pipeline, deterministic Actions and existing write engine. Watchers never initiate generation or automatic commits. No AI output adds a file, endpoint, helper, Action permission or push operation.

Exit for M7: AI-010 through 016 and ACT-001 through 003 pass with prompt-injection, partial-coverage, model failure, stale snapshot, cancellation/crash and double-increment tests. INV-009 through 013 remain enforced throughout assisted and automatic flows.

## M8 Complete backup hardening and release readiness

### M8A Implement backup portability restore and retention

Add Core archive/backup services and settings UI. Deliver versioned AI configuration export/import, full manual app-state backup/restore and change-aware automatic local backups while the process runs. Exclude repositories/credentials; exported endpoint approvals and trust/Action authority cannot become active merely through import.

Use consistent SQLite snapshots and a manifest of content/version/checksums. Reject archive traversal, absolute paths, symlinks/special files, duplicate/case-colliding paths and decompression bombs. Preview conflicts and schema compatibility, make a safety backup, validate in a temporary destination and apply atomically where possible under exclusive ownership. Recover partial application failures without overwriting the last valid state.

Define explicit handling of interrupted operation records/artifacts and current repository identity on full restore. A restored backup cannot replay operations, revive old capabilities/trust, or overwrite current unresolved recovery context. Test restores against changed repositories and newer/invalid schemas.

Proposed defaults of ten automatic backups and 90-day history retention require recorded disposition. Do not prune unresolved recovery evidence. No backup daemon persists after last-window close.

### M8B Finish product usability and scale

Complete effective hook/config inspection and explain unsupported Execute configurations without adding speculative manager support. Implement any remaining specified Overview columns/sorts/filters, density/collapse settings, subject/body composer behavior, syntax highlighting/binary summaries and trust/recovery screens. Review all UI-005 details against the spec rather than relying on initial mockups.

Benchmark generated inventories of 100, 1,000 and 10,000 repos with documented partial/timeout behavior, and histories large enough to exercise paging. These are proposed fixture sizes, not promised support limits. Record hardware, cold/warm scan, targeted refresh, watcher bursts/overflow, memory, IPC payload sizes and interaction latency. Set measurable acceptance budgets from those results before signoff. Introduce virtualization/incremental queries where measurements justify them. Never remove safety bounds to make a benchmark pass.

Test keyboard, focus restoration, VoiceOver, measured contrast, long/native paths, large reviews, zoom and narrow layouts across all completed workflows. Revisit deferred cosmetic changes only as a separately scoped polish choice.

### M8C Resolve platforms and distribution

Keep local macOS/APFS as the initial validated write target. Resolve RELEASE-003 Linux and RELEASE-004 Windows explicitly: either implement and test those platform envelopes or approve a macOS-only release scope with the remaining platform requirements visibly deferred. Do not call the entire original release matrix complete under a Mac-only test run.

For Linux support, implement the required filesystem/locking/durability/process/native-dialog behavior rather than bypassing APFS checks. For Windows, first address non-Unix compile guards, native paths, child process trees, locks/rename semantics, state-directory defaults and credentials. Each platform needs actual signing/hook/recovery and UI evidence before a support claim.

Choose final product identifier, app-data migration, license/distribution policy, supported OS/architectures, Git dependency strategy, signing identity and update policy. The Core MIT file does not settle the whole product licensing decision. Build installers/bundles, verify signatures/notarization where applicable, include licenses/notices and checksums, and test fresh install, upgrade, migration and uninstall without deleting user state. Avoid an updater service unless explicitly included in release scope.

Release validation must run against the exact packaged artifact and source/features, including ordinary write behavior, supported signing, trust review, partial staging, restart/recovery, last-window exit, secure storage and backup restore. A dev bundle is not distribution evidence. Prepare release notes and user/developer documentation; publication, tags, pushes and release upload remain explicit external actions.

Exit for M8: STATE-005 through 007, ACT-004, UI-005 and the agreed RELEASE-001 through 005 platform scope pass; all INV-001 through 015 have current supporting evidence.

## Decisions required at the relevant boundary

| Decision | Needed before | Proposed handling |
|---|---|---|
| Ordinary desktop write default | M3C | Keep disabled until native criteria pass and the default change is approved |
| Pull/Branch behavior and supported remote transports/helpers | M5B | Start with explicit fetch and non-force push; obtain exact scope for additional mutations |
| Credential and trusted hook compatibility | M5B and M8 | Native secure storage, reviewed supported helper policy and explicit fail-closed exclusions |
| AI endpoint/redirect/certificate policy and context screening | M6B/C | Approve explicit destinations; capability-gate parameters and bound payloads; verify official provider contracts at implementation time |
| Automatic AI authority versus immutable commit approval | M7C | Separate bounded run authorization from internal exact-plan approval; security review before execution exposure |
| SemVer syntax/prerelease and default bump policy | M7A | Present exact deterministic preview; do not infer a policy from AI output |
| Retention and backup defaults | M4C and M8A | Protect recovery first; record disposition of proposed 90-day/ten-backup values |
| Performance budgets and accessibility details | M3B and M8B | Set measured criteria, include VoiceOver and contrast evidence |
| Platforms, identifier, OS/Git distribution, license and signing | M8C | Decide before release artifact implementation; no implicit support claims |

Resolve each decision before its dependent work. Routine reversible implementation within an authorized package can proceed without repeated confirmation. A later instruction authorizing execution of this plan should be recorded once with its scope; preserve explicit external-action restrictions.

## Validation and evidence protocol

For each changed behavior, write focused tests that assert observable results using independent oracles. Do not replace native acceptance with component mocks or mirror implementation details in assertions. Expand validation for new failure modes and shared-boundary changes.

| Change | Required checks |
|---|---|
| Core reads/storage/identity | Relevant integration tests, default feature tests, format and strict Clippy; migration/locking/concurrency cases as affected |
| Writes/trust/recovery/Actions/bulk | Exact tree/message/index/worktree oracles, negative and interrupted paths, shared-directory contention, positive SSH/GPG, applicable feature matrix |
| Contracts/native host | Generated contract drift, handler ACL/origin tests, input bounds, feature/default graphs, native builds and actual lifecycle/dialog checks |
| React/review/styles | Component regressions, typecheck, lint, format, production build and native accessibility/visual checks appropriate to behavior |
| Remote | Isolated remotes, destination/ref and dependency proof, auth/transport failures, drift, hook stdin, cancellation and uncertain publication |
| AI | Provider mocks, approved-destination/secret tests, actual-payload comparison, hierarchy/budgets/concurrency, invalid-output and prompt-injection cases |
| Backup/release | Malicious archives, interrupted restore, recovery-protection tests, clean install/upgrade and exact artifact hashes/features |

Existing baseline commands, run from the repository root when executing validation:

```sh
GITFLEET_TEST_GPG=/absolute/path/to/isolated-test-gpg CARGO_NET_OFFLINE=true sh app/core/scripts/validate.sh
CARGO_NET_OFFLINE=true sh app/desktop/scripts/validate.sh
```

The signer path is a verified tool executable; tests use disposable keyrings. Offline mode assumes cached dependencies. Record prerequisite failures accurately. Use a new evidence directory/run identity rather than overwriting historical logs. Focused frontend iteration can use `npm --prefix app/desktop test`, `run lint`, `run build`, `run format:check` and `run contracts -- --check` before the full milestone gate.

New evidence records package/requirement, operator/time, source revision or source hashes plus dirty state, exact binary/features, environment, fixture IDs, command/action, expected/actual result and artifact hashes. Scrub secrets before writing records. Evidence may be Passed, Failed, Blocked, Unsupported or Not run; unsupported cases are accepted only with an explicit supported-envelope decision and tested rejection where applicable.

## Definition of project completion

The project is complete for its agreed release scope only when:

1. Every requirement has current implementation/evidence or an explicit accepted exclusion/deferral, with the release scope stated accurately.
2. All applicable release invariants pass across individual, batch, remote, AI and Action paths; neither renderer input nor model output can broaden authority.
3. Exact packaged binaries pass native acceptance and lifecycle/recovery tests on every claimed platform and filesystem envelope.
4. Installation, upgrade, migration, backup/restore, credential handling, retention and supported Git/tool discovery are usable and documented.
5. Open failures are triaged as fixed, release-blocking or explicitly accepted limitations; skips/blocks remain visible.
6. Source, dependency versions, licenses/notices and artifact hashes are recorded; no release claim relies solely on historical test counts or source-only review.
7. The release is prepared for review. Commit, push, signing-service use, publication and deployment occur only within explicit authorization.

## First execution package

Begin with P0A: preserve the later non-secret native evidence, establish current source/binary provenance and create the full requirement crosswalk. Then run P0B and finish M3A/B using the existing checkpoint without repeating passed checks unnecessarily. The first new functional work is the remaining review/trust/layout/window-state defects; the first unfinished native operation is read-only guarded-recovery inspection followed by an actual timed Cancel action on a fresh disposable fixture. Stop at unresolved policy or authority boundaries while continuing independent authorized work.
