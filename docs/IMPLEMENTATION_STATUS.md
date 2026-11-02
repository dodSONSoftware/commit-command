# GitFleet implementation status

Current execution follow-up (November 2, 2026): **stopped at the user's request to
restart Codex**. The [close-draft checkpoint and resume handoff](reports/M3_CLOSE_DRAFT_HANDOFF_2026-11-02.md)
records atomic presentation saving and native close/Quit coordination. Automated
gates passed: 19 Core application integration cases under each relevant feature
configuration, 42 renderer tests and 10 native host tests per configuration plus
static/build gates. Actual native acceptance of the new handshake is NOT RUN.
Resume only when requested; do not mark M3 complete or enable ordinary writes.

Earlier completion execution follow-up (same date): the user authorized the full
[completion plan](PROJECT_COMPLETION_PLAN.md), local package/milestone commits and
reports, with no push. See [execution status](EXECUTION_STATUS.md) for current gates.
P0 is committed; M3 fixes and M4 graph/activity/audit have automated evidence and
partial native evidence. M5A local commit batches have passed automated gates; see the
[M5A report](reports/M5A_LOCAL_BULK_PROGRESS_2026-11-02.md). Remaining
native checks are incomplete; ordinary desktop writes remain
disabled. M5B remote inspection and separately reviewed credential-free branch observation
are implemented with automated evidence. Fetch, authenticated transport, push and
M6–M8 remain unfinished.

## Historical M3 checkpoint

The remainder is the earlier dated checkpoint, retained for evidence provenance.
Its authorization limits, test counts and then-open observations describe that
checkpoint; current authorization and subsequent results are above and in the
linked execution tracker/reports.

Updated: 2026-11-02. **M3 implementation delivered; native acceptance is partial,
so M3 is not complete.** The user authorized M3 and its proposed decisions/gates.
Earlier organization policy denied Computer Use access to `dev.gitfleet.desktop`; the
restriction was not bypassed. Ordinary desktop write exposure remains disabled
in Rust and UI. Stop at M3; M4 and later are not authorized. No commit, push,
publication, deployment or Git write against an existing user repository occurred.

## Delivered

- The existing desktop structure now contains a Tauri 2 adapter and strict
  React/TypeScript shell, workspace management, Table/Tree Overview, explicit
  selection, repository tabs, Changes/History base, per-worktree composer drafts,
  themes/panel state, minimal operation results and recovery notices.
- Core remains one presentation-independent Rust crate. A separate required native
  adapter crate depends on it by path. No Tauri/React dependency entered Core.
  Default Core builds remain read-only for managed Git repositories.
- Non-default `application-services` provides one application/storage/M2 session
  owner, scoped versioned requests/responses, generated TypeScript contracts,
  effective trust versus hook-policy eligibility, per-operation cancellation,
  stored navigation, guarded reviews and active-workspace monitoring.
- Separate minimal authoritative review bundle, actual-caller/page/origin checks,
  explicit custom-command ACLs, bounded jobs/payloads and Rust-only confirmation
  capabilities. Only an independent native-dialog result may reach M2 approval.
- Actual generated Tauri ACLs are tested with its non-UI mock runtime; renderer
  tests cover state continuity, text-only content and a 100-repository inventory.
- Schema-2 backup before schema-3 navigation storage/index migration; invalid/newer
  state rejected. Workspace scope/removal guards protect interrupted operations.
- Notification hints, debounce/burst/overflow handling, periodic discovery/status
  fallback, automatic create/move/remove observations, inactive-workspace isolation
  and shutdown/reset policy. Trusted Rust timing options support deterministic
  tests; the renderer cannot override them.
- An explicit lock-owner guard fixes a reproduced descriptor-lifetime race without
  weakening existing workspace/M2 contention tests or adding lock retries.

