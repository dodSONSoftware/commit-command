# G06 native executable remote-helper refusal — November 7, 2026

**Selected native unsupported-transport refusal case passed after explicit fixture
trust authorization.** G06 remains partial; executable remote helpers remain unsupported.
No product patch or ordinary write-default change was needed.

## Fixture and source

Fresh disposable repository with an `ext::` origin pointing to a local executable
sentinel. It would append `helper-invocations` and exit 97 if invoked. Staged,
unstaged and untracked content are covered by exact file SHA256/mode baseline.
No real server, hooks, credential or normal user repository was configured for this
case. Existing global inspection warnings were displayed without authorizing or
executing their transport settings. No global configuration was changed.

Non-default `desktop-writes` artifact, product source unchanged from `e9eeacd`.
Preparation checkout was `484d289`; authorization-block evidence was committed at
`6932be5`. Exact acceptance and ordinary-restart hashes/paths are in `fixture.json`.
Acceptance feature build succeeded. Previous debug executable restored after copying;
ordinary release and normal user app bundle were not changed by this test.

## Authorization history

The first trust-approval click was rejected by automatic approval review because
general acceptance-testing authorization did not specifically authorize the access
grant. No bypass, seeded trust or direct Core substitute was used. The user then
explicitly authorized trust **only** for
`/private/tmp/gitfleet-g06-helper-di5z10fo/workspace/repository`, including the queued
question reply. On resumption, the native authoritative trust review already displayed
its successful result. The agent did not repeat that grant. Main UI showed granted /
current; the independent oracle confirmed exactly one unrevoked trust record for the
authorized worktree/path. The observation/refusal actions below occurred after the
explicit authorization. The prior block is resolved, not a current acceptance block.

`before-trust.json` retains the earlier zero-trust/no-invocation preparation evidence.
It is historical; its pending-approval field does not override this completed result.

## Actual native result

1. Closed the completed trust review and expanded Remotes and upstream. Local
   **Inspect remote destinations** showed origin's fetch/push destinations as
   `remote_helper`, withheld their raw executable destination, and displayed
   **Arbitrary remote helpers are not authorized**.
2. Explicitly chose origin, entered `refs/heads/main`, and clicked **Review remote
   observation**. Core refused with:
   `unsupported_configuration: Observation requires one supported destination without embedded secrets and explicit host transport policy`.
   No transport approval/review opened; controls returned to usable state. This is
   a refusal even after explicit current worktree trust, not merely an untrusted
   repository or ordinary host-gate rejection.
3. The read-only oracle confirmed exact repository file bytes/modes, intact sentinel
   script/mode, no invocation marker, SQLite integrity `ok`, precisely the one scoped
   trust record, zero local write-operation rows and zero observation/download/import/
   upstream/push-check state records.
4. Acceptance app Quit exited 0. First restart used a separate ordinary default-feature
   artifact, rendered without reload and showed write controls disabled. Ordinary Quit
   exited 0. CLI subsequently reopened the state writer lock and read one entry.
5. Final oracle exactly matched the post-refusal result. No helper execution, mutation
   or operation replay appeared across restart. Both fixture apps are closed.

## Scope and limits

This accepts the selected native `ext::` executable-helper refusal path, not support
for arbitrary helpers, other unsupported protocols, authenticated network success,
SCP/SSH facility availability, upstream topology or publication. The request was
rejected before a transport operation/journal was created; it is not a test of killing
an already running helper. Sentinel absence plus unchanged files/journals support the
bounded no-execution result; no generalized host-wide network/process audit is claimed.
The scoped trust row remains only in disposable fixture state as recorded evidence.

No product source changed and no full automated suite was needlessly rerun. Existing
Core/desktop full validation remains in the preceding cache/scan-status report; this
checkpoint adds actual native interaction, the successful acceptance build and executed
independent oracles. G02/G06 retain all other open/blocked dispositions.

[Evidence](../validation/g06-native-remote-helper-refusal-2026-11-07/README.md).
