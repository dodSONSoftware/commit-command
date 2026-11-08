# G05 three-level selected submodules — November 8, 2026

Disposition: Core integration passed; native three-level execution open. This is a
new test of M5A multi-level dependencies / BULK-004–007, not a product behavior change.
The previous native hook chain proved two-level execution only.

Added three_level_submodules_require_fresh_review_at_each_changed_parent to
app/core/tests/bulk_application.rs. The fixture uses real Git submodules nested
root → middle → leaf, all on local main branches with isolated identity, no hooks,
no signer and no network. Selection is supplied root-first deliberately. Preparation
must preserve the full repository snapshot and produce the exact leaf/middle/root
order, with only direct selected dependency edges.

First separate Core approval publishes only the leaf; middle/root remain original
and the batch needs reauthorization. Fresh middle/root review includes the new leaf
gitlink, publishes only middle and again stops for root reauthorization. Fresh root
review includes the exact new middle gitlink and completes. Independent Git checks
prove one new commit per worktree, exact content and final committed child OIDs.
Later ancestor execution preserves descendant checkouts AND their actual Git-dir
bytes/modes, which reside under root .git/modules rather than in child checkouts.

Validation: focused test 1 passed; final full bulk_application suite 13 passed,
0 failed, 1 ignored (the existing subprocess worker, invoked by interruption test).
Cargo format check and strict application-services bulk-test-target Clippy passed.
No full five-feature signing/transport matrix rerun: only this integration test
changed, no product source, contracts or binaries changed. Logs are retained in
[validation](../validation/g05-three-level-2026-11-08/).

Initial command omitted the required application-services feature and did not run.
The first enabled fixture reached safe refusal because recursive submodule init
left leaf detached. Fixture setup now explicitly checks out local main before
trust/review; production branch restrictions were preserved. These preflight logs
are retained separately from successful checks. An initial needless_range_loop
lint was corrected using iterator zip; final checks passed. Native approval and
execution for this topology are not inferred from the Core harness.

Next distinct cases: native three-level review/execution; malformed/cyclic graph
refusal coverage; remaining selection/revocation and accessibility acceptance.
G05 remains partial. Ordinary write defaults and all broader gates remain unchanged.
