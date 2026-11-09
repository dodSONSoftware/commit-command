# Commit Command implementation and completion runbook

Prepared November 8, 2026. Baseline: `17ea585` on `main`, clean working tree before documentation changes. Audience: the engineer or coding agent completing the macOS first release.

Complete the packages below in dependency order. Existing inspection, controlled Git writes, bulk commits, authenticated remote services, individual publication, guided push queues, SemVer Actions and assisted AI messages are implementation foundations to extend and validate. Do not rebuild them from the original milestone descriptions. Ordinary desktop Git trust grants, writes and remote execution remain disabled until package P14 closes.

This runbook specifies implementation work and test obligations. It does not certify acceptance, authorize commits/pushes/releases, grant access to personal credentials or approve contact with live services. Apply the current operator's authorization, not historical authorization sentences in repository documents.

## Navigation and authority

Read these three files together:

1. This file: sequencing, implementation contracts, decision tasks, file ownership and package exits.
2. [Completion test catalog](COMPLETION_TEST_CATALOG.md): fixture construction, test procedures, assertions and evidence obligations. Test IDs used below resolve there.
3. [Requirement coverage](COMPLETION_REQUIREMENT_COVERAGE.md): all 109 original requirements, 15 invariants and 20 M3 criteria assigned to packages/tests.

Product requirements remain in [the specification](../GITFLEET_SPECIFICATION.md) and [decision register](../DECISIONS_AND_OPEN_ITEMS.md). [EXECUTION_STATUS.md](../EXECUTION_STATUS.md) remains the live acceptance ledger. Dated reports establish only their stated source, fixture and envelope. This runbook is the execution specification for the remaining work; it does not replace that live ledger or redefine accepted exclusions.

If a conflict is found: identify the exact requirement/decision and conflicting implementation instruction; preserve current behavior; complete independent work; record the conflict in P00. Do not resolve product policy by guessing. An unresolved decision has an owner, exact outputs and a dependent package below. A blocked test remains blocked; it never becomes an accepted exclusion solely because a facility is unavailable.

## Baseline and existing foundations

| Area | Owning source | Preserve and extend |
|---|---|---|
| Discovery and identity | `app/core/src/discovery.rs`, `workspace.rs`, `inspection.rs`, `domain.rs` | Native byte paths, stable worktree/common-directory identities, explicit unavailable/partial status, no executable inspection side effects |
| Git reads | `diff.rs`, `history.rs`, `graph.rs`, `activity.rs`, `process.rs` | Bounded argv transport, no implicit network, exact DAG/parent comparisons, read-only index behavior |
| Local transactions | `writes/{policy,transaction,store,adapter,application}.rs` | Exact tree/message proposals, hooks/signing, durable journal, owned-lock recovery, fresh authorization |
| Bulk local writes | `application/{bulk,bulk_state}.rs` | Explicit selections, no child auto-selection, child-first ordering, changed-parent reauthorization |
| Remote operations | `remote/*.rs`, `writes/{remote,fetch,fetch_import,upstream,ssh_transport,push_*}.rs` | Separate observation/capture/import/upstream/check/publication authority, destination binding and uncertainty |
| AI | `ai/{config,change,store,template,provider,jobs,context,capture,capture_file,candidates}.rs` | Optional setup, exact reviewed source transfer, endpoint-bound native credentials, typed text-only candidates |
| Actions | `actions/{semver,version,version_file}.rs`, `writes/version_action.rs` | Frozen before/after result, retained original, single application, separate staging/commit |
| App facade and persistence | `application/{mod,state,review,audit,monitor,exploration}.rs` | One storage owner, epochs, strict IDs/scope, serialized durable updates, independent AI jobs |
| Native host | `app/desktop/src-tauri/src/{main,security,shutdown,native_termination,native_dialog,ai_credentials,ai_token_dialog,window_state,host_options}.rs` | Actual caller/page/origin checks, independent native approval, bounded jobs, guarded shutdown |
| Renderer | `app/desktop/src/views/`, `stores/model.ts`, `services/bridge.ts`, styles | Rendering/input only; authoritative eligibility, validation and execution remain Rust |

Fresh review checks at this baseline: default Core 66 passed/1 ignored; application integration 41 passed; renderer 123 passed; ordinary host 20 passed/1 ignored worker; all-feature host 23 passed/1 ignored worker. TypeScript, ESLint, contracts, frontend format/build, native format/debug build and strict all-target/all-feature Core/host Clippy passed. Loopback and Keychain suites required execution outside the restricted sandbox. Full signing/crash/transport matrix, native GUI and packaged-release checks were not rerun in that review. Logs were temporary review outputs, not durable release evidence.

Recorded unresolved observations: intermittent post-load blank startup; one completed-batch transition self-cancelled before approval; large discovery runs time out with partial inventories; real-model suggestions can overstate changes. See P02, P03, P04 and P09. Do not infer a root cause from these observations.

## Execution protocol for every package

1. Record package, requirement IDs, source revision, `git status --short`, affected files and fixture IDs in a new report/evidence directory. Preserve staged, unstaged and untracked user work.
2. Check package dependencies and required decisions. Use the decision record's explicit values; missing values block only dependent work.
3. Reuse existing behavior/tests named below. Mark each applicable catalog case `Retained evidence`, `New test required`, `Needs rerun`, `Blocked`, or `Accepted exclusion`, with a concrete source/report reference. An historical pass alone cannot attest a changed boundary or exact release artifact.
4. Implement the smallest coherent change, including errors, cancellation, migration and UI states. Keep existing one-crate Core architecture. New modules below are intended responsibilities, not a requirement for a class hierarchy or a crate split.
5. Add/update focused observable-behavior tests. For each changed shared authorization/storage/process boundary, run its dependent suites. Never weaken a safety guard to make a fixture pass.
6. Generate contracts from Rust; never edit `*.generated.ts` by hand. A new native command requires capability permissions, Rust caller checks and forbidden-caller tests. Prefer typed facade requests over adding commands.
7. Run the package validation set and relevant native catalog cases. Use test-only barriers/markers to prove an operation reached its intended phase before cancellation/interruption.
8. Review the complete diff and check links/contracts. Update live status, requirement coverage disposition and report with actual outputs. No commit unless requested in the active session.
9. Stop package work when exit criteria pass. Do not add equivalent Cancel/close variants merely to increase test counts.

### Evidence layout and records

Create `docs/validation/completion-<package>-<YYYY-MM-DD>-<unique>/` and a concise `docs/reports/COMPLETION_<PACKAGE>_<YYYY-MM-DD>.md`. Keep raw fixture roots outside the checkout. Every case result contains:

- Case/package/requirement IDs; status `passed`, `failed`, `blocked`, `unsupported` or `not_run`; operator and timestamp with timezone.
- Source HEAD plus dirty diff fingerprint, tool versions, OS/build/architecture/filesystem, features, artifact SHA-256 and exact launch arguments with credentials omitted.
- Fixture manifest, before/after oracle hashes and documented allowed deltas, phase-start proof, action, expected result, actual result and exit code.
- Repository refs/index/worktree/mode preservation, server request/ref counts where relevant, durable record comparison, process cleanup, SQLite integrity and writer-lock reopen.
- Native accessibility/visual evidence where required; a component test does not count as VoiceOver or native approval evidence.
- Explicit failure/block reason and rerun instruction. Preserve failed attempts; append corrected baselines rather than replacing them.

Never retain private signing keys, Keychains, credential tokens, raw personal app state or project source prompts in Git evidence. Fixture-generated request bodies may be retained when clearly synthetic and scrubbed. Persistent application audit must use fingerprints/metadata, not raw source payloads.

### Validation commands

Run from repository root. Offline commands assume locked dependencies are cached. Do not remove `--locked` or upgrade dependencies to resolve an environment failure.

