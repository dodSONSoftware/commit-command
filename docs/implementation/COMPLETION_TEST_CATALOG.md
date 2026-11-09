# Commit Command completion test catalog

Use with [the runbook](COMPLETION_RUNBOOK.md) and [requirement coverage](COMPLETION_REQUIREMENT_COVERAGE.md). These are required acceptance scenarios, not assertions that the tests already exist or have passed. Reuse current tests/evidence when they prove the same scenario on the applicable source; add missing implementations to the owning suite named below. Every listed variant gets a separate result. A case with multiple variants passes only when every required variant passes or has an approved exclusion.

## Common procedure and independent oracles

For every case:

1. Create a new marked fixture root in private outside-repository temporary storage. Write a fixture manifest containing canonical root, marker, workspace/data directory, repository/common-directory/worktree IDs, initial refs and owned PIDs. Never substitute an existing user checkout. Use fixture helpers already under `app/core/tests/support/` and the native generator where applicable.
2. Capture O1–O6 below before the action, declaring allowed deltas in the result. For read-only/refusal/cancel-before-mutation cases the allowed managed-repository delta is empty. App-state/audit changes, if intended, must be enumerated separately.
3. Perform the action through the actual tested layer. Unit tests exercise pure rules; integration tests call Core; renderer tests use the typed bridge; native tests use actual webviews/independent dialogs. A mock may not stand in for the native layer.
4. For cancellation/crash cases wait on a deterministic phase marker, inspect durable phase, record the exact owned process and then act. If the phase was not reached, record preparation failure, not cancellation/recovery success. SIGKILL only the exactly identified fixture host; do not kill a process by a broad name match.
5. Capture after oracles and independently compare the allowed deltas. Verify SQLite integrity, durable result, no source/credential leakage and owned process cleanup where applicable. Crash cases report any surviving descendants rather than claiming graceful cleanup.
6. Close/Quit normally after the case, reopen the state writer lock, restart the ordinary artifact on the same fixture and prove no execution/network replay. Where a case intentionally leaves unresolved recovery, inspect it read-only and preserve its evidence before cleanup.
7. Record actual source/features/binary hashes, commands, variants, outputs and disposition using the runbook evidence schema. Clean up only this run's marked fixture roots/owned processes after preserving scrubbed evidence.

| Oracle | Exact observations |
|---|---|
| O1 filesystem | No-follow recursive inventory of bytes or symlink targets, file modes and relevant inode/device identities; include actual Git directory/common directory, config, indexes/locks, working/untracked files, child/control repos, retained artifacts. Exclude only explicitly enumerated transient fixture server data, never broad `.git` exclusions. |
| O2 Git | Full OIDs from literal refs, raw commit/tree/blob/message bytes, index entries and index file hash, reflog bytes and object inventory where required. Observation Git commands use `GIT_OPTIONAL_LOCKS=0` and no executable helper/config behavior; do not refresh the real index. |
| O3 app state | Read-only SQLite snapshot/query of relevant operation/batch/queue/Action/trust/navigation/settings records before/after; preserve original payload bytes, plus `PRAGMA integrity_check` and separate writer-lock reopen after host exit. Do not use SQL mutation as the application under test. |
| O4 network | Fixture server request count/method/route/destination/body fingerprint and independent server refs/object set; counters for credential helper/hook invocations. Decline/refusal/startup/manual work must send zero unauthorized requests. |
| O5 processes | Exact host/renderer/helper/hook child PIDs, ancestry/process group, start marker, exit status and actual absence after graceful cancellation/exit. A changed UI label alone proves none of these. |
| O6 authority | Trust IDs/revisions, current plan/review/session/epoch, confirmation expiry and one-use behavior; imported/restarted records contain no usable capability. Never persist or export actual secret values. |

Fixture-only content can use `fixture@example.invalid` identities, literal baseline file contents and localhost services. Any secret sentinel is random per run, known only to the fixture, searched for in persisted outputs and then destroyed; never print its value into the report.

## Fixture recipes

Implement missing reusable builders in `app/core/tests/support/` with explicit inputs/results; reuse existing builders first. A fixture recipe is complete when its manifest and before oracles are emitted. No hardcoded personal paths/services/ports.

| Fixture | Construction and required known state |
|---|---|
| F01 local | New `main` repo with fixture identity, no inherited hooks/template/config, commit A with `selected.txt=base\n`, `excluded.txt=keep\n`, plain `version=1.4.2\n`. Stage `selected.txt=staged\n`, then write `working\n`; stage `excluded.txt=excluded staged\n`; add untracked `notes.txt=untracked keep\n`. Add second clean untrusted control repo. SHA-1 and SHA-256 variants. |
| F02 topology | Real root/middle/leaf submodules with committed `.gitmodules`/gitlinks and initialized checkouts; unrelated nested independent repo and unselected control. Add linked worktree on distinct branch sharing common directory. Record all actual metadata paths and OIDs. Advance leaf only when the case says so. Avoid making a synthetic cycle by corrupting real Git metadata; use a separate pure graph fixture. |
| F03 hooks/signing | F01 with direct fixture hooks writing invocation markers/counters. Variants: success, nonzero, held child sleep, staged-extra-file mutation, selected-content mutation, message mutation, post-commit side effect/failure, unsupported-manager signature/reference-transaction configuration. Dedicated temporary GPG home/key and SSH signing key/allowed-signers file; no personal agent/keyring. |
| F04 transport | Disposable bare server with existing `refs/heads/main` at A and client at descendant B. Explicit write-enabled HTTPS fixture with private CA/server certificate, request recorder and get-only credential helper with random fake account secret. Separate private sshd/agent/Ed25519 known_hosts fixture. Choose ephemeral loopback ports and isolated configuration. Variants SHA-1/SHA-256, linked/submodule client and server hold/reject/drop-response hooks. |
| F05 AI | Nonblocking loopback HTTP(S) mock recording exact received payload; controllable response/status/delay/size/finish reason/model list and deterministic phase barriers. Generated repo F01 or corpus with known changes. Dedicated private Keychain for auth variants. No project source or real user token. Generated semantic corpus includes rename-only, deletion, docs-only, binary and changed-code/no-test cases. |
| F06 scale | Fresh marked roots with 100/1,000/10,000 repos from the existing benchmark; separate representative histories with merges/dirty files/submodule/worktree cases. Record expected repository IDs/counts and independent generation. Add excluded tagged caches, malformed `.git`, inaccessible roots and event bursts only in stated variants. |
| F07 archive | Synthetic versioned AI/full-state archives produced by P13A writer, known manifest/components/checksums, optional history/retained originals, plus malicious variants generated without ever extracting through unsafe standard helpers. Keep old/new state and safety backup as independent manifests. |
| F08 native | Build ordinary and acceptance binaries separately; hash each. Build Core CLI and run `python3 app/desktop/scripts/native-fixture.py` for base state; extend through dedicated fixture builders for F02–F05. Use `--data-dir` from generated manifest. Test-only faults require the exact additional markers/containment checks in `native_acceptance.rs` and Core. Existing generator marker alone does not authorize arbitrary fault support. |
| F09 installed | Clean supported macOS test account/machine/VM with no app data/AI settings, selected Git/tool delivery and installed exact candidate hash. Disposable workspaces, signing material, servers and Keychains only. Preserve separate previous-version fixtures for upgrade; never use the operator's normal account state as a restore target. |

