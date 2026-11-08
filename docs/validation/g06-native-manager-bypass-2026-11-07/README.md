# Native manager-signature refusal and explicit bypass — November 7, 2026

Disposition: [acceptance report](../../reports/G06_NATIVE_MANAGER_BYPASS_2026-11-07.md).
Selected native Execute refusal, OS-declined bypass, fresh approved bypass and restart/no-replay cases passed. Manager fixture tests the detector signature, not an installed Husky distribution. No product source changed.

- `native-fixture.py`, `fixture-preparation.txt`, `native-fixture.json`, `native-baseline.json`: fresh marked factory/IDs/baselines for three nested worktrees and three bare remotes. Factory hook includes the manager signature before trust/snapshot. Run factory from repository root with built Core CLI; each run creates a new fixture. Nested snapshots overlap.
- `fixture-pre-push.txt`: exact executable hook content from this fixture. Incorrect execution would create invocation/PID markers outside repositories; none appeared.
- `artifact-provenance.json`: reused binary/source hashes, unchanged app tree and prior full desktop/build evidence; no new matrix run.
- `native-oracle.py`, `native-refused.json`, `native-declined.json`, `native-approved.json`, `native-restarted.json`, `native-final.json`: read-only exact repository/journal/draft/request assertions for each native stage. Run with stage argument while archived fixture remains available. Restart/final compare against approved baseline.
- `native-observations.json`: transcribed native error, OS decline/consumed review, fresh approval and history/restart observations, including the premature approved-stage assertion attempt and later unchanged pass.
- `native-events.jsonl`, `requests.jsonl`: two startups and a single discovery GET; no test fault actions or further requests.
- `cleanup.json`, `validation-summary.json`: normal app exits, zero hook markers, exact owned service/key cleanup and validation scope.

Native UI used Computer Use. Scripts did not seed trust/approval/results, execute hooks, mutate journals or drive UI. No private key or screenshot artifact retained. Repositories preserved; no publication/default-write/release acceptance is inferred.