```sh
git status --short
python3 app/core/scripts/check-prerequisites.py
python3 app/core/scripts/check-prerequisites.py --desktop
CARGO_NET_OFFLINE=true cargo test --manifest-path app/core/Cargo.toml --locked
CARGO_NET_OFFLINE=true RUST_TEST_THREADS=4 cargo test --manifest-path app/core/Cargo.toml --features application-services --test m3_application --locked
npm --prefix app/desktop run typecheck
npm --prefix app/desktop run lint
npm --prefix app/desktop run contracts -- --check
npm --prefix app/desktop run format:check
npm --prefix app/desktop test
npm --prefix app/desktop run build
CARGO_NET_OFFLINE=true sh app/desktop/scripts/validate.sh
```

For controlled-write/process/recovery changes, run the complete Core script with a verified installed signer and a new evidence output directory:

```sh
GITFLEET_TEST_GPG=/absolute/path/to/verified/gpg CARGO_NET_OFFLINE=true sh app/core/scripts/validate.sh /absolute/path/to/new/evidence/core
```

The executable path is a required operator input, not a literal pathname to copy. Tests use isolated keyrings, not personal keys. Rust/Cargo >=1.99, Git >=2.56.0, Python 3, C/native toolchain, Node >=22.12 and npm >=10 are current declared prerequisites. Check scripts for exact platform requirements. Local transport needs loopback permission; Keychain tests need permitted native facility access. Classify sandbox denials and rerun through approved facilities; do not change product/tests to suppress them.

The Core runner covers default, experimental, controlled, application-services and all features on macOS, plus the disposable lab. Linux explicitly does not execute macOS/APFS write acceptance. The desktop runner covers ordinary/all-feature host tests, generated contracts, lint/format/type/build, strict Clippy and ordinary release binary build. It does not perform GUI acceptance or packaging.

## Dependency order

| Package | Work | Dependencies | Gate |
|---|---|---|---|
| P00 | Reconcile executable backlog and decisions | None | All |
| P01 | Reusable fixture/evidence and reproducible validation support | P00 | G02–G11 |
| P02 | Remaining startup, shutdown and recovery criteria | P01 | G02 |
| P03 | Discovery progress, monitoring and scale | P01, performance decision | G03 |
| P04 | Complete UI behavior and accessibility | P01; P03 for large inventories | G03/G05/G06 |
| P05 | Remaining bulk commit acceptance | P01/P02/P04 | G05 |
| P06 | Supported remote/upstream/hook/auth compatibility | P01/P02/P04, supported-envelope decision | G06 |
| P07 | Individual publication acceptance | P06 | G07 |
| P08 | Guided push queue acceptance | P05/P07 | G08 |
| P09 | AI editor/provider/assisted workflow closure | P01/P04 | G09 |
| P10 | Hierarchical AI analysis | P09, analysis policy | G09 |
| P11 | Action definition and final-context integration | P09, definition policy | G09/G10 |
| P12 | Bounded automatic commits | P05/P10/P11/P13A, automatic policy | G09 |
| P13A | Backup format and consistent snapshot primitives | P01, backup policy | G10 |
| P13B | Import/restore UI, automatic backups and retention | P13A/P11; P12 if automatic records exist | G10 |
| P14 | Ordinary write exposure | Accepted applicable P02–P08/P04, explicit decision | G04 |
| P15 | Reproducible CI, hardening and license closure | P01; ongoing with other packages | G11 |
| P16 | Exact macOS package and release acceptance | P03–P15, distribution decisions | G11 |

Do P13A before any new durable schema migration, including persistent AI audit, automatic-run, Action-definition or restore schema changes. P09 editor/provider work can precede P13A; its audit migration cannot. P03/P04/P09/P13A/P15 can advance independently when another native facility is blocked. Final acceptance is integrated; this table is not a reason to skip a dependent workflow's tests.

## P00 Reconcile work and resolve blocking decisions

### P00.1 Establish the executable checklist

- Reconcile the G07–G09 rows, next-work queue and desktop README with the current implementation. Individual push, guided queues, secure AI auth/context and assisted candidates exist. Preserve dated historical sections.
- For every coverage-table row, attach exact current code/tests and the newest relevant accepted report; classify remaining implementation versus acceptance versus explicit deferral.
- For every broad phrase such as “broader topology,” create named catalog cases and identify whether it has retained evidence. Use the catalog below as the minimum explicit list; add an uncovered required scenario only with its requirement pointer.
- Keep G01 closed within its documented envelope. Rerun its relevant cases after authority/IPC changes; do not reopen it as general unfinished implementation.
- Preserve macOS-first, Pull/branch mutation deferral, no hunk staging, no selective hook toggles, no arbitrary Actions, no updater/daemon unless separately approved.

### P00.2 Decision records

Complete the supplied [decision worksheet](COMPLETION_DECISIONS.md) during execution. Each entry has decision ID, choices, owner, selected value, date/authorization, rationale, affected tests and source changes. Do not label an unselected proposal approved. The worksheet supplies concrete proposals; it does not choose unresolved user policy.

| Decision | Exact output required | Owner and blocked work |
|---|---|---|
| DCP01 performance | Supported inventory/history sizes and numeric scan/first-use/target-refresh/render/cancel/memory/IPC budgets; release benchmark hardware and warm/first-pass definition | Product owner approves measured proposal from P03; blocks scale signoff, not profiling |
| DCP02 platform/distribution | Minimum macOS, arm64/x86_64 scope, local filesystem envelope, Git bundled versus prerequisite, installer format/channel, stable identifier/data migration, signing/notary identity, update policy | Product owner; blocks packaging/signoff |
| DCP03 backup | Archive version/limits, local backup location, debounce/interval, manual history/artifact inclusion policy, disk-space floor, 10-backup/90-day proposal disposition, encryption policy if required | Product owner approves behavior; engineer specifies format; blocks P13 execution defaults |
| DCP04 automatic authority | Lifetime, selected set, maximum repos/requests/bytes/time, endpoint consent model for all stages, ranking/tie rule, exact required coverage and action integration, retry behavior | Product owner approves bounded design; blocks P12 exposure |
| DCP05 hierarchical analysis | Group/request/depth/output bounds, context allocation, complete-coverage definition, deterministic ranking order and acceptable assisted omissions | Engineer supplies bounded design, product owner approves automatic consequences; blocks P10 automatic eligibility |
| DCP06 Action definitions | Stored definition identity/version/scope, association to repositories, allowed increments/prerelease policy, verified template variable behavior, import reset UI | Product owner where new behavior is required; preserve existing explicit plain-value semantics; blocks definition/automatic integration |
| DCP07 supported Git/auth envelope | Named supported hooks/signers/helpers/transports/ref/index/worktree cases; exact tested rejection for exclusions; interactive OS prompt procedure | Engineer reconciles existing accepted envelope; product owner approves expansions/exclusions |
| DCP08 accessibility | Shortcut map, minimum window/layout behavior, required zoom settings, accessibility acceptance standard | Engineer proposes, product owner accepts unresolved choices; blocks final UI signoff |
| DCP09 licensing | Product distribution policy, third-party shipping closure and notice delivery | Product owner/legal authority; blocks distribution |

Every task depending on an undecided value must display `Blocked on DCPxx` with the exact missing field. Do not invent a default or silently implement the broadest interpretation. Routine implementation choices already fully specified here need no separate decision.

Exit: every requirement has an owning package/test and every policy dependency has a recorded selected value or explicit blocked status. Tests: DOC01–DOC03.

## P01 Validation and fixture infrastructure

Owners: `app/core/tests/support/`, Core validation/provisioning scripts, desktop fixture/native acceptance support, `.github/workflows/core.yml`.