### Commands and suite ownership

Baseline and full runners are in the runbook. Focused suite commands use actual suite names, for example:

```sh
CARGO_NET_OFFLINE=true RUST_TEST_THREADS=4 cargo test --manifest-path app/core/Cargo.toml --features application-services --test bulk_application --locked
CARGO_NET_OFFLINE=true RUST_TEST_THREADS=4 cargo test --manifest-path app/core/Cargo.toml --features application-services --test remote_observation --locked
CARGO_NET_OFFLINE=true RUST_TEST_THREADS=4 cargo test --manifest-path app/core/Cargo.toml --features application-services --test version_actions --locked
CARGO_NET_OFFLINE=true cargo test --manifest-path app/desktop/src-tauri/Cargo.toml --locked
CARGO_NET_OFFLINE=true cargo test --manifest-path app/desktop/src-tauri/Cargo.toml --all-features --locked
npm --prefix app/desktop test -- src/views/AiMessages.test.tsx
```

For a base native fixture, first build the exact prerequisites and let the generator emit its new paths:

```sh
npm --prefix app/desktop run build
CARGO_NET_OFFLINE=true cargo build --manifest-path app/core/Cargo.toml --bin gitfleet-cli --locked
python3 app/desktop/scripts/native-fixture.py
CARGO_NET_OFFLINE=true cargo build --manifest-path app/desktop/src-tauri/Cargo.toml --target-dir app/desktop/src-tauri/target/completion-ordinary --locked
CARGO_NET_OFFLINE=true cargo build --manifest-path app/desktop/src-tauri/Cargo.toml --target-dir app/desktop/src-tauri/target/completion-acceptance --features desktop-writes --locked
```

Record the generator's `data_directory` and launch the resulting `app/desktop/src-tauri/target/completion-acceptance/debug/gitfleet-desktop` with `--data-dir` followed by that exact generated directory. This fixture is for native acceptance only. For fixed crash/caller support, build in a third target directory `app/desktop/src-tauri/target/completion-faults` with `--features desktop-writes,native-acceptance-support` and use the linked G01/recovery recipe's sealing requirements. Do not assume the ordinary generator satisfies the extra containment/marker guards. Run the ordinary restart with `app/desktop/src-tauri/target/completion-ordinary/debug/gitfleet-desktop`, never an acceptance binary renamed as ordinary. Record SHA-256 of each executable before launch and keep all three feature/artifact manifests distinct.

Add new backup/automatic/hierarchical integration test targets to `Cargo.toml` with the correct feature requirements when introduced; the manifest must not accidentally run write tests in default inspection. New tests belong in existing suites if cohesive. An individual case name is chosen during implementation and recorded beside its catalog ID; catalog IDs are stable across source refactors.

## Documentation and infrastructure

Layers: repository validation/Python unit checks, manifest tests and fixture integration. Owners P00/P01.

| Case | Setup and action | Required result |
|---|---|---|
| DOC01 | Parse original requirement IDs, coverage table and package/test links | Exactly 109 original IDs, no duplicates/missing IDs; every referenced package/case resolves |
| DOC02 | Compare active capability/gate/README statements with baseline implementation | No active assertion that existing push/AI/Action foundations are absent; historical dated text preserved |
| DOC03 | Review all DCP decisions and execution dependencies | Selected values have provenance; unselected policy blocks its dependent task explicitly |
| INF01 | Create each fixture with a user-like unmarked neighboring directory | Builder/cleanup touches only marked owned root; state outside repos; mode-0700 ancestry |
| INF02 | Run O1/O2 twice on F01 without action | Exact files/index/refs/modes unchanged; oracle itself causes no refresh |
| INF03 | Force fixture preparation failure before ready marker | Result is failed/blocked preparation, never passed Cancel/crash |
| INF04 | Run evidence validator with missing hash/phase/oracle and mismatched artifact | Reject incomplete/inconsistent pass records |
| INF05 | Observe ordinary and acceptance dependency/feature graphs | Ordinary excludes experimental/crash/native-test support; enabled tests retain fixture guards |
| INF06 | Loopback denied, Keychain denied, missing GPG, long/untrusted temp ancestry variants | Clear environment/prerequisite failure; no weakened tests/guards or false skips |
| INF07 | Cancel held fixture process with unrelated same-name process present | Only owned process group stopped; unrelated process alive; no leaked fixture child |
| INF08 | Run clean locked prerequisites/fixture/signing setup and cleanup | Deterministic tools/private stores; no global config/personal keyring changes |

## Startup shutdown and recovery

Layers: Core/application and native host tests, actual F08 webview/native sessions; use existing lifecycle reports as retained evidence. Owners P02 and operation-specific packages.

