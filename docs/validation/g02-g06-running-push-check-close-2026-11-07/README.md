# Native main-window Close during running pre-push check — November 7, 2026

Disposition and transcribed native observations: [acceptance report](../../reports/G02_G06_RUNNING_PUSH_CHECK_CLOSE_2026-11-07.md).
Native `Window > Close Window` (`performClose:`) targeted the foreground main window during proven execution. No product source or approval policy changed; G02/G06 remain open for other criteria.

- `native-fixture.py`, `native-fixture.json`, `fixture-preparation.txt`, `native-baseline.json`: fresh marked fixture factory, exact IDs, three nested worktrees/three bare remotes and after-setup file/mode/symlink baselines. Factory runs from repository root with built Core CLI and creates a new fixture each time. Nested snapshots overlap. No generated private key retained.
- `artifact-provenance.json`: reused exact startup-readiness artifact/source hashes and unchanged app tree; prior full desktop validation/build links are historical, not new runs.
- `initial-host-process.json`, `prove-running.py`, `native-running-proof.json`: exact host identity, raw running/unpublished check, one invocation, saved draft and live host/hook/child process table immediately before native Close.
- `native-process-cleanup.json`: normal host exit 0 and all three exact PIDs absent 33.6 seconds after hook start, before natural completion; no external signal.
- `native-oracle.py`, `native-closed.json`, `native-restarted.json`, `native-final.json`: read-only exact repository/remote preservation, cancelled unpublished/not-passed journal, draft and unchanged observation/request evidence. Run oracle with stage argument while the archived fixture remains present.
- `hook-invocations.txt`, `hook-pid.txt`, `child-pid.txt`, `requests.jsonl`: hook effects outside managed repositories, one invocation and one GET. No completion marker appeared, including beyond the natural interval.
- `native-events.jsonl`: two native startups, hidden through initial document completion and one initial display each; no native fault-menu action.
- `cleanup.json`, `validation-summary.json`: both app exits 0, extended original-process absence, exact service/key cleanup and current validation scope.

Native UI used Computer Use. Independent scripts did not drive UI, approve, rewrite journals or kill host/hook/child. The main native Close action is distinguished from Quit, review Close/Cancel, direct AppKit and the unexercised red title-bar button interaction. No Dock/session, release, default-write or push-publication acceptance is inferred.
