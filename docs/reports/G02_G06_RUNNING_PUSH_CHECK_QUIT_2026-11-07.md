# G02/G06 Quit during a proven running pre-push check

**Selected native Cmd+Q-during-execution case passed. G02/G06 remain open.**
This follows [native running-hook Cancel review acceptance](G02_G06_RUNNING_PUSH_CHECK_CANCEL_2026-11-07.md),
committed as `d56730d`. This step exercises application Quit rather than explicit
review cancellation. No product source, feature default, approval rule or push
executor changed.

## Fixture and actual execution

Created a fresh marked temporary fixture with three real nested working repositories,
three bare remotes and a read-only loopback TLS upload-pack service. The leaf source
had one outgoing commit, distinct staged/working README bytes and an untracked file.
A supported pre-push hook records its invocation/PIDs outside managed repositories,
starts a 45-second sleeping child, and would write a completion marker after waiting.
Installed the hook before granting trust; captured full repository/remote baselines.

Reused the exact previously validated native acceptance artifact. Its SHA-256 and
relevant source hashes matched the active-sheet checkpoint, with product source
unchanged through this step's base `d56730d`. The copied bundle used fresh state and
an explicit fixture-only HTTPS CA. No global TLS/Git changes, private server,
Git authentication credentials, personal keys or user repositories were used.
The loopback server rejects publication endpoints; no Git push was invoked.

Native actions used Computer Use. Native AX states were inspected, not captured
as saved screenshots. Independent scripts only read fixture files/state/processes;
no hidden IPC, synthetic running phase, approval bypass or external hook kill.

1. Entered a composer draft and independently approved trust for **only** the leaf
   repository. Separately reviewed and approved native observation of its exact
   loopback HTTPS `refs/heads/main`; one discovery GET completed.
2. Selected only the leaf and fresh observation in the local planner. Source preview
   showed one outgoing commit and zero child requirements. Exact pre-push review
   and OS summary stated Execute, source/destination OIDs and no publication authority.
   No hook ran before the independent native approval.
3. Approved the hook-only execution. Before Quit, independently recorded the durable
   `running`/unpublished phase, a single invocation marker, hook PID **5734**, child
   PID **5735**, absent completion marker, and the exact persisted composer draft.
4. Issued **Cmd+Q directly from the authoritative review window** while the hook was
   proven running. The app exited with **code 0**, without clicking Cancel review.
   The journal became `cancelled`, `hook_passed: false`, `published: false`, issue
   `Inspection cancelled`. It reached cancellation before the natural 45-second
   sleep could complete. Independent `ps` found neither exact process afterwards.
5. Reopened the same binary and state successfully, demonstrating lock release.
   The exact saved draft appeared in the native composer. Operation results showed
   **Pre-push check · cancelled**, no publication and no retry/resume authority.
   The record and network log were identical to the post-Quit baseline; hook count
   remained one with no completion marker. Normal restart cleanup Cmd+Q exited 0.

The independent preservation oracle passed after Quit, restart and final cleanup:
every file byte/mode and symlink in all working/bare repositories matched its
original snapshot; one explicit trust, one completed observation, one cancelled
unpublished check, zero local write/fetch capture/import operations, and exactly
one read-only discovery GET. Hook side-effect markers were preserved outside the
repositories rather than rolled back. Final observations also extend beyond the
natural hook sleep interval. No hook/network/operation replay was observed.

## Validation and cleanup

[Evidence](../validation/g02-g06-running-push-check-quit-2026-11-07/README.md) records
factory, baseline, pre-Quit running proof, process cleanup, exact artifact provenance,
stage oracles and transcribed native observations.

This fixture-only step passed its actual native execution/Quit/restart checks,
independent file/journal/network/process assertions and `git diff --check`.
No automated matrix was rerun or recounted: the unchanged artifact retains the
preceding 69-renderer / 13-default-host / 16-all-feature-host full desktop validation,
and the preceding focused **9 Core push-check tests** remain dated evidence.
Neither is presented as a newly executed test run for this step.

Both app processes exited 0. Verified and stopped the exact owned fixture TLS
server and removed its generated private TLS key. No hook cleanup signal was
sent externally: the tested native shutdown path stopped the hook and child.
Ordinary executable paths, installed app and write defaults are unchanged.
No external publication or repository push occurred.

## Remaining gates

This accepts **custom Cmd+Q during a proven running supported pre-push check**,
bounded cancellation/process cleanup, durable unpublished result, lock release,
saved-draft preservation and restart without replay. It does not infer acceptance
of main-window Close, direct AppKit termination, Dock/system logout or host crash
while that hook runs. Earlier evidence for those entry points in other operation
contexts remains separate.

Next active-execution case shared with G06: **actual native host crash while a
pre-push check is running**, including explicit accounting for hook/child lifetimes,
interrupted history and no automatic replay. Native bypass/manager interaction and
remaining authentication/topology criteria stay G06. Actual Dock is policy blocked;
operator-controlled session termination remains G02. G07/G08 push publication and
uncertain server outcome recovery remain unimplemented and are not covered by a
hook check. Ordinary writes remain disabled.