| Case | Setup and action | Required result |
|---|---|---|
| LIFE01 | Fresh ordinary state, then saved multi-tab/draft state; launch | Usable UI readiness with correct tabs/drafts; no network/trust/write at startup |
| LIFE02 | Actual blank recurrence or diagnostic launch sequence after host/renderer loss | Capture pre-reload diagnostics; causal fix has regression; nonrecurrence alone is not “fixed” |
| LIFE03 | Dirty draft and active read; main close, Cmd+Q variants | Paired navigation/bulk snapshot preserved, process exits, lock reopens, no replay |
| LIFE04 | Missing close-snapshot delivery; choose Keep open then retry; separate Close without saving | First retains app/draft; retry saves; discard branch restores only last saved draft, with no invented unsaved preservation |
| LIFE05 | Trust/commit/AI source native sheet open; direct AppKit termination | No implicit approve; safe documented deferral; explicit Cancel/fresh Quit succeeds |
| LIFE06 | Actual permitted Dock Quit with idle/read/held-write states | Real Dock action recorded; guarded exit/draft/result/process oracles; policy denial remains blocked |
| LIFE07 | Coordinated session termination on disposable operator session | Actual OS event and recovery observed; no unattended user-session termination |
| LIFE08 | Review pending and confirmation already issued; close/replace/reload/main renderer loss variants | Old review/capability unusable, no implicit writes, surviving host cancels work, fresh review possible |
| LIFE09 | Held hook, credential lookup, fetch, provider request variants; native Cancel/close/Quit | Proven running; owned client/helper children reaped gracefully; phase/result truthful; drafts preserved |
| LIFE10 | Host-only SIGKILL during held operation; ordinary first restart | Interrupted/uncertain state, zero replay; record surviving descendants honestly; no broad cleanup claim |
| LIFE11 | Switch/remove workspace, revoke trust while queued/running | Old epoch/scope cannot run/update new workspace; successes preserved; no transferred trust |
| LIFE12 | Final real-provider suggestion finishes; native Quit then first restart | Clean host exit/owned client absence, exact drafts/history/settings, no request replay |
| REC01 | Local crash at each implemented pre-ref checkpoint listed in runbook | HEAD unchanged, guarded owned cleanup only, no commit/hook replay |
| REC02 | Crash after ref update before index reconciliation | Exactly one approved commit; journal and real ref distinguish publication; fresh guarded reconciliation only |
| REC03 | Crash after reconciliation and during post-processing | No duplicate commit; post-hook side effects/failure separate from publication |
| REC04 | Replace owned index/config/ref lock with same-byte foreign inode before recovery | Refuse cleanup; preserve foreign lock; manual reason visible |
| REC05 | External selected/index/HEAD edit after recovery review | Stale recovery refused; preserve newer state and require fresh inspection |
| REC06 | Prepared/cancelled/failed/unconfirmed/complete records reopened ordinary | Correct distinct labels; no authority restored or automatic execution |
| REC07 | Existing unresolved local/import/upstream/Action recovery, attempt scope removal/restore | Block destructive scope loss; inspection remains accessible; records/artifacts retained |
| REC08 | Storage error at each changed durable boundary and lock reopen | No false success, actual state classified, database coherent, next process can inspect safely |

## Discovery performance and monitoring

Layers: `m1_workspace.rs`, `m3_application.rs`, benchmark scripts, renderer/native Overview and monitoring. Owners P03/P04.

| Case | Setup and action | Required result |
|---|---|---|
| SCAN01 | F06 timeout after first bounded portion, continue repeatedly | Monotonic discovery progress reaches suffix repos; no same-prefix starvation; complete count exact |
| SCAN02 | Cancel traversal and inspection independently | Prior accepted inventory retained; late partial results not activated |
| SCAN03 | Change root/exclusion/root identity while continuation pending | Old cursor invalid; no stale identities or results in new generation |
| SCAN04 | Known repo unvisited at deadline; then complete scan with actual removal | Partial marks pending/unavailable; only completed traversal establishes missing |
| SCAN05 | Normal/nested independent/submodule/linked worktree/canonical root alias | Correct identities/relationships, no duplicate common activity, no inherited trust |
| SCAN06 | Symlink directory/root alias, valid tagged cache, malformed `.git`, denied directory | No recursive symlink escape; cache skip explicit; malformed/denied reported distinctly |
| SCAN07 | Clone/init partially written metadata, then completion | Unavailable/pending until valid; no false empty/duplicate/trust |
| SCAN08 | External repo move/rename/replacement at same path | Confident stable move preserved; replacement separate history/identity and no trust transfer |
| SCAN09 | Edit one visible file with large inventory | Targeted status updates without full rediscovery; selection/draft stable |
| SCAN10 | Burst events, queue overflow, watcher failure | Debounce/bounded reconciliation repairs misses; degraded state visible |
| SCAN11 | Switch workspace then edit inactive root | No inactive watchers/polling updates; new active root reconciles |
| SCAN12 | Incomplete topology then prepare parent/bulk/push | Dependent writes blocked until verified edges; partial never claimed complete |
| PERF01 | Release CLI 100/1,000/10,000 F06 first-pass/warm, >=3 repetitions | Exact counts/completeness, separate phase timings and medians/ranges; DCP01 budgets evaluated |
| PERF02 | Representative dirty/history/topology inventory | Approved completion/first-use latency/memory budgets pass; no empty-repo extrapolation |
| PERF03 | Large table/tree filter/sort/select/scroll | DCP01 render/interaction/DOM/IPC budgets; IDs/selection stable with bounded rendering |
| PERF04 | Large DAG/activity/audit with paging/filter changes | Stable exact results across pages and latency/payload budgets; no all-history load |
| PERF05 | Active scanning/event burst plus Cancel/close | Cancel/shutdown budget passes, storage owner responsive, no child leaks |
| PERF06 | Minimum supported hardware/OS and each shipped architecture | Stated supported envelope meets budgets; outside-envelope results explicit |

## User interface and graph

Layers: model/view component tests and actual native keyboard/VoiceOver/geometry. Owners P04 and feature UI packages.