1. Reuse current fixture generators and support modules. Add only missing reusable fixture/oracle capabilities from F01–F09 in the catalog. Fresh roots must be outside all managed repositories and carry the exact marker required by their test facility.
2. Use private mode-0700 fixture ancestry and short macOS temporary aliases for SSH/GPG where needed. Existing CI prepares private `RUNNER_TEMP` backing with a short alias; keep that security property. Do not use writable untrusted ancestry or put state inside the checkout.
3. Add deterministic phase-ready barriers only behind non-default test features. No PID/phase/approval overrides through production IPC. Native acceptance support must remain sealed to fresh fixtures.
4. Build an independent filesystem oracle: no-follow traversal, exact bytes/symlink targets/modes, selected metadata identities, explicit allowed deltas and hashes. Disable optional Git locks in observation commands. Never use an oracle that refreshes the real index.
5. Record server refs independently of the client UI and use fixture HTTP/helper/hook counters. Prove a held operation is running with marker + durable phase + owned process, rather than sleeping for an assumed interval.
6. Add a machine-readable case result validator requiring the fields above and rejecting `passed` without its oracle/phase/artifact fields. Validate only evidence schema and references; never infer success from log text alone.
7. Add cleanup ownership checks: stop only recorded fixture-owned PIDs/process groups, remove only marked roots from this run, do not kill unrelated curl/Git/SSH/GPG processes. Preserve failure evidence before cleanup.
8. Prove ordinary feature graphs exclude test support and experimental writes. Keep local/loopback fixtures explicit, not an implicit production transport option.

Tests: INF01–INF08. Exit: a new operator can create fixtures, run each test layer and collect verifiable results with the documented commands; no hidden personal services/keys required.

## P02 Startup lifecycle and recovery closure

Owners: native `main.rs`, `shutdown.rs`, `native_termination.rs`, `native_acceptance.rs`, renderer `main.tsx`, `App.tsx`, bridge, Core application review/state, existing native reports.

### P02.1 Diagnose retained startup failures

1. Extend bounded startup diagnostics, if necessary, to distinguish native launch, page-load lifecycle, React initial commit, facade readiness, renderer error and first usable screen. Keep diagnostics free of drafts/source/credentials. A native “document finished” event alone is not usable UI readiness.
2. Reproduce on fresh state, saved state, first restart after host loss, renderer loss and ordinary Quit. On an actual blank capture screenshot, native window/process state, console/error state and readiness markers before reload. Record recurrence rate and exact artifact.
3. Fix only a substantiated cause. Guard the causal ordering with a focused regression and first-launch/restart native case. Do not add arbitrary sleeps, unbounded retry reloads or speculative workarounds.
4. If not reproduced, retain the issue as open with diagnostics and a bounded observation plan. Clean repetitions cannot prove it fixed.

### P02.2 Complete phase inventory

Map every durable phase actually implemented in local writes, import, upstream, publication, queue and Action to: existing Core crash test, existing native recovery evidence, remaining case, or accepted exclusion. Local checkpoints currently include Preparing, HooksRunning, Revalidated, ObjectReady, IndexLockWritten, PublicationPending, ReferenceUpdated, Published, HeadLogWritten, IndexRenamed, IndexReconciled, PostProcessing and Complete. Use actual enums/functions, not only proposed states in the original specification.

Exercise missing native phase behavior using fixture-only fault support; add a fixed test action only where no existing facility can reach the boundary. Distinguish before-ref, after-ref/before-reconciliation and after-reconciliation/post-hook outcomes. Inspect actual refs/index and receipts before offering recovery. Never silently remove foreign locks or replay hooks/commits.

### P02.3 OS and review lifecycle

- Finish permitted actual Dock Quit and coordinated session termination cases. If access is denied, retain explicit policy-block status. Do not simulate Dock by direct AppKit and claim equivalence; do not terminate the user's session unattended.
- Preserve unanswered-sheet safe deferral: no implicit approval, explicit Cancel then fresh Quit. Immediate sheet cancellation/exit is not established by existing evidence.
- Confirm graceful exits cancel queued/running work, persist the paired draft snapshot, release storage locks and leave no owned helper children. SIGKILL is different: a hook surviving host loss is a documented limit, not proof of graceful cleanup. Any new crash-cleanup claim needs a specific design and test.
- Ordinary restart presents recovery/history but creates no authority or execution. Scope removal/switch/revocation invalidates pending/issued confirmations.

Tests: LIFE01–LIFE12, REC01–REC08. Exit: every remaining G02 criterion has accepted evidence or an explicitly approved scope disposition; retained startup issues are fixed with causal evidence or remain visible release blockers.

## P03 Discovery monitoring and scale

Owners: `discovery.rs`, `workspace.rs`, `inspection.rs`, `application/{monitor,exploration,mod}.rs`, scale script, model/Overview/scan-status views.

### P03.1 Make incomplete discovery able to progress

Existing behavior retains partial results after the 15-second deadline. Do not remove that bound or cancellation semantics. The implementation must avoid endlessly rescanning the same prefix of a large workspace.

1. Separate directory enumeration from costly repository inspection sufficiently to schedule bounded units. Maintain deterministic traversal and a pending-work queue/cursor scoped to workspace root identity, exclusion revision and scan generation.
2. Publish a partial generation with discovered/inspected counts, pending work and scan issues. Previously known unvisited repositories stay unavailable/pending, not deleted. Only a completed traversal can establish absence.
3. Continue remaining units on explicit continuation or bounded active-workspace scheduling. Rescan/root/exclusion changes invalidate the old cursor. Explicit cancellation keeps the prior accepted inventory and cannot activate stale queued results.
4. Prioritize visible/selected worktrees for status without declaring inventory complete early. Status refresh must not trigger a full recursive discovery.
5. Keep relationship classification complete only for verified edges; pending relationships block dependent write planning. A later complete generation replaces provisional state coherently.
6. Avoid a worker per repository; retain bounded process concurrency and storage-owner responsiveness. Measure before choosing concurrency changes. Do not raise read deadlines to mask serialized cost.
7. If persistence is needed for continuation, first use P13A migration backup primitives. Persist identifiers/cursors only, never trust authority. Reject stale roots and invalid cursors on reopen.

### P03.2 Watchers and measurements

- Reconcile missed/overflow events, visible watcher failures and periodic rediscovery. Only active workspace is watched. Excluded caches/build directories and Git object stores remain excluded.
- Benchmark release Core/desktop plus debug baseline at 100, 1,000 and 10,000 empty repos; add representative dirty histories/submodules/linked worktrees. Report first-pass versus warm accurately; fixture creation does not establish OS-cache-cold conditions.
- Record traversal/inspection/relationship time separately, time to first usable rows, complete generation, targeted refresh, event bursts, cancel/close latency, memory, payload sizes and DOM node count.
- Preserve the existing 88–93 entries/15-second observation as historical comparison. Run at least three repetitions per chosen fixture/temperature and report range/median; do not report percentile estimates unsupported by sample size.
- Obtain DCP01 numeric acceptance budgets after measurements and rerun against them. If 10,000 is outside the approved release envelope, document that decision and tested partial behavior rather than claiming capacity.

Tests: SCAN01–SCAN12, PERF01–PERF06. Exit: scans make measurable forward progress; no false deletions or stale publication; monitoring/status remain responsive; approved budgets pass on the stated hardware/artifact.

## P04 User interface behavior and accessibility

Owners: `views/App.tsx`, `stores/model.ts`, Core `application/state.rs`, views for graph/activity/audit/bulk/remotes/AI/Actions, styles and window state.

### P04.1 Overview and persistent shell

1. Complete sortable specified columns: repository/path, branch/HEAD, staged/unstaged/untracked/conflicts, ahead/behind, last commit, trust and error/status. Current persisted `sort` accepts only `path` and `changes`; extend it through a backward-compatible typed migration. Define stable ID/path tie breaks, null/unavailable placement and sort direction; persist a chosen sort per workspace. Show `aria-sort`.
2. Keep free-text search distinct from status filters; support the specified attention categories without changing selection. Add collapse/expand state and adjustable information density/sidebar collapse if missing; persist per workspace without losing widths. No implicit child selection.
3. Display status observation time, incomplete inventory and remote knowledge age separately. Do not label cached tracking equality “synchronized with server.”
4. Preserve selected IDs when filtered/collapsed; unavailable/missing selected entries remain visible as blocked in operation preparation. Historical superseded identities remain separate from current IDs.
5. Persist selected commit/file/parent/filter/scroll and manual Changes/History choice across tab/restart. A new tab alone gets adaptive default. Cancelled/failed operations preserve valid selection and drafts.

### P04.2 Changes composer graph and inspection

