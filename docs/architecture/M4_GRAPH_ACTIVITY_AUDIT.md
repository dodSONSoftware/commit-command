# M4 graph activity and operation audit

Implemented November 2, 2026 under the user's completion-plan authorization. These are read-only services and desktop views. Selected native checks have since passed; full accessibility/scale acceptance remains open. No M3 write default was enabled.

## Core boundaries

`graph` and `activity` are presentation-independent default Core modules. The application facade adds scoped Graph, Activity and Audit requests; generated contracts and existing actual-caller, epoch, ACL, size and queue checks remain in force. No network or generic Git command endpoint is added.

Graph uses the existing consistent history reader, true parent OIDs and topological order. Its lane cursor carries the history generation/scope/offset plus pending OIDs as layout data, never authority. Roots/new tips have no synthetic incoming edge; merges retain each actual parent. Paging preserves pending lanes. Object IDs are validated, stale history cursors fail visibly and active lanes are capped at 256 to bound derived geometry. Shallow boundaries are carried explicitly. Selecting branch/tag/tracking refs reads only locally available objects and never fetches. Worktree navigation remains in the workspace explorer.

Activity uses `rev-list --since-as-filter` so an older descendant timestamp cannot prune a recent ancestor. Date attribution is committer time. The desktop presents UTC calendar dates, seven calendar days for timeline/lanes and twelve calendar months for the heatmap. This avoids ambiguous daylight-saving buckets; the UI labels UTC explicitly. Full ref and author filters are explicit, with no regex or arbitrary arguments passed through to Git.

Workspace aggregation deduplicates by logical repository identity before reading linked worktrees. Independent repositories remain distinct even if their commit OIDs are equal. Each Git session retains its existing consistency checks; an aggregate across repositories is an observation, not an atomic global snapshot. Missing/inaccessible scopes remain issues. Query bounds are 400 days, at most 5,000 observed workspace commits, the shared read deadline and an 8 MiB response cap. Incomplete coverage is labeled partial, never a complete empty/success result. Git-derived commits do not create audit events.

Audit reads actual M2 operations and durable phase events through the existing single storage owner. It exposes type/result/worktree/UTC time filters, 50-entry pages and up to 1,000 phase events per item with an explicit completeness flag. It decodes stored phase JSON into the typed Phase enum and preserves original journal records. UI pagination remains bound to the applied filters while filter inputs are being edited. Terminal cancelled/proposed/reauthorization states do not offer unnecessary recovery execution.

No new schema or retention deletion is required for these read projections. Unresolved recovery artifacts remain protected. Configurable retention/deletion remains undecided. The additive OperationAudit projection now merges existing local, batch and remote observation sources with typed details and shared time/worktree/type/result filters before deterministic pagination. It preserves the legacy Audit API and schema 3. Only live native reviews project pending; orphaned proposals/in-progress records project interrupted without mutating journals or recreating authority. Partial publication/reconciliation/post-processing stays needs_attention. Selected raw records are capped at 4 MiB per record/page and projected details at 8 MiB. Later AI/Action/fetch/push producers require their own implementation; no fabricated events are generated. See [unified audit report](../reports/M4_UNIFIED_OPERATION_AUDIT_2026-11-02.md).

## Desktop behavior

`HistoryGraph` owns graph presentation and a bounded virtual viewport of at most 40 DOM rows. Scope, selected commit and parent remain navigation data; commit inspection still uses the existing immutable object and explicit-parent diff services. Actual graph endpoints come from Core, and unrelated repositories never share an ancestry graph.

`WorkspaceActivity` offers repository/author/ref filtering, commit-to-History navigation, chronological events, repository lanes grouped into timestamp columns and a contribution heatmap. Repository content renders as text. Partial/error coverage remains visible. Long-running activity work uses the existing application busy/cancel path. The bridge tracks pending reads in dispatch order so a queued background request cannot replace the current cancellation target.

Operation results now include the actual operation category, subject/time and expandable phase sequence. Local navigation and inspection remain available in ordinary read-only builds; no trust or execution approval can originate from these views.

## Validation and remaining work

The [M4 report](../reports/M4_READ_ONLY_PROGRESS_2026-11-02.md) records executed checks and the current native deferral. Tests use independent Git parent/tree/byte oracles, merge/tag/pagination and stale-cursor cases, SHA-256 IDs, unborn histories, clock-skewed commit dates, branch/worktree deduplication, audit filtering/phases, UTC leap days, DOM virtualization, inert content and overlapping job cancellation.

Native graph appearance, keyboard/screen-reader use across these new views and performance budgets still require acceptance when the Mac is available. Paging and explicit limits do not establish large-workspace benchmark claims. The full project, M3 acceptance, M5 remote/bulk, M6/M7 AI/Actions and M8 distribution work are not closed by these additions.
