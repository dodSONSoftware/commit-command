# Repository identity recovery evidence

- `core-workspaces.txt`: 17 workspace tests (replacement/restoration, persistence,
  incomplete/missing scan, failed inspection, repeat scans, byte preservation).
- `core-application-full.txt`: final 26 application tests, including recovery authority
  and monitoring regressions. `core-application.txt` retains the initial focused pass.
- `core-monitor.txt`: initial failed assertion about watcher availability; final
  regression uses baseline watcher fallback and is included in the full passing suite.
- `desktop-validation.txt`: full desktop validation (83 renderer / 13 default-host /
  16 all-feature host tests), format, lint, contracts, Clippy, debug/release builds.
- `core-clippy.txt`: strict Core application-services lint across all targets.
- `native-fixture.json`: disposable replacement fixture, ordinary artifact SHA256.

Native UI: successful local rescan; one active identity and a historical disclosure;
explicit replacement open; empty composer and disabled ordinary writes; normal exit 0.
See [report](../../reports/REPOSITORY_IDENTITY_RECOVERY_2026-11-07.md) for limits.
