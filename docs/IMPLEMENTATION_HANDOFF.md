# GitFleet implementation handoff

Updated: 2026-11-02. **M3 implementation exists, but M3 is not complete: native
acceptance is partial and lifecycle/accessibility gates remain open.** The user authorized M3,
not M4 or later, commits, pushes or release publication.

## Read first

1. [November 2 native checkpoint](M3_NATIVE_ACCEPTANCE_HANDOFF_2026-11-02.md) and
   [current status](IMPLEMENTATION_STATUS.md): user-observed results, inventory and open gates.
2. [M3 desktop/IPC contracts](architecture/M3_DESKTOP_AND_IPC.md),
   [M3 ledger](ACCEPTANCE_MATRIX.md#12-m3-acceptance-gates--implementation-with-native-gates-open).
3. [Native acceptance plan and results checklist](M3_NATIVE_ACCEPTANCE_PLAN.md),
   [desktop README](../app/desktop/README.md),
   [evidence](validation/m3/README.md), actual Core `application` and native host.
4. [M2 contracts](architecture/M2_CONTROLLED_WRITES.md), M1/M0 contracts and historical
   evidence still govern Git behavior. Pre-M3 status/handoff are preserved verbatim
   under `validation/m2/` as completion snapshots.

## Native blocker — do not bypass

Computer Use returned: “Computer Use is blocked from using the app
'dev.gitfleet.desktop' by your organization's policy.” The agent did not change
identity, use alternate OS automation or substitute browser mocks for native tests.
That is the historical November 1 block. The November 2 checkpoint records later
user-operated native screenshots, approvals, writes, signing and restart observations;
those were not independently reproduced by the documenting agent. VoiceOver,
contrast, guarded recovery and close/cancel/forced-interruption gates remain open.
Current process/UI state and test-time binary correspondence are unknown. Policy
restrictions still apply to any agent-operated native testing.

**Ordinary desktop writes are disabled in Rust and UI.** Native Cargo defaults are
`custom-protocol` only. Non-default `desktop-writes` is an acceptance-build opt-in;
it cannot be enabled by IPC and is not authorization to target user repositories.
Do not promote it to defaults or mark M3 complete until native gates pass. Read-only
recovery inspection remains usable, with execution confirmation disabled.

A permitted operator can follow the [native acceptance plan and results checklist](M3_NATIVE_ACCEPTANCE_PLAN.md).
The checklist is reconciled with the November 2 user observations; incomplete and
untested gates remain explicit. It does not enable writes or override access policy.
The later checkpoint records temporary fixtures/evidence separately from historical
`validation/m3/native-fixture.json`; validate availability before use. Prefer creating a new fixture for a fresh run.
Do not modify real repositories or global Git settings to force acceptance.

## Preserve architecture and user work

- Existing docs/app layout; one Core crate, no Tauri/React dependency. The separate
  native adapter crate is justified by its UI/build dependencies, not a Core split.
- Core default is read-only managed Git plus own state. `application-services`
  selects transport-neutral orchestration/monitoring/schema and M2 controlled writes.
  Original POC and fault injection remain separate non-default features and are
  excluded from the normal desktop dependency graph.
- No project Git staging/commit/push occurred. All validation writes used newly
  created disposable Git repositories. Preserve the user's `.gitignore` edit and
  earlier uncommitted M0/M1/M2 work; Git status is not solely an M3 diff.
- No M4 graph/activity/full audit, M5 bulk Git, remote, AI, Actions, hunk staging,
  destructive Git operation, installer/notarization/signing/release work.

## Owning services and authorization

`application::Application` owns one `ControlledWrites`, retaining its workspace/
app-data lock and M2 session. A single bounded native worker serializes Core jobs;
status/cancellation handlers run independently of that worker. Do not reopen the
workspace/M2 service per IPC call or expose mutable DB handles to Tauri.

Core's version-1 typed request/response schemas generate `*.generated.ts`. Incoming
IPC is size/version/unknown-field checked, scoped by actual workspace/worktree
identity and active epoch. Paths are native data; display strings never become
execution paths. Normal custom-command ACLs, actual webview label/page/origin and
platform-specific bundled-origin checks all apply. No generic shell/Git/file/SQL/
network/journal/approval-token endpoint exists.

The main React page may request a Core plan. Rust opens the separate minimal review
page, which obtains stored Core content and renders text. Only that view can ask
for the independent OS-native confirmation. A Rust-only nonce capability carries
the real callback to Core, which obtains the M2 approval immediately before use.
Neither a proposal ID, JSON boolean, event, checkbox nor digest is authority.

Main-page reload invalidates native presentation generation, cancels old jobs and
active work, closes review context and requests Core epoch reset. Callbacks recheck
generation, epoch, nonce, expiry, scope and M2 state. Review cancellation/replacement
records an unexecuted proposal as cancelled. No commit/hook is automatically replayed.
Cancellation/revocation cannot promise rollback after publication; report actual
M2 outcomes and recovery state.

M1's old inventory `untrusted` field is not effective M2 trust. Current grant status
and supported remembered hook-policy eligibility are separate. Hook validation is
shared with M2, not reconstructed in React. New/moved/replaced/linked repositories
never inherit a grant. Trusted hooks/signers remain unsandboxed user code.

## Monitoring, persistence and lock finding

Notify hints are filtered/coalesced into read-only Core reconciliation. Production
defaults: 250 ms debounce, two-second coalescing bound, ten-second status fallback,
sixty-second discovery fallback. Only the active workspace and its necessary
metadata are observed. Overflow requests discovery; old watches drop on switch;
errors remain visible; watcher paths never authorize a mutation. Trusted Rust-only
`MonitoringOptions` support shorter deterministic test schedules, not IPC overrides.
Native low-latency/performance acceptance remains unclaimed.

TOML stays version 1. M2's schema-2 migration/backup precedes schema-3 navigation
storage and operation-workspace index. Default workspace services can reopen known
schema 3. Invalid/newer state is not overwritten. Pending recovery blocks removal
or changes to workspace scope. UI drafts/paths/messages are sensitive local data;
there is no automatic pruning or encryption implementation.

Concurrent regressions exposed a prior lock lifetime bug: closing an owner's file
can leave its advisory lock held through a duplicate inherited during spawning.
Private `locking::ExclusiveLock` explicitly unlocks before close, including failed
initialization, for workspace and M2 common-directory ownership. Its unit test
proves the old close-only behavior and correct release. Existing Busy assertions
were preserved; three repeated workspace runs and the full matrix passed after
correction. Do not replace this with sleeps, broad retries or weaker assertions.

## Reproduce automated checks

```sh
GITFLEET_TEST_GPG=/absolute/path/to/gpg CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=4 sh app/core/scripts/validate.sh
npm --prefix app/desktop ci
CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=4 sh app/desktop/scripts/validate.sh
```

Requires Git 2.56+, Rust 1.99+, Node 22.12+/npm 10+ and native command-line tools.
Omit offline mode if dependencies are uncached. Local npm configuration disables
install scripts and preserves the lockfile's peer resolver. GPG-agent needs a local
socket; approved final Core validation used sandbox escalation only for disposable
fixtures/keyrings. The temporary signer is currently
`/private/tmp/gitfleet-m2-tools/install/bin/gpg`; it is not a global install or a
permanent dependency. M2 provenance/checksums remain recorded separately.

Recorded final Core results: **48/1 ignored** default, **83/2** experimental,
**71/1** controlled, **85/1** application, **125/3** all features, all with zero
failures. Historical POC lab: **23 passed/1 skipped**. Five builds/strict Clippy
configurations and formatting/layout pass. Mandatory M2 GPG/SSH positives pass.
The original optional POC GPG and non-UTF-8 filesystem exclusions remain; the
harness-ignored crash worker is invoked by passing real SIGKILL tests.

Historical November 1 desktop: **14 renderer passes**, **6 native mock passes per feature configuration**,
TypeScript/zero-warning ESLint/contracts/Prettier, debug/release builds and strict
Clippy pass. Native mock tests are not OS UI tests. Source hashes and raw results
are under M3 evidence; counts overlap. The local CI definition was updated but
never pushed or run remotely. Production npm audit found zero known advisories
at validation time; no broader supply-chain/platform assurance is implied.

## Next step

Preserve the later non-secret evidence and verify the selected binary/fixture and
owned process state. Inspect guarded recovery for the blocked HookRejected operation
without replay, then finish actual cancellation, review/main-window lifecycle,
forced-interruption and remaining security/accessibility gates using permitted access.

The workspace-removal and retained History patch defects have now been patched with
component regressions; native retest remains pending. Other checkpoint UI issues are
unchanged. Current validation is recorded in implementation status; historical counts
above remain November 1 results. No native acceptance ran during these fixes.

Keep ordinary desktop writes disabled. Do not start M4, commit, push or release
without explicit instructions.
