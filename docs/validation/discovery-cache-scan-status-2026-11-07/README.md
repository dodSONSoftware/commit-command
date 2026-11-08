# Discovery cache / scan-status evidence

- `core-workspaces.txt`: 18 passing workspace tests.
- `core-application.txt`: 27 passing application tests, including summary aggregation.
- `core-scan-summary.txt`: initial focused test failure from comparing macOS path aliases;
  corrected canonical comparisons pass in the final full application log.
- `core-clippy.txt`: strict Core application-services Clippy across all targets.
- `desktop-validation.txt`: successful full desktop checks (86 renderer / 13 / 16 host).
- `native-verification.json`: recorded native observations, independent read-only database
  assertions, and final renderer/static verification (87 tests after final integration test).
- `live-core-scan.json`: independent read-only current-workspace scan (complete, 11 repos,
  no issues, six explicitly skipped tagged caches).
- `artifact.json`: SHA256 of the ordinary release executable used for native verification.

The live application first showed two persisted issues, then completed explicit Rescan
with zero issues. No live repository byte snapshot was taken. See the report for limits.
