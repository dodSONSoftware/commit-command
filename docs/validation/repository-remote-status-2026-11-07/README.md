# Repository remote-status evidence

- `desktop-validation.txt`: successful full desktop gate (79 / 13 / 16 tests).
- `core-inspection.txt`: 12 passed, one pre-existing host-filesystem ignore.
- `artifact.json`: ordinary release executable SHA256 and disposable-state provenance.

Native UI observation: retained legacy snapshots initially showed unknown; local
Rescan changed them to No upstream. Expanded details showed origin/main/no upstream,
inspection time, local-only freshness explanation. Normal app exit returned 0.
See [report](../../reports/REPOSITORY_REMOTE_STATUS_2026-11-07.md) for scope and limits.
