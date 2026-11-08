# Commit Command — current implementation and acceptance record

Updated November 8, 2026. Current disposition incorporates the dated reports and
validated checkpoints below; the original reconciliation baseline was `db29ba6`. This is the single maintained record of **current implementation,
acceptance gaps and next work**. The specification and decision register define
requirements; the crosswalk routes requirement IDs to code/tests; dated reports
and validation artifacts preserve evidence. Older status statements do not override
this record. Reconciliation does not accept new gates or change product scope.

### November 8 G11 CI prerequisites — implemented, hosted run pending

[Provisioning checkpoint](reports/G11_CI_PREREQUISITES_2026-11-08.md): workflow now
provisions checksum-pinned Git into runner-temporary prefixes and macOS GPG with
explicit test signer/short TMPDIR. Actual isolated local Git source build, SHA256
commit/blob smoke, helper/prerequisite checks, four archive-layout tests, YAML and
shell syntax passed. Hosted CI/Linux/Brew steps and distribution acceptance unrun;
no push/release. G11 is partial. This independent prerequisite work ran while native
G05 exact fixture-trust authorization was pending; remaining gates are not skipped.

### November 8 G05 child/parent native batch — ready for fixture trust

[Prepared package](reports/G05_NATIVE_SUBMODULE_2026-11-08.md): actual disposable
parent/submodule plus dirty unselected control scanned; fresh acceptance artifact
rendered exact two-worktree native trust review. No trust/write rows. Owning Core
submodule ordering/fresh-parent regression passed. Exact fixture trust authorization
requested; native mutation awaits reply. Independent work continues; no G05 pass
is inferred and G04 remains disabled.

### November 8 G03 measured scale / partial scan recovery

[Measured correctness fix](reports/G03_SCALE_PARTIAL_INVENTORY_2026-11-08.md): all six
100/1,000/10,000-repository scans previously discarded results on deadline. Discovery
now returns inspected entries with explicit incomplete/timeout issue; cancellation
contract preserved. Repeated runs retain 88–93 entries near 15 seconds. Workspace 20,
application 27, strict Core Clippy and full desktop gate (87/13/16) passed. Fresh
ordinary native startup/live Rescan showed timeout details with retained inventory;
exact repository preservation, no trust, Quit 0 and lock reopen passed. This closes
the partial-result loss defect, not scale usability: incremental progress and broader
G03 criteria remain open. G02 facility blocks retained; G04 still disabled.

### November 8 G02–G11 continuous execution

User authorized proceeding through G02–G11 and local add/commit after each completed,
validated step without advance prompts. Gate closure still requires its owning
criteria; no blocked case becomes a pass. G04 depends on G02/G03/G05/G06, and G08
on G07, so work follows dependencies when numeric order cannot close a gate.
Live user services/credentials, publication/release and security-sensitive approvals
retain their existing boundaries. AI stays optional under D-048.

G02 disposition audited against later reports: deterministic initial-commit defect
already fixed, bounded restarts/interruption evidence retained, broad reliability
unproven. Current Dock access returned: “Computer Use is blocked from using the app
'com.apple.dock' by your organization's policy.” No alternate route used. Operator
session termination is separate and requires coordinated operator execution.
Continue unblocked G03 coverage while keeping G02 partial; do not repeat accepted
interruption variants solely to accumulate passes.

### November 8 optional AI requirement confirmed

D-048 makes AI an optional enhancement: supported manual workflows require no AI
configuration/service; provider/model failures do not block manual work or discard
drafts. Deterministic user-selected SemVer increments work without AI. Specification
and M6/M7 plan now require no-AI/offline-provider acceptance. This is a requirement
clarification, not implemented AI functionality or a new gate pass.

### November 8 native SHA-256 offline import — complete

[Final import package](reports/G06_NATIVE_SHA256_IMPORT_2026-11-08.md): retained
capture and existing authorized fixture trust reused. Cancel preserved repository;
fresh native exact import approval completed offline, adding only the full SHA256
tracking ref and one reflog entry. Ordinary restart retained complete/cancelled
chronology, writes disabled, no replay/extra request. Exact file/mode, artifact and
journal oracles passed; both exits 0 and writer lock reopened. Server remains stopped.
Objects already existed locally, so missing-object transfer is not accepted by this
case. No product patch/new trust. Native SHA256 missing-object/upstream/check and
other G02/G06 criteria remain open.

### November 8 native SHA-256 observation/download — complete

[Final native format package](reports/G06_NATIVE_SHA256_2026-11-08.md): exact fixture
trust explicitly authorized, untrusted refusal and Cancel/no-network passed. Fresh
native HTTPS observation returned the real full SHA-256 OID; separately approved
isolated download produced a verified standalone bundle (3 objects, 428 bytes).
Repository/remote file bytes/modes remained exact. Ordinary restart retained full
OID/history with writes disabled; no replay/additional request, both Quit exits 0,
writer lock reopened. Test service stopped/key removed. No import/commit/push or
product patch. Selected native SHA-256 observation/download gap closed; native
import/upstream/check topology and other G02/G06 criteria remain partial/open.

### November 7 full Core signing matrix — complete

[Final signing package](reports/G06_FULL_SIGNING_MATRIX_2026-11-07.md): isolated pinned
GPG restored; unchanged full five-configuration Core format/build/test/strict-Clippy
script passed. Tests: default 58, experimental 93, controlled 83, application 227,
all features 271 passed, zero failures. Production OpenPGP/SSH verification and
wrong-key/timeout refusal passed wherever controlled writes are enabled. Declared
worker/facility exclusions remain explicit. Legacy long-TMPDIR agent failure was
reproduced and resolved using process-local short TMPDIR; optional signing passed
explicitly in both configurations and final lab passed 24/24, zero skips. No product
patch or global install. This closes the missing-signer/full Core matrix gap;
G02/G06 remain partial for native/facility/accessibility criteria.

### November 7 current-source automated transport subset — complete

[Final compatibility package](reports/G06_CURRENT_TRANSPORT_MATRIX_2026-11-07.md):
application-services inspection/execution 10/81 passed (2 declared ignored);
all-feature inspection/execution 10/85 passed (4 declared ignored, including parent-
driven subprocess workers). Format, strict target Clippy and source-manifest check
passed. Live disposable HTTPS/SSH, isolated credential state, SHA-256, crash/storage
recovery and check-only hook/dependency coverage refreshed without a product patch.
This closes the selected automated remote-target refresh only. Full five-configuration
signing matrix still needs isolated GPG; native SHA-256/topology and facility-dependent
criteria remain open. G02/G06 remain partial.

### November 7 repository remote-status indicators

[Remote-status report](reports/REPOSITORY_REMOTE_STATUS_2026-11-07.md): the overview
table now shows no remote, missing upstream, local up-to-date/ahead/behind/diverged
counts and unknown/unavailable states, with expandable configuration/comparison/time
details. No automatic remote contact. Legacy snapshots remain unknown until local
rescan. Full desktop checks passed (79 / 13 / 16 tests); Core inspection 12 passed,
one existing filesystem ignore. Selected ordinary native presentation verified.
This is inspection/UI work; G02/G06 and ordinary write enablement remain open.

### November 7 post-load startup follow-up

[Diagnosis, patch and bounded retests](reports/G02_POST_LOAD_STARTUP_2026-11-07.md):
a deterministic initial React commit ordering defect is fixed. Full checks passed
(70 renderer / 13 default-host / 16 all-feature host tests); three native launches,
including first restarts after renderer and host loss, recovered without manual
reload and preserved exact draft/history/repository state with no replay. Diagnostic
launches did not recapture the intermittent blank, so broader startup reliability
remains open; earlier blank evidence is retained. G02/G06 stay partial.

### November 7 repository identity recovery

[Recovery implementation and acceptance](reports/REPOSITORY_IDENTITY_RECOVERY_2026-11-07.md):
local Rescan and review separates verified same-path replacements from historical
identities, retains drafts/history, blocks historical execution and invalidates pending
reviews without transferring trust. Missing/failed inspection remains unresolved.
17 workspace / 26 application tests and full desktop checks (83 / 13 / 16) passed;
selected ordinary native recovery presentation verified. G02/G06 remain partial;
broader lifecycle/accessibility/default-write gates are unchanged.

### November 7 live-workspace identity recovery verification

