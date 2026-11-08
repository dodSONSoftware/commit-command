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
First valid restart was blank: reliable recovery presentation remains open and
requires reproduction/diagnosis. G02/G06 remain partial.
Current gate dispositions remain in EXECUTION_STATUS.

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
