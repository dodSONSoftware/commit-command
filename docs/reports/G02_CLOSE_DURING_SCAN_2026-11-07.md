# G02 main-window Close during active scan — November 7, 2026

**Selected native case passed within the disposable ordinary-build envelope.**
Owning criterion: M3 native plan section 12 step 1 (A16/A17). G02/G06 remain partial.
No product code or feature default changed; no remote execution was exercised.

## Fixture and artifact

Source `e9eeacd`, ordinary default-feature release executable. Exact artifact hash,
fixture paths and initial worktree/workspace IDs are in `fixture.json` in the linked
evidence directory. A fresh disposable workspace contained 80 repositories copied
from a newly created local fixture (same commit content, distinct filesystem bindings).
Each had staged, unstaged and untracked content. No remotes, hooks, credentials or
trust grants were configured. Baseline records every repository file's SHA256/mode.
Initial CLI inventory contained one repository; subsequent native scans discovered
all 80. This workload supports interruption timing, not a performance/scale pass.

## Native actions and observations

1. Launched the exact ordinary fixture artifact and entered the draft
   `G02 Close during active scan — preserve this draft` through the composer.
2. Returned to overview and requested Rescan. The first independent process probe
   captured no live child; no interruption pass is credited to that attempt.
3. Started continuous read-only process sampling of the unique fixture host PID
   36642 before repeating the UI action. Native Rescan showed disabled controls,
   Core request in progress and enabled Cancel. Invoked native Cmd+W to close the
   **main window**, not application Quit. The window displayed Closing safely.
4. Sampling recorded 219 live Git-child observations with that exact parent PID,
   followed by host absence. Commands were reported by macOS as `(git)`, not full
   argv. This establishes live local Git work during the active scan interval;
   it does not identify an individual internal scan subphase or exclude overlapping
   periodic local inspection. Process sampling sent no signals.
5. Host exited normally with code 0. First restart of the same artifact/state
   rendered the 80-repository workspace without manual reload. Opening the retained
   repository tab visibly recovered the exact draft. Final native Cmd+Q exited 0.
6. CLI reopened the state writer lock and read all 80 inventory entries.

## Independent assertions and limits

Read-only oracles after Close and after restart/final cleanup confirmed exact file
bytes and modes across all 80 repositories; original HEAD/ref/index/reflog/config
and dirty content are therefore preserved. No managed index/other file lock appeared
in the baseline comparison. Draft persisted. Local write-operation and trust tables
remain empty; no observation, download/import, upstream or push-check state records
were created. No approval, commit, hook or network operation was requested.

The scan cache may retain a previous completed discovery; that is not an interrupted
write requiring recovery and is not a claim that the last request fully completed.
Both native exits and lock reopen passed. Startup evidence is bounded to this restart,
not universal proof against intermittent WebKit presentation failures. Actual Dock,
operator session termination, other phase-specific recovery and remote compatibility
remain open; no G06 remote/check criterion is newly closed.

No product source changed, so automated suites were not rerun. The current source's
existing full desktop/Core validation remains documented in the preceding
[cache/scan-status report](DISCOVERY_CACHE_SCAN_STATUS_2026-11-07.md).

[Evidence](../validation/g02-close-during-scan-2026-11-07/README.md).