- Clearly distinguish HEAD/index and index/worktree, including one partially staged file in both sections. Show binary, symlink/mode, rename/delete/type and gitlink summaries without treating child file contents as parent files.
- Add bounded safe syntax presentation for text diffs if missing. Render source as text; no HTML execution. Large diffs must show truncation/continuation and keep navigation usable. Do not add a heavy parser dependency without measured need.
- Provide subject/body composition required by the spec while preserving existing draft bytes. Parse on first blank line; do not trim/reformat manually entered drafts implicitly. If changing the stored representation, migrate existing raw drafts losslessly and test blank lines/trailing newlines. AI candidate application remains an explicit replace choice.
- Show applicable trust, hooks, signer, recovery and actual result rather than generic failure/success labels. Recovery details distinguish unconfirmed publication from unpublished and completed-with-post-processing-failure.
- Keep real graph parents/merge edges across pages; display shallow/missing boundaries; Activity navigation selects the correct repository/commit. Activity author/branch/date filters and labeled committer-time attribution must meet the spec. Check existing implementation before adding missing controls.
- Complete effective hooks inspection: resolved source directory, name, executable status, bounded read-only preview and policy. Preview must not execute hooks; unsupported managers explain the refusal. Remembered preference changes only through the explicit action.
- Do not add Pull/Branch/discard/reset controls as working actions. Deferred functionality stays unavailable with clear scope.

### P04.3 Accessibility and native layout

For every workflow listed in UI01–UI12: keyboard-only traversal, meaningful names/roles, focus entry/trap/restore, visible focus, announced errors/progress/results and Escape/Return behavior. Use native VoiceOver as well as component tests. Graphs/heatmaps need accessible textual equivalents. Color alone cannot encode trust, readiness or results.

Measure foreground/background contrast for all state/control pairs; test Light/Dark/System, increased text/approved zoom, narrow minimum window, long native paths/messages, large reviews and mixed-DPI monitor disconnect/reconfiguration. Use DCP08 shortcuts and geometry acceptance. Preserve deferred cosmetic preferences; implement usability behavior without broad redesign.

Tests: UI01–UI12, GRAPH01–GRAPH05. Exit: every UI-001–005 detail is accepted or explicitly scoped, all specified controls function and persist, full native accessibility checklist passes on the release-supported environment.

## P05 Bulk local commits

Existing Core/native evidence already covers two-repository success, submodule child/fresh-parent chain, three levels, linked worktrees, partial staging, child-hook rejection/cancel/retry, partial publication crash/recovery and exclusions. Preserve those cases; rerun only when their boundary changes or for exact final artifact acceptance.

Owners: `application/{bulk,bulk_state}.rs`, `writes/transaction.rs`, `views/BulkCommit.tsx`, shared publication presentation.

1. Close explicit remaining selection cases: unavailable/missing/replaced identity; deselection/reselection during preparation; parent-only/unselected child; unrelated nested repo; excluded gitlink; mixed modes; shared versus per-repo message; blocked/ineligible selected item.
2. Preserve deterministic selected-only graph order. Malformed/self/ancestor/cyclic synthetic graphs refuse safely; distinguish real Git-valid fixtures from synthetic fallback-only cases. Return a cycle witness or actionable blocked reason, not an invented executable order.
3. Diagnose the reported completed-batch transition self-cancel: capture batch ID, review identity, component mount/key, active job/cancel token and native dialog lifecycle before patching. Add a regression only after causality is established. Test completed results → new proposal directly, and close/reopen as a comparison.
4. Verify revocation/scope loss cancels pending work, preserves successes and does not propagate trust to children/linked worktrees. Parent review after child success contains the new gitlink and exact parent message.
5. Finish keyboard/accessibility for exclusions, per-repo modes/messages, dependency reasons, results and unfinished-only retry. No empty retry of completed items.

Tests: BULK01–BULK12 and relevant REC cases. Exit: explicit G05 residual list is exhausted, actual Git objects/index/files/control repositories match the proposal and allowed deltas, failure/retry never repeats a success.

## P06 Supported remote authentication fetch upstream and checks

Owners: `remote/*.rs`, `writes/{remote,ssh_transport,fetch,fetch_import,upstream,push_check}.rs`, `authentication.rs`, credentials bridge, native settings/review and remote views.

### P06.1 Supported envelope table

Build a case-by-case table under DCP07: local macOS/APFS; SHA-1/SHA-256; ordinary and linked worktrees; initialized submodules; supported files refs and ordinary indexes; HTTPS verified TLS/get-only helper; SSH explicit Ed25519 agent identity/known_hosts. Retain existing unsupported reftable/advanced-index/filter/manager/interactive-signer cases as specified by actual M2 contracts. Do not declare a new support expansion merely because a Git CLI command succeeds.

For each supported combination, point to Core tests and native cases. Use mandatory pairwise topology/format/auth coverage plus explicit risk combinations in REMOTE01–REMOTE14; do not claim an exhaustive Cartesian product if it was not run. Named exclusions need a refusal test and approved disposition.

### P06.2 Remaining acceptance and errors

- Operator-assisted actual secure-store Deny/Allow/unlock/cancel must use a dedicated disposable credential store/account. Observe actual OS prompt and client/process outcome. Do not edit real user credentials or assume silent helper failure exercises the prompt.
- Unknown/changed SSH host key must stop before Git data transfer; no fallback identity, password or automatic known_hosts edit. Give external verification instructions. Strict host-key and agent snapshots remain destination-bound.
- Finish native SHA-256 missing-only upstream and pre-push check cases across supported ordinary/linked topology; existing SHA-256 observation/download/import and linked upstream/owned-lock recovery are retained evidence.
- Native import must handle truly missing objects, packed expected-old tracking refs, stale/foreign locks, cancellation and conflicting ref changes. Existing missing-object native pass is not a request to implement it again.
- Network work must never originate from status polling, reopening history or startup. A fresh observation, capture/import and upstream/check approval each remains separate.
- Preserve hook-manager Execute refusal and separately explicit BypassAll; pre-push check results never authorize publication. Verify exact argv/stdin, rejected/cancelled/interrupted states and reaping for graceful cancellation.
- Actual SCP port-22 acceptance remains a named facility block if unavailable. URI/argument/parser tests or alternate-port SSH cannot close that specific criterion.

Tests: REMOTE01–REMOTE14, LIFE/REC as mapped. Exit: supported cases pass, unsupported cases refuse before execution, prompt/facility blocks are resolved or explicitly approved release exclusions, authentication cannot be triggered in background.

## P07 Exact individual publication acceptance

Owners: `writes/push_publication.rs`, `remote/push_publication.rs`, `writes/push_check.rs`, publication review/results/recovery views and authenticated fixture support.

Do not replace the existing isolated export/ancestry/expected-old implementation. Close remaining acceptance:

1. Native verified HTTPS helper and SSH-agent publication, in both object formats across the approved topology pairs. Independently inspect server ref/object format and exact source OID. Use fake accounts/private fixture keys only.
2. Stale branch/source/index/config/destination/auth/tool/dependency/check review refuses; branch switch at the same OID still invalidates generation where required. Check approval cannot be submitted as publication authority.
3. Server expected-old change or rejection must never overwrite a newer ref. Local pack preparation must exclude unrelated object content, remain bounded, preserve managed repository refs/index/worktree and avoid raw server-output leakage.
4. Interruption before send is distinguished from after transport may have reached server. Lost response after server success is unconfirmed; no automatic retry. Fresh independently reviewed observation allows read-only reconciliation as expected-old, approved-source-present, different or missing without inventing a successful receipt.
5. Run the remaining native recovery presentation/cancel/foreign-state cases. Preserve the existing actual host-loss/server-completed case; use it as a recipe, not proof for every changed artifact.

Tests: PUSH01–PUSH10. Exit: G07 criterion-level coverage passes in supported envelope and all uncertain outcomes remain non-replaying and truthfully displayed.

## P08 Guided push queues

Owners: `writes/push_batch.rs`, `remote/push_batch.rs`, dependency planning/reachability, `views/GuidedPushQueue.tsx` and planner UI.