| Case | Setup and action | Required result |
|---|---|---|
| UI01 | Table/tree filter/search/collapse; select parent and child explicitly | Shared selection, no descendant auto-select, collapse/filter never deselect silently |
| UI02 | Sort every specified column ascending/descending with ties/unavailable values | Stable documented order and `aria-sort`; persisted old/new schema compatible |
| UI03 | Tabs/views/commit/file/parent/filter/scroll; switch/restart/remove workspace | Overview permanent, correct adaptive default only for new tab, exact saved state/drafts retained |
| UI04 | F01 staged+working same file; select each diff/stage source | Correct HEAD/index versus index/worktree labels/content, partial staging preserved |
| UI05 | Rename/delete/mode/symlink/binary/gitlink/large diff | Accurate bounded summary/source; no HTML/code execution or nested content leakage |
| UI06 | Subject/body edit with blank/trailing lines, AI replace/cancel, save failure | Lossless manual draft and explicit replacement; no silent trim/normalization |
| UI07 | Hooks/trust/recovery inspect, remember policy versus one-off override | Exact effective paths/executable content/policy; read-only preview; no hidden preference write |
| UI08 | Keyboard-only every workspace/Changes/graph/bulk/remote/AI/Action/backup flow | Reachable controls, visible focus, correct Escape/Return, focus restored to trigger |
| UI09 | VoiceOver same flows including graph/heatmap and native approval/error | Correct names/roles/order, text alternatives, announced results/progress, no color-only meaning |
| UI10 | Light/Dark/System, disabled/error/selected/hover/focus states | Measured accepted contrast, readable native sheets, theme persisted |
| UI11 | Minimum/narrow window, approved zoom, long paths/messages, large reviews | No clipped required actions, scroll and density/collapse usable, limits visible |
| UI12 | 1×/2× negative-coordinate displays; disconnect/reconnect/change primary | Window clamped to available display and minimum; persisted restore no shrink/lost window |
| GRAPH01 | Root/merge/disconnected/shallow/missing parent DAG with page boundary | Actual OID edges and continuity; missing boundary shown, no invented parent |
| GRAPH02 | Select merge parent and compare each actual parent | Diff matches raw Git oracle and selected repository only |
| GRAPH03 | Switch repo/ref/filter with delayed previous response | Old result discarded, selected commit scoped, no cross-repo graph |
| GRAPH04 | Linked worktrees, all local branches, author/branch/date filters | Activity dedup by repository/common identity and exact committer time attribution |
| GRAPH05 | Timeline/lanes/heatmap selection and navigation | Seven-day/twelve-month defaults and chosen dates correct; lanes no cross-repo ancestry; History opens right commit |

## Bulk commits

Layers: `bulk_application.rs`, `m2_writes.rs`, `BulkCommit.test.tsx`, actual F08 native. Owner P05.

| Case | Setup and action | Required result |
|---|---|---|
| BULK01 | F01/F02 prepare review-all/index-only with explicit selected set | Preparation changes no staging; exact source blobs shown, excluded/control untouched |
| BULK02 | Shared message plus distinct override; multiline/control-looking text | Actual raw commit messages exactly approved, no accidental shared override |
| BULK03 | Selected parent only with unselected child/new gitlink | No child commit or auto-selection; exact parent proposal or explicit dependency block |
| BULK04 | Real root/middle/leaf selected; commit leaf then ancestors | Child-first; each changed ancestor requires fresh exact gitlink approval; one commit each |
| BULK05 | Linked worktrees same common directory, one untrusted/unselected | Independent trust/index/branch; serialize shared conflicts, preserve unselected checkout |
| BULK06 | Child hook rejects, then held child hook cancelled | Pending parent blocked, no published child, actual process reaped and exact side effects reported |
| BULK07 | First item published then host SIGKILL | Preserve first success, remaining untouched; restart unconfirmed/interrupted truthful, no replay |
| BULK08 | Fresh unfinished-only retry after failure/recovery | New proposal/authorization; no re-commit of successful items; empty retry disabled |
| BULK09 | Missing/replaced/ineligible selected repo, deselect/reselect and exclusions | Explicit blocked item/reason; stale selection review invalid; excluded gitlink no child content |
| BULK10 | Synthetic self/cycle/ancestor/malformed graph and unavailable edge | Refuse invalid order with actionable witness/reason; no writes or speculative fallback |
| BULK11 | Completed results → prepare next batch directly, repeated with close/reopen | No unexplained self-cancel; if reproduced record exact lifecycle causal state before fix |
| BULK12 | Revoke/scope switch during pending/running batch; keyboard/VoiceOver result retry | Pending authority cancelled, successes retained, no trust propagation; accessible controls |

## Remote observation fetch upstream and checks

Layers: `remote_inspection.rs`, `remote_observation.rs` with support modules, authentication/remote view tests and actual native. Owner P06.

Mandatory format/topology pairs: SHA-1 ordinary, SHA-256 ordinary, SHA-1 linked worktree, SHA-256 initialized submodule. Run HTTPS and SSH success for both formats; run native missing-only upstream and pre-push in both formats and at least one linked-worktree case. All other exclusions remain named in DCP07; extra combinations are added for a specific changed boundary, not an implicit Cartesian-product claim.

| Case | Setup and action | Required result |
|---|---|---|
| REMOTE01 | Inspect configured remotes/rewrites/push URLs/auth profiles offline | Exact redacted policy/origins, no helper/contact, ambiguous destinations blocked |
| REMOTE02 | Approve observation over HTTPS and SSH format variants | Server full OID/time recorded; managed repo unchanged, exact selected endpoint/auth |
| REMOTE03 | Native observation Cancel/decline and ordinary restart | Zero new traffic/authority, history inspection read-only |
| REMOTE04 | Config/auth/tool drift, branch switch same OID after review | Stale/context refusal before transfer, current trust handled correctly |
| REMOTE05 | Actual private credential prompt Deny/Allow/unlock/Cancel | Distinct OS actions observed, no fallback/store/erase, client reaped, no secret logs |
| REMOTE06 | Wrong credentials, locked/missing store, missing/wrong agent identity | Clear bounded failure, zero alternate identity/password fallback |
| REMOTE07 | Unknown/changed host key and malformed known_hosts | Stop before data transfer; files unchanged, external verification instruction |
| REMOTE08 | Separately approve capture after observation, then server stops before import | Verified bounded standalone bundle, only approved objects; import offline and separate |
| REMOTE09 | SHA-256 genuinely missing objects + packed old tracking ref | Exact objects/ref/reflog added; HEAD/index/work/config/packed-refs preserved |
| REMOTE10 | Missing-only upstream success, standard/linked formats | Only approved branch config keys change, metadata/modes preserved, shared scope/trust effects truthful |
| REMOTE11 | Upstream/import empty/ready/pre-publication owned lock; stale/foreign replacement | Cancel preserves; fresh owned abort only; foreign lock never removed |
| REMOTE12 | Supported pre-push hook success/reject/held Cancel/manager Execute refusal/explicit bypass | Exact argv/stdin; no publication; refusal/bypass counters correct; check token not publication token |
| REMOTE13 | Background monitor/startup/history/manual refresh with configured helper | No authentication/network initiated; zero request/helper counters |
| REMOTE14 | Direct SCP port-22 criterion under permitted facility | Actual required network route observed or explicit block; parser/alternate route is not a pass |

