# Post-load startup evidence

See [report](../../reports/G02_POST_LOAD_STARTUP_2026-11-07.md) for interpretation.

- `bootstrap-before.txt` / `bootstrap-after.txt`: deterministic initial shell commit regression.
- `desktop-validation.txt`: successful full desktop checks; first formatting attempt retained separately.
- `artifact-provenance.json`, `support-build.txt`: exact source/frontend/support artifact provenance.
- `diagnostic-events.jsonl`: diagnostic baseline; seven launches did not recapture blank.
- `prior-blank-host-diagnostics.txt`: inconclusive OS diagnostics, not root-cause proof.
- `fixed-native-events.jsonl`: three fixed launches, actual renderer kill/reset, no explicit reload action.
- `fixed-host-loss.json`: exact fixture host SIGKILL; no new running hook claimed.
- `native-oracle.py`, `native-baseline.json`, `native-recovered.json`, `native-fixed-after-*.json`: independent preservation/no-replay assertions.

Native UI observations: three fixed launches restored workspace/draft without manual
reload; last restart also displayed cancelled/unpublished operation history.
The startup probe observed a healthy shell, never an actual blank document.
