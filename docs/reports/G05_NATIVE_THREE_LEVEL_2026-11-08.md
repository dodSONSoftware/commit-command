# G05 native three-level submodule preparation — November 8, 2026

Disposition at preparation: awaiting exact path-specific trust; superseded by the
authorized native checkpoints below. Selected chain execution/restart now passed;
review-window self-cancellation finding remains open.
Core counterpart passed in G05_THREE_LEVEL_2026-11-08.md. This native case targets
actual leaf-first review/publication, a fresh middle gitlink approval, a fresh root
gitlink approval and ordinary restart with no replay. Distinct from two-level case.

Exact selected roots under /private/tmp/gitfleet-m3-native-g05-three-level-xadtjgtr:

- workspace/parent
- workspace/parent/middle
- workspace/parent/middle/leaf

workspace/unselected is independent, unselected and untrusted. Four actual Git
repositories discovered, two actual submodule edges. Each selected worktree is on
main, signing disabled, no applicable hooks. File changes are disposable content.
Application state stays outside workspace, in this fixture's state directory.
Existing test binaries match fully validated product source 4c52595; subsequent
source changes are documentation/tests only. No new build or permission bypass.

Actual native Overview explicitly selected only the three roots. Review trust for
selected opened authoritative Rust review listing exactly those paths. Stopped
before Continue to native confirmation: zero trust rows, zero write operations,
all repository file hashes/modes exactly match prepared baseline. User authorization
for the old hook-enabled pair does not cover these paths. No access was granted.
If the review expires before authorization, prepare the same exact review afresh.

Initial fixture lacked the native support marker/layout and was refused at startup
(exit 1) before UI. Its records are retained under initial-invalid-layout. Corrected
preparation creates the required marked disposable root; no product guard changed.
The corrected app is open only at the pending trust review, owned subprocess session
69518 (PID 98575). Do not query a stopped app through CUA, which can launch default
state. Reuse the exact fixture.json app/state paths. No commit or publication yet.

[Evidence](../validation/g05-native-three-level-2026-11-08/) retains reproducible
preparation, fixture, exact initial repository snapshot, scan and preflight. This
preparation is reviewable and validated; the pending security-sensitive trust grant
and subsequent separately approved local commits are not recorded as passes.


## Authorized native leaf publication — passed

User granted exact root/middle/leaf trust scope. Authoritative Rust review and
independent OS confirmation granted those three only; control remains untrusted.
Native bulk shared message is `G05 native three-level reviewed commit` followed by
`Require fresh approval at each changed parent.` as a separate paragraph.

Batch cc497a22-f182-484a-8568-4e87c2de89c8 prepared exact leaf/middle/root order;
each ancestor has only its direct selected-child dependency. All three proposals
initially select only working file.txt; no future gitlink silently approved.
Preparation preserved every repository byte/mode. Separate native batch approval
published leaf operation 85f67967-c1ff-4fef-8e36-3502580ce513 as
4b6e1ea1dbeabd9350368a24ad83206366dc7608, then needs_reauthorization.
Middle operation f85efb29-aec9-439b-8be2-99077b6c3a7b and root operation
b499c6fb-c8af-42be-bf03-a509c907e5e3 cancelled_before_publication, neither started.

verify.py leaf passed: exactly one leaf commit, exact message/content, clean leaf;
every file outside leaf's actual Git metadata remains original, proving root/middle
heads/index/config unchanged, all working bytes/modes unchanged, full unselected
control unchanged, exactly three trusts and SQLite integrity ok. Native durable
results visibly match this stop. Fresh middle and root approvals remain pending;
this section does not accept the complete chain or ordinary restart.


## Fresh middle publication and second ancestor barrier — passed

Select unfinished worktrees excluded completed leaf and selected middle/root.
First preparation 258bbae2-f235-4b80-88ac-0085901dd931 unexpectedly became cancelled
before any approval; both proposals cancelled_before_publication, no writes. Full
repository exactly matched leaf-files.json after both preparations. This is retained
as a review-window lifecycle finding, not claimed as an intentional Cancel pass.

