# G02 active-scan host-loss evidence

- `fixture.json`, `baseline.json`: fresh marked-by-path disposable workspace/state,
  ordinary artifact SHA256/features/source and exact managed file bytes/modes.
- `host.json`: exact fixture host PID, separately verified against full command.
- `interrupt.py`, `interruption.json`: guarded arming/live-child proof/host-only SIGKILL.
  This script requires the archived fixture and is not a generic process killer.
- `after-crash.json`: integrity/draft/repository assertions; one retained inventory entry.
- `original-processes-gone.json`: original host and sampled Git child absent.
- `oracle.py`, `oracle-final.json`, `oracle-after-lock-reopen.json`: executed read-only
  final preservation/no-operation/coherent 80-repository scan assertions.
- `native-summary.json`: native busy/first restart/draft/fresh scan observations,
  host -9 versus launcher 0, final Quit 0 and successful writer lock reopen.

No product patch or automated test rerun. No new hook, remote or write execution.
[Report](../../reports/G02_HOST_LOSS_DURING_SCAN_2026-11-07.md).
