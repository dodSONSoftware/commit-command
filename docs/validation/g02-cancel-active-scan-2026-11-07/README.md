# G02 active-scan Cancel evidence

- `fixture.json`, `baseline.json`: fresh disposable workspace, exact ordinary artifact
  SHA256/source/features and all managed file SHA256/modes. Baseline content was
  reused from the source fixture and independently matched against the fresh copies.
- `host.json`, `process-samples.json`: unique fixture host and live child read proof.
- `after-cancel.json`: one-entry completed inventory retained, no partial scan cache,
  exact repository preservation and saved draft after native Cancel.
- `oracle.py`, `oracle-after-retry.json`, `oracle-final.json`: executed read-only final
  repository/SQLite/coherent scan/draft/no-operation assertions.
- `native-summary.json`: native busy/Cancel/error/restored-controls/fresh scan/draft
  observations, exit 0 and writer-lock reopen. No signals, code fix or test-suite rerun.

[Report](../../reports/G02_CANCEL_ACTIVE_SCAN_2026-11-07.md).
