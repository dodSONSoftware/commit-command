# Native running pre-push host crash evidence — November 7, 2026

Disposition and transcribed native observations: [report](../../reports/G02_G06_RUNNING_PUSH_CHECK_CRASH_2026-11-07.md).
Preservation/no replay passed; first valid restart blank, second restored interrupted history/draft. G02/G06 stay open.

- `native-fixture.py`, `native-fixture.json`, `fixture-preparation.txt`: fresh marked fixture factory (run from repository root with built CLI), exact IDs and loopback service. Three worktrees/three remotes; nested baseline subtrees overlap. No private key retained.
- `native-baseline.json`: original full file/mode/symlink snapshots after hook setup.
- `prove-running.py`, `native-running-proof.json`: durable running/unpublished phase, actual hook/child PIDs, invocation and saved draft before crash.
- `initial-host-process.json`, `crash-host.py`, `host-crash-proof.json`: exact host identity checks, SIGKILL to host only and before/after process tables. Crash script requires proof under 20 seconds old.
- `native-oracle.py`, `native-crashed.json`, `native-restarted.json`, `native-final.json`: read-only repository/state/request preservation checks. Raw journal stays running; interrupted is the native audit projection. Run oracle with stage argument from repository root; archived fixture must remain present.
- `hook-invocations.txt`, `hook-pid.txt`, `child-pid.txt`, `hook-finished.txt`, `process-lifecycle.json`: original orphan hook continued and finished naturally; no external child kill. Exactly one invocation.
- `requests.jsonl`, `native-events.jsonl`: one independently approved discovery GET and native startup support audit. No fault menu used.
- `artifact-provenance.json`: verified reused binary/source hashes, unchanged app tree and links to historical automated checks; no new matrix run.
- `unexercised-initial-launch.json`: excluded initial pretest launch, exited 0 before test execution.
- `validation-summary.json`: current checks and unresolved presentation finding.

Native actions used Computer Use; scripts did not inject approvals or mutate journals. Host loss, first blank restart, second successful draft/history recovery, and both normal restart exits are explicitly distinguished in the report. Fixture TLS service stopped and generated key removed; repositories preserved.
