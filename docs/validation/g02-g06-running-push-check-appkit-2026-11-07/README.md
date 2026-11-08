# Direct AppKit termination during running pre-push check — November 7, 2026

Disposition and native observations: [acceptance report](../../reports/G02_G06_RUNNING_PUSH_CHECK_APPKIT_2026-11-07.md).
Actual `NSApplication.terminate` delivery/interception verified during proven execution; G02/G06 remain open for other criteria. No product source changed.

- `native-fixture.py`, `native-fixture.json`, `fixture-preparation.txt`, `native-baseline.json`: fresh marked fixture factory, IDs and bytes/modes/symlinks after setup. Three nested worktrees/three bare remotes (snapshots overlap). Run factory from repository root with built CLI; each run creates another fixture. No private TLS key retained.
- `artifact-provenance.json`: reused exact artifact/source hashes and unchanged app tree; previous full desktop/build logs are historical evidence.
- `initial-host-process.json`, `prove-running.py`, `native-running-proof.json`: unique host identity, durable running/unpublished state, exact draft, one invocation and live host/hook/child table immediately before the native test action.
- `native-process-cleanup.json`: host exit 0 and all exact processes absent at 43.7 seconds after hook start, before natural 45-second finish; no external host/hook/child signal. Includes proof of AppKit delegate sequence and absent sheet.
- `native-oracle.py`, `native-closed.json`, `native-restarted.json`, `native-final.json`: read-only repository/remote/journal/draft/request preservation. First stage is named closed for consistency with the prior shutdown oracle, not because a window Close action was used. Run with stage argument while archived fixture remains available.
- `native-events.jsonl`, `check-native-events.py`, `native-event-assertions.json`: actual request/dispatch/intercept/return ordering, default native run-loop mode, no attached confirmation sheet, exactly one fixture termination action, startup readiness on both hosts. Run `python3 docs/validation/g02-g06-running-push-check-appkit-2026-11-07/check-native-events.py` against the captured audit; it drives no UI.
- `hook-invocations.txt`, `hook-pid.txt`, `child-pid.txt`, `requests.jsonl`: hook effects outside repositories and one read-only observation GET. No completion marker appeared, including beyond the natural interval.
- `cleanup.json`, `validation-summary.json`: final original-process absence, both app exits, exact service/key cleanup and current validation scope.

Native UI used Computer Use; independent scripts did not inject approvals, mutate journals, or kill app/hook/child. This is not an actual Dock/session termination pass. No release/default-write/push-publication acceptance is inferred.
