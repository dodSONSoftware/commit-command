# M3 native acceptance plan and results checklist

Current gate disposition and next work are maintained in
[EXECUTION_STATUS.md](EXECUTION_STATUS.md), gates G01–G04. This document supplies
test procedures and a historical checklist. Selected later Cancel/recovery,
close/Quit, theme and geometry passes are linked from the current record; older
untested labels below do not reopen those cases. Remaining criteria still require
evidence, and ordinary writes remain disabled.


Prepared: 2026-11-01. **Original procedure and historical checklist.** No native
tests were run when first saving it. Current execution/results supersede only the
explicitly retested cases; see [execution status](EXECUTION_STATUS.md) and the
[current acceptance ledger](ACCEPTANCE_MATRIX.md#12-m3-acceptance-gates--implementation-with-native-gates-open).
M3 remains incomplete. Organization policy previously blocked Computer Use access
to `dev.gitfleet.desktop`; this plan does not override that restriction.
Ordinary desktop writes must remain disabled in Rust and the UI. The native Cargo
feature default remains `custom-protocol` only; `desktop-writes` is a non-default
acceptance opt-in. This document grants no Git authority; the user subsequently
authorized local milestone commits in chat. Pushing remains prohibited.

Use two separate runs: first the ordinary read-only desktop, then an explicitly
feature-enabled acceptance build targeting fresh disposable repositories. Keep
ordinary feature defaults unchanged throughout.

Latest November 7: [G01 closure / G02 recovery](reports/G01_NATIVE_SUPPORT_G02_RECOVERY_2026-11-07.md)
closes G01 within its documented envelope using fixed non-default native test
support. Subsequent [actual renderer-loss acceptance](reports/G02_ACTUAL_RENDERER_LOSS_2026-11-07.md)
passed the auto-reload path with pending/issued confirmation and saved draft.
G02 still requires permitted Dock/system termination and remaining phase coverage;
Dock access is organization-policy blocked. Injected close-delivery fallback and
selected native host crash/recovery remain separate passes.
The subsequent [native termination fix](reports/G02_GUARDED_APPKIT_TERMINATION_2026-11-07.md)
accepts selected direct AppKit fallback/retry/restart cases; unanswered-sheet direct
termination, actual Dock and session termination remain distinct open cases.
[Active-sheet follow-up](reports/G02_ACTIVE_SHEET_TERMINATION_2026-11-07.md)
now verifies actual native delivery and safe deferral with trust/commit sheets,
explicit Cancel, fresh termination and restart; no immediate sheet cancellation or
Dock/session pass is inferred. Earlier open-sheet statements are historical.
The [shared G02/G06 running-hook case](reports/G02_G06_RUNNING_PUSH_CHECK_CANCEL_2026-11-07.md)
has actual native Cancel review/process cleanup/no-replay evidence. The subsequent
Quit case is recorded below; subsequent host-crash findings follow.
[Cmd+Q during a running pre-push check](reports/G02_G06_RUNNING_PUSH_CHECK_QUIT_2026-11-07.md)
now passed with process cleanup, draft/lock preservation and no replay. Older open
Quit labels above are historical. [Actual running-check host crash](reports/G02_G06_RUNNING_PUSH_CHECK_CRASH_2026-11-07.md)
now has preservation/no-replay evidence and interrupted history/draft on second
restart. Original hook survived and finished naturally; no crash cleanup guarantee.
First valid restart was blank at that checkpoint. The subsequent
[startup readiness fix/retest](reports/G02_STARTUP_READINESS_2026-11-07.md) reproduced
premature display before document completion and fixed that boundary. A fresh
running-hook SIGKILL and another idle-host crash both recovered on the first
restart with draft/interrupted history and no replay. Three audited startups
remained hidden through document completion and displayed once each; normal
renderer reload and actual renderer loss also recovered. Fresh full desktop
validation passed. Other G02/G06 criteria remain open; this does not establish
hook cleanup after host loss or solve every possible WebKit failure.
[Main-window Close during running pre-push execution](reports/G02_G06_RUNNING_PUSH_CHECK_CLOSE_2026-11-07.md)
now passed using native Window > Close Window (`performClose:`) with the main window
foreground. Live running phase/process proof preceded Close; native shutdown removed
hook/child before natural finish, persisted cancelled/unpublished/not-passed history,
and preserved exact draft/repositories through first restart without replay. No code
change or fresh automated matrix run. The subsequent
[direct AppKit running-check case](reports/G02_G06_RUNNING_PUSH_CHECK_APPKIT_2026-11-07.md)
now passed: actual termination dispatch/delegate interception/return, no attached
sheet, normal exit, hook/child cleanup before natural finish, cancelled/unpublished
history and exact draft/repository preservation through first restart without replay.
No product change; prior automated evidence retained. Actual Dock/session, remaining
phase-specific recovery and G06 bypass/manager/compatibility remain separate.
The subsequent [native detected-manager/bypass case](reports/G06_NATIVE_MANAGER_BYPASS_2026-11-07.md)
now passed: Execute refused the detected unsupported-manager signature before review;
explicit BypassAll was reviewed natively, declined at OS confirmation, then freshly
proposed/approved. Cancelled and complete records remained unpublished/not passed,
with zero hook execution and no restart replay. This closes selected native refusal/
bypass interaction, not installed-manager Execute compatibility. Other G02/G06
criteria retain their dispositions; no product code changed.
[Actual renderer loss during running pre-push execution](reports/G02_G06_RUNNING_PUSH_CHECK_RENDERER_LOSS_2026-11-07.md)
now passed cancellation/recovery in the surviving host: exact renderer kill, old
hook/child absent before natural finish, stale review closed, saved draft and
cancelled history restored with no replay. However the subsequent normal restart
was blank despite document Finished/show and needed explicit reload. Reliable
startup presentation is reopened; prior document-readiness passes remain bounded.
Next investigate the retained post-load blank-window evidence. No source fix claimed.
Current gate dispositions remain in EXECUTION_STATUS.

### November 8 native SHA-256 — prepared, execution pending

[Native format package](reports/G06_NATIVE_SHA256_2026-11-08.md): real SHA-256
repository/remote and read-only loopback HTTPS fixture prepared; native initialized
with zero trust grants/requests. Cancel, observation/download and restart follow.
G02/G06 remain partial.

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

## Safety, execution and evidence rules

- Perform interaction manually or through an organization-permitted native testing
  facility. If policy prevents access, record **Blocked**. Do not change the app
  identifier, enable another automation route, or substitute browser tests to
  bypass a restriction.
- Fixture setup and controlled-commit cases necessarily create test commits only
  inside newly created disposable repositories. Never commit or push the GitFleet
  project, use existing user repositories, configure remotes, or use personal keys.
- Preserve M0–M2 inspection/helper restrictions, supported-state checks, trust,
  native approval, exact-plan execution and guarded recovery. Do not add generic
  IPC, enable crash injection in the desktop, or weaken checks for acceptance.
- Use only local macOS/APFS repositories within the documented support envelope.
  Unsupported hooks/indexes must fail closed. Do not change global configuration.
- For each case record its ID, operator/date, binary path/hash/features, fixture
  path, steps, expected/actual result, screenshots where useful, and independent
  Git evidence. Do not copy signing private keys into evidence.

| Result | Meaning |
|---|---|
| Pass | Expected behavior observed, with required independent verification |
| Fail | Observed behavior violates the criterion |
| Blocked | Policy, tooling or another prerequisite prevents execution |
| Not run | No execution evidence |
| Unsupported | Outside the documented M0–M3 support envelope |

A checked checklist box means the result and evidence were recorded, not that the
case necessarily passed. Failures, blocks and unsupported cases do not close gates.

## 1. Build the ordinary desktop and create a fresh fixture

Run from the project directory, only when executing this plan:

```sh
cd /Users/randydodson/Documents/_code/git-fleet

M3_EVIDENCE=$(mktemp -d /private/tmp/gitfleet-m3-evidence-XXXXXX)

cargo build --locked \
  --manifest-path app/core/Cargo.toml \
  --bin gitfleet-cli

npm --prefix app/desktop ci
npm --prefix app/desktop run build

cargo build --locked \
  --manifest-path app/desktop/src-tauri/Cargo.toml

python3 app/desktop/scripts/native-fixture.py \
  > "$M3_EVIDENCE/fixture.json"

cat "$M3_EVIDENCE/fixture.json"
```

Read and validate the generated paths:

```sh
M3_FIXTURE=$(python3 -c \
  'import json,sys; print(json.load(open(sys.argv[1]))["root"])' \
  "$M3_EVIDENCE/fixture.json")

test -f "$M3_FIXTURE/.gitfleet-m3-fixture" || exit 1
test -d "$M3_FIXTURE/workspace/engine/.git" || exit 1
```

The fixture contains `engine`, `engine/plugins/audio`, `tooling` and `documentation`.
Each has two baseline commits. `engine/README.md` has different staged and working
contents; the nested audio repository is independently dirty. Application state
is outside the repositories at `"$M3_FIXTURE/state"`.

Keep fixture/evidence directories outside the project. Use a new fixture for
unrelated mutation cases; save each JSON under a unique evidence name and reassign
`M3_FIXTURE` from that case's JSON. Never reuse the old recorded temporary fixture
without validating its existence and baseline.

## 2. Capture independent baseline evidence

For each of the four repositories record:

- Symbolic HEAD, HEAD OID and HEAD tree.
- Complete index entries and porcelain status with optional refresh disabled.
- Tracked/untracked file contents and file/symlink modes/targets.
- Hashes of repository metadata, including existing objects, refs, reflogs,
  configuration and index. Snapshot the nested child separately.

Example read-only commands:

```sh
git --no-optional-locks -C "$M3_FIXTURE/workspace/engine" \
  symbolic-ref -q HEAD

git --no-optional-locks -C "$M3_FIXTURE/workspace/engine" \
  rev-parse HEAD 'HEAD^{tree}'

git --no-optional-locks -C "$M3_FIXTURE/workspace/engine" \
  ls-files --stage

git --no-optional-locks -C "$M3_FIXTURE/workspace/engine" \
  status --porcelain=v2 --untracked-files=all
```

Save both versions of `engine/README.md` independently:

```sh
git --no-optional-locks -C "$M3_FIXTURE/workspace/engine" \
  show :README.md > "$M3_EVIDENCE/engine-index-before.txt"

cp "$M3_FIXTURE/workspace/engine/README.md" \
  "$M3_EVIDENCE/engine-worktree-before.txt"
```

Redirect outputs and metadata/content snapshots into case-specific evidence files.
These sample Git summaries alone do not prove the absence of object/index writes;
the repository byte snapshots supply that assertion. Ignore access timestamps,
not content or mode changes. Do not follow symlinks outside fixture boundaries.

Do not use `git write-tree` as an inspection assertion: it can create objects.
Read existing trees using `rev-parse`, `ls-tree` and `show`. Repeat baseline capture
before each mutation case and compare expected changes independently of UI labels.

## 3. Verify the ordinary read-only build — A02/A09

Launch the ordinary binary:

```sh
app/desktop/src-tauri/target/debug/gitfleet-desktop \
  --data-dir "$M3_FIXTURE/state"
```

1. Verify the saved workspace opens and displays all four repositories.
2. Inspect changes, history, diffs and previews without granting trust.
3. Verify trust/write controls are disabled; no UI preference enables writes.
4. If a read-only recovery review is available, verify execution is disabled.
   If no recovery record exists, exercise this check in the recovery case later.
5. Close the application and compare repository snapshots.

**Pass:** Inspection changes only application-owned state. Repository contents,
index, refs, reflogs, configuration and objects remain unchanged. Do not enable
writes in this binary to continue testing.

## 4. Exercise workspace management and native picker — A04

Using the ordinary build:

1. Click **Open folder…**, then cancel the native picker. Verify no saved/active
   workspace changed.
2. Create another fresh fixture and open its workspace through the native picker.
3. Rename its saved workspace through **Workspace settings**.
4. Exercise **Change root…**, including cancellation.
5. Add a fixture-directory exclusion and verify inventory reconciliation.
6. Switch between the two saved workspaces; verify inventories stay separate.
7. Use **Remove saved definition** for the secondary workspace.
8. Verify its repository directories and contents still exist.

**Pass:** Picker cancellation is harmless; accepted paths identify the intended
workspace; switching never mixes inventories; removal never deletes repository data.
Do not point the acceptance build at personal projects.

## 5. Verify navigation, drafts and actual restart — A05/A06

1. Select `engine` in Overview and switch Table → Tree → Table.
2. Confirm selection remains explicit: selecting `engine` does not select audio.
3. Open tabs for `engine`, audio and `tooling`; enter distinct unfinished drafts.
4. Select different change sources/files and history commits.
5. Change filter/sort, theme and panel dimensions.
6. Close one repository tab and verify other drafts remain.
7. Close the main window normally and verify the process exits.
8. Relaunch with the same data directory.
9. Verify saved tabs, drafts, navigation and supported layout state; check focus.

**Pass:** State follows the correct workspace/worktree identity, drafts never
migrate between repositories, and restart executes nothing. Record restoration
failures precisely; SQLite persistence tests do not establish native acceptance.

## 6. Verify Changes, History and monitoring — A07/A08

1. Inspect both index and working-tree sources for `engine/README.md`; compare
   displayed content against the independent baseline files.
2. Inspect tracked changes and untracked `notes.txt`.
3. Open History and inspect both commits. Compare OIDs, messages, parents and
   changed paths against read-only Git commands.
4. Exercise available parent selection and commit diffs.
5. Edit a disposable file externally and observe automatic status reconciliation.
6. Rename a fixture repository directory; verify identity/draft continuity.
7. Move a repository outside the workspace temporarily, but within the disposable
   fixture root. Verify a missing/unavailable state, not silent reassignment.
8. Restore it and observe reconciliation.
9. Switch to the secondary fixture workspace; verify old-workspace results do not
   overwrite the active view.

Allow approximately ten seconds for status fallback and sixty seconds for discovery
fallback; notification delivery may update sooner. Record actual latency and any
visible degradation. External moves/edits are intentional fixture mutations and
must be distinguished from application writes when comparing snapshots.

**Pass:** Content/source identity is accurate, external changes reconcile without
write authorization, and stale workspace results cannot replace active context.

## 7. Build a separate write-enabled acceptance binary

Close the ordinary application. Generate a new write-test fixture and update
`M3_FIXTURE` from its JSON. Capture a new baseline.

```sh
cargo build --locked \
  --manifest-path app/desktop/src-tauri/Cargo.toml \
  --features desktop-writes \
  --target-dir "$M3_EVIDENCE/acceptance-target"

"$M3_EVIDENCE/acceptance-target/debug/gitfleet-desktop" \
  --data-dir "$M3_FIXTURE/state"
```

The separate target directory preserves the ordinary binary. Record the explicit
feature selection. Never promote `desktop-writes` into defaults or launch this
binary against an existing user repository. If native access is denied, stop the
native case and record **Blocked**; do not change identity or use another driver.

## 8. Test trust and independent human approval — A12/A14

1. Open `engine` and **Inspect trust & eligibility**. Verify workspace membership
   does not imply a grant.
2. Click **Review trust**. Verify a separate **AUTHORITATIVE RUST REVIEW** window
   displays the intended canonical repository.
3. Cancel the review and verify no grant.
4. Create another review and click **Continue to native confirmation**.
5. Verify the separate OS dialog offers **Approve exact operation** and **Cancel**.
6. Cancel the dialog and verify no grant.
7. Create a fresh review and explicitly approve it through the OS dialog.
8. Verify grant/currentness and execution eligibility are distinct.
9. Confirm audio/other repositories did not inherit trust.
10. Revoke trust and verify subsequent write proposals are blocked. Re-grant only
    when needed for another case.
11. Close an unapproved review and replace a pending review with another. Neither
    action may approve the abandoned plan.

**Pass:** Only explicit native approval grants trust; a selection, checkbox,
cancelled dialog or closed review never does.

## 9. Test exact review and index-only commit — A13/A15

Use a fresh fixture preserving the original staged/working distinction.

1. Capture baseline HEADs, index entries and contents for all four repositories.
2. Trust only `engine`.
3. Include `README.md` from the **index** source. Leave `notes.txt`, unrelated
   files and audio unselected.
4. Enter a distinctive multiline message and select **Use configured signing**;
   the initial fixture is configured unsigned.
5. Click **Review commit**. Inspect canonical path, selected source, message,
   patch, hook policy and signing policy in the authoritative window.
6. Cancel once; independently prove HEAD/index/worktree are unchanged.
7. Create a fresh proposal, inspect it, and approve through the native dialog.

Verify independently, using this case's baseline files:

```sh
git --no-optional-locks -C "$M3_FIXTURE/workspace/engine" \
  show HEAD:README.md > "$M3_EVIDENCE/engine-head-after.txt"

cmp "$M3_EVIDENCE/engine-index-before.txt" \
  "$M3_EVIDENCE/engine-head-after.txt"

cmp "$M3_EVIDENCE/engine-worktree-before.txt" \
  "$M3_FIXTURE/workspace/engine/README.md"

git --no-optional-locks -C "$M3_FIXTURE/workspace/engine" \
  ls-tree -r HEAD
```

Also inspect actual commit message/parent and before/after index entries.
**Pass:** Exactly one intended commit is published with the correct message,
parent and tree. Working content stays different, excluded files and the child
remain untouched, and the UI result matches actual Git state.

Repeat with harmless HTML-like text and control/bidirectional characters. Content
must render inertly and unambiguously without changing the approved message bytes.

## 10. Test stage, unstage and stale proposals — A13/A15

Use fresh fixtures for separate cases; capture baseline and result evidence.

| Case | Procedure | Required result |
|---|---|---|
| Stage | Select working-tree path, **Review stage**, native approval | Only selected index entries change; HEAD/worktree unchanged |
| Unstage | Select staged path, **Review unstage**, native approval | Selected index entries match HEAD; unrelated staging preserved |
| Stale content | Review working-tree commit, externally edit selected file, approve | Old plan rejected; no unapproved publication |
| Same-OID switch | Create branch at HEAD, open review, switch branch, approve | Old plan rejected despite same commit OID |
| Workspace change | Open review, then switch workspace if allowed | Old context invalidated or switching safely prevented |
| Expiry | Leave review beyond displayed expiry, attempt confirmation | Fresh proposal required |

Same-OID branch setup uses only the disposable repository:

```sh
git -C "$M3_FIXTURE/workspace/engine" branch m3-same-oid
# Open the review in GitFleet, then:
git -C "$M3_FIXTURE/workspace/engine" switch m3-same-oid
```

Record whether rejection occurs before/after the dialog. The decisive criterion is
that stale authority cannot publish. Intentional external fixture changes are not
application mutations and must be accounted for separately.

## 11. Test hooks, bypass and signing — A14/A15

Use fresh fixtures and repository-local configuration only. Install hooks before
granting trust, or expect configuration changes to require fresh trust. Hooks are
unsandboxed executable code; keep them minimal and confined to the fixture.

| Hook case | Required result |
|---|---|
| Pre-commit writes sentinel and exits nonzero | Visible rejection, sentinel confirms execution, no published commit |
| Explicit **Bypass all — explicit review** with rejecting hooks | Review states bypass; no configured hook sentinel executes |
| Post-commit writes sentinel and exits nonzero | Commit remains published; post-processing failure reported separately |
| Hook changed after review | Stale trust/review rejected |
| Reference-transaction configured with Execute requested | Unsupported policy fails closed; no false eligibility |

For signing, complete both positive cases:

1. **SSH:** Generate a disposable Ed25519 key using `ssh-keygen`; configure
   `commit.gpgsign`, `gpg.format=ssh` and `user.signingkey` locally. Create a
   fixture-only allowed-signers file. Approve a desktop commit and independently
   verify it using `git verify-commit` with that allowed-signers file.
2. **OpenPGP:** Create a new mode-0700 `GNUPGHOME` outside the repositories, generate
   a disposable signing key, configure the exact executable/fingerprint locally,
   and launch the acceptance process with that isolated keyring. Approve a desktop
   commit and independently verify its signature under the same isolated keyring.
3. For each format, make the configured key unavailable before a second attempt.
   Verify no publication and no unsigned fallback.

Use [existing M2 signing tests](../app/core/tests/m2_writes.rs) as the configuration
reference. Record resolved signer identity and the authoritative review. Never
substitute a personal keyring. Missing isolated tooling means **Blocked**, not Pass.

## 12. Test cancellation, shutdown and guarded recovery — A16/A17

Use fresh fixtures for separate lifecycle cases.

1. Close the main window during inspection/scan; verify actual process exit.
2. Relaunch immediately with the same data directory; verify lock release.
3. Close an unapproved review; verify no mutation.
4. For an approved in-progress operation, use a minimal supported hook that writes
   a started sentinel and waits briefly, within the supported execution timeout.
5. While waiting, test cancellation, review close and main-window close in
   separate runs. Verify bounded termination and actual HEAD/index/result state.
6. For forced interruption, record the exact PID of the launched acceptance
   executable. Kill only that PID after the fixture hook's started sentinel appears.
   Never use a broad process-name kill.
7. Relaunch with the same state directory. Inspect recovery notices or
   **Operation results**, then **Inspect guarded recovery**.
8. Cancel recovery once and prove it made no repository changes.
9. Create a fresh recovery review; approve only the displayed supported action.
10. Verify actual refs/index/owned locks and sentinel counts. Reopen the recovered
    fixture using the ordinary read-only binary and verify recovery execution is
    disabled there.

**Pass:** Interrupted state is visible on restart; recovery never replays a commit
or hook; foreign/stale state cannot force unsafe recovery. Cancellation must not
promise rollback after publication. Read-only inspection must not repair state.

A manual kill does not prove a particular publication boundary. Record the actual
observed phase. Retain the automated thirteen-phase SIGKILL matrix as separate
evidence. Do not enable crash-injection desktop features or manually delete locks
to make recovery pass. Preserve unexpected artifacts for diagnosis.

## 13. Complete native accessibility and visual checks — A19

Test in the actual application:

- Keyboard-only workspace, selection, tabs, Changes/History and review navigation.
- Visible focus, readable errors and usable disabled controls.
- VoiceOver labels/roles, dialog announcements and exact-review reading.
- Initial review focus on **Cancel review**; no unexpected approval from Return or
  Escape. Exercise native-dialog keys deliberately and record their actual effect.
- Main/review/dialog focus transitions and focus return after dismissal.
- Light, dark and system themes, contrast, resizing and scrolling.
- Long paths/messages without hidden approval details.
- Responsiveness during refresh and safe close/cancellation behavior.

Capture main/review screenshots and observations. Screenshots alone do not prove
accessibility. Do not mark untested VoiceOver/focus behavior as passed.

## 14. Final verification and gate disposition — A20

1. Stop the acceptance executable and only fixture-owned signer agents.
2. Relaunch the ordinary binary against disposable state; verify writes disabled.
3. Verify no fixture application processes remain and the data lock can be reacquired.
4. Preserve evidence; avoid broad temporary-directory deletion commands.
5. Record every result, unresolved failure and supporting artifact below.
6. Do not enable ordinary writes, start M4, commit or push as part of this plan.

M3 can close only when all required native criteria pass or an explicit scope
decision resolves an exception. A blocked/unsupported/not-run case is not an
implicit waiver. Passing this plan does not itself authorize changing defaults.

## Current execution follow-ups — November 2, 2026

The checklist below preserves the earlier checkpoint reconciliation and its
user-observed provenance. It is historical; current results supersede its older
"not run"/"pending" labels only for the exact checks recorded in the
[acceptance matrix](ACCEPTANCE_MATRIX.md#12-m3-acceptance-gates--implementation-with-native-gates-open)
and [execution status](EXECUTION_STATUS.md). Current agent-operated evidence includes
actual foreground Cancel, close during held reads/running writes, native guarded
abort after forced interruption, no replay, and serialized refresh during writes.
See [latest report](reports/M3_POLLING_2026-11-02.md). M3 remains incomplete; no
ordinary-write default change or push is authorized by this acceptance document.
The user's later chat authorization governs local milestone commits.

## Historical results checklist

Reconciled: 2026-11-02 against the [native checkpoint](M3_NATIVE_ACCEPTANCE_HANDOFF_2026-11-02.md).
**User-observed** means the checkpoint records terminal output/screenshots; the
current agent has not independently reproduced the native case. Partial rows include
untested criteria and do not close gates. Historical November 1 evidence remains
separate. Exact steps, OIDs, fingerprints and fixture paths are in the checkpoint.
Checkboxes remain unchecked pending full evidence reconciliation and gate review;
the status column records the currently available observations, including failures.

Run date/operator: November 1–2 / user, as recorded in checkpoint
Host/tool versions: user-reported build output; exact test-time attestation absent
Ordinary/acceptance binary hashes/features: no exact test-time attestation established
Evidence directory: `/private/tmp/gitfleet-m3-evidence-niCdnj` (temporary; durable preservation pending)

| Recorded | Case | Gates | Status | Evidence / actual result |
|---|---|---|---|---|
| [ ] | 1. Ordinary build and fresh fixtures | A02/A04 | Partial | User-observed builds and separate fixtures; exact binary hashes/features unestablished. |
| [ ] | 2. Independent snapshots for all four repositories | A09/A13 | User-observed pass | Four-repository before/after snapshots in checkpoint; timestamps/every OS property not covered. |
| [ ] | 3. Ordinary read-only behavior and unchanged repository bytes | A02/A09 | User-observed pass | Read-only session snapshots matched for all four repositories. |
| [ ] | 4. Native picker cancel/accept, scope, exclusions and definition removal | A04 | Partial / defect | Picker cancellations, naming/exclusions and removal observed; removal produced stale UI. Renderer fix passes; native retest pending; full scope criteria unfinished. |
| [ ] | 5. Table/Tree, explicit selection, tabs, drafts and restart restoration | A05/A06 | Partial | Table/Tree selection, tabs/drafts and restart observed; geometry/focus/lifecycle criteria remain. |
| [ ] | 6. Index/working content and History/parent diffs | A07 | Partial / defect | Independent staged/working and History/root comparisons observed. Retained patch fixed in renderer; native retest pending. |
| [ ] | 6. External edit/move/missing reconciliation and workspace isolation | A08 | User-observed pass | External edits, rename/move/missing restoration and inactive workspace isolation observed; no formal performance claim. |
| [ ] | 7. Separate acceptance build; ordinary defaults unchanged | A02 | Partial | User used acceptance build and ordinary read-only restarts; exact test-time feature/hash attestation absent. |
| [ ] | 8. Cancel/approve trust through real native dialog | A12/A14 | User-observed pass | Trust cancel and native grant observed with snapshot preservation. |
| [ ] | 8. No inherited grant, revocation and abandoned-review invalidation | A12/A14 | Partial | Engine-only trust, audio no inheritance, revocation/regrant observed; complete abandoned-review lifecycle untested. |
| [ ] | 9. Exact index-only commit with independent tree/message/index checks | A13/A15 | User-observed pass | Independent parent/tree/message/index/working/exclusion checks for index-only publication passed. |
| [ ] | 9. Inert HTML-like/control/bidi display and exact approved bytes | A13 | Not run | Native hostile-content case remains; automated escaping tests do not substitute. |
| [ ] | 10. Stage and unstage | A15 | User-observed pass | Stage changed approved README index entry; unstage restored prior entries/status. |
| [ ] | 10. Stale content and same-OID branch switch | A13/A15 | User-observed pass | Selected-content and same-OID branch changes rejected without publication/index changes. |
| [ ] | 10. Workspace context change and expired review | A12/A13 | Not run | Remaining native context-switch and review-expiration criteria untested. |
| [ ] | 11. Rejecting, bypassed, post-failure and changed hooks | A14/A15 | User-observed pass | Rejecting pre-hook, explicit bypass, published post-hook failure and changed-hook rejection observed. |
| [ ] | 11. Unsupported reference-transaction Execute fails closed | A14 | User-observed pass / unsupported execution | Execute rejected before proposal; sentinel absent and snapshots matched. Hook execution remains unsupported. |
| [ ] | 11. Isolated SSH positive verification and missing-key rejection | A15 | User-observed pass | Signature independently verified with expected SSH fingerprint; missing key blocked without unsigned fallback. |
| [ ] | 11. Isolated OpenPGP positive verification and missing-key rejection | A15 | User-observed pass | Signature independently verified with expected VALIDSIG fingerprint; missing secret key blocked without fallback. |
| [ ] | 12. Close during read; exit and immediate lock reacquisition | A17 | Not run | Close during read, process exit and immediate lock reacquisition not established. |
| [ ] | 12. Cancel/review-close/main-close during approved execution | A17 | Not run | Cancel was never clicked. Slow hook ended HookRejected; neither cancellation nor timeout passed. Review/main close during execution remains open. |
| [ ] | 12. Forced interruption, native recovery review and no replay | A16/A17 | Not run | Native forced interruption and guarded recovery/no replay remain open. |
| [ ] | 12. Ordinary binary cannot execute recovery | A02/A16 | Partial | Ordinary read-only footer observed; recovery inspection/execution gating still needs native verification. |
| [ ] | 13. Keyboard, focus, VoiceOver and native approval safety | A19 | Partial | Keyboard focus/no trap reported; native approvals exercised; VoiceOver untested. |
| [ ] | 13. Themes, contrast, resizing, long content and responsiveness | A19 | Partial | Themes accepted for now; contrast unmeasured and visual/layout defects remain; no complete responsiveness gate. |
| [ ] | 14. Processes stopped; ordinary write gate reverified | A02/A17 | Not run | Current process state unknown; final shutdown/lock and ordinary write-gate recheck pending. |
| [ ] | 14. Evidence and unresolved gates reconciled | A20 | Partial | Documents reconciled; durable non-secret evidence preservation, binary attestation and unresolved gates remain. |

### Failure / blocker record

| Case/gate | Actual behavior or blocker | Artifact | Resolution / retest |
|---|---|---|---|
| Historical agent access | November 1 organization denial; later user-operated evidence does not establish current agent permission | [Prior block record](validation/m3/native-acceptance.json), [later checkpoint](M3_NATIVE_ACCEPTANCE_HANDOFF_2026-11-02.md) | Honor policy for further native work |
| Workspace removal | Mixed/stale UI and stale_review until restart | November 2 checkpoint | Renderer fix and regression pass; native retest pending |
| History inspector | Changes patch retained before selecting a commit | November 2 checkpoint | Renderer fix and view-switch/late-response regression pass; native retest pending |
| Running cancellation | Cancel never clicked; blocked HookRejected outcome | November 2 checkpoint | Actual cancellation and preservation test still required |

Final disposition: **M3 incomplete; partial user-observed native evidence, required gates still open.**
No new native tests were performed during this reconciliation.
Ordinary desktop writes remain disabled by default. No commit or push authorized.

## Owning references

- [M3 acceptance ledger](ACCEPTANCE_MATRIX.md#12-m3-acceptance-gates--implementation-with-native-gates-open)
- [M3 desktop and IPC contracts](architecture/M3_DESKTOP_AND_IPC.md)
- [Desktop build/workflow](../app/desktop/README.md)
- [Current status](IMPLEMENTATION_STATUS.md)
- [Conversation handoff](IMPLEMENTATION_HANDOFF.md)
- [Existing M3 evidence](validation/m3/README.md)