1. Native uninterrupted real root/middle/leaf queue: explicit selected set, leaf publication, separately approved fresh observation of actual leaf availability, middle review/publication/observation, then root. A stale parent plan stops safely; do not silently regenerate/approve it.
2. Repeat supported authenticated topology variants selected in DCP07. Include linked worktree unrelated branch and unselected child whose required OID is absent/present on the intended remote.
3. Validate cycle/malformed/unavailable/ambiguous historical mapping reasons; no implicit dependency selection. Requirements from all outgoing trees, including removed/replaced gitlinks and merged side history, remain complete despite displayed commit caps.
4. Test stop after first failure/cancel/unconfirmed result; persist all successes, preserve server state, and require a new unfinished-only plan. Restart between items grants no execution authority.
5. Finish inspection/paging/accessibility: show exact per-item destination/source/check/dependency state and why a parent is blocked. A conditional order is never displayed as child availability success.

Tests: QUEUE01–QUEUE09. Exit: uninterrupted and stopped/retry native chains pass; all per-item publications retain independent authority; no successful item is republished by restart/retry.

## P09 AI settings providers and assisted workflow closure

Owners: `ai/*.rs`, facade Request/Data/review, `AiSettings.tsx`, `AiDiagnostics.tsx`, `AiMessages.tsx`, native secure token adapter.

### P09.1 Complete editor behavior

1. Preserve the implemented separate connection/profile/template entities, scoped assignment precedence, additive instructions, one-off overrides, duplicate profile and restore-system-default controls. Verify before adding equivalents.
2. Resolve saved entity selection by stable returned ID, not a name lookup. Current `AiSettings.tsx` uses `.find(...name === change.name)` after save; if duplicate names are permitted, a different entity can be selected. Add a duplicate-name reproduction and either return the changed identity in the typed save result or select it by an unambiguous identity delta. Do not impose a new uniqueness policy just to hide the ambiguity.
3. Preserve all unsaved input after failed save/discovery/inference; stale revision reload must not silently overwrite it. Deleting an in-use connection/profile/template must return actionable dependent IDs and leave settings intact unless an explicit reassignment/deletion plan is approved.
4. Search installed model IDs with manual fallback and an unavailable indicator. Do not erase a stored model when discovery fails. Present capability support accurately: additional adapter parameters are accepted only when declared; unsupported keepalive/thinking controls remain absent or explicitly unsupported, never sent opportunistically.
5. Complete structured subject/body template rules, examples, scope/type choices, inherited effective configuration and one-off instructions. Do not infer a new template language; extend current typed rules only for specified missing behavior.
6. Implement “Test with AI” for a selected read-only repository snapshot using current unsaved profile/template/instructions. It must have a frozen exact filtered viewer and fresh native transfer approval, use simulated Action preview values only, and perform no file/Git mutation or settings save. Extend `ReviewAiMessages`/capture planning or a typed equivalent; do not silently save editors to reuse the existing saved-profile route.

### P09.2 Provider and secure-auth acceptance

- Verify fixed Ollama and compatible adapter routes against official provider documentation when implementing protocol changes; cite version/date in the implementation report. No generic arbitrary HTTP endpoint executor is added.
- Add actual compatible-provider synthetic/generated-fixture acceptance and HTTPS/trusted certificate/credential cases. Explicitly consent to any real endpoint; use generated fixture source, never this repo by implication.
- Keep no redirects/fallback, bounded request/response/time, output-stop reason validation and cancellation. Test token/context exhaustion, malformed partial output, unsupported finish reason, absent model, unavailable endpoint, 401/403 and response-size limit.
- Verify endpoint-bound Keychain set/read/replace/delete, wrong endpoint, missing/locked item, cancel/timeout and imported auth reset. Private test Keychain ownership and helper attestation remain mandatory. No token through renderer/argv/log/audit/backup.
- Finish final native Quit/restart after completed real-provider suggestions; the latest real Ollama checkpoint does not establish that final closure.

### P09.3 Candidate behavior and durable audit

1. Preserve 1–3 typed candidates, strict unknown-field/provenance/template validation, duplicate rejection and explicit draft replacement. A valid schema does not prove semantic correctness; expose concerns/unreported chunks and keep assisted human review.
2. Add a fixture corpus with known generated changes and expected factual assertions (rename, bug fix, docs-only, deleted file, binary, no changed tests). Native/model evaluation records unsupported claims, omitted changed groups and meaningful distinction; never use model self-confidence as proof.
3. Add persistent safe metadata for generation/test/cancel/failure and later hierarchical/automatic runs to `application/audit.rs`. Existing audit kinds contain no AI generation kind; AI jobs/results currently expire in memory. Persist request/run IDs, purpose, model/profile/template versions, context fingerprints/accounting, timestamps, phase/result/error class and explicit candidate-application event if required. Do not persist full request/source or credentials by default.
4. Keep run-level versus per-stage records distinguishable and joinable without fabricating Git operations. Audit filter/order/page across tied timestamps must remain stable. Reopening audit cannot replay a request.
5. Corrupt/missing/unavailable AI remains isolated: manual individual/bulk composition, deterministic Actions and all supported manual Git operations work with zero provider traffic and preserved drafts.

Tests: AI01–AI18; AUDIT01–AUDIT05. Exit: missing editor/test/audit behavior implemented, real-provider/TLS/secure-auth selected acceptance recorded, no implicit source transfer or manual-work dependency.

## P10 Hierarchical analysis

This is new implementation. Reuse exact capture/filter/provider/jobs/candidate validation. Add cohesive `ai/analysis.rs` and `ai/analysis_plan.rs` modules or equivalent focused modules within the existing Core crate. Do not give model summaries executable authority.

### P10.1 Frozen analysis plan

1. Input: captured selected snapshot, exact filtered chunks/provenance, resolved effective profile/template/instructions and version, explicit limits and endpoint/auth binding. Define serializable inspection DTOs separate from nonserializable one-use authorization.
2. Partition selected diff chunks deterministically by byte-safe path, content source and hunk/chunk ID; pack groups in stable order under the exact serialized request budget. Related source/docs/history can accompany a group but never count as coverage of an omitted selected diff.
3. Each leaf has ID, ordered input chunk IDs, exact request bytes/fingerprint, omission/accounting, model/profile/template versions and maximum output budget. Each reduction lists child IDs/digests and retained source IDs. Detect a chunk that cannot fit; never silently discard it.
4. DCP05 fixes group/request/depth/bytes/time limits before execution. Apply checked arithmetic and reject oversized plans before consent. Account for system/template/instruction/envelope bytes, not source bytes alone. Token estimates are labeled estimates unless a supported tokenizer exists; enforce exact byte bounds regardless.
5. Build the exact viewer for every leaf/reduction/final stage as the stage becomes available. Summaries are unknown before execution: use fresh per-stage approval or a separately approved bounded multi-request contract that explicitly covers derived summaries/endpoint/limits. Do not extend an existing one-request native token to authorize subsequent requests.

### P10.2 Execution and validation

- Schedule every individual provider request through existing global/connection admission limits. Persist safe run metadata; source payloads and summaries remain ephemeral unless explicit retention is separately selected.
- Leaf response schema: bounded factual summary, concerns and subset of supplied chunk IDs. Reject unknown IDs, duplicates, controls, extra tool/command fields and malformed/unfinished output. Count validated protocol attribution separately from semantic review.
- Reduction/final schema preserves source IDs transitively. An invented citation cannot restore missing coverage. Include omitted/redacted/failed groups and actual completed stage counts in the viewer/results.
- Coverage complete for automatic eligibility means every selected diff chunk required by the frozen policy survived capture/filtering and has validated stage attribution through final synthesis, with no failed/cancelled/truncated required stage. Context exclusions cannot be converted to success by a model statement. Semantic accuracy still requires the accepted automatic policy and testing; protocol coverage is not semantic proof.
- On failure: cancel dependent stages, discard stale candidate authority, preserve completed metadata and manual draft. Explicit retry creates a fresh plan/approval against current source/settings; no background fallback or retry.
- Cancel, scope switch, revocation, main renderer/host loss and expiry revoke the run generation. Late responses cannot update another workspace/draft. Restart displays interrupted history and sends no requests.
- Produce final 1–3 candidates through existing template validation. If DCP04 uses ranking, apply its deterministic rule only to valid eligible candidates; rationale/confidence cannot override validation.

