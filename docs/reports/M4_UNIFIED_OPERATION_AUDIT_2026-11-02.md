# Unified operation audit checkpoint

November 2, 2026. Requirement STATE-004 / completion-plan M4C. Base source `0d4aa0d`; local checkpoint commits authorized, no push. Inputs are existing M2 operation/event rows, version-1 batch records and version-1 remote observation records. The new endpoint is read-only and adds no schema, migration, retention deletion, network or execution authority.

## Delivered behavior

Added typed OperationAudit request/results alongside the unchanged legacy Audit endpoint. Core unions the three existing sources, scopes them to the active workspace, applies optional worktree/type/result/inclusive UTC time filters before sorting/paging, and returns at most 50 records. Equal timestamps use deterministic source/ID tie breaking. Historical worktree IDs remain queryable without opening their checkout. Raw selected records are bounded to 4 MiB each and per page; detailed response entries are bounded to 8 MiB. Unsupported stored versions, inconsistent IDs, malformed filters and oversized records fail visibly.

The result classification preserves distinctions: complete, blocked, cancelled, pending, interrupted and needs_attention. Partial batches and unresolved publication/reconciliation/post-processing require attention; they are not marked complete. Only a currently live native review projects pending. Orphaned local proposed/approved records and remote/batch in-progress reviews project interrupted. No journal phase, session authority or operation is rewritten/replayed by reading history. The exact local event sequence/outcome, batch item results and observation metadata remain typed details; batch/member local operations are intentionally separate traceable records.

Extracted OperationHistory from App and replaced its separate local/remote lists with a single filtered chronology. Pagination and refresh use the applied filter snapshot; unsaved edits do not change older-page scope. Native completion refresh replaces pages, superseded/unmounted responses are ignored, and failed replacement reads cannot leave old records displayed under new filters. Existing guarded local recovery only requests a fresh independent review. Repository observation inspection reuses its existing shared record renderer. No fetch/push/AI/Action operation has been invented.

## Validation actually run

Full Core build/test/strict-Clippy matrix passed, using the existing test-only GPG executable and disposable fixtures:

| Configuration | Passed | Failed | Ignored |
|---|---:|---:|---:|
| Default | 53 | 0 | 1 |
| Experimental writes | 88 | 0 | 2 |
| Controlled writes | 76 | 0 | 1 |
| Application services | 128 | 0 | 2 |
| All features | 168 | 0 | 4 |

Counts overlap. Existing non-UTF-8 filesystem limitations remain ignored; subprocess-worker ignores are invoked by parent crash tests. Positive SSH/OpenPGP and crash matrices passed. The separate disposable lab passed 23 cases with one explicit optional diagnostic skip. No ignore was converted into a pass.

New Core cases use actual reviewed/cancelled stage, batch and network-observation proposals without server contact. They cover exact per-source detail/filters, live review classification, repository preservation/no authority, deterministic mixed-source pagination over 56 tied historical records, cross-workspace exclusion, orphan projections without journal changes, time/offset/input bounds, newer record rejection and 4 MiB storage limits. Synthetic history is explicitly identified in the tests.

Full final desktop gate passed: 45 renderer tests, 10 native host tests under both feature configurations, typecheck/lint/contracts/format, production frontend/native builds, strict Clippy and release build. Renderer tests cover pagination/applied filters, refresh, stale completion responses, exact partial-publication display, recovery review requests, inclusive date/filter serialization and inert content. Initial compile/Clippy/format corrections are not counted as successful gates; successful final logs are retained separately.

Native ordinary bundle: observed one real prior cancelled operation plus three synthetic remote records and one synthetic partial batch. Combined remote-observation/interrupted/engine filters returned exactly the expected historical record. Refresh retained applied filters; scrolling reached the result in the unexpectedly small restored window. Native Quit exited. All journal payload/event rows and all four repositories' file hashes/modes matched the prelaunch snapshots. No server contact, write execution, real batch completion or new native network acceptance is claimed. AX/screenshots were inspected inline; no PNG saved.

Evidence: [Core gate](../validation/m4-unified-audit/core-validation.txt), [desktop final gate](../validation/m4-unified-audit/desktop-validation-final.txt), [summary](../validation/m4-unified-audit/summary.json), [native fixture/summary](../validation/m4-unified-audit/native-summary.json), [source/artifact](../validation/m4-unified-audit/source-artifact.json).

## Remaining work

M4 retention policy/deletion, later AI/Action/fetch/push audit producers, phase-event retention for batch/remote histories, scale and full accessibility remain open. During native inspection a separate M3 finding appeared: saved 1440×940 placement restored as a 720×470 viewport and was then recaptured as 720×470. Investigate native DPI/geometry conversion before accepting restored minimum dimensions. Ordinary writes remain disabled. Preserve concurrent user .gitignore, docs/graphics, docs/README.md and docs/APPLICATION_OVERVIEW.md work; none belongs in this checkpoint. No push.
