# Behavioral Test Matrix

This lab translates 24 cases from GitFleet Hook Lab v4 into a Rust diagnostic harness. In addition, `tests/core_behaviour.rs` covers discovery and path safety.

| Category | Scenarios |
|---|---|
| Selected staged/worktree | `selected_staged_preserves_other_staging`, `selected_worktree_preserves_other_staging`, `mixed_selected_sources` |
| Partial staging | `partial_staging_commits_index_only`, `partial_staging_commits_worktree` |
| Path kinds | `untracked_selected`, `selected_unstaged_deletion`, `selected_rename_pair`, `space_in_path` |
| Hooks | `hook_modifies_only_worktree`, `hook_stages_unapproved`, `hook_rewrites_message`, `post_commit_hook`, `hook_changes_selected_worktree`, `prepare_hook_rewrites_message` |
| Races | `selected_worktree_changed_after_review`, `external_index_changed`, `stale_head_before_cas` |
| True termination | `real_sigkill_before_ref`, `real_sigkill_after_ref`, `real_sigkill_after_reconcile`, `crash_then_external_index_edit` |
| Signing | `signing_missing_key`, `successful_gpg_signing` (skips if GPG missing) |

Each scenario runs in a fresh OS temporary directory, initializes a Git repo, executes the controlled pipeline, checks expected outcome and core invariants, then deletes the fixture.

Current command: `cargo test --workspace --all-targets --features experimental-writes`.
Each executed acceptance case uses a subprocess. Positive GPG is explicitly ignored
by default; run it with `--ignored --exact successful_gpg_signing` only with working
GPG. A requested test may not silently skip. `cargo run --features experimental-writes
--bin gitfleet-lab -- --output target/lab-results.json` retains all 24 scenarios and
reports missing-GPG skips. Key generation failure now fails.

Default `cargo test --workspace --all-targets` exercises only the inspection boundary.
See [M0 evidence ledger](../../ACCEPTANCE_MATRIX.md#9-m0-evidence-and-identifier-reconciliation)
and [current exact results](../../IMPLEMENTATION_STATUS.md).

## Intended invariants

- A created commit has exactly the approved tree and message.
- A failed pre-ref operation does not advance the branch.
- Unrelated staged/unstaged/untracked contents remain represented correctly.
- Controlled recovery does not silently rerun an already published commit.
- Hook mutations outside approved tree/message cause a block before commit creation.
- External index/ref edits are not silently overwritten.

## Repository discovery regression tests (0.1.1)

- Root and nested paths are compared after `fs::canonicalize`, because macOS may expose temporary folders using `/var/...` while the canonical path is `/private/var/...`.
- A symlinked workspace-root alias returns the canonical repository path.
- A linked worktree with a `.git` **file** is discovered alongside the parent repository.

These tests are read-only except for creating disposable local Git fixtures.

## M0 additions and evidence limits

- All successful original scenarios compare full actual tree/blob contents against
  independently captured baseline plus selected index/worktree bytes; excluded
  logical index entries are checked independently of the journal.
- `tests/write_regressions.rs` adds six tests: same-OID branch switch, literal
  filenames, nonzero hook rejection, independent partial/binary/raw-message checks,
  unknown journal phase rejection, and legacy phase spelling preservation.
- `tests/inspection.rs` adds twelve tests for no repository mutation, configured
  helper sentinels, filters (local/include/worktree), inherited overrides, missing
  promisor objects, unborn/detached/unmerged status, root validation and unusual
  filenames. Non-UTF-8 filename creation is explicitly ignored on macOS after EPERM.
- The three original crash locations are retained. The “after reconciliation”
  location is after finalization; intermediate durability boundaries remain untested.
- Cargo ignored tests, diagnostic skipped scenarios, unsupported configurations
  and untested production requirements are separate categories, never extra passes.
