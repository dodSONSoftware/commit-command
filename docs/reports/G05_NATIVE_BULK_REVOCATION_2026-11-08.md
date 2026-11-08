# G05 native bulk trust revocation — November 8, 2026

Disposition: selected pending-batch revocation, stale confirmation and known-untrusted
whole-batch refusal passed. Broader G05 remains partial. No product source change.
Used existing authorized parent/middle/leaf fixture and instrumented build from
64b348c; no new trust/hooks/signing or push. Only parent trust intentionally revoked.

## Setup and separate review-window finding

With app stopped, saved original parent/middle file.txt and deliberately changed
working content to G05 revoked batch must not publish. Explicit setup snapshot;
previous operation/batch records retained. Selected only middle and parent.
First preparation a58cd244-116d-46a6-b377-10404e40d52b disappeared before any trust
change or approval. Native trace shows fresh review 1d973616-6e8a-40a6-accc-ff9b61605ae5
built, then WindowEvent::Destroyed, then successful cancellation of that same review.
Both proposals cancelled_before_publication. This reproduces the disappearing-window
symptom with a trace; it is NOT the intentional revocation pass. The trigger for
window destruction is unknown, so no speculative fix and no lifecycle closure.

## Pending review revoked in the main window

Fresh retry 41af4a3c-5178-4b1a-9cb5-c3c3e24e5459 remained open, 2 selected / 0 blocked,
with middle then parent exact working-file proposals. No confirmation requested.
Used native Window menu to activate main window, opened parent tab, and clicked
Revoke trust. Before action, records prove this batch and both operations proposed.
After action, only parent trust revoked; middle/leaf remained trusted, control absent.
Both operations 80df7f79-905d-44ce-a412-a7cd4b64db8d and
ab60a1d0-878d-443d-bd50-1a5170bd59aa cancelled_before_publication; batch cancelled.

Returned to original authoritative review via native Window menu. Clicking Continue
to native confirmation returned Not completed: Application context or review changed;
no OS confirmation sheet, no execution. Closed stale review and prepared a fresh
batch with both explicit selections unchanged.

## Fresh blocked batch and restart

Batch 3b57e486-9ce8-4bcb-b871-60a862fac981 displayed 2 selected / 1 blocked with
No operations execute while a selected item is blocked. Parent item had
untrusted_repository and no operation; middle had proposed operation
1c79614e-4f44-4ad9-aff0-c8ad22b5cc83. Separate OS native approval returned blocked,
cancelling the trusted middle before execution, no publication/index reconciliation.
This proves the untrusted parent prevents even its trusted selected child from running.

verify.py refusal passed exact setup repository bytes/modes, five new operations
all cancelled_before_publication without new OID/publication/reconciliation, exact
historical payloads, only parent revoked and integrity ok. Native Quit exited 0.
Restored ONLY deliberately changed working files; full repository again matches
pre-test diagnostic final snapshot, no Git metadata or other contents changed.
Records retained, parent remains revoked; no regrant requested or performed.

Ordinary restart displayed clean parent and revoked · revoked on inspection; read-only
status, blocked/cancelled batch histories and disabled native bulk preparation.
Ordinary Quit exit 0. Final exact file/mode and payload oracle passed with no replay;
writer-lock reopen passed. No full matrix rerun for native-only evidence; latest
full desktop validation remains 91/13/16, and latest bulk suite 14 passed/1 worker
ignored from the immediately preceding test-only package.

[Evidence](../validation/g05-native-bulk-revocation-2026-11-08/) includes original/setup
files, before/prepared/revoked/blocked/final raw records, derived plans, native trace,
exit codes, restored scan and exact verifiers. Summary preserves blocked items even
when they have no operation; raw records are authoritative. Earlier fixture current
oracles expecting three active grants are now historical: parent revocation is
intentional and must not be bypassed or silently undone. Original window-destruction
trigger, remaining selection/accessibility and other gates remain open.
