# Native observation configuration-drift evidence

- `fixture.json`: same explicitly authorized repository, isolated state, exact artifacts,
  source and reviewed/drifted supported loopback destinations.
- `original-config.txt`: original disposable config bytes, restored during cleanup.
- `reviewed-baseline.json`, `drift-baseline.json`, `restored-baseline.json`: expected
  file SHA256/modes at the three intentional fixture stages. Only config was changed.
- `proposed.json`, `blocked.json`: actual durable observation before/after native approval;
  original reviewed URL retained, blocked untrusted_repository, no OID/time.
- `oracle.py`, `oracle-blocked.json`, `oracle-restarted.json`, `oracle-restored.json`:
  executed read-only file/SQLite/record/trust/transport-storage assertions. The current
  fixture is restored: use `oracle.py restored` to check its present state.
- `prior-case-restored.json`: previous helper-refusal oracle passed after config restore.
- `cleanup.json`, `native-summary.json`: native same-path trust renewal, actual stale
  observation approval/refusal, history on ordinary restart, both exits 0 and cleanup.

No live transport success or independent later URL-digest-guard pass is claimed.
[Report](../../reports/G06_NATIVE_OBSERVATION_CONFIG_DRIFT_2026-11-07.md).