Tests: HIER01–HIER14, relevant AI/LIFE/AUDIT. Exit: deterministic grouping/accounting and exact stage consent are proven; every request counted; omissions/failures visible; automatic-ineligible results cannot be used by P12.

## P11 Action definitions and final context

Existing typed invocation, original retention and no-double-increment recovery remain authoritative. Add only missing product integration.

Owners: `actions/`, `writes/version_action.rs`, AI template/context planning, `VersionAction*.tsx`, settings/import services.

1. Introduce a versioned typed SemVer definition record under DCP06: stable ID/revision, owning scope/target native-relative path, supported increment set, prerelease policy and approval metadata. Store definitions separately from immutable invocation records. A definition is not a reusable invocation token.
2. Add create/edit/inspect/disable/reauthorize UI. Editing a definition changes revision and invalidates pending invocations. Imported definitions always start disabled/unapproved with no local trust or endpoint authority.
3. A manual deterministic increment must still work without any AI settings. Existing plain-value file syntax and LF/CRLF rules remain; do not add TOML/JSON package-version mutation implicitly.
4. Expose a verified resulting version as an explicit bounded template value after successful Action or deterministic read-only preview. Never trust a model-supplied version variable. Show which preview/result/profile/template produced it.
5. After mutation, recapture selected source and index state; invalidate pre-Action AI context/candidates/commit review. Stage only the exact resulting approved path via existing separate manual review, or the P12 bounded contract if enabled. Do not silently stage after an ordinary manual Action.
6. Native interruption before/after atomic exchange, external edit conflict, retained-original inspection after revocation/move and recovery with index drift must remain inspectable. Do not prune unresolved originals or auto-restore over newer content.

Tests: ACT01–ACT12, IMPORT06, AUTO cases. Exit: definition/import semantics and verified version/context integration pass without broadening existing manual authority; native recovery/inspection usability accepted.

## P12 Bounded automatic commits

This is a new authority boundary. Do not reuse an ordinary immutable manual-review token for an unknown future message. Keep ordinary builds gated until its own acceptance and product exposure decision. No arbitrary code/tool output is executable.

### P12.1 Contract and state

Implement a focused Core automatic-run coordinator and immutable plan/inspection DTOs. Exact module names may be `ai/automatic.rs` plus application integration. Required frozen contract fields:

| Category | Required binding |
|---|---|
| Identity | Run ID, version, session, active workspace/epoch, selected worktree/common-directory identities and branch refs |
| Scope | Explicit selected paths and index/worktree sources per repository; allowed exclusions; initial HEAD/index/content/config/trust snapshots |
| AI | Effective connection/endpoint/auth binding, profile/template/instruction revisions, all-stage limits and consent policy, capture/screening/coverage rules |
| Action | At most the explicitly supported typed definition revision/target/expected-old/frozen output or allowed deterministic increment chosen before authorization |
| Execution | Hooks/signing policy, sequential selected dependency order, stop-on-failure, max commits/repos, candidate ranking/tie rule and exact disqualifiers |
| Lifetime | Approved-at monotonic/session-bound expiry, cancellation generation, maximum requests/bytes/time and no restart resumption |
| Results | Durable safe phase/item IDs, derived exact commit proposal digest, publication/recovery references; never serialize reusable capabilities |

DCP04 supplies numeric/product choices; validate all fields in Rust. Native review shows exact selected scope, pending AI message selection, Action before/after, endpoint/source transfer, limits, expiry and stop/review conditions. Only an independent native human action creates the nonserializable run capability.

State machine: planned → native review → authorized → optional frozen Action → recapture → analysis → candidate validation/selection → exact proposal → final revalidation → controlled commit → recorded item result → next item → complete. Terminal states distinguish cancelled, expired, blocked/needs-reauthorization, partial failure, interrupted and uncertain publication. Existing transaction journal is still the truth for Git outcomes.

### P12.2 Execute safely

1. Check whole run constraints and current item identity/trust before every sensitive step. Expiry or changed settings/source/index/HEAD/config/endpoint/auth/Action invalidates authority; stop with exact mismatch.
2. Apply only the frozen Action output through the existing guarded executor. Expected Action mutation is accounted for explicitly and recaptured. Any additional external edit still stops the run. Action failure or uncertain exchange blocks dependent commits.
3. Perform approved analysis requests under P10 scheduling/consent. Require complete policy-compliant coverage and valid candidate; no fallback to manual text or another model.
4. Build a normal exact M2 proposal from the final tree and selected message. A private Core-only approval conversion verifies the derived proposal is entirely within the current run capability. No renderer flag, JSON digest or serialized import can call that conversion with authority.
5. Execute the existing hook/signing/transaction engine unchanged where possible. Revalidation after hooks still enforces exact tree/message. Persistent result references the real operation and actual published OID/tree/message.
6. Child success changes a parent's gitlink. Preserve existing fresh parent reauthorization: stop and show updated parent proposal unless a later specifically approved contract explicitly enumerates and accepts that dynamic change. This handoff does not authorize silently widening parent scope.
7. Stop on first failed/cancelled/uncertain item; leave successes. Retry is a new run for unfinished selected work with fresh native authorization. Restart never resumes AI or writes. Recovery uses existing separately reviewed transaction/Action recovery.
8. No watcher path, commit-completed event, candidate response or run-completed handler may call push/publication. Automatic run has no push capability.

### P12.3 Renderer and audit

Separate Automatic from assisted message generation. Show review, progress per repository/stage, limits, Cancel, preserved successes, block reason and “Prepare new run” for unfinished work. No auto-selected files/dependencies. Store safe audit metadata, actual decision/coverage and operation references; do not log raw prompts/source/candidates by default. Expired/interrupted records are history only.

Tests: AUTO01–AUTO20, HIER/ACT/REC/AUDIT. Exit: separate native bounded authority, exact derived transactions, injection/drift/interruption refusals and no-replay/no-push invariants pass in Core, renderer, native and final enabled artifact.

## P13 Backup portability restore and retention

Implement P13A before new durable-run migrations. Existing migration copies are not a product backup system. New focused `backup`/`portability` modules may live in Core; UI only supplies typed choices and presents previews.

### P13A.1 Archive and snapshot contract

1. Produce and approve DCP03 format v1 before writing export/import code. Required manifest fields: format/schema versions, creation timestamp, source app version, logical export kind, component list with relative canonical path/type/size/SHA-256, database schema version, included history/artifact policy and settings generation ID. No absolute source data-directory paths needed for extraction authority.
2. Support separate AI configuration portability and full app-state manual backup. Portable AI content includes definitions/profiles/templates/instructions/assignments and disabled Action definitions; credentials and destination approvals are excluded/reset. Unresolvable workspace/repo assignments are shown in preview, not rebound by matching names.
3. Full backup includes coherent TOML/config generations, navigation/workspaces, consistent SQLite snapshot and chosen operation history. Repositories/keys/Keychains/agent sockets/reusable trust/approval capabilities are excluded. Historical trust facts may be displayed only as inactive history, never imported grants.
4. Inventory durable operation artifacts (write originals/receipts, fetch captures/import recovery, upstream receipts, Action originals, automatic metadata). For each type record “included as inert recovery evidence” or “omitted with explicit restore consequence.” Never prune live unresolved artifacts to make an archive small. A backup that omits needed recovery evidence must advertise that limitation before creation/restore.
5. Use the owning SQLite connection's backup API for a coherent snapshot, not a copy of a live `.db` without WAL consistency. Coordinate TOML/AI-generation snapshots under the storage owner so manifest/db/settings represent one revision. Keep workers for bounded I/O away from mutable connection ownership.
6. Write archive to a new private temporary file, flush/fsync, validate manifest/checksums, then atomic final publication on the destination filesystem. Failed backup leaves the previous valid backup and reports no success. Never export into a managed repository by implicit default.
7. Define bounded archive implementation and dependencies after checking existing standard-library/library capabilities. Required reader supports count/per-file/total-expanded/ratio/path-depth limits and streaming bounded hashing. Do not trust compressed size or metadata alone.

