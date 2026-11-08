# G02 active-scan main-window Close evidence

- `fixture.json`: fresh disposable workspace, ordinary artifact SHA256/source/features.
- `baseline.json`: exact repository file bytes/modes before native interaction.
- `host.json`: unique fixture host PID and command; no broad process selection/signals.
- `process-samples.json`: continuous live Git-child sampling; host absent at completion.
- `after-close.json`: initial read-only preservation assertions and normal exit.
- `oracle.py`, `oracle-after-restart.json`, `oracle-final.json`: executed read-only
  repository/draft/no-operation preservation assertions after restart/final cleanup.
- `native-summary.json`: native busy/Close/first-restart/draft observations, normal
  exit codes, final state writer lock reopen and validation limits.

Native actions used CUA, including main-window Cmd+W. First process probe was
inconclusive and is not counted as a pass. No product fix or new automated test run.
[Report](../../reports/G02_CLOSE_DURING_SCAN_2026-11-07.md).
