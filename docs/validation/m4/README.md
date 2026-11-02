# M4 validation evidence

Recorded November 2, 2026. [Implementation report](../../reports/M4_READ_ONLY_PROGRESS_2026-11-02.md) and [contracts](../../architecture/M4_GRAPH_ACTIVITY_AUDIT.md) describe scope and deferred native acceptance.

- `core-validation-final.txt`, `final-core/environment.json`, `final-core/lab-results.json`: final full passing Core gate with five configurations, mandatory isolated signing and process-crash regressions.
- `desktop-gate-final.txt`: final complete passing desktop gate.
- `final-summary.json`: exact overlapping counts and native visual deferral.
- `focused-tests-final.txt`, `clippy-final.txt`, `renderer-tests-final.txt`: focused checks.
- Earlier `core-validation.txt`/`matrix-summary.json` precede two final graph/activity regression additions; they are intermediate evidence, not final totals. The earlier desktop log records a formatting check failure corrected before the final passing run.
- `source-sha256.json`: finalized application source hashes; generated/build/dependency directories excluded. No native visual binary attestation is claimed while the Mac is locked.

All Git writes in tests use new disposable fixtures. No new schema, network operation, ordinary write-default change, push or release occurred. Existing ignored cases retain their documented meanings; none are counted as passes.
