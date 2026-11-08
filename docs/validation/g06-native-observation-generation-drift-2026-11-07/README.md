# Same-OID observation generation-drift evidence

- `fixture.json`: authorized path, isolated state, exact artifacts/features/source.
- `original-baseline.json`, `reviewed-baseline.json`, `switched-baseline.json`:
  expected managed file SHA256/modes across deliberate setup and branch switch.
- `restore-files.json`: original disposable config/HEAD/log bytes for exact cleanup.
- `proposed.json`, `blocked.json`: actual durable observation before/after approval.
- `switch-proof.json`: same OID, config/index unchanged, changed HEAD/log paths.
- `generation-proof.json`: current read-only snapshot generation differs from plan,
  despite the same HEAD OID.
- `oracle.py`, `oracle-blocked.json`, `oracle-restarted.json`, `oracle-restored.json`:
  executed read-only exact repository/SQLite/journal/trust/no-transport assertions.
  Current fixture has been restored; use `oracle.py restored` for its present state.
- `prior-case-restored.json`: previous helper-refusal oracle passed after cleanup.
- `native-summary.json`, `cleanup.json`: native current-trust/stale-review result,
  history on ordinary restart, both exits 0, state-lock reopen and exact restoration.

No product fix or new full automated suite. No network success/publication claimed.
[Report](../../reports/G06_NATIVE_OBSERVATION_GENERATION_DRIFT_2026-11-07.md).
