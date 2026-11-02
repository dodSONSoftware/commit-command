# Known Limitations and Release Gates

**This is a controlled lab, not a production-safe commit engine. Never use it against live projects.**

1. **Evidence scope.** Original artifact authors could not compile Rust. Subsequent macOS/Git 2.56.0 build/test evidence exists; M0 results are recorded in [implementation status](../../IMPLEMENTATION_STATUS.md). Linux/Windows execution and universal Git compatibility remain unverified.
2. **Manual Git hook subset.** Invokes `pre-commit`, `prepare-commit-msg`, `commit-msg`, and `post-commit`. Does not reproduce all native Git lifecycle context, `pre-merge-commit`, `post-rewrite`, hook manager semantics, or hook invocation from another process.
3. **Executable hooks are not sandboxed.** A malicious or defective hook can modify arbitrary accessible paths, spawn processes, or contact the network. Repo trust and hook policy must gate production writes.
4. **Local branch only.** Detached HEAD and unborn repositories are unsupported for controlled commits. Merge/rebase sequencer states and unmerged index entries are not supported.
5. **Ordinary index only.** Split index, sparse index, intent-to-add, skip-worktree, special extensions, index v4, unusual file mode filters, multiple alternate indexes, and non-UTF8 filenames require dedicated tests or explicit rejection.
6. **Concurrent external Git processes.** Index lock and branch compare-and-swap mitigate but do not eliminate all races, especially processes that ignore Git's locks or act between validation and ref update.
7. **No atomic branch + index update.** Durable journal and recovery cover tested SIGKILL windows but not all filesystem, write-reordering, corruption, or cross-volume failures.
8. **Journal authorization is not authenticated.** Test data are trusted; tampered journals may redirect operations. Production must use opaque, backend-owned approval identities and a durable operation database.
9. **Signing.** GPG positive case runs only when GPG is installed; its generated key is isolated in the disposable fixture. SSH signing is not tested. Key-agent and `gpg.format` behavior remain to be tested.
10. **Post-commit hook.** An at-most-once attempt is journaled. A crash between setting attempted and actual execution may suppress the hook; there is no exactly-once delivery guarantee.
11. **No GitFleet Actions or AI.** Future deterministic SemVer updates, AI templates, profiles, and bulk authorization will be separate application services.
12. **Discovery is intentionally narrow.** Ignores directory symlinks and common directories; doesn't classify linked worktrees/submodules or implement a watcher.
13. **No production write CLI.** `gitfleet-cli` exposes only read-only commands. `gitfleet-lab` is explicitly disposable-only.
14. **Path handling.** Selected paths must be UTF-8 and cannot have symlinked ancestor directories; non-UTF8 paths must be supported or explicitly blocked in production.
15. **No remote operations.** No fetch/pull/push, dependency-aware bulk commit/push, or credential handling in POC.
16. **Recovery stop policy.** If unexpected index/ref state occurs, the lab stops with `needs_manual_reconciliation`; it never attempts destructive cleanup.

## Hard release gates

- Compile and run the Rust tests on macOS and Linux.
- Reconcile outcomes with the v4 Python baseline and investigate every discrepancy.
- Add index/worktree/HEAD stress and fuzzer tests; validate all supported index features.
- Implement production repository trust, scoped authorizations, and immutable approved snapshots.
- Test real-world hooks, GPG/SSH signing, linked worktrees, submodules, crash/restart and power-loss-like faults.
- Define branch, lock, journal, and index cleanup contracts before any UI enables live Git writes.

## M0 boundary and remaining defects

The normal build exposes `inspection` and `discovery`; experimental modules and
`gitfleet-lab` require `experimental-writes`. This feature is a build boundary,
not repository trust or production write approval. Existing `results.json` is
historical and is not overwritten by M0 validation.

See [M0 safety contracts](../M0_SAFETY_BOUNDARIES.md) for supported inspection and
configuration restrictions. `git::log_state` remains a write-producing **lab**
diagnostic; no production-oriented inspection path calls it.

Remaining write blockers include mutable journals/scratch indexes, no trust store
or opaque approval, incomplete unsupported-state preflight, non-comprehensive
hook bypass (including reference-transaction hooks), global signing configuration
suppression, index flags/extensions, shared-worktree races, path TOCTOU,
scratch/temp cleanup, discarded directory-fsync errors, and post-hook state
revalidation. Unknown journal phases now fail deserialization, but that does not
make journals authenticated. Full message equivalence still uses trimming in the
legacy engine; a new independent fixture checks raw message bytes for its case.

`real_sigkill_after_reconcile` terminates **after finalization/complete journal**,
not immediately after index rename. Intermediate journal writes, failed writes,
corruption, power loss and post-hook delivery gaps remain untested. No production
write guarantee follows from the three existing crash locations.


## M1 reconciliation

The default service now includes saved workspaces, typed identity/inventory/status,
history and diffs. [M1 contracts](../M1_READ_ONLY_WORKSPACES.md) supersede historical
read-only limitations above where explicitly tested. No production write limitation
was lifted. The lab adapter additionally suppresses auto-maintenance during fixture
Git commands to make before/after state evidence deterministic. CLI services never
call that adapter.
