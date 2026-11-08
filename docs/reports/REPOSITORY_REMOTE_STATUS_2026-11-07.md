# Repository remote status — November 7, 2026

The Workspace Overview table now distinguishes configured remotes from repositories
with none, and shows local tracking comparison status with symbols, text and color.

| Indicator | Meaning |
|---|---|
| ○ No remote | Local inspection found no configured remotes |
| ◇ No upstream | Remotes exist, but the current branch has no upstream |
| ✓ Up to date | HEAD equals its locally recorded upstream ref |
| ↑ Ahead N | N local-only commits |
| ↓ Behind N | N upstream-only commits |
| ↕ Diverged · ↑ N ↓ M | Both local-only and upstream-only commits |
| ? Comparison unavailable | Upstream configured but counts unavailable, e.g. missing local ref |
| ? Remote status unknown | No inspection yet or an older saved snapshot |

Each row has an expandable Remote details disclosure with configured remote names,
branch, upstream ref, both counts where available, inspection time and freshness
explanation. Counts are local ref comparisons, not a claim about the current server.
No fetch or other network operation is started. Existing separately reviewed remote
operations remain in the repository's Remotes and upstream panel. This change adds
no push executor or write authority. Indicators are in the table view.

Core snapshots now include optional remote names. Absent metadata in older persisted
snapshots means unknown, not no remote. Local refresh/rescan supplies current names.
The existing restricted read session and consistency fingerprint cover the read.

## Validation

Full desktop gate passed: 79 renderer tests, 13 default-host tests, 16 all-feature
host tests, typecheck/lint/contracts/format, strict Clippy and debug/release builds.
Core inspection suite: 12 passed, one existing non-UTF-8 filesystem test ignored.
The new real-Git test covers no remote, remote without upstream, missing upstream
ref, equal/ahead/behind/diverged refs, legacy deserialization, unreachable configured
URL without network contact, and exact fixture preservation. Nine renderer cases
cover labels/colors/details and unknown states.

Native ordinary release artifact with copied disposable state showed unknown metadata
before local rescan, then No upstream for the three retained fixture repositories.
Expanded details visibly showed origin, main, missing upstream and local-inspection
freshness explanation. Exit was 0. Other indicator states have automated coverage;
no claim of native visual acceptance for every state/layout/theme. Installed app
was not replaced. G02/G06 and ordinary write enablement remain open.

[Logs and artifact hash](../validation/repository-remote-status-2026-11-07/README.md).
