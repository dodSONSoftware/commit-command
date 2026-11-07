# G02 selected native crash boundaries — November 7, 2026

See [report](../../reports/G01_NATIVE_SUPPORT_G02_RECOVERY_2026-11-07.md) for the
native review/confirmation/recovery observations and remaining gates. These were
real feature-enabled native app processes using disposable repositories and
separate state. Core's existing fault guard was retained unchanged.

- `before-fixture.json`, `after-fixture.json`: compatible system-temp fixtures,
  exact app binary hashes, initial HEAD/index and expected message/blob. Both use
  `custom-protocol,desktop-writes,native-acceptance-support`. Final source and native
  artifact hashes are in [the shared manifest](../g01-native-support-2026-11-07/source-artifact-manifest.json).
- `*-baseline.json`: all fixture repository files' original hashes/modes.
- `*-crashed-verdict.json`: observed raw journal phase `publication_pending` or
  `reference_updated`; actual ref untouched or advanced respectively. This records
  observed state after self-SIGKILL, not a simulated journal import.
- `*-restarted-verdict.json`: same HEAD/commit count/phase after native restart;
  no automatic replay. After-ref-update journal `published: false` is deliberately
  distinguished from the independently observed advanced ref.
- `*-recovered-verdict.json`: before-publication explicitly aborted/unpublished;
  after-publication complete/published with the same single new commit. Working
  files, index entries and other repositories preserved. Both owned lock paths
  absent after recovery. Native separately approved recovery showed `abort_unpublished`
  or `reconcile_index` and no hook/commit replay. Recovery processes exited 0.
- `oracle.py`: read-only independent preservation/Git/SQLite checks, invoked as
  `python3 docs/validation/g02-crash-boundaries-2026-11-07/oracle.py before recovered`
  (or `after recovered`). Earlier crash/restart verdicts are historical snapshots;
  they cannot be regenerated after recovery. A later strengthened final-phase check
  initially used UI outcome field `index_reconciled`; corrected to the journal's
  actual `reconciled` field, then both final verdicts passed. No product change.
- `*-native-events.jsonl`: actual menu fault arm and fixed phase audit.
- `compatible-build.txt`: build of the exercised compatible native artifact.
- `create_compatible_crash_fixture.py`, `prepare_crash_bundles.py`: fixture creation
  and development bundle/baseline preparation snapshots (run from repo root).
  Before preparation, add root/workspace symlink to gitfleet-workspace and write
  engine/.gitfleet-m2-fixture. Never rerun bundle/baseline preparation on an exercised
  fixture: it intentionally captures pre-operation evidence.
- `core-crash-matrix.txt`: existing aggregate
  `sigkill_matrix_recovers_from_actual_ref_index_and_owned_locks_without_replay`,
  `cargo test --manifest-path app/core/Cargo.toml --locked --offline --features application-services,write-test-support --test m2_writes sigkill_matrix_recovers_from_actual_ref_index_and_owned_locks_without_replay`.
  One test passed covering 13 checkpoints; it is automated Core evidence, not 13
  separately exercised native UI cases.

Setup exceptions remain preserved: `before-initial-fixture.json` and its baseline,
`before-refused-verdict.json` / `before-refused-events.jsonl` record a refused
fault-injection attempt using the standard incompatible fixture layout. No ref
advanced and the guard returned InvalidRequest; this is not a crash pass.
`after-initial-fixture.json` / baseline describe an unused initial fixture, never
claimed as exercised. Native app loss was observed following compatible fault
approval; the fixed fault calls SIGKILL on its own process. No signal exit status,
power-loss durability, remote transport or general renderer crash acceptance is
inferred. Actual renderer loss and Dock/system termination remain open G02 work.
