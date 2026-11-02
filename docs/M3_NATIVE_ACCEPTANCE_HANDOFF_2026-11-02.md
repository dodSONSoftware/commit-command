# GitFleet M3 native acceptance handoff — 2026-11-02

## Start here

This checkpoint preserves the native acceptance evidence supplied by the user during the November 1–2 session. **M3 is not complete.** These results were observed by the user and supported by pasted terminal output and screenshots; they were not independently reproduced by the agent writing this document. Current process state, temporary-file availability, and source/binary correspondence must be checked before continuing.

The next agent should operate the application and terminal directly. The user explicitly requested this because relaying scripts and timing-sensitive clicks was slow and error prone. Do not require the user to repeat already demonstrated checks without a concrete reason. Honor tool approval restrictions; do not bypass a computer-control denial or change application identity to evade one.

The previous conversation became stuck in repeated reading and compaction. Use this document as the checkpoint, inspect only the necessary current sources, and work in bounded steps. Do not reload the entire conversation or repeatedly reread the same documentation.

## Scope and safety boundaries

- Project: `/Users/randydodson/Documents/_code/git-fleet`.
- Preserve `docs/` and `app/`, existing functionality, and all unrelated staged, unstaged, and untracked work. The implementation tree was dirty at handoff.
- Core remains independent of Tauri, React, and presentation technologies; do not split it into more crates without a demonstrated requirement.
- Ordinary desktop writes remain disabled by default. Use an explicitly opted-in acceptance build only under the existing plan, against verified disposable fixtures.
- No Git writes against existing user/project repositories. No commit, push, packaging, release, global configuration changes, or M4 implementation is authorized.
- Do not delete fixtures, keys, journals, locks, or evidence as cleanup without understanding their ownership and purpose. Never print private-key contents.
- Native tests are unfinished. This handoff does not authorize silently marking skipped or untested gates as passed.

## Owning documents and evidence

Read the relevant portions of `docs/M3_NATIVE_ACCEPTANCE_PLAN.md`, `docs/IMPLEMENTATION_HANDOFF.md`, `docs/IMPLEMENTATION_STATUS.md`, and `docs/architecture/M3_DESKTOP_AND_IPC.md` before resuming. Earlier contracts are in `docs/architecture/M0_SAFETY_BOUNDARIES.md`, `docs/architecture/M1_READ_ONLY_WORKSPACES.md`, and `docs/architecture/M2_CONTROLLED_WRITES.md`. Reconcile final outcomes with `docs/ACCEPTANCE_MATRIX.md` and `docs/DECISIONS_AND_OPEN_ITEMS.md`.

This document supplements historical handoff/status statements that may still describe all native gates as blocked. It does not assert that those other files were updated.

User-reported evidence directory: `/private/tmp/gitfleet-m3-evidence-niCdnj`. It contains fixture descriptions, the snapshot script, baseline/after snapshots, independent verification results, signature output, and selected byte-level evidence. Temporary paths can disappear. Preserve relevant evidence in an appropriate durable project location before further testing, excluding private keys, secret-key directories, and sensitive application state. Historical `docs/validation/m3/` evidence must not be confused with this later session.

Snapshots compare symbolic HEAD, HEAD OID/tree, index entries, porcelain status, and filesystem entries including modes, file sizes/hashes, and symlink targets. They do not prove unchanged timestamps or every OS-level property. No exact test-time source/binary attestation was established in the conversation.

## Verified session outcomes

### Ordinary read-only desktop

Core CLI, frontend, and native desktop builds succeeded in the user's output. npm audit reported 10 moderate development-dependency findings and zero production-dependency findings; no dependency fixes were applied during these checks.

Original fixture: `/private/tmp/gitfleet-m3-native-d0hufzd0`.

- Discovery showed four independent repositories, three with changes, and zero needing attention. Table/tree selection and nested repository separation were exercised.
- Staged and working README previews matched independently captured bytes: staged content said `Reviewed native fixture content.`; working content said `Working content remains outside an index-only commit.`
- History and root-commit patches matched independent Git output.
- Original repository snapshots matched both after initial inspection and after the complete read-only session: all four repositories unchanged.
- Separate repository drafts, tab close/reopen, application restart, active tab, and draft restoration were demonstrated.
- Folder-picker cancellation and Change-root cancellation preserved state and returned focus without errors.
- Light/dark appearance was accepted by the user for now. Keyboard navigation had visible focus and no reported focus trap. VoiceOver was not tested; contrast was not measured.

### Workspace management and monitoring

Secondary fixture: `/private/tmp/gitfleet-m3-native-kr2b9lbc`.

- Renaming to `GitFleet M3 Secondary` updated both name displays without changing the root. Drafts stayed associated with their workspace.
- Excluding `plugins` marked audio missing; restoring the exclusion list restored normal discovery.
- External file edits and restoration were detected automatically, from nearly immediate to approximately four seconds.
- Tooling rename and rename-back updated the repository/tab in about two seconds, retaining the draft with no missing duplicate.
- Moving tooling outside the workspace produced a missing/unavailable state; restoring it cleared the error automatically in about two seconds and preserved the draft.
- Inactive workspace changes remained isolated.
- Removing the saved secondary definition left all four repositories unchanged by full snapshot comparison. **The UI entered a mixed/stale state with `stale_review`; restart recovered, the secondary definition was absent, and the original drafts remained. This is a defect, not an unqualified pass.**

