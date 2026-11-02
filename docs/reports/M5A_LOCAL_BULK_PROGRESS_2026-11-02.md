# M5A local bulk commit implementation checkpoint

November 2, 2026. Execution and local commits are authorized by the user; no push. This implements the local M5A workflow with automated coverage. M5 as a whole remains incomplete, and native acceptance of the new bulk view is deferred while the Mac is locked.

## Delivered behavior

- Explicit 1–20 worktree batch preparation with fresh complete discovery, review-all or index-only content, exact exclusions, shared messages and per-repository draft overrides. Preparation preserves managed HEAD/index/working files; it does not silently enroll nested repositories or submodules.
- Immutable per-item M2 proposals combined in one authoritative native review. Known blocked items prevent all execution. Renderer payloads cannot supply approval, and ordinary desktop builds reject batch writes through the existing Rust feature gate.
- Actual submodule dependency order, with selected children before selected parents. A test initially exposed that a child's successful publication could leave a prepared parent executing without renewed batch review. The coordinator now stops and requires a fresh parent review after selected-child publication, preserving the successful child commit.
- Sequential execution with per-item trust/content/policy revalidation and existing shared-common-directory locks. First failure or cancellation stops successors; successful commits remain published. Results distinguish publication from reconciliation. Unfinished-only retry selects worktrees for a new review and excludes every published item.
- Versioned durable batch records in the existing SQLite state table, referencing M2 outcomes rather than duplicating them. No new schema or state owner. Restart never restores confirmation authority or replays a batch. Orphaned proposals and executing records display as interrupted. Pagination and record/review/result payload budgets are explicit.
- Desktop Bulk Commit form, native-review preparation and durable batch results. Existing individual review rendering is reused. Trust revocation and hook-policy changes also abandon/cancel pending batch proposals.
- Current status documentation now clearly separates current authorization/results from the earlier historical M3-only checkpoint. The original dated evidence is retained.

Contract: [M5A local bulk commits](../architecture/M5A_LOCAL_BULK_COMMITS.md).

## Verification actually run

The complete Core validation script passed using cached dependencies and isolated signing fixtures. Counts overlap across feature configurations and must not be summed as unique tests.

| Feature configuration | Passed | Failed | Ignored subprocess workers |
|---|---:|---:|---:|
| Default | 53 | 0 | 1 |
| Experimental writes | 88 | 0 | 2 |
| Controlled writes | 76 | 0 | 1 |
| Application services | 104 | 0 | 2 |
| All features | 144 | 0 | 4 |

All applicable Core builds, formatting and strict Clippy passed. The experimental lab passed 23 scenarios with one optional GPG diagnostic skip; controlled-write positive signing tests ran in the mandatory matrix. Ignored subprocess workers are invoked by their parent acceptance tests, including the new batch interruption worker.

Eleven new bulk integration cases use independent Git HEAD/blob/index/worktree oracles. They cover explicit selection and independent nesting; hook failure and fresh unfinished-only retry; known untrusted worktrees; decline and stale working content; partial staging and stale exclusions; duplicate selection and restart capability revocation; real selected submodule order and fresh parent review; cancellation after a hook actually starts; actual process-group SIGKILL after the first item publishes with restart/no replay; linked-worktree commits to distinct branches; and trust revocation of a pending batch.

The desktop validation script passed type checking, lint, generated-contract drift, formatting, 26 renderer tests, production frontend build, default/all-feature native tests (eight each), strict native Clippy and release build. No new dependencies were added. A separate renderer run under concurrent build load exposed a real-time polling test timeout; its assertions now use explicitly advanced polling intervals. The focused regression passed (nine tests), then the final full renderer run passed all 26 tests, followed by type/lint/format checks. The app polling interval itself was unchanged.

Evidence: [Core matrix](../validation/m5a/core-validation.txt), [desktop gate](../validation/m5a/desktop-validation.txt), [final renderer run](../validation/m5a/renderer-tests-final.txt), [timer regression](../validation/m5a/renderer-timer-regression.txt), [machine-readable summary](../validation/m5a/summary.json), [source hashes](../validation/m5a/source-sha256.json). Source base is `55563fe` plus the recorded checkpoint changes; this is source/build evidence, not packaged native acceptance.

## Remaining gates and next work

Native bulk visual/keyboard/VoiceOver checks are deferred while the Mac is locked. Dedicated multi-level submodule/malformed dependency fixtures remain useful acceptance work; existing real one-level ordering and discovery tests do not establish every graph shape. M3 close/interruption/accessibility gates and M4 native checks also remain open. Ordinary desktop writes remain acceptance-only and disabled by default.

Next is M5B/C remote inspection/policy, explicit fetch and separately reviewed individual/bulk push, using isolated remotes and credential-free fixtures. This package neither implements remote operations nor authorizes pushing this development repository. AI, Actions, backup/retention, release and platform gates remain unfinished under the completion plan.

The pre-existing unrelated `.gitignore` edit is preserved outside this commit. No credentials, signer keyrings or fixture state databases are included in reports/evidence.