Second fresh preparation f1f3975f-97e9-44bd-9590-62446ba0444f opened normally,
reviewing middle file.txt plus exact leaf gitlink e9bfc641cb50a16e8914d1fe4b6b8be519560c37
→ 4b6e1ea1dbeabd9350368a24ad83206366dc7608. Separate native OS approval published
middle operation ab6c12b3-effd-4056-8ae0-d026bb4ef994 as
215c2e1b5e47bbb237b1ee48ff53afd2f7c7ff9b. Root operation
4a33f20e-d7a8-4af9-80de-6d8f50261f1c cancelled_before_publication; batch again
needs_reauthorization. Native result and verify.py middle passed: exactly one commit
in leaf/middle, original root HEAD/index/config, exact file content/message and
committed leaf OID, unchanged leaf checkout AND actual Git metadata, working files
and full control preserved, prior leaf records unchanged, exact trusts/integrity.
Root-only fresh approval and final ordinary restart remain next. No product change.


## Fresh root approval and ordinary restart — passed

Select unfinished from the middle batch selected only root. Fresh native review
b94b213e-9087-4b2e-b776-d62dc734e6e8 included file.txt and the exact middle gitlink
2dbf7c743f1cf3e9b9120683d40dfedbb9301eed →
215c2e1b5e47bbb237b1ee48ff53afd2f7c7ff9b. Separate OS approval published operation
134973e2-b8d7-4a5d-8311-9c5427dad4a8 as
98712d632e0130ce37720ede6e31fc84981244b6, complete/index reconciled.

verify.py parent passed: exactly one new commit in each worktree, exact shared
message/file content, exact committed child OIDs, clean chain. All descendant
checkouts and actual Git metadata preserved during ancestor execution, original
working-file bytes/modes and full unselected control preserved, prior operation/
batch payloads unchanged, three authorized trusts only, SQLite integrity ok.

Acceptance native Quit exited 0. Ordinary restart showed all three chain repos clean,
only root selected and dirty control unselected; read-only status. Bulk retained
complete root, two needs_reauthorization histories and the unexpected cancelled
preparation; Prepare native bulk review disabled. Ordinary Quit exited 0. verify.py
restart passed exact repository files/modes and all operation/batch payloads against
pre-restart snapshot, with no replay; writer-lock reopen passed. Logs/exit codes and
oracles retained in the evidence directory. No product change/full matrix rerun for
this native-only step. Current-state earlier stage oracles are historical after
later intentional approvals; final verifier owns the final state.

This accepts the selected native three-level publication/reauthorization/restart
chain. It does not accept all G05 or dismiss the unexpected first middle-review
cancellation. Next diagnose that lifecycle finding with focused reproduction before
changing code; preserve the safely cancelled record as evidence.


## Bounded review-window diagnosis — original finding still open

After the completed chain, reopened the same isolated acceptance app and prepared
five root-only trust reviews, cancelling each without native confirmation. Four
close/immediate-fresh-review transitions all displayed the new authoritative review
with enabled confirmation and No operation is approved yet. No trust grant or commit
was requested; exact final repository files/modes and operation/batch payloads still
matched parent-files.json / parent-records.json. Native Quit exited 0.

review-repeat-proof.json records all five observed review IDs and the narrower
coverage. This did not reproduce the original completed-batch-close → fresh-bulk
preparation cancellation. Code inspection found review-ID guards on queued
cancellation, asynchronous window teardown/reopen handling and Core cancellation
journaling; inspection alone establishes no root cause. No speculative patch.
Keep the original safely cancelled batch as an open lifecycle finding and use a
focused completed-batch transition reproduction/instrumentation if it recurs. These
pending-trust repetitions do not establish that original transition passed.