## Individual push and guided queues

Layers: remote publication/batch Core support, renderer planning/results, actual native server. Owners P07/P08.

| Case | Setup and action | Required result |
|---|---|---|
| PUSH01 | Fresh separate check then exact publication approval | One server ref advance to approved source; check/commit alone send no push |
| PUSH02 | Native HTTPS helper and SSH agent, SHA-1/SHA-256 approved topology pairs | Actual server full OID/object format and authentication match; local files/refs/index unchanged |
| PUSH03 | Source/index/HEAD/config/auth/endpoint/tool drift after review | Refuse before unsafe publication; cannot substitute a different destination or source |
| PUSH04 | Server ref advances/diverges after review | Expected-old/ancestry rejection, no overwrite or force; actual state reported |
| PUSH05 | Export with unrelated secret-sentinel object and bounded object graph | Only approved reachable export, no unrelated object content, limits enforced |
| PUSH06 | Server pre-receive rejects; transport fails before sending | No false success; server ref unchanged; retries fresh |
| PUSH07 | Host kill before transport and after server accepts but response lost variants | Before-send unpublished versus uncertain correctly distinguished, no automatic retry |
| PUSH08 | Fresh recovery observations expected-old/source-present/different/missing | Distinct recorded state, no invented receipt or remote rollback |
| PUSH09 | Recovery observation predates attempt or source/destination mismatch | Refuse reconciliation; require fresh exact observation |
| PUSH10 | Native recovery Cancel, ordinary restart/history and accessible outcome | No replay, success/unconfirmed labels accurate, controls/focus usable |
| QUEUE01 | Uninterrupted native root/middle/leaf guided queue | Child publish/fresh observed availability before each parent; explicit fresh per-item approvals |
| QUEUE02 | Authenticated HTTPS and SSH selected topology chains | Intended server refs only, no wrong-account/destination fallback |
| QUEUE03 | Unselected child's required OID absent versus present on intended remote | Absent blocks parent; present proof permits exact scope; no implicit child push |
| QUEUE04 | Removed/replaced historical gitlink and merged-side requirement beyond display cap | Full required object set verified; UI cap cannot hide dependency |
| QUEUE05 | Cycle/ambiguous mapping/missing observation/unavailable selected node | Block exact reason; no executable partial order claimed ready |
| QUEUE06 | Fail/cancel second item after first succeeds | Stop remaining items, first server success retained, exact counts |
| QUEUE07 | Lost response/unconfirmed item | Queue stops; separate observation/recovery before any fresh retry |
| QUEUE08 | Host kill between items, restart then new unfinished-only plan | No automatic next item; successes not republished; new plan and approvals |
| QUEUE09 | Filter/page saved queues, keyboard/VoiceOver and stale parent plan | Inspection grants no authority; stale parent stops visibly; accessible reasons/results |

## AI settings providers assisted messages and audit

Layers: `ai/tests.rs`, provider/candidate unit tests, `m3_application.rs`, AI view tests, private Keychain host tests and actual native F05/F08. Owners P09/P10/P12.

| Case | Setup and action | Required result |
|---|---|---|
| AI01 | Fresh no-AI state; unavailable/corrupt configuration variants | Manual drafts/reads/individual/bulk workflows and deterministic Action remain available; zero provider traffic |
| AI02 | Create/edit/duplicate profiles/templates/connections, including duplicate names | Stable selected ID, no accidental overwrite; original/revision preserved on failure |
| AI03 | Stale settings fingerprint/revision and unsafe generation pointer/link | Refuse overwrite/unsafe read, prior generation active, unsaved editor input retained |
| AI04 | App/workspace/repo/one-off assignments and additive instructions | Most specific explicit profile wins, instructions order correct, no model fallback |
| AI05 | Delete referenced connection/profile/template; refresh unavailable model list | Actionable dependent IDs, no dangling state or erased model selection |
| AI06 | Current unsaved editor discovery and synthetic inference independently | Exact editor payload used without save; zero source/Git mutation, native consent per request |
| AI07 | Read-only selected-repository Test with AI using unsaved template/profile and simulated version preview | Exact filtered viewer/request match, fresh source consent, no settings/Action/index/files write |
| AI08 | Compatible versus Ollama supported/unset/unsupported parameters | Only declared params sent; absent values use provider defaults; unsupported explicit params refused |
| AI09 | 401/403/missing model/unreachable endpoint/timeout/redirect/wrong TLS certificate | Clear bounded errors, no fallback/redirect/retry, drafts intact |
| AI10 | Valid complete output, token limit, unsupported finish reason, malformed/oversized body | Only complete bounded supported response accepted; partial output discarded |
| AI11 | Exact source viewer; decline/approve transfer; raw excluded source drifts | Decline zero HTTP; approved received fingerprint matches viewer; any raw snapshot drift invalidates |
| AI12 | Sensitive paths, secret/control/bidi fragments, nested repo/symlink/hardlink/race | Required exclusions/accounting, no unsafe traversal/source transfer, raw drift checked |
| AI13 | 0/1/3/4 candidates, duplicate messages, unknown chunk/command field/template violations | 1–3 valid structured choices only; invalid/duplicate/authority fields rejected or invalid candidate labeled per schema |
| AI14 | Apply/edit/regenerate/cancel candidate with an existing manual draft | Only explicit choice replaces draft; no staging/commit/push; late/stale result not applied |
| AI15 | Private Keychain set/get/replace/delete, wrong endpoint, missing/locked, timeout/Cancel | Exact binding, no unauthenticated fallback, bounded owned helper, no secret outputs |
| AI16 | Import connection metadata and try old token/approval | Auth/approval reset; new binding/local secure enrollment required |
| AI17 | Actual approved Ollama and compatible HTTP/HTTPS generated-fixture corpus | Record model/protocol/settings, unsupported factual claims/coverage; no project source by implication |
| AI18 | Concurrent queued/running requests, manual refresh and scoped cancellation | Global/per-connection limits and storage responsiveness, no cross-workspace late result, correct job expiry |
| AUDIT01 | Complete/cancel/fail AI diagnostic and source generation | Durable safe purpose/model/revisions/fingerprints/coverage/result; no raw prompts/token |
| AUDIT02 | Hierarchical and automatic multi-stage run with per-item Git operations | Run/stage/item references accurate, no duplicated/fabricated Git success |
| AUDIT03 | Mixed local/remote/Action/AI tied timestamps and filters/pages | Stable global ordering/filter-before-page; every expected event exactly once |
| AUDIT04 | External Git commit and app restart/history open | External activity affects Git-derived view only; no fabricated app operation or request replay |
| AUDIT05 | Retention/restore with orphaned/referenced/interrupted records | Readable truthful history and preserved live recovery; safe metadata only |

