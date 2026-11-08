# Read-only diff index preservation — November 8, 2026

An acceptance oracle's git diff --name-only refreshed a reconciled index despite
GIT_OPTIONAL_LOCKS=0 and --no-optional-locks. Independent copy proved bytes and
0600→0644 mode change; the original final-restart baseline was taken before the
oracle query, making its exact-preservation comparison invalid. The incorrect pass
statement was withdrawn and corrected native repeat accepted from an explicitly
named post-oracle baseline. Original snapshots and failed results remain retained.

The same behavior was reproduced through actual Core diff inspection: after read-tree
leaves correct entries with empty stat cache, read-only unstaged diff changed index
and returned ConcurrentChange. Core's restricted Git command now explicitly overrides
diff.autoRefreshIndex=false. This alone prevents writes but raw diff still reports
stat-only misses as modifications. Unstaged inspection therefore obtains bounded
NUL-delimited numstat paths and excludes only same-mode M entries absent from actual
content changes. Rename/binary/tab-containing native paths are parsed without display
normalization. Existing raw metadata and file-fingerprint stability checks remain;
raw-only entries are still included in the bounded concurrent-change fingerprint check.
No private index refresh, repository writes, new dependency or authority change.

Regression failed before fix; final owning read/diff suite passed 17 tests, including
clean empty-stat cache with repository autoRefreshIndex=true, exact bytes/mode
preservation, and a simultaneous genuine tab-path content change. Initial config-only
and exit-code trials failed accurate metadata/inspection tests and were superseded;
those outputs are retained, not counted as acceptance passes. All other existing
status/history/diff/helper/conflict/CLI contracts passed.

Full Core validate.sh exit 0: default 61, experimental 96, controlled 86, application
231, all features 275 passed. Exact ignored reasons in summary.json remain exclusions
or subprocess workers; no silent promotion. Lab 24 passed / zero skipped. All builds,
formatter, mandatory isolated signing/real transports and strict Clippy passed.
Full desktop gate 91 renderer / 13 default-host / 16 all-feature host, all type/lint/
contracts/format/build/Clippy passed. Separate all-feature Core strict Clippy passed.
No Core source changed during this full matrix; hashes match the native artifact.

A separate ordinary bundle with this fix rendered the retained batch state/history,
kept native preparation disabled, exited 0, and passed corrected exact restart files/
records plus lock reopen. This verifies restart preservation; direct native clean-file
diff interaction is not claimed where no changed-file row exists. Core regression
provides actual command-level no-mutation evidence. Existing bounded preview/output/
deadline limits remain, and broader G03 scale/usability/accessibility gates stay open.

[Core/desktop evidence](../validation/g03-readonly-diff-2026-11-08/), [retained failed
oracle and early trials](../validation/g05-native-batch-host-crash-2026-11-08/) and
[corrected native chain](G05_NATIVE_BATCH_HOST_CRASH_2026-11-08.md). No push or ordinary
write-default change. The evidence workflow now stops shell sequences on validation
failures and inspects the result before committing acceptance claims.
