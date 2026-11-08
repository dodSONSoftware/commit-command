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

## Current-workspace follow-up

Explicit **Rescan and review** was exercised in the normal `__cosmic-v` workspace
with the validated ordinary build. Separation was already present at initial
observation (periodic discovery or earlier interaction may have established it);
the explicit scan verified stability, not a newly observed 21-to-11 transition.
Table, Tree and sidebar showed 11 current repositories. The expanded historical
panel showed 10 original/replacement ID pairs and explicit open actions. No
repository selections or replacement-open actions were made in this follow-up.

A read-only SQLite oracle confirmed 21 retained memberships, 11 unique current
paths, all current identities available, and 10 historical links targeting distinct
current identities at the exact same path. No product fix was required for duplicate
separation. The app remains open in Table overview with history collapsed.

**Discovery limitation remains open:** last scan is incomplete with two metadata
inspection failures under `coding/labs/npc-personality/app/.tools/cache/sdists-v10`
and `coding/labs/npc-voice/.tools/cache/sdists-v10`. The overview's zero attention
count covers known repository entries and does not establish complete discovery.
Investigate these cache traversal errors and presentation of scan-level issues;
this checkpoint does not waive them or automatically change discovery exclusions.

No new automated suite was run for this documentation-only follow-up; the previous
implementation tests remain its automated evidence. No before/after Git byte
snapshot was taken in the live workspace, so exact preservation is not newly claimed.
See `live-workspace-verification.json` in the evidence directory.
