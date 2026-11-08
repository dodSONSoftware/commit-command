# G05 native three-level submodule preparation — November 8, 2026

Disposition: prepared; awaiting exact path-specific trust. No execution acceptance.
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
