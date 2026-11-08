# Repository identity recovery — November 7, 2026

## Behavior

Repository replacement is a legitimate local event (reclone, restore, or replacement
of Git metadata). Identity mismatch continues to block targeted refresh and execution.
The recovery flow does not treat a shared path as proof of the same repository.

- Unavailable/missing current locations show **Repository identity recovery** with
  **Rescan and review**, local-only guidance and the unresolved path/error list.
- A successful local scan of a different identity at the exact same canonical path
  records the old identity's `superseded_by` relation in existing application state,
  atomically with inventory updates. No database schema migration is needed.
- Historical identities remain in storage and operation-history repository choices,
  but are excluded from active table/tree/sidebar, counts, bulk/planner candidates
  and periodic monitoring. Their refresh/read/write entry points reject them.
- The historical disclosure shows original/replacement IDs, an explicit **Open
  inspected replacement** action only when available, and any old draft read-only
  for manual copying. Tabs, selections and drafts are not silently remapped.
- Rescan abandons pending review/confirmation. Existing trust and operation records
  are retained under their original IDs; the replacement gets no inherited trust.
- Missing volumes, denied access and incomplete/failed inspection stay unresolved.
  A failed inspection cannot establish a replacement. Repeated scans/restart preserve
  historical classification. Restoring the original filesystem identity makes that
  original ID current again after successful inspection; other identities become
  historical. A verified move continues to use existing binding rules, without
  guessing matches from names, paths or commit IDs.

No Git files are changed, no remote is contacted, no history is deleted, and no
ordinary write capability is enabled. This flow does not repair or bypass spurious
identity mismatches: recurring warnings without actual replacement need diagnosis.

## Validation

See [evidence](../validation/repository-identity-recovery-2026-11-07/README.md).
Full desktop validation passed with 83 renderer, 13 default-host and 16 all-feature
host tests, static/contract checks, strict Clippy and debug/release builds.
Core workspace suite: 17 passed. Core application suite: 26 passed, including
pending-confirmation invalidation, preserved draft/tab ownership, fresh replacement
trust, exact fixture preservation and periodic monitoring of current identities only.

The first monitoring assertion assumed filesystem watching was available; this host
reported its existing watcher fallback. The final test compares with the baseline
watcher status and detects any added recovery-related degradation. The first failure
log is retained with that interpretation; it is not a product pass.

Native default release acceptance used a disposable repository with replaced `.git`.
Local Rescan displayed one active repository and one separate historical identity;
expanded recovery showed both IDs and explicit open action. Opening the replacement
showed its untracked file and empty composer, with ordinary writes disabled. Exit 0.
Native draft-copy, all filesystem failure modes and accessibility combinations were
not exhaustively exercised; automated evidence covers preserved draft separation.
Installed/user app and normal workspace state were not changed by this acceptance run.

G02/G06 remain partial. This closes the implemented local identity recovery behavior
within the tested envelope, not broader lifecycle, accessibility or write enablement.