### P13A.2 Safe decoding

Before extraction reject absolute/drive/UNC paths, `..`, NUL/control-invalid names, alternate separators escaping normalization, duplicate/case-colliding/Unicode-alias paths on the target filesystem, symlinks/hardlinks/special files, unexpected entries, encrypted unsupported archives, impossible sizes, checksum mismatch, truncated members and unsupported schema. Extract descriptor-relatively into new private staging without following links. Validate actual bytes/types/counts against manifest; reject the entire archive on mismatch. Nothing activates during parsing.

Tests: BACKUP01–BACKUP12, IMPORT01–IMPORT08. Exit: consistent bounded snapshots and adversarial reader tests pass; policy and format documented; reusable migration backup primitive adopted where affected.

### P13B.1 Preview and import

- Preview archive source/version/components, credentials/authority reset, assignment mappings, conflicts and unsupported content. AI-only import supports deliberate rename/replace/skip decisions by stable IDs; never merges different entities because names match.
- Compute a preview digest binding archive bytes, current state revision, choices and conflicts. Native approval is required where import changes trusted settings/restore state through an authority-sensitive path; the exact trusted-settings policy must be explicit in DCP03. Preview itself is read-only. Do not add unnecessary native prompts to harmless inspection.
- Revalidate current revision and archive hash on apply. Atomically activate a complete new AI settings generation using existing store primitives. Run `Settings::imported()` semantics or stricter validated equivalent: reset endpoint approval/auth and regenerate revisions. Imported Actions disabled.
- Cancellation before apply leaves current state unchanged. Partial apply failure must leave either old complete state or recoverable staged state, never a mixed active configuration.

### P13B.2 Full restore and crash recovery

1. Refuse restore while active reviews/writes/AI requests or unresolved current recovery could be overwritten. Do not destroy current unresolved evidence; offer inspection/manual resolution first. A future merge strategy requires its own explicit design.
2. Obtain exclusive ownership of the data directory and create a verified safety backup of the current state. If safety backup fails, stop.
3. Restore into a sibling staging generation, validate all components/schema/SQLite integrity and sanitized identity/authority state. Strip live capabilities; mark imported in-flight history interrupted/inert. Current on-disk repos must be freshly discovered before binding current identities or enabling trust.
4. Write and sync restore intent containing old/new generation IDs and phase. Activate with an atomic generation pointer or equivalent independently recoverable file-set protocol. Keep immutable old generation/safety backup until post-restore validation completes. Never rename individual live files without a recovery protocol for partial replacement.
5. Restart opens the selected complete generation, handles interrupted restore deterministically and starts no Git/AI action. Mixed/newer/corrupt schema fails without destroying old state. App/data identifier migration follows the same safety principles.
6. Show outcome, preserved safety backup and required reauthorization. Verify exact manual drafts, workspaces, settings, optional history and recovery limitations against the approved preview.

### P13B.3 Automatic backups and retention

- Change-aware backup schedules only while the app runs; revisions coalesce under DCP03 timing. Do not create repeated backups when persistent logical state is unchanged. Write activity cannot indefinitely postpone a required maximum backup interval if policy specifies one.
- Only completed validated automatic backups count toward rotation. Never delete manual/safety backups or unresolved recovery artifacts as a side effect. Disk-full/permission errors preserve the last valid backup and remain visible; no background daemon.
- History retention prunes only eligible terminal records under the configured policy. Compute references from batches/queues/automatic runs/Actions/recovery before deletion. In-flight, uncertain, unresolved or still-referenced evidence is pinned regardless of age.
- Prune database rows and owned unreferenced artifacts coherently and idempotently; crash between phases cannot make live recovery impossible. Show/record what was removed without secrets. Defer deletion until decisions and tests pass; no implicit 90-day/10-backup cleanup.

Tests: RESTORE01–RESTORE12, RETAIN01–RETAIN08, LIFE/AUDIT. Exit: manual/automatic/portable backup, previewed restore, interruption safety and retention pass exact content/authority/recovery oracles and native UI acceptance.

## P14 Ordinary write availability

Owner: native feature definitions/dispatch/capabilities, UI capability gating, packaging and support documentation.

1. Compile a closure table for applicable G01/G02/G03/G05/G06 and publication criteria, including accepted support exclusions and remaining facility blocks. A blocked required criterion prevents enablement unless the product owner explicitly changes release scope.
2. Prepare the exact proposed default/distribution feature change and supported-envelope text. Keep native acceptance/crash support and experimental lab features non-default and excluded from shipping artifacts.
3. Obtain the explicit default-change decision against that concrete diff and closure table. This documentation request does not grant it.
4. Apply the approved change. Run ordinary artifact feature/caller/refusal tests and native untrusted→trust-decline→fresh-trust→stage/commit/fetch/push scenarios, with independent approvals and no test menu. Verify supported writes actually work and unsupported targets fail closed.
5. Keep a buildable inspection-only configuration and test its Rust rejection path. Capability UI reflects trusted native response, not a persisted frontend switch. AI remains optional.

Tests: ENABLE01–ENABLE06 plus required final invariant suite. Exit: explicitly approved ordinary behavior passes on the exact distribution candidate; source feature inspection alone is insufficient.

## P15 CI security and license closure

Owners: workflow, provisioning/prerequisite/license scripts, lockfiles, manifests and release documentation.

- Execute hosted macOS Core and desktop jobs and Linux supported read/lab job after separately authorized publication/workflow access. Preserve full logs/tool versions/artifact hashes and real exit statuses. YAML validation and local workflow fragments are not hosted passes.
- Validate checksum-pinned Git build, required GPG/SSH/private temporary directories, exact Rust version and locked npm install behavior on fresh runners. Check actual `.npmrc`/lock behavior rather than assuming install scripts are disabled by a comment.
- Ensure a failing test, missing signer, failed provisioning or license scan fails the job. Upload evidence even on failure; record ignored tests with reasons. Tune timeouts based on observed runner behavior without hiding failures.
- Refresh full dependency/build-tool advisory analysis with timestamp and source. Triage reachable shipped versus development-only issues, record disposition, make focused justified updates and rerun affected gates. Do not advertise a clean audit based on an old checkpoint or production-only npm scan.
- Derive shipping license closure from the actual packaged artifact/features/platform, not all possible locked dependencies alone. Current inventory contains 580 declarations; actual texts/notices and distribution policy remain. Resolve first-party npm declaration deliberately; do not guess licenses for missing metadata.
- Include license texts/notices as accessible shipped files/About content, verify attribution for brand assets, and document any reciprocal/source-offer obligations through qualified review. Product MIT files alone do not decide third-party obligations.
- Confirm native CSP/ACL/caller checks, no shell/Git argv/raw HTTP/SQL endpoints, no dev origin in release, source prompts not logged, secrets excluded from artifacts and test-only facilities absent.

Tests: CI01–CI06, SEC01–SEC08, LICENSE01–LICENSE04. Exit: real hosted results, current dependency dispositions and actual shipping notice closure are available for the candidate source.

## P16 Exact macOS package and release acceptance

DCP02/DCP09 must be resolved. Tauri `bundle.active` is currently false; development `.app` scripts are not a distribution pipeline.

