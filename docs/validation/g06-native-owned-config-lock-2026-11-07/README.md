# Native owned-config-lock recovery evidence

- `worker-build.txt`, `worker.json`: existing all-feature subprocess worker build.
- `prepare.py`: actual checkpoint SIGKILL fixture generator; no fabricated receipts.
- Per-case `*-worker.txt`, `*-marker.json`, `*-fixture.json`, `*-upstream-before.json`:
  real phase/receipt provenance, exact artifacts and worker -9 exit.
- Per-case repository/remote/network baselines: exact file SHA256/modes after crash.

Preparation passed; native package outcomes initially pending. See the report for
current results as subsequent validated steps are committed.

Final package passed. `oracle.py CASE STAGE PHASE` asserts exact file/mode and durable
journal preservation; `*-final.json` and `*-final-inventory.json` are final results
for all four aborted fixtures. `foreign-oracle.py` records historical stale/manual
refusal stages before restoration; those stages cannot be rerun against the final
aborted fixture. `foreign-owned-restored.json` identifies external fixture restoration;
`foreign-final-native.json` records final native and ordinary restart observations.
`*-exit.json` and native summaries retain process outcomes. The foreign replacement
remains preserved outside the repository. Worker faults are automated Core evidence;
recovery decisions and independent confirmations were exercised through native UI.
