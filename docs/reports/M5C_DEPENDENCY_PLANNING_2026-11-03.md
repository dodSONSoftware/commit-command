# M5C multi-level dependency planning

November 3, 2026. Implementation and automated validation checkpoint. Explicit selected repositories can now produce a bounded, read-only child-first plan across multiple dependency levels. Unselected children are never silently enrolled. A closed cycle witness or any blocked dependency removes the entire proposed order. No push implementation, execution authority, network request, or ordinary-write default change was added.

## Behavior

The Workspace Overview has a separate local dependency planner. Its selections are independent of bulk commit selection. Each selected repository requires an explicitly chosen completed branch observation and a fresh source preview. Every outgoing historical child mapping requires an explicit checked child repository/observation binding; the operator adds that checked binding to the draft plan. Changing the selected scope, observation, or inspected sources clears dependent drafts and results. Nothing is loaded on mount, and the plan is not persisted or replayed.

Core recomputes previews and validates workspace identity, trust, source OIDs, destination branches, configuration, and observation freshness. The renderer submits only selections, IDs and source OIDs. It cannot supply authoritative reachability or an execution plan. The request is classified as read-only by the native host and creates no review capability, journal operation or network observation.

An already remotely reachable child is satisfied by recent scoped observation plus local ancestry proof. An unselected child whose required commit is unavailable blocks the plan. For a selected child, Core additionally proves that the selected source contains the required child commit and uses the exact selected observation/destination. Such a dependency is labelled conditional: the child still needs successful separately reviewed publication before its parent can proceed. Local source containment does not claim the commit is already remote.

The graph covers all explicitly selected sources and their historical mapping contexts. Selected relationships are conservatively ordered child-first even where existing reachability is sufficient. Unselected children remain terminal evidence checks; their already-published histories are not recursively enrolled. A grandchild → child → parent chain produces that conditional order. Selection order does not determine execution order. Duplicate selected worktrees, duplicate/mismatched/extra bindings and two sources targeting the same destination branch refuse. Behind/diverged roots block non-force planning.

Stable topological sorting returns only selected repositories. Bounded DFS reports an actual closed cycle, excluding merely dependent or independent nodes. Any cycle, blocked edge or global issue clears the entire order; partial successful orders are not returned.

## Bounds and revalidation

Planning allows 1–32 selected repositories, at most 256 explicit bindings and 256 historical mapping contexts in aggregate, and a 2 MiB aggregate preview/result budget. The application request has a 60-second deadline and cancellation propagation. Existing preview and child-evidence bounds remain in force. Observations must be no older than five minutes, including at return. Limit exhaustion refuses without a successful partial plan.

Child-evidence provenance is rechecked and full selected source previews are recomputed before return. Core internally reuses its own preview snapshots during child checks to avoid repeatedly inventorying parent history for every edge; no public snapshot-input interface was added. The existing single-child API retains its final full parent revalidation. The planner retains small provenance summaries rather than cloned whole previews for each edge. Cross-repository reads are not an atomic publication transaction: a future executor must independently revalidate, reobserve after child publication and block parents after failure or uncertainty.

## Validation

Actual results and source provenance are saved in `docs/validation/m5c-dependency-planning/validation-summary.json` and the adjacent logs. Seven Git-backed integration tests cover available unselected children, unavailable unselected children, pending selected children, a source missing the required commit, a real two-repository cycle, a three-level chain with selected/unselected grandchild, stale observations, non-fast-forward roots, duplicate targets, the selected-node limit, binding validation and cancellation. SHA-1/SHA-256 pending-child cases and full worktree/bare-remote preservation are checked. Two graph unit tests cover deterministic child-first order, closed/self cycles and exclusion of noncycle nodes. Eight existing child-evidence tests passed after the snapshot refactor.

Three new renderer tests cover explicit independent selection, binding adoption, selected/unselected behavior, scope invalidation, conditional order and cycle presentation. The full desktop gate passed 63 renderer tests and 13 native-host tests per feature configuration, plus typecheck, lint, contracts, formatting, production renderer build, native debug/release builds and strict Clippy. The full Core gate passed 54 default, 89 experimental-write, 79 controlled-write, 208 application-services and 252 all-feature tests; direct ignores were respectively 1/2/1/4/8. The research lab recorded 23 passes and one skip. All Core format/build/strict Clippy gates passed. Initial test-only SQLite integer type failures are retained in earlier logs; fixed tests use the database's signed integer representation.

No new actual native UI acceptance was run for this planner. Automated renderer behavior and Git-backed Core integration are not native acceptance. No new native artifact is claimed accepted. Earlier child-evidence native acceptance remains historical evidence for that earlier source.

## Remaining gates

Run a fresh disposable three-level native acceptance: separately approve observations for all intended branches; plan a parent with an unavailable unselected child and verify blocking/no auto-selection; explicitly select child and grandchild, adopt both checked bindings and verify conditional grandchild → child → parent order; change selection and verify clearing. Capture exact repository/remote snapshots and observation/journal counts before and after local planning, then ordinary restart/no replay. Do not reuse deleted credential fixtures or infer native acceptance from mocks.

Actual individual/bulk non-force push still needs exact native approval, hook policy/stdin, revalidation, child-success gating, cancellation/uncertain-publication recovery, server rejection and no replay. The current no-push instruction remains binding. Existing operator-only SecurityAgent Deny/Allow/unlock acceptance and OS-blocked direct SCP port-22 evidence remain open. Ordinary desktop writes remain disabled; M5C and the overall completion plan remain incomplete.

## Preservation

The user's existing local commit `6ebadb9` was preserved. The unrelated untracked `LICENSE` was left untouched and excluded from this checkpoint. No branch change, push, credential mutation, system trust change or external-server operation was performed.
