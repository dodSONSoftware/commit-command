# G03 scale measurement and partial inventory fix — November 8, 2026

## Diagnosis and behavior

Repeatable Core CLI first-pass/warm scans over 100, 1,000 and 10,000 empty independent
repositories all hit the default 15-second traversal deadline. Before the fix all
six returned Timeout with no inventory JSON, discarding the inspected entries.
The new expired-scan regression failed against that behavior. Discovery now retains
inspected entries, records a timeout issue/path and returns complete=false. It does
not start relationship classification after the exhausted traversal deadline. Explicit
cancellation remains an error and does not replace saved inventory. Workspace partial
scan handling retains known unvisited identities as unavailable rather than missing.
No safety deadline was removed and incomplete topology is not presented as complete.

## Validation

All six repeated scans now return explicit partial JSON, retaining 88–93 inspected
repositories in 15.04–15.27 seconds, rather than none. This is a visibility/correctness
fix, not a full scale pass: incremental scanning, actual large-workspace usability,
relationship/history paging, watcher/IPC/memory and interaction budgets remain open.
The first pass follows fixture creation and is not an OS-cache-cold measurement.
Fixtures use empty repositories and a debug CLI, so results are not release capacity
claims. Benchmarks remove only their new disposable marked roots after execution.

Workspace suite 20 passed (new deadline/availability and cancellation regressions);
application suite 27 passed; strict all-feature Core Clippy passed. Full desktop gate
passed: 87 renderer, 13 default-host, 16 all-feature host tests, generated contracts,
format/type/lint, frontend/native builds, strict Clippy and ordinary release build.

Fresh ordinary development bundle on a separate 100-repository fixture displayed
Discovery incomplete / 1 scan issue and the actual timeout path alongside 93 retained
repositories and zero repository-attention count. Live native Rescan showed running
controls then persisted a fresh partial scan; exact repository file bytes/modes stayed
unchanged, SQLite integrity ok, no trust grants. Native Quit exit 0 and writer lock
reopened. Narrow-window drag did not establish changed geometry and is not accepted.
Old temporary app copy was unavailable; new ad-hoc fixture bundle assembled from the
fresh release executable with unchanged bundle identifier. No distribution claim.

Initial sandbox benchmark process failed before JSON; the runner was corrected to
retain nonzero-exit diagnostics and the permitted host run captured the actual Core
Timeout. Historical failed attempt retained. Full signing/transport matrix was not
rerun for this inspection-only patch; the owning workspace/application and full
desktop checks above were run. G03 and G02 remain partial; G04 stays disabled.

[Evidence and benchmark results](../validation/g03-scale-2026-11-08/).
