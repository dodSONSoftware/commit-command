# M3 desktop implementation and native acceptance progress

Recorded November 2, 2026 after baseline commit `7130ced`. This is an implementation checkpoint; M3 retains deferred native gates and ordinary desktop writes remain disabled.

## Implemented changes

- Replaced the serialized proposal dump with operation-specific Rust-produced review text: scope, exact selected sources/paths, expected HEAD/tree, hook manifest, commit identity/signing only when applicable, exact JSON-escaped message and patch. Non-UTF-8 patch bytes remain inspectable as exact JSON bytes. Rust confirmation/nonce/approval authority is unchanged.
- Moved trust inspection out of the Changes/History tab strip. Acceptance builds automatically refresh authoritative trust/eligibility for the active workspace, rejecting old-workspace results.
- Derived file counts and submitted paths from changes that still exist for their selected source, and remove obsolete selections so future changes cannot silently become selected again. Workspace removal resets the monitoring label as well as prior UI state.
- Constrained narrow composer controls to their content height/width.
- Added native window placement persistence through the existing single Core storage owner, without renderer IPC authority or a second state writer. Placement survives restart; native restoration clamps to an available display. Core validates persisted bounds. Maximized/minimized/fullscreen geometry does not overwrite the normal saved placement.

## Automated validation

The complete Core validation script exited zero: default 48 passed/1 ignored; experimental 83/2; controlled writes 71/1; application services 87/1; all features 127/3. Zero failed. Five builds/strict Clippy configurations, formatting/layout checks, mandatory isolated SSH/OpenPGP positives and the 23-pass/one-optional-skip historical lab passed. Ignores retain their documented meanings.

The final desktop validation script passed: 17 renderer tests; seven native mock/unit tests for each feature configuration; typecheck, lint, generated contracts, formatting, frontend build, native debug/release builds and strict Clippy. Added Core regressions cover operation-specific review applicability and placement persistence/rejection. Native unit coverage checks changed-display clamping. The frontend polling regression waits for two real three-second production refresh intervals; its per-test deadline was corrected from five to ten seconds without reducing assertions. The earlier timeout output is retained as unsuccessful evidence.

## Native observations and independent verification

Using the hashed acceptance binary and a newly created disposable fixture:

1. Granted trust only to engine through the independent native dialog. Automatic trust refresh displayed `granted · current` without an inspection click.
2. Reviewed an index-only commit through the new authoritative review, including exact message, source and hook policy. After native approval, waited for the installed slow hook's start sentinel, switched to the primary window and actually clicked Cancel current operation while it ran.
3. Durable operation `98922127-d098-4884-bb57-4e662044c68a` ended `cancelled_before_publication`, published false. The sentinel records exactly one execution. Full before/after snapshots matched for all four repositories.
4. Closed/restarted the application. The cancelled outcome remained durable, no hook replay occurred, and repository snapshots still matched. Moved-window position/size persisted exactly across the restart and second close.
5. Loaded a staged Changes patch, entered History without choosing a commit, and observed the selection prompt without the Changes patch. Removed the active saved definition and observed Welcome, zero repositories, no stale tabs and no stale-review error. Repository snapshots remained unchanged.

A final renderer-only adjustment permanently removes obsolete selections and resets the monitoring label. These changes have automated coverage; the tested acceptance binary predates those last adjustments. Screenshots were inspected in the chat but are not saved as standalone evidence images. Exact binary hash, fixture descriptors, snapshots and result JSON are under `docs/validation/m3-followup/`.

## Deferred work

The Mac locked before the next fresh-fixture native session. Computer Use reported that automatic unlock could not unlock it. Under the user's instruction to skip reasonable permission/facility blocks and revisit later, the remaining UI interactions are deferred rather than requiring the user to operate them now.

Still open: close-during-running-write and forced-interruption/native recovery cases, remaining hostile-content/context/expiration review criteria, final artifact retesting, actual VoiceOver and measured contrast. Physical monitor removal is not reproduced; clamping has unit coverage only. Trust reviews still use the existing structured JSON detail, while write reviews are now operation-specific. General hook-manager support remains excluded. M3 is not declared complete, and ordinary write defaults were not changed.

Subsequent authorized independent read-only M4 work can proceed while these gates remain explicit. No push, release upload or user-repository Git mutation occurred. The only Git writes outside disposable fixtures are the explicitly authorized local project commits.