## Hierarchical analysis

Layers: new focused Core plan/analysis tests, application scheduler/provider mock integration, renderer context tree and native multi-stage consent. Owner P10.

| Case | Setup and action | Required result |
|---|---|---|
| HIER01 | Same mixed paths/hunks supplied in different discovery order | Identical deterministic group/order/IDs/payload hashes and explicit chunk sources |
| HIER02 | Requests exactly at byte budget and one byte over incl. system/envelope | Exact fit accepted, overflow split/refused without silent omission |
| HIER03 | Single oversize chunk, binary/redacted required selected diff | Explicit omitted/incomplete result, no automatic eligibility |
| HIER04 | Max group/count/depth/time/bytes/response limits and arithmetic overflow | Bounded rejection before unauthorized request; no unbounded recursion |
| HIER05 | Leaf summaries and reduction/final stages | Supplied provenance preserved transitively; actual stage payload viewable |
| HIER06 | Unknown/duplicate/missing chunk attribution at each stage | Refuse invented attribution; incomplete selected coverage remains incomplete |
| HIER07 | Native single-request approval replayed for second stage | Second request refused; fresh approval or specifically bounded run contract required |
| HIER08 | User-approved bounded all-stage contract and newly derived summaries | Every transmitted stage inside endpoint/content/limit contract, exact payload inspection available |
| HIER09 | N groups on same/different connection with configured limits | Every actual HTTP request counted; concurrency never exceeds limits |
| HIER10 | Middle leaf timeout/malformed/unfinished response | Dependent synthesis stops, metadata records failure, no silently dropped group |
| HIER11 | Cancel/expire/scope switch/revoke while held stage and late response | No new stages, owned client reaped gracefully, late results ignored, draft unchanged |
| HIER12 | Host loss between stages then restart | Interrupted metadata only, zero new requests, fresh plan for retry |
| HIER13 | Injection in source/history/summary asks to add endpoint/path/Action | Text stays data, no expanded authority or extra tool/network operation |
| HIER14 | Final candidates with incomplete versus complete required coverage | Assisted warnings truthful; automatic blocked on incomplete; deterministic accepted ranking only among valid choices |

## SemVer Actions

Layers: semver/version-file units, `version_actions.rs`, application barriers, Action renderer tests and actual native. Owner P11.

| Case | Setup and action | Required result |
|---|---|---|
| ACT01 | Preview major/minor/patch from `1.4.2`, LF and CRLF | `2.0.0`, `1.5.0`, `1.4.3`; exact newline preserved, no mutation |
| ACT02 | Prerelease/build metadata, whitespace, malformed/binary/oversized file | Existing explicit policy enforced; invalid input refused with original intact |
| ACT03 | Traverse/symlink/hardlink/nested repo/foreign owner target | Reject unsafe target before write, no outside-root changes |
| ACT04 | Approve definition/invocation then edit target/config/index/HEAD/root identity | Stale invocation refused; preserve external changes |
| ACT05 | Typed definition create/edit/disable, pending invocation | New revision/disable invalidates prior authority, no arbitrary script capability |
| ACT06 | Execute approved frozen output | One exact atomic replacement, original privately retained, modes preserved |
| ACT07 | Crash before/after exchange, then read-only restart and reviewed recovery | Before/after/conflict distinguished, no recomputed second bump/replay |
| ACT08 | External edit during last precheck/exchange or after apply | Newer edit preserved, conflict/manual action visible, no blind rollback |
| ACT09 | Separate stage then exact commit after manual Action | Only intended path/result staged/committed; untracked/excluded/index content preserved |
| ACT10 | Pre-Action AI capture/candidate then Action applies | Old context/candidate/commit review invalid, regenerated context includes actual resulting version |
| ACT11 | Verified version template variable and read-only simulated Action test | Value from deterministic preview/observed result only, model cannot substitute version |
| ACT12 | Revoke/move repo then inspect retained original; native accessibility | Read-only original bytes/fingerprint available via operation ID, no authority or prune |

## Bounded automatic commits

Layers: new Core automatic contract/coordinator tests, real controlled-write/signing/crash fixtures, renderer and independent native authorization. Owner P12. Every case checks no push route is reached and no automatic retry after restart.

| Case | Setup and action | Required result |
|---|---|---|
| AUTO01 | Build contract for selected F01/F02 items | Exact identities/paths/sources/revisions/limits/expiry/hooks/signing/Actions inspected; no action yet |
| AUTO02 | Decline native run or forge serialized run/approval fields | Zero provider/Action/Git writes; renderer/import cannot create capability |
| AUTO03 | Approve bounded single repo, complete analysis/valid candidate | One exact approved-scope commit, tree/message verified independently |
| AUTO04 | Three valid/invalid/tied candidates | DCP04 deterministic valid-only rank/tie selection; no valid eligible candidate blocks |
| AUTO05 | Partial/redacted/failed/truncated required coverage | Automatic commit blocked even if model says complete/high confidence |
| AUTO06 | Unreachable/missing model/auth failure at any stage | Stop affected pending work, no model/manual fallback or push; drafts retained |
| AUTO07 | Mutate selected bytes, excluded raw fingerprint, index, HEAD, branch same OID variants | Material snapshot/generation drift stops for renewed authorization |
| AUTO08 | Change profile/template/instructions/endpoint/auth/tool/trust after consent | Old run invalid, no new destination/fallback, safe metadata records reason |
| AUTO09 | Expire queued/run capability; exceed repo/request/byte/time limits | Stop before next sensitive step; limits exact including hierarchical requests |
| AUTO10 | Cancel queued, analysis, Action, prepublication and postpublication phases | Phase-aware truthful outcome; successes retained, no unsafe rollback or next item |
| AUTO11 | Frozen SemVer Action then reanalysis/commit | Exactly once approved result, verified context/version, only approved path; no old capture |
| AUTO12 | Action fails/conflicts/uncertain exchange | Dependent analysis/commit stops; original and external edit preserved; explicit recovery |
| AUTO13 | Hook stages extra file/rewrites message/changes selected worktree | Reject before ref update; side effects visible, no authority expansion |
| AUTO14 | Positive isolated GPG/SSH; signing failure/wrong-key/timeout | Exact signed commit or clear nonpublication; no unsigned fallback |
| AUTO15 | Selected child success changes parent gitlink | Parent stops for fresh approval under this contract; no silent scope widening |
| AUTO16 | Second repo fails after first success; fresh unfinished retry | First success preserved/not repeated, fresh contract/native consent required |
| AUTO17 | Host SIGKILL before Action, after Action, after ref and between items | Durable truthful interrupted/uncertain history, restart sends/runs nothing automatically |
| AUTO18 | Source/model injection asks to run shell/stage extra/push/add endpoint | Strict schema/scoped conversion refuses; zero unauthorized operation |
| AUTO19 | Watcher event, app startup, candidate completion without approved run | No automatic run/commit/push; capability conversion private to trusted Core |
| AUTO20 | Ordinary installed run UI and audit/restore of old automatic record | Accessible limits/results/Cancel; old persisted/imported run grants no live authority |

