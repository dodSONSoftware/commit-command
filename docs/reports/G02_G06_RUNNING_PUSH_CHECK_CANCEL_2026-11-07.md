# G02/G06 native cancellation of a proven running pre-push check

**Selected running-hook cancellation case passed. G02 and G06 remain open.**
This extends the [November 3 pre-push-check checkpoint](M5C_REVIEWED_PUSH_CHECKS_2026-11-03.md),
which had automated running-hook cancellation but no corresponding native test.
No product code, permissions, feature defaults or publication executor changed.

## Fixture, authority and exact artifact

Created a fresh marked temporary fixture with three real nested repositories,
three bare remotes and a read-only loopback TLS Git HTTP service. It accepts only
upload-pack discovery/read endpoints and refuses publication endpoints. No external
or private server, Git authentication credentials, user repository, global Git/TLS
configuration, or user signing identity was used. An ephemeral fixture TLS CA was
passed explicitly through `--https-ca-file`; hostname/certificate verification
remained enabled. The generated server private key was removed after cleanup.

The leaf repository had one outgoing commit, different staged/working README
content and an untracked file. Its fixture-only pre-push hook writes its invocation
and PIDs outside the managed repositories, starts `sleep 45`, waits, then would
write a completion marker. The hook was installed before native trust approval.
All repository/remote hashes and modes were captured after setup.

Reused the exact acceptance binary validated in
[the active-sheet checkpoint](G02_ACTIVE_SHEET_TERMINATION_2026-11-07.md).
Binary hash and relevant source hashes were independently matched; product source
is unchanged at base commit `0000156`. The copied bundle used new fixture state.
Features were custom-protocol, desktop-writes and native-acceptance-support; no
special fault action or hidden IPC was used for hook cancellation.

## Actual native sequence

Native UI actions used Computer Use. AX observations were inspected and excerpts
are transcribed in the evidence record; no saved screenshots are claimed.

1. Independently reviewed and natively approved trust for **only the leaf**
   `parent/child/grand`. Parent/child worktrees remained untrusted.
2. Inspected its effective remote, then separately reviewed and natively approved
   observation of `refs/heads/main` on the loopback HTTPS grand.git endpoint. Exactly
   one GET discovery request completed with the expected baseline OID.
3. Selected only that worktree and fresh observation in the local planner. Its
   source preview showed one outgoing commit and zero child requirements. Kept
   Execute supported pre-push hook, prepared the exact no-publication review, then
   approved the independent native OS dialog. No hook marker existed beforehand.
4. **Proved active execution before cancellation:** independently read the real
   durable record as `running`, unpublished, with exactly one invocation marker,
   hook PID 87858 and sleeping child PID 87859; completion marker absent. This is
   actual execution, not a synthesized journal phase or precancelled operation.
5. Clicked **Cancel review** in the real native review window. The normal host
   cancellation path closed the review and cancelled its active token. The journal
   became `cancelled`, `published: false`, `hook_passed: false`, issue
   `Inspection cancelled`. Independent `ps` found neither exact process afterwards;
   no external kill or hook-release signal was used to obtain that result.
6. Native Operation results displayed **Pre-push check · cancelled**, no publication
   and no retry/resume authority. Ordinary Cmd+Q exited 0. Reopened the same binary/
   state and inspected the same history; no replay control or new operation appeared.
   Final Cmd+Q also exited 0.

The independent oracle verified exact full bytes/modes for all working repositories
and all bare remotes throughout, zero local write operations/fetch capture/imports,
one explicit trust grant, one completed observation, one cancelled unpublished
check, one hook invocation and no completion marker. After Quit/restart all records
and request-log bytes were unchanged; there was still just the one read-only GET.
That verifies no replay even beyond the hook's natural 45-second finish interval.
The hook's external invocation/PID markers are preserved effects, not rolled back.

## Validation and cleanup

[Evidence](../validation/g02-g06-running-push-check-2026-11-07/README.md) includes
fixture factory, baseline, running proof, process checks, stage oracles, exact
artifact provenance and raw Core test output.

Focused locked/offline Core `remote_observation` push_check suite: **9 passed**,
including actual Core running-hook cancellation, one-use authority, no replay,
decline/bypass/manager refusal, source/hook drift and child evidence cases. The
preceding full desktop gate (69 renderer, 13 default / 16 all-feature host tests
and static/Clippy/build checks) remains the validated unchanged-artifact evidence;
it was **not rerun or re-counted as a new full gate** for this fixture-only step.
`git diff --check` passed. Native observation/hook/preservation checks passed.

Both fixture app processes exited 0. Verified the loopback server's exact command/
root before stopping it, and removed only its generated private TLS key. No managed
repository was deleted. Ordinary executable paths/build defaults and installed
app bundle remain unchanged; ordinary writes remain disabled. No Git push occurred.

## Remaining acceptance queue

This accepts the **native Cancel review path during a proven running supported
pre-push check**, including process-group cleanup, durable unpublished cancellation
and restart without replay. It does not accept all interruption mechanisms:

- Native Quit/main-close/direct native termination during a running pre-push check.
- Actual native host crash during a running pre-push check, orphan cleanup and
  interrupted history/no replay. Earlier injected-journal history is not crash proof.
- Native explicit bypass and unsupported hook-manager interaction (automated
  coverage remains separate), plus remaining supported authentication/topology cases.
- Actual Dock/session termination retains its G02 restrictions.

Next shared G02/G06 case: Quit during the same kind of proven running pre-push
check, followed by process cleanup/lock release/restart assertions. Publication
and uncertain server outcome recovery remain unimplemented G07/G08 work; a
completed or cancelled pre-push check never grants push authority.
