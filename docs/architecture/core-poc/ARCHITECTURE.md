# GitFleet Rust Core POC Architecture

## Layer boundaries

- `gitfleet_core::inspection`: fixed read-only queries and conservative untrusted configuration policy.
- `gitfleet_core::git` (opt-in experimental): narrow Git CLI adapter (`Command`, argv only, no shell), index inspection, path validation, read-only repository state.
- `gitfleet_core::commit`: controlled selective commit planning, isolated indexes, hook invocation, signing via Git, compare-and-swap ref updates, guarded index reconciliation.
- `gitfleet_core::journal`: versioned, fsync-backed journal writes and restart reconciliation.
- `gitfleet_core::discovery`: read-only nested repository discovery, skipping symlinks and common high-volume directories.
- `gitfleet_core::lab`: disposable test fixtures and v4-compatible behavioral scenarios.
- `gitfleet-cli`: read-only adapter (`discover`, `status`).
- `gitfleet-lab`: test-only write adapter, restricted to disposable fixture repositories.

M0 gates `git`, `commit`, `journal`, `lab` and the lab binary behind the non-default
`experimental-writes` feature. The CLI depends only on inspection/discovery.
Typed journal phases preserve the original JSON spellings; typed error categories
cover inspection and hook rejection. Outcome strings remain the legacy lab protocol.

The core contains no Tauri, React, browser, or HTTP server dependencies. A future `gitfleet-tauri` adapter can call service interfaces over typed requests. A web API would be a separate trusted boundary.

## Commit pipeline

1. Record current branch OID, logical index entries, and selected working-tree fingerprints.
2. Create a scratch index starting at `HEAD`, overlay precisely selected index/worktree paths.
3. Compute approved tree and message. Invoke supported validation hooks against scratch index.
4. Reject hooks changing the approved tree, message, or selected working-tree content.
5. Build a reconciled index containing approved selected entries plus untouched other staged entries.
6. Persist journal, then on execution revalidate branch/index/selected worktree.
7. `git commit-tree` creates the immutable commit object. This may leave an orphan if later checks fail.
8. Write an owned `index.lock` containing reconciled index, revalidate state, then use `git update-ref` with expected old HEAD.
9. Promote index lock to real index, fsync, verify commit tree and message, attempt post-commit hook, finish journal.
10. Recovery examines current ref, owned lock, actual index and journal; it never replays a successful commit.

## Authorization and threat boundary

This POC tests a reviewed tree/message boundary, but does **not** implement a production approval token or trust store. The test harness represents a trusted actor. Journal JSON is not signed and must not be accepted from a hostile client. Hooks are executable local programs, not sandboxed. See `LIMITATIONS.md`.

## Why not run ordinary `git commit`?

Normal Git commit hooks may mutate the index or message *after* user review. GitFleet's manually controlled pipeline examines hook effects before creating a commit object. It intentionally supports only a subset of Git's native lifecycle until compatibility is demonstrated.

## Rust production evolution

Before exposing write actions to Tauri, implement a typed request/authorization protocol, privileged service checks, durable operation IDs in SQLite, hook compatibility policy, index-format validation, portable path bytes, locking and race stress tests, plus explicit repository trust.

## M0 contracts

See [service and safety contracts](../M0_SAFETY_BOUNDARIES.md). The typed error
categories and journal enum are implemented; production trust/approval/execution/
recovery service contracts in that document are proposals for M1/M2, not shipped
services. There are no new crates or presentation dependencies.