## Backup archives and imports

Layers: new Core backup/portability integration suites, adversarial archive pure tests, application/settings views/native inspection. Owners P13A/P13B.

| Case | Setup and action | Required result |
|---|---|---|
| BACKUP01 | AI-only export of configured metadata/definitions | All selected nonsecret entities present, credentials/approvals absent; schema/checksums exact |
| BACKUP02 | Full backup with workspaces/drafts/UI/history option | Exact chosen components and manifest; no repository bytes or secret store |
| BACKUP03 | SQLite WAL writes during snapshot and settings revision change | Coherent db/config generation, integrity valid, no lost/half snapshot |
| BACKUP04 | Retained Action/import/upstream/local recovery artifacts | Inclusion/omission policy explicit and verifiable; unresolved live evidence never deleted |
| BACKUP05 | Destination disk full/permission denied/write/fsync failure | Prior valid archive remains, no success/rotation on partial file |
| BACKUP06 | Crash before temp completion/before final rename/after publication | New backup either absent or verified complete; incomplete never used as valid |
| BACKUP07 | Deterministic content with new timestamps/revision manifest | Checksums/content correctly mapped, logical change detection not based solely on file mtime |
| BACKUP08 | Count/per-entry/expanded/ratio/depth limits at and above bounds | Boundary accepted; overflow rejected while streaming within memory budget |
| BACKUP09 | Repository/credential/agent/Keychain/token sentinel in source state | Forbidden components not exported, no secret in archive/manifest/errors |
| BACKUP10 | Backup attempt during active write/unresolved recovery | Coherent supported snapshot or explicit safe refusal; operation state not falsified |
| BACKUP11 | Validate migration safety backup before new schema | Backup verified/synced before mutation; failure prevents destructive migration |
| BACKUP12 | Cancel backup before/while writing; native inspect outcome | No partial success, active app responsive, previous backup kept |
| IMPORT01 | `../`, absolute, drive/UNC, NUL, mixed-separator traversal members | Reject whole archive before activation, no file outside staging |
| IMPORT02 | Symlink/hardlink/device/FIFO/socket entry variants | Refuse without following/opening attacker target |
| IMPORT03 | Duplicate/case/Unicode collision and manifest extra/missing member | Refuse ambiguous layout, no last-entry-wins |
| IMPORT04 | Wrong checksum/truncated/compression bomb/unsupported encryption/schema | Bounded clear failure, current app data untouched |
| IMPORT05 | Valid AI import rename/replace/skip conflicts by stable ID | Preview exact resulting definitions/mappings, no name-based accidental rebind |
| IMPORT06 | Imported endpoints/auth/Action definitions/trust-shaped metadata | No active endpoint/auth/Action/trust authority, fresh revisions/reauthorization required |
| IMPORT07 | Archive/current settings change after preview; then apply | Preview digest stale, apply refused, no partial state |
| IMPORT08 | Cancel/apply failure during new AI generation activation | Old complete generation active or deterministically recoverable, never mixed settings |

## Restore automatic backups and retention

Layers: new backup/restore crash tests in separate workers, application UI/native and installed candidate. Owner P13B.

| Case | Setup and action | Required result |
|---|---|---|
| RESTORE01 | Preview valid full archive with conflicts/history options | Read-only exact contents/authority resets/limitations and chosen resolution visible |
| RESTORE02 | Current active review/write/AI job or unresolved recovery | Apply safely refused; no current evidence overwritten |
| RESTORE03 | Safety backup fails | Restore does not activate or alter current state |
| RESTORE04 | Valid restore into fresh state | Exact approved drafts/workspaces/config/history, valid DB, no credentials/capabilities |
| RESTORE05 | Restore older known schema and reject newer/corrupt schema | Supported migration backed up; unknown state preserved/refused, no downgrade guessing |
| RESTORE06 | External app acquires writer ownership/change after preview | Exclusive ownership/revision failure, no competing replacement |
| RESTORE07 | SIGKILL at staging, intent, preactivation, postactivation, cleanup boundaries | Restart selects old or new complete generation deterministically; safety copy remains |
| RESTORE08 | Partial file-set rename/fsync/permission/disk-full failure | No mixed active config, recovery instruction and valid old state available |
| RESTORE09 | Restored repo moved/replaced/deleted; same display name elsewhere | Fresh discovery identity, no inherited trust or wrong-name binding |
| RESTORE10 | Restore incomplete/uncertain local/remote/automatic history | Inert interrupted evidence only, no replay/retry/endpoint requests |
| RESTORE11 | Restore Action originals and inspect selected backup limitations | Correct retained bytes or explicit omitted-artifact consequence; no double increment |
| RESTORE12 | Native preview/confirm/cancel/error/restart and installed restore | Keyboard/VoiceOver accessible; exact outcome matches preview; user state not silently destroyed |
| RETAIN01 | No logical state changes over multiple scheduling intervals | No redundant automatic backups/no post-exit daemon |
| RETAIN02 | Burst logical edits, continuous edits and idle flush | DCP03 coalescing/max interval observed, coherent latest accepted snapshot |
| RETAIN03 | More than configured successful automatic backups | Rotate only eligible successful automatic files, keep manual/safety/newest valid |
| RETAIN04 | Disk full/invalid new backup during rotation | Never remove last valid backup to count failed candidate; visible error |
| RETAIN05 | Terminal history older/newer than configured cutoff, exact boundary | Only eligible records pruned per policy, deterministic cutoff/timezone |
| RETAIN06 | Old unresolved/in-flight/uncertain or batch/Action/run-referenced records | Pinned regardless of age; recovery artifacts still inspectable |
| RETAIN07 | Crash between record prune/artifact cleanup and repeated maintenance | Idempotent coherent cleanup, no live artifact deletion/foreign file touch |
| RETAIN08 | Last-window close while backup/prune pending then restart | Owned worker stops safely, no daemon, no destructive work replay without current valid policy |