1. Implement reproducible package commands for the selected macOS/architecture/installer scope, with frontend build, locked native build, explicitly recorded features and tool delivery. Validate Git discovery/version diagnostics before first use. Do not silently substitute an unsupported system Git or install global configuration.
2. Freeze product identifier/state location decision. If it changes from `dev.gitfleet.desktop`, implement previewed migration preserving all old state, refusing active ownership/newer/corrupt/conflicting destinations, retaining rollback safety and requiring fresh trust where identity changes. Test install coexistence rules explicitly.
3. Produce a concrete local unsigned/ad-hoc candidate first where appropriate; inspect entitlements/resources/binary architectures/feature graph. Use approved signing/notary credentials only after authorization. Verify signature, notarization/staple and Gatekeeper behavior of the exact exported installer on a clean supported machine.
4. Generate source/artifact/dependency/notice/checksum manifest and installation instructions. Hash the final package after signing/stapling. Any artifact change invalidates its hash-based acceptance claims and needs affected reruns.
5. Perform fresh install, first launch/no-AI manual workflow, update from the supported prior state, migration failure rollback, reinstall and uninstall retaining user state. Verify no daemon/login item/global Git config/credential alteration.
6. Run RELEASE01–RELEASE12 and INV-001–015 against the installed candidate, including exact partial staging, supported positive GPG/SSH, actual native approval, fetch/push/auth, automatic/Action scopes if shipped, interruption/recovery, backup/restore, accessibility and last-window shutdown. Use new disposable workspaces/accounts on the clean machine.
7. Publish support limitations, recovery/backup instructions, known issues and release notes with exact version/platforms/tool requirements. Deferred Linux/Windows/Pull/branch/hunk/selective hooks remain explicit; they are not empty checklist passes.
8. Prepare the release for review. Tags, pushes, signing-service use, uploads and publication require the applicable current authorization; this runbook does not grant them.

Exit: no unresolved required release criterion, all 109 requirements have accepted implementation/exclusion/deferral disposition, all 15 invariants have current exact-artifact evidence, and product owner has the concrete candidate and closure record for release approval.

## Existing acceptance recipes and source checkpoints

Use these concrete recipes to reuse fixture builders, phase markers and independent oracles. A linked historical pass is not automatically new acceptance. Copy only the needed setup into a fresh run; never reuse a historical temporary root, account, process ID or local trust authorization.

| Package | Existing recipe or evidence to start from |
|---|---|
| P01 complete Core matrix | [AI source regression](../reports/G02_G11_AI_SOURCE_REGRESSION_2026-11-08.md), [private temporary storage](../reports/G11_PRIVATE_FIXTURE_TEMP_2026-11-08.md), [CI prerequisites](../reports/G11_CI_PREREQUISITES_2026-11-08.md) |
| P02 native caller/recovery | [G01 closure and G02 recovery](../reports/G01_NATIVE_SUPPORT_G02_RECOVERY_2026-11-07.md), [actual renderer loss](../reports/G02_ACTUAL_RENDERER_LOSS_2026-11-07.md) |
| P02 startup/termination | [Post-load startup](../reports/G02_POST_LOAD_STARTUP_2026-11-07.md), [guarded AppKit termination](../reports/G02_GUARDED_APPKIT_TERMINATION_2026-11-07.md), [active sheet termination](../reports/G02_ACTIVE_SHEET_TERMINATION_2026-11-07.md) |
| P02 proven running hook | [Cancel](../reports/G02_G06_RUNNING_PUSH_CHECK_CANCEL_2026-11-07.md), [host crash](../reports/G02_G06_RUNNING_PUSH_CHECK_CRASH_2026-11-07.md), [renderer loss](../reports/G02_G06_RUNNING_PUSH_CHECK_RENDERER_LOSS_2026-11-07.md) |
| P03 discovery | [Scale and partial inventory](../reports/G03_SCALE_PARTIAL_INVENTORY_2026-11-08.md), [cache and scan status](../reports/DISCOVERY_CACHE_SCAN_STATUS_2026-11-07.md), [identity recovery](../reports/REPOSITORY_IDENTITY_RECOVERY_2026-11-07.md), [read-only diff index fix](../reports/G03_READONLY_DIFF_INDEX_2026-11-08.md) |
| P04 native layout/accessibility | [Accessibility](../reports/M3_ACCESSIBILITY_2026-11-02.md), [mixed DPI restore](../reports/M3_MIXED_DPI_WINDOW_RESTORE_2026-11-02.md), [theme](../reports/M3_NATIVE_THEME_2026-11-02.md) |
| P05 dependency and partial staging | [Native three levels](../reports/G05_NATIVE_THREE_LEVEL_2026-11-08.md), [linked worktrees](../reports/G05_LINKED_WORKTREE_2026-11-08.md), [partial staging](../reports/G05_NATIVE_PARTIAL_STAGING_2026-11-08.md), [exclusions](../reports/G05_BULK_EXCLUSIONS_2026-11-08.md) |
| P05 failure/retry | [Child hook](../reports/G05_NATIVE_CHILD_HOOK_2026-11-08.md), [partial batch host crash](../reports/G05_NATIVE_BATCH_HOST_CRASH_2026-11-08.md), [prepublication crash](../reports/G05_NATIVE_BATCH_PREPUBLICATION_2026-11-08.md), [revocation](../reports/G05_NATIVE_BULK_REVOCATION_2026-11-08.md) |
| P06 auth and upstream | [Authentication compatibility](../reports/M5B_AUTHENTICATION_COMPATIBILITY_2026-11-02.md), [linked upstream](../reports/G06_NATIVE_LINKED_UPSTREAM_2026-11-07.md), [owned config lock](../reports/G06_NATIVE_OWNED_CONFIG_LOCK_2026-11-07.md), [full signing](../reports/G06_FULL_SIGNING_MATRIX_2026-11-07.md) |
| P06 import and check | [SHA-256 missing objects](../reports/G06_NATIVE_SHA256_MISSING_OBJECTS_2026-11-08.md), [manager refusal/bypass](../reports/G06_NATIVE_MANAGER_BYPASS_2026-11-07.md), [helper refusal](../reports/G06_NATIVE_REMOTE_HELPER_REFUSAL_2026-11-07.md) |
| P07 individual publication | [Exact publication](../reports/G07_EXACT_PUBLICATION_2026-11-08.md), [authenticated Core publication](../reports/G07_AUTHENTICATED_PUBLICATION_2026-11-08.md), [actual native host crash](../reports/G02_G07_NATIVE_PUSH_HOST_CRASH_2026-11-08.md) |
| P08 queues | [Guided push queue](../reports/G08_GUIDED_PUSH_QUEUE_2026-11-08.md), [dependency planner native](../reports/M5C_PLANNER_NATIVE_ACCEPTANCE_2026-11-03.md) |
| P09 settings and credentials | [Settings storage/editor](../reports/G09_AI_SETTINGS_STORAGE_2026-11-08.md), [provider diagnostics](../reports/G09_AI_PROVIDER_DIAGNOSTICS_2026-11-08.md), [secure authentication](../reports/G09_AI_SECURE_AUTH_2026-11-08.md) |
| P09 source and real provider | [Context capture](../reports/G09_AI_CONTEXT_CAPTURE_2026-11-08.md), [context filter](../reports/G09_AI_CONTEXT_FILTER_2026-11-08.md), [assisted messages](../reports/G09_AI_ASSISTED_MESSAGES_2026-11-08.md), [real Ollama](../reports/G09_OLLAMA_REAL_PROVIDER_2026-11-08.md) |
| P11 Actions | [SemVer Action](../reports/G09_SEMVER_ACTION_2026-11-08.md), [retained original inspection](../reports/G09_RETAINED_VERSION_INSPECTION_2026-11-08.md) |
| P15 licenses | [Locked inventory](../reports/G11_LICENSE_INVENTORY_2026-11-08.md) |

## Package completion template

Use this exact information in each package report; fields may link to structured evidence rather than duplicate it:

```text
Package and requirement IDs:
Source revision and dirty state:
Decision dependencies and approved values:
Implemented behavior and affected files:
Retained acceptance references and why still applicable:
New or changed case IDs and actual outcomes:
Commands, tool versions, features and artifact hashes:
Native operator, fixture and phase proof:
Allowed versus observed file/ref/index/server/record deltas:
Cancellation, recovery, integrity and lock reopen:
Failures, blocks, accepted exclusions and approval reference:
Remaining criterion IDs:
Live ledger and coverage updates:
Next executable package:
```

## Handoff completion rule

A package is complete only when its implementation steps, mapped catalog cases, evidence and disposition updates are complete. A broad “all tests passed” statement cannot replace specific native/failure/restore checks. An unimplemented feature cannot be deferred merely to declare the release ready. If a policy remains undecided, return its concrete decision record and continue independent work; do not make the next implementer rediscover what question is blocking execution.
