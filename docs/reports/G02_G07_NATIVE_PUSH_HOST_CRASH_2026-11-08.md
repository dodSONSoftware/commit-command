# Native host loss during actual push publication

November 8, 2026. Selected G02/G07 acceptance passed on source `984a89b`, with
unchanged production code and the exact hash-recorded acceptance/ordinary artifacts
from the [individual publication checkpoint](G07_EXACT_PUBLICATION_2026-11-08.md).
This tests actual publication, distinct from earlier pre-push hook-check interruptions.
G02/G07 remain partial; this is not a release or ordinary-write enablement decision.

Only the previously authorized disposable leaf worktree was used. No new trust was
granted. Parent stayed revoked; middle and the independent control were untouched.
For setup, only the disposable bare server ref was reset to its known old commit.
A fixed, hash-recorded server pre-receive hook checked exactly the old/new/ref tuple,
recorded a running marker, and waited on a local release marker. It performed no
network operation. The original server checkout and earlier artifacts remain saved.

## Proven interruption and remote outcome

A new separately approved native observation found old destination
`e9bfc641cb50a16e8914d1fe4b6b8be519560c37`. Rust review and independent macOS approval
bound leaf source `4b6e1ea1dbeabd9350368a24ad83206366dc7608` and that exact old tip.
Publication `d7132f9a-d376-4cf6-967f-c2d3c3321fd0` entered durable `publishing` with
`may_have_published=true`, while the actual server check was proven running and the
server ref still old. PID/start-time/exact executable/state checks bound SIGKILL to
only the owned fixture host. Its observed exit was 137.

After observing host exit, the fixed server check was released. The remote then
advanced to the approved source while the application was absent. The journal still
said `publishing`. No sampled owned child processes remained after release. This is
concrete evidence that host loss does not imply push failure or permit a blind retry.

## Actual restart and guarded reconciliation

The first acceptance restart rendered real workspace content and one recovery notice.
Operation results projected `unconfirmed`, explained that publication may have
occurred and offered observation-based recovery; no push or hook was replayed.

The pre-attempt observation was deliberately selected for recovery. Native UI refused
with `stale_review` before opening a recovery review/native approval, while that
observation was still only 221,317 ms old. Its age was inside five minutes and the
source/configuration were unchanged, so the new post-attempt freshness barrier was
exercised rather than hidden by an unrelated generation or expiry failure.

A separately approved fresh observation then found the approved source. A separate
Rust recovery review and independent native approval recorded only that evidence,
explicitly authorizing no push, hook, ref change or retry. The durable result is
`reconciled`, `remote_state=approved_source_present`, `may_have_published=true`, and
`published=false`. The last field deliberately does not invent retrospective causal
proof from a server observation. It does not deny the independently observed server
update in this controlled experiment.

The recovery notice cleared. Native Quit exited 0. Ordinary restart retained the
exact reconciled record, displayed read-only status, and offered no default writes.
Ordinary Quit exited 0; storage lock reopened and SQLite integrity was `ok`.

## Preservation and limits

Every managed repository file and mode matched the pre-case baseline, including
the dirty leaf file and actual submodule metadata. All prior operation/batch/trust,
publication/check/observation records remained byte-exact. Exactly one new publication
and one new check were present; neither restart added/replayed one. The server tip
matched the approved source; all server files/modes matched the post-recovery
baseline across ordinary restart. The owned server hook was hash-verified and removed.
Source/artifact hashes matched the preceding validated checkpoint, so the full Core
and desktop suites were not repeated for this evidence-only step.

This fixture now extends the older G07 fixture history from two publication records
to three; older dated oracles remain historical baselines, not current fixture checks.
Broader authenticated/native/topology/accessibility and remaining lifecycle criteria
retain their tracker disposition. This one passing first restart does not erase
older intermittent blank/review-window counterevidence.

[Evidence and reproducible guards](../validation/g02-g07-native-push-host-crash-2026-11-08/)
include the exact running journal, host/child identity, signal/exit, post-loss server
state, rejected pre-attempt observation, fresh observation, reconciled value, final
preservation proof, native trace and lock reopen. All repositories and process
interruption targets were disposable fixture resources; no user remote was contacted.