[Current-workspace follow-up](reports/REPOSITORY_IDENTITY_RECOVERY_2026-11-07.md#current-workspace-follow-up):
explicit native Rescan and review preserved separation of 11 unique available current
repositories and 10 historical identities. Table/Tree/sidebar and read-only database
assertions agree; no duplicate-separation code fix needed. **Discovery remains
incomplete:** metadata inspection failed in two `.tools/cache/sdists-v10` locations.
Zero repository-attention count does not establish scan completeness. Cache traversal
and scan-level issue presentation remain open. No new automated suite; G02/G06 partial.

### November 7 cache discovery / scan-status closure

[Diagnosis and verification](reports/DISCOVERY_CACHE_SCAN_STATUS_2026-11-07.md):
the two cache errors came from empty `.git` markers within valid tagged cache subtrees.
Discovery now skips and lists verified tagged caches, preserving real malformed-marker
errors. Native current-workspace rescan reports complete: 11 repositories, zero issues,
six tagged caches skipped; 10 historical identities remain separated. Last-scan summary,
error paths and exclusions are now visible independently of repository attention counts.
18 Core workspace / 27 application tests passed; full desktop gate passed (86 renderer /
13 / 16), followed by 87 renderer tests and static checks after final integration coverage.
The previously recorded cache/scan-presentation findings are closed within this envelope;
broader G02/G06/accessibility/scale remain open.

### November 7 G02 active-scan main-window Close

[Native case report](reports/G02_CLOSE_DURING_SCAN_2026-11-07.md): fresh ordinary
80-repository fixture; Rescan in progress, live Git children sampled, then native
main-window Cmd+W entered safe shutdown and host exited 0. First restart rendered
without reload and recovered the exact draft; final Quit 0 and state writer lock
reopened. Exact repository file bytes/modes preserved, zero trust/write/remote records.
Selected M3 plan section 12 step 1 passed within this envelope. No source change or
new automated suite. G02 remains partial; G06 remote compatibility is unchanged.

### November 7 G02 active-scan native host loss

[Forced read-interruption report](reports/G02_HOST_LOSS_DURING_SCAN_2026-11-07.md):
fresh ordinary 80-repository fixture, native scan request active and live Git child
proven before exact host-only SIGKILL. SQLite integrity and prior one-entry completed
inventory preserved; first restart rendered and recovered saved draft without reload.
Explicit fresh Rescan completed with 80 entries/zero issues; final Quit 0 and writer
lock reopened. Repository file bytes/modes match baseline; zero trust/write/remote
records. Selected bounded G02 case passed; no code fix or new automated suite.
G02 remains partial, including Dock/session and broader startup/phase coverage;
G06 remote compatibility is unchanged.

### November 7 G02 active-scan Cancel

[Native cancellation report](reports/G02_CANCEL_ACTIVE_SCAN_2026-11-07.md): fresh
ordinary fixture, active Rescan plus live Git child proof, then native Cancel returned
Inspection cancelled and restored controls. Prior one-entry completed inventory retained;
no partial discovery published. Explicit fresh Rescan completed with 80 entries/zero
issues, exact draft retained, normal Quit 0 and writer lock reopened. File bytes/modes
preserved, SQLite valid, zero trust/write/remote records. Selected G02 read-cancellation
case passed within this envelope; no code fix or new automated suite. G02 remains
partial; G06 remote compatibility is unchanged.

### November 7 G02 missing-snapshot explicit fallback exit

[Native fallback-exit report](reports/G02_CLOSE_WITHOUT_SNAPSHOT_2026-11-07.md): fresh
marked fixture and existing non-default test support (desktop writes disabled), one
suppressed close-request delivery, then explicit native Close without saving exited 0.
Saved presentation payload and repository file bytes/modes preserved; ordinary first
restart rendered/recovered saved draft without reload, normal Quit 0 and lock reopened.
Selected exit branch passed, complementing prior Keep app open evidence. Already saved
draft used: no unsaved-edit loss/recovery claim. No product fix or new full automated
suite. G02 remains partial; G06 remote compatibility is unchanged.

### November 7 G06 executable-helper refusal preparation — authorization blocked

[Prepared native case](reports/G06_NATIVE_REMOTE_HELPER_REFUSAL_2026-11-07.md): fresh
disposable `ext::` remote-helper sentinel fixture and acceptance build prepared.
Automatic approval review rejected the native fixture trust grant because specific
access-change authorization is missing. User authorization requested; no bypass used.
Read-only checks confirm unchanged repositories, no helper invocation and zero
trust/write/observation rows. **No remote-refusal pass claimed.** Resume only after
specific fixture authorization; G02/G06 remain partial. No product patch.

### November 7 G06 executable-helper refusal — authorized native pass

[Completed native case](reports/G06_NATIVE_REMOTE_HELPER_REFUSAL_2026-11-07.md): user
explicitly authorized only the prepared disposable worktree trust, resolving the
previous automatic-review block. Native inspection classified/withheld the executable
helper destination; origin/main observation preparation refused unsupported configuration
before transport review. Exact files/modes preserved, sentinel never invoked, exactly
one scoped trust record and zero observation/write/other remote-operation records.
Acceptance and ordinary-restart Quit 0, final oracle unchanged and writer lock reopened.
Selected unsupported-helper refusal passed; no code fix/full automated rerun. G06 remains
partial; unsupported helpers remain unsupported and other G02/G06 limits persist.

### November 7 G06 observation review / configuration drift

[Native stale-configuration case](reports/G06_NATIVE_OBSERVATION_CONFIG_DRIFT_2026-11-07.md):
same explicitly authorized disposable repository, isolated state; renewed scoped trust,
prepared exact observation review, then deliberately changed only remote config.
Old native confirmation retained reviewed URL; approval returned blocked/untrusted
before transport initialization, revoked trust, no OID/time. Ordinary restart retained
identical history without replay; both exits 0, writer lock reopened. Exact expected
repository baselines preserved; original config restored and previous case reverified.
Selected trust/config-drift barrier passed; later URL-digest guard not independently
isolated. No source fix/full-suite rerun. Other G02/G06 criteria remain open.

### November 7 G06 same-OID observation generation drift

[Native generation-barrier report](reports/G06_NATIVE_OBSERVATION_GENERATION_DRIFT_2026-11-07.md):
same authorized disposable repository/isolated state; prepared observation, switched
to an existing alternate branch at the identical OID without config/index changes,
then approved the old exact review. Core blocked stale_review while trust stayed
current, with no transport storage/OID/time. Ordinary restart retained identical
history, both exits 0 and writer lock reopened. Expected file baselines preserved;
fixture restored and earlier case reverified. Selected generation guard passed beyond
stale-trust coverage; URL/auth/tool drift checks not independently isolated. No product
fix/full-suite rerun. Other G02/G06 requirements remain open.

### November 7 G06 linked-worktree upstream native chain

[Native shared-config topology report](reports/G06_NATIVE_LINKED_UPSTREAM_2026-11-07.md):
real approved loopback HTTPS observation, separately approved missing-only upstream
setup completed/updated, exactly two shared-config keys visible from both worktrees.
Both checkouts/common metadata otherwise unchanged; linked branch policy untouched,
main trust revoked, no linked trust grant. Ordinary restart preserved both completed
records/config with only one server GET total. Focused Core topology test passed;
independent file/journal/request oracles passed, both exits 0 and lock reopened.
Selected native linked-worktree upstream success gap closed; fixture restored/service
stopped/key removed. No product patch/full-suite rerun. G06 remains partial; prioritize
named recovery/topology packages rather than additional already-covered guard variants.

### November 7 owned-config-lock recovery package — complete

[Final recovery package](reports/G06_NATIVE_OWNED_CONFIG_LOCK_2026-11-07.md): native
Cancel preserved the ready lock; empty, ready and pre-publication owned-lock recovery
returned aborted/unchanged without replay or config publication. Stale approval and
same-byte foreign-inode replacement were refused; fresh review required manual
investigation. After the fixture operator restored the original owned inode, fresh
native recovery passed. Ordinary restarts retained resolved history with writes
disabled. All four final file/mode/journal/remote oracles and writer-lock reopens
passed; foreign evidence remains intact. No native trust grant or product patch.
This selected package is closed; G02/G06 remain partial. Earlier progress entries
below/above are historical checkpoints, not remaining work in this package.

### November 7 G06 native owned-config-lock recovery — in progress

[Recovery package](reports/G06_NATIVE_OWNED_CONFIG_LOCK_2026-11-07.md): existing Core
subprocess worker created actual interrupted empty/ready/pre-publication locks and
ownership receipts. SIGKILL/phase/config/receipt assertions passed; exact baselines
and non-default artifact provenance retained. Preparation is automated Core evidence,
not native recovery acceptance. Continue Cancel, owned recovery, foreign/stale refusal
and ordinary restart preservation. No product patch; G02/G06 remain partial.

### November 7 owned-lock recovery — ready candidate checkpoint

[Recovery package step 2](reports/G06_NATIVE_OWNED_CONFIG_LOCK_2026-11-07.md): actual
ready-candidate lock preserved on startup and Cancel. Fresh native abort_unpublished
approval removed only owned config.lock, returned aborted/unchanged; config/repo/remote/
transport snapshots otherwise exact. Ordinary restart retained resolved history with
no replay/pending action; both exits 0, writer lock reopened. Empty/pre-publication
and foreign/stale cases still in progress. No product patch; G02/G06 partial.

### November 7 owned-lock recovery — boundary cases passed, negative cases ongoing

[Recovery package steps 3–4](reports/G06_NATIVE_OWNED_CONFIG_LOCK_2026-11-07.md): native
LockCreated empty-lock and PublicationPending candidate-lock recoveries both returned
aborted/unchanged, removed only receipt-proven config.lock, preserved original config
and all other repo/remote/transport files/modes. Both normal exits 0 and writer locks
reopened. Foreign-lock/stale-fingerprint negative cases continue; package not yet closed.

## Status meanings and availability

- **Accepted within envelope:** the recorded milestone criteria passed only for
  the documented environment and exclusions.
- **Implemented / acceptance partial:** source exists and has automated and/or
  selected native evidence; the full product criterion remains open.
- **Not implemented:** required behavior still needs code and acceptance.
- **Deferred / unsupported:** explicitly outside first-release scope or the
  supported execution envelope; this is not a pass.
- **Environment blocked:** a check could not exercise its intended behavior;
  historical passes remain historical and are not replaced by a new pass.

Ordinary desktop builds allow local inspection and app-state persistence, but
reject trust/write/network execution in Rust. `desktop-writes` remains a non-default
acceptance feature. Default Core builds expose no managed-repository writes.
Local publication means a local ref advanced; it does not mean a server push.
No push executor exists. Neither planning nor a completed hook check grants push
authority. No default change, commit, push or release is authorized by this record.

## Current capability and milestone disposition

| Area | Current disposition | Evidence / remaining gate |
|---|---|---|
| P0 baseline | Accepted historical baseline | [P0 report](reports/P0_BASELINE_2026-11-02.md); current checks below do not replace the full baseline matrix |
| M0–M1 Core inspection/workspaces | Implemented and accepted within recorded local envelope | [M1 contracts](architecture/M1_READ_ONLY_WORKSPACES.md), [M1 ledger](ACCEPTANCE_MATRIX.md#10-m1-acceptance-gates--completed-local-implementation); desktop/scale criteria remain separate |
| M2 controlled stage/unstage/commit | Implemented and accepted within bounded macOS/APFS envelope | [M2 contracts](architecture/M2_CONTROLLED_WRITES.md), [M2 ledger](ACCEPTANCE_MATRIX.md#11-m2-acceptance-gates--controlled-writes); ordinary desktop exposure blocked by G01–G04 |
| M3 desktop/lifecycle/reviews | Implemented; acceptance partial | [M3 ledger](ACCEPTANCE_MATRIX.md#12-m3-acceptance-gates--implementation-with-native-gates-open), [close/Quit](reports/M3_NATIVE_CLOSE_QUIT_2026-11-02.md), [review lifetime](reports/M3_REVIEW_LIFETIME_2026-11-02.md), [geometry](reports/M3_MIXED_DPI_WINDOW_RESTORE_2026-11-02.md); G01–G04 |
| Branding/About | Selected automated/native visual acceptance passed | [Branding report](reports/COMMIT_COMMAND_BRANDING_ABOUT_2026-11-03.md); internal storage identifier preserved; release branding/signing work remains G11 |
| M4 graph/activity/audit | Implemented; acceptance partial | [Graph/activity](reports/M4_READ_ONLY_PROGRESS_2026-11-02.md), [unified audit](reports/M4_UNIFIED_OPERATION_AUDIT_2026-11-02.md); G03, G10 |
| M5A bulk local commits | Implemented; acceptance partial | [Bulk implementation](reports/M5A_LOCAL_BULK_PROGRESS_2026-11-02.md), [native bulk](reports/M5A_NATIVE_BULK_2026-11-02.md); G05 |
| M5B inspection/authentication/fetch/upstream | Implemented for supported subset; acceptance partial | [Fetch native chain](reports/M5B_NATIVE_FETCH_ACCEPTANCE_2026-11-02.md), [authentication](reports/M5B_AUTHENTICATION_COMPATIBILITY_2026-11-02.md), [SSH](reports/M5B_SSH_AGENT_TRANSPORT_2026-11-02.md), [upstream](reports/M5B_GUARDED_UPSTREAM_2026-11-02.md); G06. Artifact initialization finding [fixed](reports/M5B_ARTIFACT_INITIALIZATION_FIX_2026-11-02.md) |
| M5C push preview/child evidence/planning | Implemented; selected native acceptance passed | [Relative child evidence](reports/M5C_RELATIVE_CHILD_ACCEPTANCE_2026-11-02.md), [multi-level planning](reports/M5C_DEPENDENCY_PLANNING_2026-11-03.md), [native planner](reports/M5C_PLANNER_NATIVE_ACCEPTANCE_2026-11-03.md); G03, G06–G08 remain; no publication |
| M5C independently reviewed pre-push hook check | Implemented; selected automated/native acceptance passed | [Check report](reports/M5C_REVIEWED_PUSH_CHECKS_2026-11-03.md); hook approval only, unpublished history/no replay; selected native interruption/manager-refusal/bypass cases passed; broader G06 compatibility remains |
| M5C individual/bulk push publication | Not implemented | Exact publication authority, transport, rejection, uncertain outcome reconciliation and child-success coordination: G07–G08 |
| M6 AI foundation | Not implemented | `app/core/src/ai/` contains only `.gitkeep`; profiles, providers, templates and exact context pipeline: G09 |
| M7 AI commits / SemVer Actions | Not implemented | `app/core/src/actions/` contains only `.gitkeep`; deterministic recovery and separate bounded automatic authority: G09 |
| M8 backup/retention/usability/release | Partial foundations only | Atomic storage, migrations and development bundles exist; general backup/restore, retention and release acceptance do not: G03, G10–G11 |

## Open acceptance and implementation gates

These stable gate IDs group the acceptance queue; they supplement, rather than
renumber, original requirement IDs and M3-A criteria. G01 is **accepted within its
recorded envelope**; G02–G11 remain **open**. Accepted rows remain here for traceability. Close a row only
with criterion-level evidence, exact source/artifact/features where applicable,
and explicit disposition of exclusions. A facility block is not a waiver.

| Gate | Remaining work and closure evidence | Requirement / plan ownership |
|---|---|---|
| G01 — native authority/review fidelity | **Accepted within envelope, November 7:** remaining actual-webview caller/page/origin and pending/issued-confirmation reload cases passed using non-default native test support. Combined with prior exact-content/signing/lifecycle evidence, the enumerated G01 cases are closed. [Closure report](reports/G01_NATIVE_SUPPORT_G02_RECOVERY_2026-11-07.md). Accessibility, broader compatibility and ordinary write exposure remain G03/G06/G04. | M3-A12–A15; SEC-001–012, SEC-018–020; M3A/B |
| G02 — lifecycle and recovery | **Partial; facility blocks retained.** Initial React commit ordering defect fixed; renderer/host-loss and bounded restart/no-replay cases passed ([post-load report](reports/G02_POST_LOAD_STARTUP_2026-11-07.md)). Earlier intermittent blank remains counterevidence; capture diagnostics on recurrence before reload, rather than treating document Finished as proof of content. Actual Dock access rechecked November 8: organization policy still denies com.apple.dock; no surrogate accepted. Operator session termination and specifically uncovered phase/recovery cases remain open. | M3-A16–A17; SEC-013–017; WS-014; M3B |
| G03 — accessibility, layouts and scale | Full VoiceOver/focus/review/dialog coverage, long/byte paths and large content, narrow/zoom layouts, display disconnect/reconfiguration, measured inventory/history/monitoring budgets. Selected contrast/theme/1×–2× geometry passes remain valid only for their recorded cases. | UI-001–005; M3-A04/06/07/19; M4; M8B |
| G04 — ordinary write enablement | Reconcile all applicable safety prerequisites with G01–G03/G05–G06 and support exclusions, then obtain the explicit default-change decision. Validate the exact ordinary artifact with supported writes and independent approval. No current gate completion is inferred. | D-045; M3C; RELEASE-002 |
| G05 — native bulk commit failures/dependencies | Actual submodule child-before-parent/fresh parent review, partial publication/failure, cancellation and reviewed retry of unfinished items; per-repo overrides and linked-worktree cases as applicable. Preserve successful/unselected content and do not replay published items. | GIT-013; BULK-001–007; M5A |
| G06 — supported remote/check compatibility | **In progress:** selected native executable-helper refusal after explicit fixture trust passed ([report](reports/G06_NATIVE_REMOTE_HELPER_REFUSAL_2026-11-07.md)); helpers remain unsupported.  native supported pre-push Cancel review, Cmd+Q, main-window Close and direct AppKit termination during proven running execution passed, including process cleanup and restart without replay ([AppKit report](reports/G02_G06_RUNNING_PUSH_CHECK_APPKIT_2026-11-07.md)). Actual running-check main-renderer loss now passed with host survival/cancellation/no replay; a subsequent normal restart blank reopens G02 presentation ([renderer report](reports/G02_G06_RUNNING_PUSH_CHECK_RENDERER_LOSS_2026-11-07.md)). Earlier running-check host crash and first-restart recovery passed in their fixture; original hook survival remains explicit ([startup retest](reports/G02_STARTUP_READINESS_2026-11-07.md)). Selected native detected-manager Execute refusal, OS-declined bypass and fresh approved bypass passed with zero hook execution and restart/no replay ([bypass report](reports/G06_NATIVE_MANAGER_BYPASS_2026-11-07.md)); unsupported managers remain unsupported for Execute. Operator SecurityAgent choices, broader fetch/upstream topology/recovery, planner cycle/accessibility/scale and direct SCP facility block retain their limits. | M5B/C; PUSH-002; AUTHENTICATION.md; M8B |
| G07 — individual push | Implement independent exact non-force publication approval, isolated transport, source/destination/auth/hook revalidation, rejection/drift handling, cancellation and durable uncertain-outcome reconciliation using fresh observation. Existing hook checks never become publication tokens. | PUSH-001/003/005/006; INV-009; M5C |
| G08 — bulk push | Compose accepted individual push with fresh per-item authority, actual child availability before parents, no implicit child selection, cycle/block handling, partial-success preservation and fresh retry. Conditional planning is not remote success. | PUSH-004–006; INV-014; M5C |
| G09 — AI and Actions | Implement M6 profiles/providers/templates/bounded exact context; M7 assisted candidates, deterministic SemVer preview/execution/recovery and separately bounded automatic authorization. Provider/model failures and imported definitions cannot broaden authority. | AI-001–017; ACT-001–003; INV-010–013; M6/M7 |
| G10 — backup/portability/retention | Versioned export/import, consistent manual/rotating backups, previewed safe restore, malicious/interrupted archive tests and protection of unresolved recovery. Decide proposed 90-day history/ten-backup defaults before implementing cleanup. Migration backups alone do not meet this gate. | STATE-005–007; ACT-004; M4C/M8A |
| G11 — reproducible release | Provision declared Git/Rust/signing prerequisites in CI and record actual results; resolve OS/architecture/tool delivery/identifier/signing/update policy; verify licenses/notices and exact packaged install/upgrade/migration/uninstall, signing/notarization and release invariants. A development bundle is not a release. | RELEASE-001–005; INV-001–015; M8C |

### Latest G02/G06 running-hook renderer loss / startup counterevidence — November 7, 2026

[Renderer-loss report](reports/G02_G06_RUNNING_PUSH_CHECK_RENDERER_LOSS_2026-11-07.md)
and [evidence](validation/g02-g06-running-push-check-renderer-loss-2026-11-07/README.md)
record actual main WebKit process SIGKILL after independent running-hook proof.
Same host survived; original renderer/hook/child were absent 18.7 seconds after hook
start, before natural finish. Main reset/reloaded, obsolete review closed, and exact
draft/cancelled unpublished history recovered automatically with no replay.

**Reliable startup presentation is reopened.** After a later normal Quit/restart,
the window was blank across native AX/screenshot observations despite a Finished
page-load event and successful initial show. Explicit native Reload main renderer
restored draft/history in that same host without replay. Page completion/display
therefore cannot by itself establish painted application content. The earlier
readiness patch remains, but its passes do not close this broader startup symptom.
Next: investigate the post-load blank restart using this retained fixture/evidence.

Exact repository/remote/draft/journal/request assertions passed after renderer
recovery, restart and final cleanup; one hook invocation and one discovery GET.
Both app exits were 0; original hook/child remained absent past natural finish,
owned service stopped and generated TLS key removed. No product source changed;
prior automated validation is historical. G02/G06 remain open; this checkpoint
accepts running renderer-loss cancellation, not automatic restart presentation.

### Latest G06 native manager refusal / explicit bypass — November 7, 2026

[Native bypass report](reports/G06_NATIVE_MANAGER_BYPASS_2026-11-07.md) and
[evidence](validation/g06-native-manager-bypass-2026-11-07/README.md) close the
selected detected-manager refusal/decline/bypass cases. A fresh fixture hook had a
Husky signature installed before trust. Execute returned unsupported state before
proposal/approval. Explicit BypassAll was visible in authoritative and OS reviews;
OS Cancel left a cancelled record and consumed review. A fresh separately approved
BypassAll check completed with hook not passed and no publication. Neither ran the
hook. This is detector/refusal coverage, not installed-manager Execute support.

Native history and first restart preserved distinct cancelled/complete bypass
records and the exact draft without retry/publication authority or replay. All
repository/remote bytes/modes and journal/request assertions passed at refusal,
decline, approval, restart and final cleanup: zero hook markers and one observation
GET. Both app exits were 0; exact owned service stopped and generated key removed.
No product source changed; unchanged validated artifact reused. Prior automated
matrix remains historical. One early final-phase assertion was run while pending;
the unchanged oracle passed after native completion, with the timing note retained.

G06 remains open for broader authentication, fetch/upstream topology/recovery,
planner cycle/accessibility/scale and SCP facility limits. G02 dispositions are
unchanged; actual Dock/session and remaining phase-specific recovery remain open.
Ordinary writes stay disabled; no publication authority was added.

### Latest G02/G06 direct AppKit termination during running hook — November 7, 2026

[AppKit report](reports/G02_G06_RUNNING_PUSH_CHECK_APPKIT_2026-11-07.md) and
[evidence](validation/g02-g06-running-push-check-appkit-2026-11-07/README.md)
record the fixed native test action calling actual NSApplication termination from
the active review while a supported hook was independently proven running. Audit
confirms request, dispatch, delegate interception and native return in order, with
no attached confirmation sheet. The host exited 0; exact host/hook/child were absent
at 43.7 seconds after hook start, before natural finish. No external host/hook/child
signal was sent. The raw check became cancelled/not passed/unpublished.

First restart restored the exact draft and cancelled history, released the state
lock and caused no hook/request/operation replay. Exact working/bare repositories,
journal, observation and request evidence were preserved through final cleanup.
One hook invocation and one discovery GET; no completion marker beyond the natural
interval. Both app exits were 0, owned service stopped, generated TLS key removed.
No product code changed; exact artifact/source provenance matched the prior full
desktop gate. Earlier automated counts remain historical, not a new test run.

This direct AppKit running-check case passed. G02/G06 remain open for actual Dock
(policy blocked), operator-controlled session termination, remaining phase-specific
recovery requirements, native bypass/manager and broader compatibility criteria.
Ordinary writes stay disabled; hook acceptance grants no publication authority.

### Earlier G02/G06 main-window Close during running hook — November 7, 2026 (historical)

[Close acceptance report](reports/G02_G06_RUNNING_PUSH_CHECK_CLOSE_2026-11-07.md)
and [evidence](validation/g02-g06-running-push-check-close-2026-11-07/README.md)
record native Window > Close Window (`performClose:`) with the main window foreground
and the authoritative review still open. Independent proof immediately before Close
found raw running/unpublished state and exact live host/hook/child. Host exited 0;
all three PIDs were absent 33.6 seconds after hook start, before its 45-second
natural finish. No external host/hook/child signal was sent.

The check became cancelled/not passed/unpublished. First restart restored the exact
draft and cancelled history, released the state lock, and added no hook invocation,
request or operation. Exact repository/bare-remote snapshots were preserved through
final cleanup beyond the natural hook interval. The service stopped and generated
TLS key was removed. No product source changed; artifact/source hashes matched the
startup-readiness checkpoint. Prior full automated checks remain historical.

This main-window Close case passed; the subsequent direct AppKit running-check
pass is recorded above. G02/G06 remain open. Actual Dock is
policy blocked; session termination and native bypass/manager criteria remain
separate. Ordinary writes stay disabled; no publication authority was added.

### Earlier G02 startup readiness fix and native retest — November 7, 2026 (historical)

[Fix/retest report](reports/G02_STARTUP_READINESS_2026-11-07.md) and
[evidence](validation/g02-startup-readiness-2026-11-07/README.md) record the
reproduced transient startup exposure, committed baseline `a89ecc6`, and fix:
keep the initial main window hidden until its document finishes loading. An
HTML startup status covers React mounting. Later renderer navigation does not
repeat initial display; approval and write defaults are unchanged.

A fresh actual running-hook SIGKILL test passed **on the first restart**, restoring
draft and interrupted/not-passed history with no replay. A further idle-host crash
also passed first restart. Normal renderer reload and actual renderer-process loss
recovered. Three audited startups were hidden through document completion, each
with exactly one initial display. Repository/remote/draft/request/journal oracles
passed. Original hook survived and finished naturally, invocation count one.

Fresh full desktop gate passed: 69 renderer / 13 default-host / 16 all-feature-host
tests, type/contract/format/lint, strict Clippy and debug/release builds. Fixture
service/key cleaned up. Premature-display finding was closed within that envelope;
the newer blank normal restart above reopens reliable startup presentation despite
Finished/display audit. G02/G06
remain open for other criteria. The subsequent main-window Close pass is recorded
above; direct AppKit termination, Dock/session and bypass/manager remain separate.

### Earlier G02/G06 running-hook host crash — November 7, 2026 (historical)

[Crash report](reports/G02_G06_RUNNING_PUSH_CHECK_CRASH_2026-11-07.md) and
[evidence](validation/g02-g06-running-push-check-crash-2026-11-07/README.md)
record actual SIGKILL to the exact host after independent running-phase/PID proof.
Original hook and child survived, then finished naturally; no external child signal.
Exact repositories, remotes, draft and journal/request counts were preserved with
one invocation and no replay/publication. Raw check stays running/unpublished;
audit projects interrupted, not passed, with fresh approval required.

First valid restart showed a blank window; second clean restart restored draft and
interrupted history. First-restart presentation was open at that checkpoint; the
startup readiness fix/retest above supersedes that finding. Other running-check entry points,
Dock/session and G06 bypass/manager criteria remain separate. No product changes;
artifact/source hashes verified, prior automated tests historical. Fixture cleaned
up; ordinary writes stay disabled. G02/G06 remain open.

### Earlier G02/G06 Quit during running hook — November 7, 2026 (historical)

[Quit acceptance report](reports/G02_G06_RUNNING_PUSH_CHECK_QUIT_2026-11-07.md) and
[evidence](validation/g02-g06-running-push-check-quit-2026-11-07/README.md) record
actual Cmd+Q from the review window after a real pre-push hook was independently
proven running. Host exited 0, hook and child stopped without an external kill,
and the check became cancelled/unpublished/not passed before natural hook finish.
Exact draft and repository/bare-remote snapshots were preserved. Restart acquired
the state lock, restored the draft and cancelled history, and caused no hook,
request or operation replay. Fixture server stopped and generated TLS key removed.

No product code changed. Reused artifact/source hashes match the prior full desktop
gate; this step executes native/independent acceptance checks rather than claiming
a fresh automated matrix. The preceding nine Core pre-push tests remain historical.
At that checkpoint the next shared case was native host crash, now recorded above,
with hook/child lifetime, interrupted-history and no-replay assertions. Main Close,
direct AppKit/Dock/session entry points during this exact operation are not inferred.
Ordinary writes remain disabled; no push or publication authority was added.

### Earlier G02/G06 native running-hook cancellation — November 7, 2026 (historical)

[Running pre-push-check report](reports/G02_G06_RUNNING_PUSH_CHECK_CANCEL_2026-11-07.md)
and [evidence](validation/g02-g06-running-push-check-2026-11-07/README.md) establish
actual native cancellation of a supported pre-push hook after independent proof
of running phase and hook/child start. Cancel review stopped both exact processes;
durable result was cancelled, unpublished, hook not passed. Native history and
restart retained no retry authority or replay. One explicit worktree trust, one
completed loopback observation, one hook invocation and one cancelled check; every
repository/remote byte/mode and request-log byte preserved after execution/restart.
No production code or write default changed and no push ran.

Nine focused Core push-check tests passed. Reused exact prior validated artifact/
source hashes; earlier full desktop gate retained, not relabeled as rerun. Fixture
apps/server stopped and generated private TLS key removed. Next shared case is
Quit during a proven running pre-push check, followed by native host-crash cases.
Native bypass/manager interaction remains G06; Dock/session restrictions remain G02.

### Earlier G02 unanswered-sheet termination — November 7, 2026 (historical)

[Active-sheet report](reports/G02_ACTIVE_SHEET_TERMINATION_2026-11-07.md) and
[evidence](validation/g02-active-sheet-2026-11-07/README.md) resolve the delivery
uncertainty in the prior checkpoint. A fixed feature-only request now audits actual
native dispatch, attached-sheet state and return. AppKit received two calls with
unanswered trust/commit confirmations attached and returned without consulting the
termination delegate. Both sheets remained responsive/unanswered; no implicit grant
or publication occurred. The run-loop starvation hypothesis was not confirmed:
observed dispatch used default mode. No production behavior changed.

After explicit native Cancel, the commit was durably cancelled before publication.
A fresh sheetless terminate request entered guarded shutdown and exited 0. Restart
restored the exact saved draft without replay, and all four repository byte/mode
snapshots matched baseline. Full desktop checks passed: 69 renderer, 13 default /
16 all-feature host tests and all static/Clippy/build checks; default feature/
artifact isolation and nonfixture refusal passed. Ordinary writes remain disabled.

This closes the selected active-sheet **safe deferral/decline/retry** evidence gap,
not immediate termination or automatic sheet cancellation. Remaining G02 steps:
actual Dock Quit (policy blocked), operator-controlled session logout/restart, and
running-execution interruption combinations shared with G06. No Dock/session or
unrun native phase is waived. A future immediate-exit requirement needs separate
supported design/acceptance.

### Earlier G02 guarded native termination — November 7, 2026 (historical)

[Fix/retest report](reports/G02_GUARDED_APPKIT_TERMINATION_2026-11-07.md) records
public applicationShouldTerminate interception into the existing save/cancel/Core
shutdown handshake, with original Tao delegate forwarding. Native missing-delivery
fallback now supports Keep app open and a successful direct termination retry;
restart restored the draft, representative forwarding probe passed and clean direct
termination exited 0. All repository bytes/modes remained identical, zero grants/
operations. Full desktop gate passed (69 renderer, 13 default / 16 all-feature host,
static/Clippy/debug/release checks). Ordinary writes remain disabled.

G02 remains open for Dock Quit (policy blocked), unanswered-sheet direct native
termination (request did not reach the decision callback), operator-controlled
session logout/restart and remaining active-execution combinations with G06.
Do not substitute own-app fixture requests for actual Dock/session evidence.
Completed checkpoint commits: `aad07d2` (G01/renderer recovery) and `904a8e5`
(direct termination diagnostic); the validated fix is recorded by the commit
containing this checkpoint.

### G02 direct AppKit termination diagnostic — November 7, 2026 (historical)

[Diagnostic report](reports/G02_APPKIT_TERMINATION_2026-11-07.md) records a corrected
native terminate request that exited while bypassing the guarded close handshake.
An armed missing-close-delivery fault was never consumed; no fallback appeared.
No repository mutation, grant or operation occurred. This is a failed graceful
termination criterion, not acceptance. A first test-control reentrancy hang was
identified separately by stack sample and corrected through run-loop scheduling.
Next: intercept the cancellable AppKit termination decision and route it into the
existing shutdown handshake, then natively retest. Dock remains policy blocked.

### Latest G02 actual renderer-loss checkpoint — November 7, 2026

[Renderer-loss report](reports/G02_ACTUAL_RENDERER_LOSS_2026-11-07.md) and
[evidence](validation/g02-renderer-loss-2026-11-07/README.md) record two actual
WebKit main-renderer terminations through a fixed native acceptance action targeting
only the PID reported by the owned fixture WKWebView. Host survived, Tauri's normal
termination handler reloaded main, and the existing reset path invalidated review
authority. Pending trust granted nothing; an issued commit confirmation was
cancelled before publication. The saved draft returned, a fresh OS dialog worked,
and normal Quit/restart preserved state without replay. All four repositories'
bytes/modes remained unchanged. No production lifecycle behavior changed.

Full desktop validation passed: 69 renderer tests, 13 default / 16 all-feature host
tests and all static/strict-Clippy/build checks. Default feature/artifact isolation
and nonfixture refusal passed. No full Core/signing/remote matrix rerun is claimed.
Dock access via Computer Use was denied by organization policy on `com.apple.dock`;
no alternate UI driver was used. Dock remains blocked, system/session termination
and further required phase-specific native cases remain open. Actual renderer loss
is accepted only for the observed auto-reload path; persistent restart failure and
unsaved keystroke durability are not established. Ordinary writes remain disabled.

### Earlier G01 closure / G02 recovery — November 7, 2026 (historical)

[Closure/recovery report](reports/G01_NATIVE_SUPPORT_G02_RECOVERY_2026-11-07.md),
[G01 evidence](validation/g01-native-support-2026-11-07/README.md) and
[G02 evidence](validation/g02-crash-boundaries-2026-11-07/README.md) close the
remaining G01 live-native cases with fixed, non-default test support. Actual
webviews rejected cross-caller IPC; wrong main page/origin navigations were denied.
Real main reload invalidated pending trust and an issued native commit confirmation,
with no unauthorized grant/publication and all repository bytes preserved.
No IPC permission, independent-approval rule or default write policy changed.

G02 additionally passed one suppressed close-delivery fallback with Keep app open,
persisted draft and successful Quit retry, and actual native process SIGKILL at
`publication_pending` / `reference_updated`. Restart did not replay either operation.
Separately approved recovery aborted the first without publication and reconciled
the already published second commit/index without creating a second commit.
A refused incompatible crash fixture is recorded separately; Core's guard was
not relaxed. Actual renderer-process loss and Dock/system termination remain open.

Final checks passed: 69 renderer tests, 13 default / 15 all-feature host tests,
24 Core application cases, the existing aggregate 13-checkpoint Core crash matrix,
all desktop static/build/strict-Clippy checks, and default feature/artifact isolation.
Native artifacts/source chronology and exclusions are explicit in the report.
G01 acceptance is bounded; M3 overall, G02–G11 and ordinary write enablement stay open.

### Earlier G01/G02 continuation — November 7, 2026 (historical)

[Latest report](reports/G01_G02_CONFIRMATION_LIFECYCLE_2026-11-07.md) and
[evidence](validation/g01-g02-2026-11-07/README.md) record three native lifecycle
findings and fixes: synchronous confirmation blocked Quit, closing an async sheet's
parent leaked its dialog lease, and replacement raced window-label teardown.
Confirmation now uses a responsive native sheet, cancels that sheet explicitly on
review teardown, and waits for Destroyed before revalidating/showing a replacement.
No renderer authority, IPC command or write-default change was added.

Actual native tests passed unusual filename/exact SSH-signed publication with a
direct pre-commit hook, outstanding-confirmation Quit, workspace change followed
by a fresh native dialog, main close with outstanding confirmation, and direct
replacement with a different review ID and fresh native decline. Independent
checks found exactly one fixture publication/hook run, five local operation records
(one complete, four cancelled), no tooling trust grant and preserved excluded state.
Full final desktop gate passed (69 renderer, 13 host per configuration and all
static/build checks); Core application suite passed 24 cases. Generated private
key removed and fixture processes exited. Ordinary writes remain disabled.

At this earlier checkpoint G01 remained open for live renderer reset/reload and
native caller/page adversarial cases; automated coverage is not substituted for those native cases. G02 remains
open for missing-renderer fallback, Dock/system termination and additional recovery
phases. Preserve earlier historical results and support exclusions; no full signing,
crash, platform or release gate is closed by this continuation.

### First G01 execution checkpoint — November 7, 2026

[Acceptance report](reports/G01_REVIEW_AUTHORITY_2026-11-07.md) and
[durable evidence](validation/g01-2026-11-07/README.md) record the first G01 pass.
No production code or write default changed; two Core integration tests and expanded
native origin assertions were added. The separate disposable native build published
one explicitly approved local commit with exact hostile-text message/staged blob
bytes and preserved excluded content. Decline, same-OID branch drift, replaced
review and workspace change produced no additional publication. The fixture exited.

This first checkpoint remains historical. The continuation above supersedes its
then-open unusual-filename, selected signer/hook and outstanding-confirmation
close/replacement observations. Mock-runtime/Core coverage is not relabeled as
native acceptance. All gate-level dispositions are in the current rows above.

### Supported scope and explicit deferrals

First release is macOS; Linux/Windows are deferred by D-040 and are not accepted
platforms. Controlled writes retain the M2 local APFS/files-ref/ordinary-index
limits. Advanced indexes, unsupported hook managers/reference-transaction Execute,
interactive/hardware-token signing, hostile same-user manipulation and OS/power-loss
durability are not implicitly supported by process-crash tests. Non-UTF-8 filesystem
acceptance remains unproven on the current host. See the M2 contract for exact rules.
SSH uses the documented selected Ed25519 agent identity/strict known-host policy;
HTTPS uses the documented scoped helper/TLS policy. Unsupported configurations must
fail closed. Pull, branch mutations, destructive Git operations, hunk staging,
individual hook toggles and arbitrary Actions remain deferred/outside v1.
AI, SemVer Actions and backups remain in the approved product scope, not deferred.

## Next work and exit conditions

1. **Acceptance closure:** G01 is accepted within its recorded envelope. Continue
   G02 with permitted Dock/system termination and remaining required phase-specific
   cases, then G03/G05/G06; attach
   criterion-level native evidence and classify facility blocks. Restore the full
   reproducible automated matrix with isolated GPG and permitted local transport
   fixtures. Do not reopen selected historical passes merely because an old intro
   still lists them as untested. Ordinary writes stay disabled until G04 closes.
2. **Next functional package — G07:** design and implement exact individual
   non-force push and uncertain-outcome recovery. Validate with disposable local
   remotes and loopback services, never a user repository or private server.
3. **G08:** build bulk push on verified individual outcomes and fresh child-success
   evidence. Exit only with partial-failure/retry and dependency acceptance.
4. **G09:** AI foundation, deterministic Actions, then assisted/bounded automatic
   commits, following the completion plan's separate authorization contracts.
5. **G10–G11 and remaining G03:** complete state portability, measured usability and
   exact macOS release acceptance. Strengthen backup primitives before later schema
   migrations; do not postpone data safety until packaging.

### G02/G06 queue audit after linked-worktree upstream acceptance

Recent guard/interruption cases are evidence of distinct barriers, not completion of
whole gates. Group completed variants; do not repeat them without a new failure,
source change or an explicitly uncovered requirement. Choose the next package using
its owning criterion and state what its completion closes before execution.

| Package | Current disposition | Concrete remaining work / handling |
|---|---|---|
| G02 shutdown/read interruption | Selected graceful Close, active scan Cancel/host loss, running-hook lifecycle and both missing-snapshot choices passed within reports | Do not add more equivalent scan/close variants merely to accumulate passes |
| G02 startup | Initial React commit ordering defect fixed; bounded native restarts passed; earlier intermittent blank retained | Capture diagnostic state on an actual recurrence before reload; bounded clean launches do not prove a universal cause or guarantee |
| G02 real OS entry points | Direct AppKit covered; actual Dock policy blocked; operator session termination distinct | Keep blocks explicit; no surrogate or destructive session termination without operator coordination |
| G06 upstream topology | Selected missing-only setup/recovery history and now native linked-worktree success passed | Native owned-config-lock recovery package now passed (Cancel, empty/ready/pre-publication abort, stale/foreign refusal and ordinary restart); broader named topology/failure/accessibility criteria remain open |
| G06 fetch/auth/format compatibility | Selected real HTTPS/SSH/auth chains have dated evidence; identified artifact initialization race already fixed | Full current-source five-configuration Core signing/transport matrix passed with restored isolated GPG. Native SHA-256 HTTPS observation/download now passed; native offline import with existing objects now passed; missing-object/upstream/check topology remains; do not reopen the already diagnosed artifact race without new counterevidence |
| G06 OS credentials / other transport exclusions | SecurityAgent choices and direct SCP facility limits retain prior blocks | Operator/facility-dependent work is not an unattended pass; unsupported helpers stay unsupported |
| Planner accessibility/cycles/scale | Broader native criteria remain open; overlap with G03/G08 | Track as their own criteria, not repeated observation-guard tests; publication remains G07/G08 |

The latest linked-worktree native chain closes one concrete topology criterion.
Configuration-trust drift and same-OID generation drift are now grouped guard evidence;
further URL/auth/tool variants are not the default next work unless tied to an explicit
remaining criterion. G02/G06 are not closed by this queue audit.

The user has authorized proceeding through acceptance steps and making a local
Git commit after each completed, validated step. This includes committing the
completed documentation/G01/G02 renderer-loss checkpoint. The queue does not
itself change write defaults or authorize live services, pushes, release publication
or deployment. Acceptance writes use only explicitly disposable fixtures.
Organization-policy blocks remain blocks, not acceptance waivers.

## Latest review validation — November 7, 2026

These results were obtained during the preceding read-only project review at
`db29ba6`, before this documentation reconciliation. They are a focused snapshot,
not the full five-configuration/signing/crash/release gate. No native GUI acceptance
was performed. Temporary raw logs are not durable release evidence.

| Check | Observed result |
|---|---|
| Renderer tests | 69 passed across 18 files |
| TypeScript, ESLint, generated contracts, frontend production build | Passed |
| Native host tests, default configuration | 13 passed |
| Core application integration | 21 passed |
| Core units / bulk / graph / inspection / reads / workspaces | 22 / 11 / 5 / 11 / 16 / 15 passed; bulk subprocess and non-UTF-8 filesystem cases explicitly ignored |
| Controlled-write integration | 24 passed, 1 failed: mandatory positive OpenPGP gate lacked `GITFLEET_TEST_GPG`; not a signing pass |
| Remote inspection | 10 passed |
| Remote observation | 52 passed, 29 failed, 2 ignored. HTTPS fixtures failed to bind loopback sockets (`Operation not permitted`); SSH fixtures timed out at startup. Transport acceptance remains unverified here |

Historical passing matrices below remain evidence for their recorded sources and
environments; this snapshot neither erases them nor attests the current release.
Documentation-only validation for this reconciliation is recorded in the final
change summary; application tests need rerunning only for subsequent code changes
or unresolved acceptance work.

## Historical checkpoint index — November 3, 2026

The following original tracker is retained for provenance. Its counts, "next"
labels and pending observations are dated; use the current tables above for work
selection. Later reports listed here can supersede earlier checkpoint limitations.


Updated November 3, 2026. Execution resumed under the user's fresh-chat authorization. Local checkpoint/milestone commits and reports are authorized; no push. Reasonable permission/facility blocks may be deferred; an unrun test is not a pass. Latest [native close/Quit report](reports/M3_NATIVE_CLOSE_QUIT_2026-11-02.md).

| Package | State | Evidence and next action |
|---|---|---|
| P0 baseline | Accepted | Commit `7130ced`; [report](reports/P0_BASELINE_2026-11-02.md); full baseline matrix passed |
| M3 desktop fixes | Implemented with native retests partial | [Progress report](reports/M3_DESKTOP_PROGRESS_2026-11-02.md); 17 renderer, 87 application-services and 127 all-feature passes; seven native tests per configuration |
| M3 native acceptance | Partial; execution resumed | Actual running cancellation, no replay, geometry, History/removal retests plus ordinary close and proven held-read close passed. Close during a running pre-publication hook and forced interruption/guarded abort now passed; broader review/interruption/accessibility gates remain open; graph-read Cancel fixed |
| M3 native lifecycle follow-up | Selected gates passed | [Report](reports/M3_NATIVE_LIFECYCLE_2026-11-02.md); exact PID termination, durable cancellation, native guarded abort, no replay and four-repository preservation; background polling queue finding subsequently fixed and retested |
| Mixed-DPI window restore | Fixed; automated and actual 1×/2× native retests passed | [Report](reports/M3_MIXED_DPI_WINDOW_RESTORE_2026-11-02.md); target-display conversion, shared minimum, repair/restart/no shrink and exact negative-coordinate Retina restore; 45 renderer/11 native host tests per configuration; physical disconnect/full accessibility remain open |
| Commit Command branding/About | Implemented; automated and native visual acceptance passed | [Report](reports/COMMIT_COMMAND_BRANDING_ABOUT_2026-11-03.md); supplied theme banners, pointer/click About, license and native menu, macOS icons and visible rename; 69 renderer/13 host per configuration. Internal identifier/storage preserved |
| M3 ordinary writes | Disabled | No change to default features; requires remaining acceptance and explicit default decision |
| M4 graph/activity/audit | Automated gates passed; selected native retests passed | [Original report](reports/M4_READ_ONLY_PROGRESS_2026-11-02.md); timeline/lanes/heatmap and basic graph/parent navigation now observed natively; foreground Cancel retest passed; remaining accessibility/scale gates open |
| M5A local bulk commit | Implemented; automated coverage and ordinary native form retests | [Original report](reports/M5A_LOCAL_BULK_PROGRESS_2026-11-02.md); new shared-form persistence fix passed automated/native navigation and restart retests; native two-repository publication/decline/restart now passed; broader dependency/failure/accessibility gates remain open |
| M5A native bulk follow-up | Selected native gates passed | [Report](reports/M5A_NATIVE_BULK_2026-11-02.md); exact staged/working blobs and messages, exclusions, unselected preservation, native Escape decline and restart/no replay; result refresh/empty retry gaps subsequently fixed and natively retested |
| M5B local remote inspection | Automated gates and ordinary native redaction/gate retest passed | [Report](reports/M5B_REMOTE_INSPECTION_2026-11-02.md); 113 application, 153 all-feature, 27 renderer and eight native host passes per configuration; no server contact |
| M5B authentication compatibility/artifact fix | Implemented and selected real acceptance passed; manual SecurityAgent interaction open | [Compatibility report](reports/M5B_AUTHENTICATION_COMPATIBILITY_2026-11-02.md), [artifact fix](reports/M5B_ARTIFACT_INITIALIZATION_FIX_2026-11-02.md), [setup guide](AUTHENTICATION.md). Actual osxkeychain/Gitea password + approved read-only PAT, native chain/Quit/ordinary restart passed; 56 renderer/13 host, Core 54/89/79/183/227. Native Deny/Allow policy blocked; lingering dialog dismissal unverified. Ordinary writes disabled |
| M5B authentication profiles/settings | Implemented; selected actual native acceptance passed; artifact finding resolved in later checkpoint | [Report](reports/M5B_AUTHENTICATION_PROFILES_2026-11-02.md); separately reviewed nonsecret HTTPS/SSH profiles, explicit destination-bound per-remote assignments, persistence and tool/revision revalidation. Native decline/save/assign/authenticated read/unassign/ordinary restart passed without auth startup flags. Desktop 54 renderer/13 host per configuration; real OS helper/Gitea now passed in later checkpoint; direct SecurityAgent interaction open. Final Core 54/89/79/179/223 passed; initial all-feature artifact-guard failure retained; now diagnosed/fixed in later checkpoint |
| M5B HTTPS credential helper | Automated matrix and selected actual native gates passed | [Report](reports/M5B_HTTPS_CREDENTIAL_HELPER_2026-11-02.md); host-selected scoped get-only helper/account, verified TLS/Basic, no secret IPC/journal, no store/erase forwarding, inherited process-group cleanup. Core 54/89/79/173/217; desktop 51 renderer/13 host per configuration. Native decline/rejection/authenticated observation/download/offline import and held-lookup Cmd+Q passed. OS secure-store/settings compatibility remains open; ordinary writes disabled |
| M5B reviewed branch observation | Automated and selected actual native HTTPS gates passed | [Report](reports/M5B_REMOTE_OBSERVATION_2026-11-02.md); exact credential-free network read through separate native review and isolated transport; local/loopback TLS tests pass; actual native HTTPS chain now passed in [native fetch report](reports/M5B_NATIVE_FETCH_ACCEPTANCE_2026-11-02.md) |
| Workspace remote history | Implemented; automated and native presentation passed | [Report](reports/M4_WORKSPACE_REMOTE_HISTORY_2026-11-02.md); existing scoped history endpoint, shared records, pagination/refresh/stale-response handling; 44 renderer tests and full desktop gate; synthetic native history only, no network execution claim |
| Unified operation audit | Implemented; automated matrix and selected native filters passed | [Report](reports/M4_UNIFIED_OPERATION_AUDIT_2026-11-02.md); one typed read-only chronology of local/batch/remote records; SQL filters before pagination; 45 renderer tests, full Core matrix and desktop gate; retention/later producers remain open |
| M5B reviewed fetch download | Implemented; automated real transport and native presentation/default gate passed | [Report](reports/M5B_FETCH_CAPTURE_2026-11-02.md); exact observed branch to isolated verified bundle; 16 transport cases, full Core matrix, 46 renderer/11 native host cases per configuration; separate import checkpoint implemented; actual native HTTPS download/import chain passed in [native report](reports/M5B_NATIVE_FETCH_ACCEPTANCE_2026-11-02.md) |
| M5B reviewed fetch import | Automated and selected real native gates passed | [Report](reports/M5B_FETCH_IMPORT_2026-11-02.md); separate exact-bundle review, bounded offline object import, direct expected-old tracking-ref transaction, durable journal/recovery and no replay. Full Core matrix 54/89/78/146/188 passed; desktop 47 renderer/11 native host tests per configuration and complete gate passed. [Native report](reports/M5B_NATIVE_FETCH_ACCEPTANCE_2026-11-02.md): actual HTTPS observation/download/import, decline, prepared Cancel/Cmd+Q, receipt-owned lock recovery, ordinary default gating and restart/no replay passed; ordinary writes remain disabled |
| M5C reviewed pre-push checks | Implemented; automated and selected native check acceptance passed; publication open | [Report](reports/M5C_REVIEWED_PUSH_CHECKS_2026-11-03.md); one-use independent hook approval, exact argv/stdin/fingerprint, already-reachable child gate, cancelled/interrupted unpublished history and restart/no replay. No push executor or publication authority; ordinary writes disabled |
| M5C native planner acceptance | Selected actual native gates passed | [Report](reports/M5C_PLANNER_NATIVE_ACCEPTANCE_2026-11-03.md); fresh TLS three-level conditional order, unselected-child blocking, selection/observation invalidation, Cmd+Q and acceptance/ordinary restart/no replay; exact worktree/remote/record/request preservation. No source patch, push or default-write change |
| M5C multi-level dependency planning | Implemented; automated and selected native acceptance passed | [Report](reports/M5C_DEPENDENCY_PLANNING_2026-11-03.md); explicit selected graph, conditional child-first order, actual closed cycle witness and blocked unavailable unselected children. No auto-selection, partial order, push authority or new network operation; ordinary writes disabled |
| M5C relative/SCP child acceptance | Implemented; selected actual native/SSH acceptance passed | [Report](reports/M5C_RELATIVE_CHILD_ACCEPTANCE_2026-11-02.md); explicit-parent relative resolution, exact SCP syntax, Git oracle and redaction regression. Native verified-TLS relative child evidence/clearing/ordinary restart/no replay and real agent SSH evidence passed. Direct SCP port-22 network acceptance OS-blocked; supported-subset limitations retained. No push/default writes |
| M5C historical mapping/child evidence | Implemented; validation checkpoint | [Report](reports/M5C_CHILD_MAPPING_EVIDENCE_2026-11-02.md); committed mapping blobs preserve URL changes; explicit child/approved observation selection and bounded local ancestry evidence with destination/context/freshness checks. No new network action, inferred relative URL, push authority or push. Native form/authenticated chain acceptance remains open |
| M5C outgoing-history child requirements | Implemented; local verification checkpoint | [Report](reports/M5C_HISTORY_DEPENDENCIES_2026-11-02.md); unique byte-path/child-OID requirements from all outgoing commit trees, including removed/replaced and merged-side links, independently of displayed commit cap. Bounded 256 distinct trees/1,024 requirements; no remote reachability proof, destination authority or push. Native presentation not run |
| M5C source child requirements | Implemented; automated verification checkpoint | [Report](reports/M5C_SOURCE_DEPENDENCIES_2026-11-02.md); exact source-tree gitlink paths/OIDs, SHA-1/SHA-256 and byte-safe rendering, bounded refusal. No child checkout/network access, remote proof, history-wide dependency completeness or push authority. Native presentation not run |
| M5C local push preflight | Full automated matrix and ordinary native checks passed | [Report](reports/M5C_LOCAL_PUSH_PREVIEW_2026-11-02.md); current local branch vs explicit past observation, exact counts/outgoing details, divergence/refusal checks, no network or authority. Actual native raw-parent oracle and full repository/artifact/journal preservation passed |
| M5B guarded upstream setup | Automated matrix and selected actual native gates passed | [Report](reports/M5B_GUARDED_UPSTREAM_2026-11-02.md); independent missing-only config review, exact byte/metadata preservation, durable phases and fresh recovery/no replay. Core 54/89/79/156/200; desktop 50 renderer/11 native host per configuration. Native decline/publication/default restart and actual post-publication journal-failure recovery passed; broader compatibility/accessibility remains open |
| M5B reviewed SSH-agent reads | Implemented narrow policy; selected actual native gates passed; artifact finding subsequently fixed | [Report](reports/M5B_SSH_AGENT_TRANSPORT_2026-11-02.md); host-selected Ed25519 agent identity/strict known hosts, isolated snapshots, no credential fallback. Core 54/89/79/163/207 and desktop 50 renderer/12 native host cases per configuration passed; real native chain, held-read Quit/no replay and final hardened authentication passed. Prior artifact refusal now captured/diagnosed/fixed; retain original evidence |
| M5B/C fetch/push workflows | Next | Continue operator-assisted SecurityAgent interaction, explicitly supported compatibility and broader upstream acceptance; artifact initialization refusal now fixed; retain native accessibility/topology/recovery gaps. General authentication workflows, individual/bulk push and dependency proof remain unfinished |
| Previous native follow-up | Validated checkpoint | [Report](reports/M5_NATIVE_FOLLOWUP_2026-11-02.md); 18 application integration tests per configuration, 29 renderer tests, eight native host tests per configuration and native preservation checks pass |
| Foreground cancellation | Validated, automated and native | [Report](reports/M3_FOREGROUND_CANCEL_2026-11-02.md); 30 renderer tests, eight native host tests per configuration; actual graph cancellation/process termination/reload and preservation passed |
| Background polling | Validated, automated and native | [Report](reports/M3_POLLING_2026-11-02.md); 31 renderer/eight native host tests per configuration; serial refresh/no queueing during native write, native Cancel and preservation passed |
| Native completion/results | Validated, automated and native | [Report](reports/M3_NATIVE_RESULTS_2026-11-02.md); 33 renderer/eight native host tests per configuration; fast native batch automatically refreshes to Published, empty retry disabled; ordinary read-only restart preserved |
| Selected accessibility | Fixes validated; full acceptance open | [Report](reports/M3_ACCESSIBILITY_2026-11-02.md); 37 renderer/eight native host tests per configuration, measured CSS contrast, keyboard/AX and Light rendering; interactive native Light/Dark switching and ordinary Dark restart now accepted in [theme report](reports/M3_NATIVE_THEME_2026-11-02.md); VoiceOver/broader cases open |
| Review lifetime/context | Selected native gates passed; presentation timer validated | [Report](reports/M3_REVIEW_LIFETIME_2026-11-02.md); actual late native approval and removed-workspace review rejected; zero trust/write records and preserved repositories; full current Core matrix plus 40 renderer/eight native host tests per configuration |
| Obsolete review window | Fixed and natively retested | [Report](reports/M3_STALE_REVIEW_WINDOW_2026-11-02.md); epoch-changing workspace removal closes review immediately; no grants/writes or repository mutation; 40 renderer/eight native host tests per configuration |
| Final close draft snapshot | Selected native gates passed; Quit defect fixed | [Restart handoff](reports/M3_CLOSE_DRAFT_HANDOFF_2026-11-02.md); atomic paired presentation save, one-use close token and main-only listener/ack, safe unsaved choice and Quit/close coordination; 19 Core integration cases per relevant configuration, 42 renderer/10 native host cases per configuration; actual immediate close/Quit, save failure/retry, held read and pre-publication write passed; [native report](reports/M3_NATIVE_CLOSE_QUIT_2026-11-02.md); broader lifecycle/accessibility gates remain open |
| M6 through M8 | Planned | Follow dependency and acceptance packages in completion plan |

Historical logs remain immutable. Current [requirement crosswalk](REQUIREMENT_EXECUTION_CROSSWALK.md), [completion plan](PROJECT_COMPLETION_PLAN.md), reports and per-run validation directories track progress. Do not infer a full milestone or product completion from an implementation checkpoint.
