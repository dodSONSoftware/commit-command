# M5A explicit local commit batches

Implemented November 2, 2026 under `application-services`. Ordinary desktop writes remain disabled; `desktop-writes` is still an acceptance-only opt-in. No remote operation is part of a commit batch.

## Preparation and authority

`ReviewBatch` requires an active workspace and 1–20 distinct, explicitly selected worktrees. Complete fresh discovery is required. Core derives eligible paths from status using either all changed sources or index-only sources. Working content takes precedence for a partially staged path in review-all mode; index-only mode preserves its working version. Rename source/destination paths remain explicit. Exact per-worktree exclusions are checked against current eligible paths; stale exclusions and empty selections block that item. Conflicts block preparation. Nested repository directories are excluded from untracked parent selections and are never implicitly selected.

The desktop supplies a shared message with optional existing per-worktree draft overrides. Messages, sources, hook policy and signing preference become exact immutable M2 proposals. Preparation leaves managed HEAD, index and working files unchanged. All selected item issues appear in one authoritative review; any known blocked item prevents the entire batch from starting. Native confirmation holds a Rust-only capability. Renderer requests can prepare or inspect, but cannot approve or execute.

Dependency edges come only from discovery's actual submodule relationships. Independent nested repositories have no dependency edge. A deterministic topological order places selected children before selected parents. Dependencies are persisted with the batch. If a selected child publishes, the parent's old proposal cannot authorize a future changed gitlink; execution stops for fresh parent review. Omitting a child never enrolls it. Sequential M2 execution retains existing common-directory and native Git locks, including for linked worktrees.

## Workspace composer persistence

Shared messages and per-worktree content/exclusion/hook/signing choices survive
view navigation and restart. A separate versioned `bulk-composer:<workspace-id>`
record uses the existing state table and single owner; the existing desktop-state
schema and per-repository message drafts remain unchanged. Typed load/save requests
require the active workspace, cap the message at 512 KiB, configurations at 100,
exclusions at 10,000 per configuration and the record at 2 MiB. Invalid versions,
relative paths, duplicate native path aliases and oversized values fail without
replacing the last valid draft. Preferences confer no operation authority.

The app retains the composer across view changes, saves with the existing
navigation debounce and flushes before switching/creating workspaces. Selecting
unfinished worktrees retains explicit content policy. A clear-exclusions control
lets users remove obsolete exclusions without resetting index-only mode. Saving a
draft does not stage or write Git content. An immediate close before the existing
debounce is not newly guaranteed by this patch; close/save lifecycle acceptance
remains open.

## Execution and durable results

A versioned `batch:<uuid>` record uses the existing single-owner SQLite state table. It links exact M2 operation IDs, paths, worktree IDs, dependencies and preparation issues. Outcomes are read from M2's journal rather than duplicated. Record size is capped at 4 MiB. Results paginate at up to 50 records and bound raw records to 4 MiB and projected results to 8 MiB per page.

Before execution, Core compares the pending record with persisted content and rechecks active workspace, operation scope, current worktree, trust, policy and M2 content bindings. It persists `executing` before issuing any per-item authority. Only one item receives approval at a time. Cancellation or the first failure stops successors; pending proposals are cancelled. Published commits are preserved, with publication and index reconciliation reported separately. Unexpected failures leave a durable interrupted state rather than implying completion.

No batch is atomic. A result may contain published successes and unpublished failures. The UI can select unfinished worktrees for a new review, excluding all published operations, including publication with incomplete reconciliation. It cannot replay an old operation ID. M2 recovery remains a separate review.

Restart does not restore native confirmation capabilities. An executing record projects as interrupted; a proposed record without its live, unexpired review also projects as interrupted. Merely reading batch results never resumes execution or modifies managed Git state. Trust revocation and hook-policy changes abandon and cancel pending batch proposals.

## Validation boundary

`bulk_application.rs` uses real disposable Git repositories to cover explicit selection, independent nesting, partial staging, exclusions, known untrusted items, decline/stale content, hook failure and unfinished-only retry, real submodule ordering/parent reauthorization, linked-worktree branches, trust revocation, cancellation after a hook starts, and actual process-group interruption after the first item publishes. Restart preserves repository bytes and does not replay. The ignored subprocess worker is invoked by the interruption test.

Renderer tests verify the review payload, native-only authorization boundary, ordinary-build write gate, durable publication display and unfinished-only selection. Native host tests reject nested forged approval fields and apply the existing write feature gate to batch preparation. Existing individual-write regressions remain applicable because batch execution reuses the same M2 engine.

Native visual/keyboard/VoiceOver acceptance is deferred while the Mac is locked. Multi-level submodule and malformed relationship fixtures need further dedicated coverage. Remote fetch/push and assisted/automatic AI batches belong to subsequent packages; this checkpoint does not implement or accept them.