### Controlled writes, trust, staging, and stale reviews

Write fixture: `/private/tmp/gitfleet-m3-native-ug8vk650`.

- Trust review cancellation, native trust grant, engine-only trust without inheritance to audio, revocation rejection, and regrant were exercised. Trust operations and cancelled proposals left all four repository snapshots unchanged.
- Index-only publication created `e09e95cc2fd91b3fdaf7c388eb2a94c4b65d33a0` directly above baseline `51bb784c9e8c1ba1cb1d199dd57fdc33e251a492`.
- Independent checks verified exact message, main branch, full committed tree equal to the baseline index, preserved index entries, unchanged working README, unchanged nested contents, and unchanged other repositories.
- Exact message was `M3 native index-only acceptance\n\nCommit the reviewed staged README; preserve working content.\n`.
- Ordinary restart retained durable results and did not replay or modify repository state; all snapshots matched the post-publication snapshot.
- Stage changed only the README index entry to the approved working bytes. Unstage restored complete pre-stage index entries and status. HEAD, branch, working contents, and other repositories remained unchanged.
- Editing selected working content after review produced `needs_reauthorization` with `StaleReview: Selected working content changed`; HEAD and index bytes stayed unchanged. Test edits were restored.
- Switching to `m3-same-oid` after review, while keeping the same commit OID, produced `needs_reauthorization` with `StaleReview: Checked-out branch changed`. Both branch refs and index entries stayed unchanged. The fixture was switched back to main; the test branch remains.

### Hooks

Pre-commit fixture: `/private/tmp/gitfleet-m3-native-hqehpkyh`.

- Trust/inspection did not execute its sentinel hook.
- A pre-commit hook exited 23 after writing a sentinel. Execute mode returned `HookRejected`, ran it exactly once, and preserved HEAD, branch, index bytes, working contents, and other repositories.
- Explicit reviewed BypassAll published `dad4c4c0943c3cf98caf9e71af25332670fca430` with exact message `M3 explicit hook bypass acceptance\n`. Independent checks passed; the sentinel still recorded exactly one execution and the installed hook remained unchanged.

Post-commit fixture: `/private/tmp/gitfleet-m3-native-5k4erxeb`.

- Publication created `e81bd36e5981331bfd640cf3414ae000635af4cb`; the post-commit hook exited 23 once.
- Result correctly distinguished `published_with_post_processing_failure`, published true, and index reconciled true. Independent parent/tree/message/index/working-content/exclusion checks passed, and durable results displayed the post-hook failure.
- Changing the hook after review caused `needs_reauthorization` / `UntrustedRepository`, without another publication, index change, or hook execution. The hook was restored; trust was reported revoked.

Unsupported reference hook fixture: `/private/tmp/gitfleet-m3-native-22lusee_`.

- Execute mode rejected the installed `reference-transaction` hook before a write proposal. Its sentinel remained absent and all four full snapshots matched.
- **Fail-closed rejection passed; execution of this hook remains unsupported.**

### SSH signing

Fixture: `/private/tmp/gitfleet-m3-native-de5h4dpa`; disposable key under `signing/`; signer `/usr/bin/ssh-keygen`.

- Expected fingerprint: `SHA256:nz0tI1UNHADN/+ZjygZ2EwwIFQGJ7xmjsXkL8m0gZ6k`.
- Signed publication: `c3ca010d6caeea3fe7f8f1c1c4bab1bc6c871704`, exact message `M3 SSH signing acceptance\n`.
- Independent signature verification exited zero with the expected fingerprint. Full tree, parent, branch, message, signature presence, index, working contents, and other repository checks passed.
- Moving the private key after review produced blocked `SigningFailed`, no unsigned fallback, and no publication. HEAD, branch, index bytes, working contents, and other repositories remained unchanged. The disposable key was restored.

### OpenPGP signing

Fixture: `/private/tmp/gitfleet-m3-native-13rg7n0e`; isolated home `gnupg/`; executable `/private/tmp/gitfleet-m2-tools/install/bin/gpg` (reported GnuPG 2.5.24).

- Expected fingerprint: `EB661ECB86143CD81D98B8442927E80107A4F80C`.
- Signed publication: `5e66ec2233096d0680331a854f0929854d31aec0`, exact message `M3 OpenPGP signing acceptance\n`.
- Independent verification exited zero with exactly the expected VALIDSIG fingerprint. Full tree, parent, branch, message, signature presence, index, working contents, and other repository checks passed.
- Stopping the fixture agent and temporarily replacing its secret-key directory with an empty directory caused blocked `SigningFailed`, no unsigned fallback, and no publication. Preservation checks passed. The original secret-key directory was restored after stopping the isolated agent.