The [M3 contracts](architecture/M3_DESKTOP_AND_IPC.md),
[acceptance ledger](ACCEPTANCE_MATRIX.md#12-m3-acceptance-gates--implementation-with-native-gates-open)
and [desktop README/manual steps](../app/desktop/README.md) define behavior and scope.
Default desktop builds select `custom-protocol`; the separate `desktop-writes`
feature is non-default and remains an acceptance-only opt-in. No renderer input
can enable it. Read-only recovery inspection is available; execution stays gated.

The [November 2 native handoff](M3_NATIVE_ACCEPTANCE_HANDOFF_2026-11-02.md)
records later user-observed results supported by terminal output and screenshots;
the documenting agent did not independently reproduce them. The
[native checklist](M3_NATIVE_ACCEPTANCE_PLAN.md#results-checklist) now distinguishes
those observations from failed, unsupported and untested criteria. Desktop write
defaults are unchanged. Historical November 1 evidence is preserved verbatim.

## Latest native observations and open work

User-observed read-only snapshots, discovery, picker cancellation, tab/draft restart,
workspace monitoring/isolation, trust and revocation, exact index-only publication,
stage/unstage, stale content/branch rejection, hook rejection/bypass/post-failure,
and isolated SSH/OpenPGP signing and missing-key failures are recorded in the handoff.
Unsupported reference-transaction Execute failed closed; execution remains unsupported.
These observations have no exact test-time source/binary attestation.

Actual running-operation cancellation was **not tested**. The slow-hook operation
ended blocked `HookRejected`; neither Cancel nor timeout was demonstrated. Guarded
recovery/no replay, close-during-read/write, review close, forced interruption,
remaining hostile-content/context/expiration cases, VoiceOver and measured contrast
remain open. Current process/UI state is unknown.

Removing the active saved definition produced stale workspace UI. The current
renderer patch preserves the previous context until `load()` can clear the removed
workspace, tabs, inventory, trust and operation view. The History patch clears shared
patch/preview/commit details before loading each view or comparison, retaining the
existing late-response guards. Two component regressions cover workspace removal,
subsequent workspace drafts, view switching and a late Changes response. Native
retesting of both fixes is pending; other reported UI defects remain open.

Later evidence is reported at `/private/tmp/gitfleet-m3-evidence-niCdnj`; it must be
preserved durably without keys or sensitive app state before further acceptance.
Existence was checked during the repository scan, but evidence integrity and binary
correspondence were not established. The historical 88-file source manifest matched
before these fixes; its hashes are not an attestation of the updated renderer.

## Follow-up validation — 2026-11-02

For the two renderer fixes and documentation reconciliation:

| Command/check | Result |
|---|---|
| `npm --prefix app/desktop test` | 16 passed across 4 files, 0 failed; includes 2 new regressions |
| `npm --prefix app/desktop run lint` | Passed with zero warnings |
| `npm --prefix app/desktop run build` | TypeScript check and Vite production build passed |
| `npm --prefix app/desktop run format:check` | Passed |
| `CARGO_NET_OFFLINE=true npm --prefix app/desktop run contracts -- --check` | Passed; generated contracts unchanged |
| Local Markdown target check | 89 links checked; no missing file targets (anchors not validated) |
| `git diff --check` | Passed |

No Core/native Rust implementation changed. The full Rust feature matrix and native
host/debug/release builds were not rerun for this renderer-only patch. No new native
GUI acceptance was performed. The two native defect retests and all remaining gates
stay open. Historical test logs, hashes and summary JSON below remain unchanged.

## Historical automated results — 2026-11-01

Executed on macOS 27.2 arm64/APFS, Git 2.56.0, rustc/Cargo 1.99.1, Node 22.22.3 and
npm 10.9.8. Command-line native tools were sufficient for builds; full Xcode was
not installed. M2's isolated temporary GPG executable was reused with new disposable
keyrings. No personal signing keyring, global Git setting or global npm install
was changed. All integration Git writes used new disposable fixtures.

| Configuration/check | Result |
|---|---|
| Core default all-target tests | **48 passed, 0 failed, 1 ignored** |
| Core experimental-only tests | **83 passed, 0 failed, 2 ignored** |
| Core controlled-only tests | **71 passed, 0 failed, 1 ignored** |
| Core application-services tests | **85 passed, 0 failed, 1 ignored** |
| Core all-feature tests | **125 passed, 0 failed, 3 ignored** |
| Core build and strict Clippy, all five configurations | Passed |
| Core formatting/layout checks | Passed |
| Historical disposable POC lab | **23 passed, 1 skipped** |
| Frontend renderer/component tests | **14 passed, 0 failed** |
| Tauri handler/security mock-runtime tests, default | **6 passed, 0 failed** |
| Same native tests with all features | **6 passed, 0 failed** |
| TypeScript, ESLint with zero warnings, contract drift, Prettier | Passed |
| Frontend production build | Passed |
| Native default debug and optimized release builds | Passed |
| Native strict Clippy, default/all features; Rust formatting | Passed |
| Production npm advisory query | **0 known production advisories** at validation time |
| Live native visual/accessibility/approval/lifecycle tests | Historical November 1 policy block; later partial user observations are recorded above. |

Configurations overlap; totals must not be summed. The all-feature Core harness
contains 128 entries: 125 passes and three explicit ignores. M3 adds one lock unit,
two monitor units and 12 application integration tests to the prior M2 baseline.
The six native mock tests run twice with different feature selections and are not
12 unique tests. The 100-entry renderer test is a component observation, not native
latency/performance acceptance. No remote CI run is claimed.

The three Core ignores retain their meaning: optional original POC GPG Cargo case;
known macOS non-UTF-8 filesystem fixture limitation; and the crash child driver
explicitly invoked by passing real SIGKILL tests. The lab skips optional GPG because
it is absent from default PATH. **Mandatory M2 GPG/SSH positives execute and pass**
through the explicit isolated signer path. No skip is counted as a pass.

[Raw evidence and commands](validation/m3/README.md) preserve exact results,
source/dependency information, development findings and the native policy block.
M0/M1/M2 evidence and pre-M3 status/handoff snapshots remain historical and intact.

## Files changed for M3

Added Core application modules: `src/application/{mod,monitor,review,state}.rs`,
`src/locking.rs`, `src/writes/application.rs`, `src/bin/gitfleet_contracts.rs`, and
`tests/m3_application.rs`. Existing domain/read/write DTOs gained optional generic
schema derives; workspace/read APIs gained compatible cancellation-aware wrappers.
Workspace supports known schema 3 and bounded config saves. M2 execution/recovery
use explicit lock ownership and shared hook-policy validation. Cargo adds optional
notify/schemars application dependencies and pins SQLite to the existing 0.41.0.

Added desktop manifests/lockfiles, native host/security/permissions/capabilities,
React shell/store/bridge, generated contracts, isolated review, styles, tests,
development icon and scripts/README within the pre-existing directories. Core and
desktop validation, local CI, root/core/docs READMEs, specification, decisions,
acceptance, architecture and current status/handoff were reconciled.

The user's pre-existing `.gitignore` edit and all earlier uncommitted M0/M1/M2 work
were preserved. Git status includes historical changes, not only M3. Original POC
source listings, historical results, mockups and licensing remain untouched.

## Open acceptance and safety boundary

M3-A01/02/03/05/08/09/10/11/18 retain the bounded automated evidence in the ledger.
A04/06/07/12–17/20 have partial native evidence or unresolved criteria; A19 is partial
with keyboard/theme observations but untested VoiceOver/contrast and open visual
issues. The workspace-removal and History fixes have renderer coverage only until
native retest. No remaining criterion is waived by the later handoff.

The ordinary binary must stay read-only until the documented native steps are
completed with permitted access or a human operator. Mock runtime, source review,
renderer tests and release compilation do not replace those gates. Do not promote
`desktop-writes` into default features or mark M3 complete based on this report.

M2's local APFS/files-ref/ordinary-index support envelope, unsupported filters,
advanced indexes, hook managers, signing restrictions, native-lock/reflog recovery
and unsandboxed trusted-code caveats remain. No OS/power-loss durability, other
platform, network filesystem, hardware-token or generic hook-manager claim was
added. Private messages/patches/drafts/paths remain unencrypted local app data;
automatic retention cleanup and release packaging are not implemented.

No M4 graph/activity/full audit, M5 bulk Git, remote operations, AI, Actions, hunk
staging, discard/reset/amend or distribution work was performed. Resume only the
remaining M3 acceptance work when authorized/permitted; M4 is not ready for
implementation on the basis of a completed M3 gate.
