# G02 actual native host loss during active scan — November 7, 2026

**Selected forced read-interruption/restart case passed within the ordinary-build
fixture envelope.** This complements the preceding graceful main-window Close case.
G02 remains partial; G06 remote compatibility is unchanged. No product patch was needed.

## Artifact, fixture and interruption proof

Product source is the validated `e9eeacd` implementation; checkout checkpoint
`00547b2` adds only preceding acceptance documentation. Used the exact ordinary
(default-feature) release executable, identified by SHA256 in `fixture.json`.
No acceptance feature, crash hook, write default or approval rule was enabled.

Fresh disposable state initially inventoried one local dirty repository. Another
79 copies were created before launch, giving 80 distinct filesystem identities with
identical commit content. Each contains staged, unstaged and untracked content.
The independent baseline records all managed file bytes/modes. No remote, hook,
credential or trust grant was configured. This is not a scale/performance benchmark.

Native CUA entered the exact draft `G02 host loss during scan — saved draft`,
returned to overview and requested Rescan. UI showed Core request in progress,
disabled Rescan and enabled Cancel. A read-only database assertion verified the draft
had already persisted before arming interruption; unsaved keystroke recovery is not
claimed. A guarded external sampler verified the unique fixture host command/PID,
waited for explicit arming after the native busy observation, and observed a running
Git child PID 69962 of host 66331. It then sent SIGKILL **only to that host**.
Proof and signal timestamps are saved. macOS reported the child command as `(git)`;
its exact internal scan subphase is not identified, and overlapping periodic local
inspection is not excluded. No child, unrelated app or broad process group was signalled.
The host returned -9; its Python launcher exited normally after reporting that signal.

## Recovery and assertions

Immediately after the crash, SQLite integrity was `ok`, and the saved membership and
last completed scan both contained exactly **one** repository. The interrupted scan
did not expose a partially published inventory. All 80 repositories' file bytes/modes
matched baseline, and the exact saved draft was retained. Zero local write operations
and trust grants were present.

First restart with the same binary/state rendered without manual reload and showed
the retained one-repository inventory. Opening the retained tab visibly recovered
the exact draft. Explicit native Rescan then completed with **80 repositories and zero
scan issues**. Final Cmd+Q exited 0; the CLI subsequently acquired the state writer
lock and read all 80 entries. Independent final oracles again confirmed SQLite
integrity, coherent scan/membership counts, exact repository file preservation, draft
preservation and zero trust/write/remote-operation records. The original host and
sampled child were independently confirmed absent; no cleanup signal was needed.

Ordinary monitoring may perform new local reads; this is not execution replay.
A stale completed scan remains a last-recorded observation, not proof that current
discovery has finished. No Git mutation or recoverable publication was started, so
no guarded write-recovery approval was applicable. No read-only restart repair of
managed Git content occurred. This case does not establish interruption at a specific
SQLite commit boundary, power-loss durability or orphan cleanup for arbitrary hooks.

## Validation limits / disposition

Native steps, exact process proof, SQLite checks and reusable read-only oracles all
passed. No product source changed and no automated suite was needlessly rerun;
previous full desktop/Core results remain in the cache/scan-status report. Both startup
and forced interruption acceptance are bounded to this fixture/build/run. Actual
Dock, operator-session termination, other phase-specific recovery, broader startup
reliability and remote/auth/topology criteria keep their existing open dispositions.
The fixture is closed; the user's normal app was not terminated or reconfigured.

[Evidence](../validation/g02-host-loss-during-scan-2026-11-07/README.md).