## Cancellation checkpoint — do not misclassify

Fixture: `/private/tmp/gitfleet-m3-native-_yii1o77`; workspace ID `e296a36c-2eaf-4047-a598-41f188653756`; data directory is its `state/` directory.

- Engine HEAD: `51d563931025e6f2efe8b26662554bb4ca16e719`.
- Operation: `ac99b8ec-85e0-4310-8a4b-b2a87b127e46`.
- Pre-commit wrote `cancellation-hook-started.txt`, slept 60 seconds, then exited 23.
- **The user did not click Cancel. Actual running-operation cancellation was not tested.** The observed durable outcome was blocked `HookRejected`; do not call this a verified timeout either.
- Full before/after repository snapshots matched for all four repositories.
- The last supplied screenshot, after ordinary restart, displayed one recorded operation needing attention, blocked HookRejected, published false, index reconciled false, an `Inspect guarded recovery` button, and the read-only desktop footer.
- No subsequent restart-preservation/sentinel check was supplied. Current application state is unknown.

## Remaining work for the next conversation

Reconcile this list with the actual acceptance plan; it is not a claim that every remaining gate is enumerated.

1. Inspect current Git status, evidence availability, exact executable paths, launch configuration, and owned process/UI state. Preserve evidence without copying disposable secrets into the repository.
2. Inspect guarded recovery for the blocked operation in read-only mode and verify no replay, repository mutation, or repeated hook execution.
3. Use a fresh disposable fixture to test actual cancellation while the hook is running, with the agent handling timing and terminal checks. Record the action, terminal outcome, durable operation state, and preservation evidence separately.
4. Complete applicable review-window/main-window close and restart/forced-termination gates. Target only an exactly identified owned process. Do not use broad kill commands, manually edit journals, remove locks, or introduce crash injection without a justified approved procedure.
5. Complete remaining applicable IPC/hostile-content, context-switch, review-expiration, accessibility, and lifecycle gates. VoiceOver remains untested. Historical automated recovery tests do not substitute for native lifecycle evidence.
6. Update status, handoff, and acceptance records with precise pass/fail/unsupported/untested distinctions. Keep desktop writes disabled by default and stop at the M3 boundary.

Previously used executable paths, to verify before use:

- Ordinary: `/Users/randydodson/Documents/_code/git-fleet/app/desktop/src-tauri/target/debug/gitfleet-desktop`.
- Acceptance: `/private/tmp/gitfleet-m3-evidence-niCdnj/acceptance-target/debug/gitfleet-desktop`.
- Launch data must be the chosen disposable fixture's absolute `state/` path via the documented `--data-dir` option; consult the existing plan for the acceptance opt-in mechanism.

## Reported UI defects and deferred polish

Do not silently fix these while gathering acceptance evidence; track and scope them explicitly.

- Window position/size did not restore.
- Entering History initially retained a Changes patch until a commit was selected.
- Removing a saved definition produced mixed workspace UI and stale-review errors until restart.
- Narrow composer layout stretched controls into oversized buttons.
- `Inspect trust & eligibility` looks like a tab despite acting as an inspection action; the user explicitly rejected this confusing presentation.
- Authoritative review dumps expose long raw JSON/byte arrays and are difficult to review.
- Trust labels needed explicit inspection to refresh even when Core had the correct state.
- Stage/unstage review summaries included irrelevant message/signing fields; a selected-file count also appeared stale after publication.

Light/dark cosmetic adjustments were explicitly deferred by the user. No implementation fixes, builds, native tests, commits, or pushes were performed as part of writing this handoff.

## Suggested fresh-chat instruction

> Continue GitFleet M3 native acceptance from docs/M3_NATIVE_ACCEPTANCE_HANDOFF_2026-11-02.md. Read the existing acceptance plan and relevant safety contracts, verify current evidence and application state, and operate the GUI and terminal yourself. Preserve passed evidence; actual running-operation cancellation is still untested. Keep ordinary desktop writes disabled by default, use only disposable repositories for write tests, do not commit or push, do not implement M4, and update the handoff/status with exact outcomes before stopping.

## Follow-up implementation — 2026-11-02

The user subsequently authorized the workspace-removal and History patch fixes,
status reconciliation and relevant tests, without commit or push. The renderer now
lets `load()` detect removal of the active workspace and reset the workspace,
inventory, tabs, trust and operation view. Changes/History inspector state is cleared
before loading the current view/comparison; late-response guards remain intact.
Two component regressions cover removal/subsequent-workspace drafts and patch/preview
isolation, historical comparison restoration and a late Changes response.

These two defects are patched with automated renderer evidence; **native retest is
pending**. The earlier session narrative and other UI issues above remain historical
observations. No new native acceptance, cancellation, recovery, signing, process
termination or build-default change occurred during the fixes. Current validation
is recorded in [implementation status](IMPLEMENTATION_STATUS.md). Historical
`validation/m3/` logs and hashes are unchanged and do not attest the updated renderer.
The current status, ledger and plan checklist now reference the later user evidence;
M3 is still incomplete and ordinary desktop writes remain disabled.
