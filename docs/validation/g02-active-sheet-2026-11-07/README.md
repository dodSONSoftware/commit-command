# Active-sheet native termination acceptance evidence

See [report](../../reports/G02_ACTIVE_SHEET_TERMINATION_2026-11-07.md).

- `fixture.json`, `manifest.json`: new marked disposable fixture/state/bundle,
  exercised source hashes and exact native binary SHA-256. Features:
  custom-protocol,desktop-writes,native-acceptance-support. Base commit 83cbb12.
- `repositories-before.json`: all bytes/modes of four real fixture repositories,
  including Git metadata; baseline preserved throughout.
- `native-events.jsonl`: three actual dispatch events. First two have attached_sheet
  true, return and no delegate interception; last has false, return and interception.
  This is actual NSApplication terminate delivery, not direct delegate invocation.
  All observed modes were default; no modal-mode dispatch or starvation cause is claimed.
- `trust-open-verdict.json`: historical zero grants/operations after active trust
  sheet termination request. Cannot regenerate that snapshot after explicit trust.
- `commit-open-verdict.json`: historical proposed/unpublished commit after actual
  termination delivery with OS confirmation still unanswered.
- `commit-cancelled-verdict.json`, `closed-verdict.json`, `restarted-verdict.json`,
  `final-verdict.json`: one explicit engine trust, one cancelled unpublished operation,
  exact saved draft and complete repository preservation. Native restart observed.
- `oracle.py`: independent read-only file/SQLite assertions; e.g.
  `python3 docs/validation/g02-active-sheet-2026-11-07/oracle.py final`.
  Earlier stage labels describe historical snapshots, not repeatable final-state checks.
- `support-build.txt`, `desktop-validation.txt`: successful support build and full
  final desktop script (exit 0), 69 renderer / 13 default host / 16 all-feature host
  tests and all static/Clippy/debug/release checks. These suites overlap.
- `default-isolation.json`: default artifact hashes and test request/PID/audit absence,
  default graph test-feature exclusion and acceptance nonfixture refusal.
- `validation-summary.json`: native and automated results, bounded disposition.

Native UI was operated through Computer Use; AX states were inspected, no saved
screenshots. No actual Dock, session/logout, forced active-sheet cancellation,
immediate native exit or full Core/remote/release acceptance is inferred.
Normal native termination and cleanup processes exited 0. Test-only control changes
do not alter the production decision or Core authority. Installed app unchanged;
ordinary writes disabled. Local step commit authorized; no push.
