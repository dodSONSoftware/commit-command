# G02/G05 completed-batch review transition — November 8, 2026

Disposition: instrumented diagnostic repeat passed; original intermittent
self-cancellation remains open. This is diagnosis, not a claimed runtime fix.

## Change and validation

Added native-acceptance-support-only trace points for request start/finish (kind,
job ID, result success/epoch), review open/build/old-window wait/destruction,
queued ShowReview and processed CancelReview (review ID, cancellation result).
No request payload, repository content, credential or command arguments logged.
No IPC, access grant, default feature or approval policy changed. Ordinary builds
compile out trace points. Cancellation still calls Core once with the same result.

Full desktop validate.sh passed: 91 renderer tests, 13 default host tests, 16
all-feature host tests, TypeScript, ESLint, generated contracts, formatting,
frontend/default/debug/release builds and strict default/all-feature Clippy.
Instrumented acceptance build passed separately. No Core source change or five-
configuration signing/transport rerun required. Binary/source hashes in fixture.json.

## Actual native transition

Reused already authorized parent/middle/leaf under
/private/tmp/gitfleet-m3-native-g05-three-level-xadtjgtr. No new trust, hooks,
config/signing changes or push. Deliberately changed parent/middle file.txt while
app stopped; recorded a fresh explicit setup snapshot. Earlier three-level current
oracles are historical after this intentional setup; their results/records preserved.

Selected middle and parent, reviewed exact new working content with no hooks/signer,
message G02 review transition diagnostic round 1. Separate native approval for batch
b1dd9019-742b-450c-84ca-18b959d6f372 published middle
949253cc316c4811271f4eb272f1196a8027d10e then stopped needs_reauthorization;
parent unstarted. Closed completed review, selected unfinished parent and prepared
fresh review. Batch b973671c-2438-49c3-8203-18a9e5f7235a contained exact middle
215c2e1b5e47bbb237b1ee48ff53afd2f7c7ff9b → 949253cc316c4811271f4eb272f1196a8027d10e
gitlink and file.txt. Separate native approval published parent
8df29fca0127da252cf1fb2c3923670b3e37d715, complete/index reconciled.

Trace shows two review windows built once each. Old consumed review CancelReview
calls were rejected; queued ShowReview for consumed review did not rebuild it.
Fresh review stayed open and completed; no successful unexpected cancellation.
This successful trace does not explain the original uninstrumented cancellation.

## Preservation and rebuilt ordinary artifact

verify.py checks one additional middle/root commit from independently verified
previous heads, zero leaf commits, clean chain and exact content/messages/gitlinks.
All working bytes/modes preserved; entire unselected control and leaf actual Git
metadata preserved. Exact earlier operation/batch payloads, three trusts and integrity
ok. Acceptance exit 0. Rebuilt ordinary release artifact visibly read-only with
native bulk preparation disabled, complete and needs_reauthorization results
retained; exit 0. Exact final repository hashes/modes and all payloads survived
restart, no replay; writer lock reopened. The oracle checks it does not modify files.

[Evidence](../validation/g02-review-transition-2026-11-08/) retains checks, source/
binary provenance, setup/final snapshots, exact records/plans, native trace, exit
codes and reproducible inspection/verification. Continue to a distinct uncovered
criterion; retain this finding for targeted reproduction if it recurs. No G02/G05
closure, ordinary write enablement or release acceptance inferred.