## Enablement CI security licensing and installed release

Layers: feature graph/native tests, real hosted jobs, dependency/license tooling and actual installed F09. Owners P14/P15/P16.

| Case | Setup and action | Required result |
|---|---|---|
| ENABLE01 | Inspect prerequisite closure table/default-change decision | Every applicable criterion closed/scoped with approval; no inferred enablement |
| ENABLE02 | Ordinary enabled feature graph/binary inspection | Writes enabled only as approved; no crash/lab/native-test feature/menu |
| ENABLE03 | Untrusted ordinary app attempts trust decline, then fresh approved trust/write | No write on decline; exact separate operations after approval only |
| ENABLE04 | Enabled app unsupported filesystem/config/auth and forged requests | Rust fail-closed unchanged, no renderer bypass |
| ENABLE05 | Inspection-only build receives direct mutating request | Rust refuses regardless of UI state/persisted preference |
| ENABLE06 | Enabled no-AI manual individual/bulk/Action/fetch/push native chain | Complete supported manual workflows without AI or automatic publication |
| CI01 | Fresh hosted macOS Core/desktop and Linux supported jobs | Real exit statuses/logs/artifacts with exact source; no local substitute |
| CI02 | Checksum-pinned tool download/build wrong checksum/missing signer variants | Fail clearly before tests/signoff; no unverified global tool fallback |
| CI03 | SSH/GPG private short-temp setup and cleanup | Correct private ancestry/isolated stores; no long-socket/hardening workaround |
| CI04 | Deliberate failing test/format/contract check | Workflow fails and still uploads evidence; no pipe exit masking |
| CI05 | Locked npm/Cargo clean install/cache miss | Declared dependency graph reproducible; install-script behavior known and recorded |
| CI06 | Current full dependency advisory scan and remediation | Timestamped full build/runtime scope, justified dispositions and affected regressions |
| SEC01 | Main/review/unprivileged actual callers and wrong page/origin | ACL plus Rust checks reject forbidden commands; no generic executor |
| SEC02 | Unknown nested fields/version/oversize/foreign workspace IDs | Bounded strict refusal before side effects |
| SEC03 | Replayed/expired/replaced/closed native confirmations | One-use scoped capability unusable; no serialized approval shortcut |
| SEC04 | Hostile filenames/message/source with HTML/control/bidi/argv-looking bytes | Text escaped for presentation, exact authorized raw bytes retained, no command execution |
| SEC05 | Path traversal/symlink/race/foreign lock/native byte encoding | Literal safe paths or explicit supported refusal; no outside mutation |
| SEC06 | Renderer/network request queues/resource limits and cancellation | Bounded memory/jobs/processes; no authority from backlog/late response |
| SEC07 | Search app logs/audit/backup/package for fixture secret/source sentinel | No forbidden retention/export/argv exposure; safe metadata only |
| SEC08 | Release CSP/capabilities/dev origins/assets/test routes | Intended IPC only; dev/test facilities absent, no remote content execution |
| LICENSE01 | Inventory lockfiles then actual shipping feature/platform closure | All shipped components accounted, declaration gaps explicitly resolved |
| LICENSE02 | Obtain actual required license texts/notices and brand attribution | Exact shipped notices match obligations; no guessed third-party license |
| LICENSE03 | Product/first-party npm metadata and distribution decision | Approved consistent policy, no accidental license change |
| LICENSE04 | Inspect installed package/About/offline notice access | Required texts available and readable, source-offer obligations handled if applicable |
| RELEASE01 | Clean supported install and first launch | Correct identifier/architecture/tools, no-AI manual work, no inherited state/trust |
| RELEASE02 | Unsupported Git/version/OS/architecture prerequisite | Clear diagnostic before unsafe writes, no automatic global installation/config |
| RELEASE03 | Upgrade prior supported app/data, cancel/fail migration | Exact old data preserved/safety backup; supported migration coherent |
| RELEASE04 | Identifier/path conflict/newer/corrupt destination during migration | Refuse guessing/overwrite, old state intact, actionable resolution |
| RELEASE05 | Signed/notarized/stapled exported installer on clean machine | Signature/entitlements/Gatekeeper verified on exact hashed package |
| RELEASE06 | Installed partial-stage/mixed selection/positive GPG+SSH workflows | Exact tree/message/index/work/untracked preservation and signatures |
| RELEASE07 | Installed authenticated fetch/upstream/individual and queue push | Correct independent approvals/server refs, no implicit publication |
| RELEASE08 | Installed Action/assisted/hierarchical/automatic scope if shipped | Exact scopes/no injection/no fallback, optional AI manual invariant |
| RELEASE09 | Installed crash/recovery and last-window shutdown | No replay/double commit/bump, actual processes stop gracefully and lock reopens |
| RELEASE10 | Installed backup/import/restore/retention | Exact preview/content/authority resets and interruption safety |
| RELEASE11 | Installed accessibility/layout/performance supported envelope | DCP01/DCP08 budgets and native VoiceOver checks pass |
| RELEASE12 | Reinstall/uninstall and final manifest/notes/checksum review | User state preserved, no daemon/global config changes, all 109/15 dispositions current; release prepared without unauthorized publication |

## Case coverage and signoff

Each package report lists its case IDs, implementation test names/native evidence paths and every variant result. Retained historical evidence is acceptable only for its actual criterion and unchanged boundary; exact installed release acceptance is always tied to the new package hash. Unknown/blocked fields cannot be converted to passed.

No artificial test count is an exit condition. The required result is that the specific assertions above and all mapped requirements hold under the approved support envelope, with independent observations and reproducible evidence.
