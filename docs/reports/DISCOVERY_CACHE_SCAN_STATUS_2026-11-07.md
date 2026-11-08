# Cache discovery and scan status — November 7, 2026

## Diagnosis

The two `sdists-v10` directories contained zero-byte regular `.git` files.
Discovery treated every `.git` marker as a potential repository, so restricted Git
metadata inspection failed there. Their parent `.tools/cache` directories each
contained a valid CACHEDIR.TAG signature. These were generated cache subtrees, not
additional workspace repositories. No managed Git files or cache contents were repaired
or deleted. The issue was discovery traversing tagged caches as ordinary folders.

The marker format follows the [Cache Directory Tagging Specification](https://bford.info/cachedir/).
Only the exact 43-byte signature is recognized; a filename alone is insufficient.

## Patch

Discovery skips child subtrees with a verified regular, non-symlink CACHEDIR.TAG and
records every skipped path in `Inventory.skipped_caches`. It reads only the signature,
uses no-follow/nonblocking opening on Unix, and does not suppress normal discovery
when a marker is invalid/unreadable. Explicitly selected workspace roots are still
inspected, including a root that is itself tagged. Existing name/glob exclusions stay
unchanged; no user configuration was rewritten. Untagged malformed `.git` files still
produce issues rather than being globally ignored.

Inventory responses now include an optional compact summary of the last recorded
scan: completeness, discovery errors, repository-inspection errors and skipped caches.
Old saved scans default to no recorded cache skips; absent scans remain unknown.
The desktop shows a distinct **Discovery complete/incomplete/not yet scanned** status,
issue/cache counts and expandable **Discovery details** with paths, error kind/message,
exclusion scope and retry guidance. Repository attention counts cannot imply complete
discovery. This introduces no network operation or ordinary write capability.

## Validation

Core workspace suite: 18 passed, including tagged-cache skip, byte preservation,
invalid/symlink marker behavior, long comments, explicit tagged root and real malformed
Git marker detection. Core application suite: 27 passed, including aggregation and
persistence of scan-level/repository errors and cache skip metadata. Initial new test
assertions used noncanonical macOS temporary paths; comparisons were corrected to the
canonical paths returned by discovery. The initial focused failure log is retained.

Full desktop gate passed (86 renderer tests at that run, 13 default-host tests,
16 all-feature host tests, typecheck/lint/contracts/format, strict Clippy and
debug/release builds). After adding one final App integration regression, the full
renderer suite passed 87 tests with typecheck/lint/format. Strict Core application-services
Clippy across all targets also passed. No production code changed after the full gate.

The read-only Core scanner on the current `__cosmic-v` workspace reports complete,
11 repositories, zero scan/repository issues and six skipped tagged caches. Native
ordinary build verification first displayed the saved incomplete scan with its two
error paths; explicit Rescan then showed **Discovery complete · 0 scan issues ·
6 tagged caches skipped**, with all skipped paths listed. Identity separation remains
11 current repositories and 10 historical identities. The app was restarted with the
updated ordinary build and left open with Discovery details expanded.

No live before/after Git byte snapshot was taken, so exact live preservation is not
newly claimed; disposable-fixture tests cover read preservation. This closes the two
recorded cache discovery errors and missing scan-summary presentation within this
workspace/test envelope. Broader accessibility, scale and G02/G06 gates remain open.

[Evidence](../validation/discovery-cache-scan-status-2026-11-07/README.md).
