# G02 Cancel during active workspace scan — November 7, 2026

**Selected native read-cancellation case passed within the ordinary-build fixture
envelope.** G02 remains partial; G06 remote compatibility is unchanged. No product
fix, permission change or default write enablement was needed.

## Fixture, proof and native behavior

Used the current validated ordinary default-feature release executable (product code
`e9eeacd`; checkout `2cc62bd` includes subsequent evidence-only commits). Exact SHA256,
source and fixture IDs/paths are in `fixture.json`. Fresh disposable state initially
inventoried one local dirty repository; 79 additional copies were then created with
distinct filesystem identities. Staged, unstaged and untracked content is covered by
the file SHA256/mode baseline. No remotes, hooks, credentials or trust were configured.
This workload creates an interruption opportunity; it is not a scale benchmark.

Native CUA entered `G02 Cancel active scan — retain this draft`, returned to overview,
and requested Rescan. The UI showed disabled Rescan, Core request in progress and
enabled Cancel. A read-only sampler started before the request captured **14 live Git
child observations** belonging to exact fixture host PID 87775. macOS reported `(git)`
rather than full argv; exact internal read subphase and overlap with periodic local
inspection are not established. No process was signalled or artificially paused.

Clicked **Cancel current operation** using its freshly observed enabled control.
The app returned `Error: cancelled: Inspection cancelled`, restored controls and
kept the window/host available. A read-only SQLite assertion immediately afterward
confirmed that the previous **one-repository** membership and completed scan cache
were intact; no partially discovered 80-repository inventory was published. All
managed file bytes/modes and the persisted draft matched their expected values.

Dismissed the cancellation message and explicitly requested a new Rescan. It completed
with **80 repositories and zero scan issues**, proving cancellation did not remain
latched for the next request. Opening the original tab visibly retained the exact
draft. Final Cmd+Q exited 0; the CLI then acquired the writer lock and read 80 entries.

## Independent validation and limits

Final read-only oracles passed after retry and after lock reopen: exact file bytes
and modes across all 80 repositories; SQLite integrity `ok`; matching completed scan
and membership count of 80; exact saved draft; zero write-operation/trust rows and
zero remote observation/download/import/upstream/push-check records. No Git mutation,
publication or guarded write recovery was involved. No restart is newly claimed in
this checkpoint; prior Close/crash restart evidence remains separate.

Known inventory and last-completed discovery can remain visible after Cancel; that
is retained observation, not a claim that the cancelled scan completed. No blanket
rollback or universal timing guarantee is inferred. Ordinary periodic inspection may
continue reading locally; no write/hook/network replay was requested or observed.

No product source changed, so no automated suite was rerun. The existing full gate
results remain in the cache/scan-status report. Actual Dock/session termination,
broader startup and phase-specific recovery, accessibility/scale, and G06 remote
compatibility remain open. The fixture is closed; the normal user app was untouched.

[Evidence](../validation/g02-cancel-active-scan-2026-11-07/README.md).
