# M4 read only implementation progress

Recorded November 2, 2026 after M3 checkpoint `c826256`. Graph/activity/audit services and UI are implemented; native visual/accessibility acceptance remains deferred because the Mac locked during the previous native session. This checkpoint does not declare the whole project complete.

## Delivered

- Default Core graph and activity modules, with scoped application requests and regenerated TypeScript contracts. No presentation dependency entered Core and no write/remote endpoint was introduced.
- A true-parent commit graph with deterministic lane continuity across pages, roots/merges, branch/tag/local tracking-ref selection, explicit shallow boundaries, SHA-1/SHA-256 identities and bounded geometry. The desktop virtualizes loaded rows to at most 40 DOM nodes while preserving exact commit/parent inspection.
- Workspace activity over local branches, deduplicated by logical repository identity across linked worktrees. UTC committer-time timeline and seven-day repository lanes, a twelve-calendar-month heatmap, repository/author/ref filters, partial-coverage warnings and navigation into repository History.
- Time-bounded activity queries that retain recent ancestors behind clock-skewed older descendants; explicit record/time/payload bounds avoid false completeness.
- Audit queries over actual durable M2 operation/phase records, with operation/worktree/result/time filters, paged UI, phase details and stable applied-filter pagination. Git-derived activity never fabricates audit records. Existing write journals and schemas are preserved.
- Dispatch-order tracking in the native bridge fixes cancellation targeting when a later background request overlaps the current Core read.

## Validation

See `docs/validation/m4/` for complete output and final summaries. The focused final Core suite passes five graph/activity tests plus fifteen application tests. These include independent merge-parent and byte-preservation oracles, page continuity/stale cursors, SHA-256, unborn history, malformed lanes, clock-skewed activity, shared-worktree deduplication and real audit phase filtering.

The final renderer suite passes 23 tests across six files. Added cases cover real graph-edge rendering, virtualized scrolling and selection, leap-day UTC calendar coverage, inert source text, partial activity and navigation, applied audit pagination and cancellation ordering. Typecheck, lint, formatting, contract generation/drift and production build are required in the final gate. The complete Core matrix and native build/mock/Clippy results are recorded in the final validation logs; counts from earlier intermediate runs are not promoted to the final source state.

Final full Core matrix: default **53 passed / 1 ignored**, experimental **88 / 2**, controlled **76 / 1**, application services **93 / 1**, all features **133 / 3**, with zero failures. All five builds/strict Clippy configurations and the lab gate passed. Final desktop gate exited zero with **23 renderer passes** and **7 native tests per configuration**, typecheck/lint/format/contracts, frontend build, native debug/release builds and strict Clippy. Counts overlap and must not be summed. Historical ignore meanings are unchanged.

## Remaining acceptance

The Mac's locked state prevents native screenshots/interaction for these new views. Per the user's instruction, this is deferred for later verification rather than blocking independent development or being marked passed. Native graph readability, accessible keyboard/screen-reader flow and large-history/workspace performance must still be checked. Final ordinary desktop defaults remain read-only.

Configured retention is deferred to the existing M8 package; no recovery journal/artifact is pruned. Audit projections for future remote/bulk/AI/Action kinds will be added with those features. M5 through M8 are still outstanding. No project push, release publication or write to an existing developer repository occurred.
