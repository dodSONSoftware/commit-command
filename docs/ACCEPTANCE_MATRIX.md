# GitFleet — Acceptance Matrix and Evidence Ledger

**Version:** 1.0 · **Date:** November 1, 2026  
**Companion:** `GITFLEET_SPECIFICATION.md`

Current acceptance disposition and remaining work are maintained only in
[the current record, gates G01–G11](EXECUTION_STATUS.md#open-acceptance-and-implementation-gates).
This matrix preserves original criteria and dated evidence; it is not a second
current task queue. No gate is newly accepted by documentation reconciliation.

Historical execution updates are recorded in [execution status](EXECUTION_STATUS.md),
[the requirement crosswalk](REQUIREMENT_EXECUTION_CROSSWALK.md) and milestone reports.
The original rows below retain the baseline assessment; dated follow-up ledgers and
reports supersede their earlier test status. A later automated pass does not imply
native or release acceptance.

## How to read the status

- **POC-tested:** Behavior exercised by the existing Python labs / Rust v0.1.1 *in a constrained experimental subset*. Does not mean production ready.
- **Not yet tested:** Requirement accepted, but the production implementation and acceptance test remain outstanding.
- **Partial:** One necessary mechanism was tested; broader behavior remains unproven.

These status labels are assessment of known evidence, not promises of implementation. GitFleet Rust Core v0.1.1 was reported to compile on macOS/Git 2.56.0 with **24 acceptance tests + 6 core tests passing** and **23/24 diagnostic scenarios passing (one positive GPG test skipped)**. Each production test must independently validate its own outcome.

## 1. Workspace, discovery, monitoring and navigation

| ID | Acceptance criterion | Status | Target |
|---|---|---|---|
| WS-001 | Save named workspaces with exactly one root each | M1-tested: saved definitions and restart | M1 |
| WS-002 | Switch between multiple saved workspaces while only one monitors | Partial: one active ID persisted; monitoring remains M3 | M1/M3 |
| WS-003 | Restore last active workspace and tabs after restart | Partial: active workspace restored; tabs remain M3 | M3 |
| WS-004 | Discover normal Git repositories recursively | M1-tested | M1 |
| WS-005 | Discover independent repositories nested under parent repos | M1-tested | M1 |
| WS-006 | Recognize `.git` file / linked worktree marker | M1-tested | M1 |
| WS-007 | Classify submodule vs nested independent repository | M1-tested: index gitlinks/declarations/shared metadata | M1 |
| WS-008 | Canonicalize symlinked root without duplicate path or false failure | M1-tested | M1 |
| WS-009 | Avoid traversing symlinked directories by default | M0/M1-tested | M1 |
| WS-010 | Respect configurable excluded directory patterns | M1-tested: basename and workspace-relative globs | M1 |
| WS-011 | Detect add/remove/move/rename while active | Not yet tested | M3 |
| WS-012 | Report inaccessible/missing repo without treating it as empty | M1-tested explicit scans; watcher behavior remains M3 | M1/M3 |
| WS-013 | Monitor status changes from external tools without resetting UI state | Not yet tested | M3 |
| WS-014 | Quit and stop monitoring on last window close | Not yet tested | M3 |
| WS-015 | Compare canonical repo identity while showing friendly relative path | M1-tested native bindings and friendly paths; UI remains M3 | M1/M3 |
| UI-001 | Table/Tree share selection state, filters, and repository identity | Not yet tested | M3 |
| UI-002 | Selecting parent repository does not select children | Not yet tested | M3 |
| UI-003 | Overview permanent, repo tabs closable/restorable | Not yet tested | M3 |
| UI-004 | Adaptive Changes/History selection and manual view persistence | Not yet tested | M3 |
| UI-005 | Responsive resizable panels; light/dark themes and accessible controls | Not yet tested | M3/M8 |

## 2. Status, staging and controlled commits

| ID | Acceptance criterion | Status | Target |
|---|---|---|---|
| GIT-001 | Read-only status returns staged/unstaged/untracked accurately | M1-tested under the restricted inspection policy | M1 |
| GIT-002 | Stage full selected paths only (no unrelated paths) | Partial | M2 |
| GIT-003 | Distinguish staged/unstaged versions of same file | M1 read/diff tested; experimental commit coverage retained | M2 |
| GIT-004 | Commit selected index content, preserving unrelated staged paths | POC-tested | M2 |
| GIT-005 | Commit selected worktree content, preserving unrelated staging | POC-tested | M2 |
| GIT-006 | Preserve partially staged file after index-only commit | POC-tested | M2 |
| GIT-007 | Handle selected worktree version of partially staged path | POC-tested | M2 |
| GIT-008 | Handle mixed index/worktree selections | POC-tested | M2 |
| GIT-009 | Handle untracked selected path and preserve excluded untracked | POC-tested | M2 |
| GIT-010 | Handle deletions, rename pairs and spaces in filenames | POC-tested | M2 |
| GIT-011 | Handle unusual filenames/non-UTF-8, binary and mode changes | Partial: M1 bytes parser/binary/mode/symlink tested; non-UTF-8 filesystem skipped; production writes pending | M2 |
| GIT-012 | Require user review for individual commit | Not yet tested | M3 |
| GIT-013 | Require user review and explicit batch selection for bulk commit | Not yet tested | M5 |
| GIT-014 | Actual committed tree equals authorized tree | POC-tested | M2 |
| GIT-015 | Actual commit message equals authorized message | POC-tested | M2 |
| GIT-016 | External selected-worktree change invalidates approval | POC-tested | M2 |
| GIT-017 | External index entry change invalidates approval | POC-tested | M2 |
| GIT-018 | Stale HEAD prevents unsafe ref update | POC-tested | M2 |
| GIT-019 | Actual index reconciling preserves unaffected staged/unstaged work | POC-tested | M2 |
| GIT-020 | Unmerged/split/sparse indexes and linked worktree conflicts fail safe | Not yet tested | M2 |
| GIT-021 | Successful SSH and GPG signing on supported local config | Partial (missing GPG verified; positive GPG not tested on Mac) | M2/M8 |
| GIT-022 | Post-commit side effects are separate, visible and not misattributed | POC-tested | M2 |
| GIT-023 | Git log, diff, branch, tag and graph reflects real DAG | Partial: M1 history/parent/ref/diff tested; graph rendering remains M4 | M1/M4 |

## 3. Hooks, trust, transaction and crash recovery

| ID | Acceptance criterion | Status | Target |
|---|---|---|---|
| SEC-001 | Newly discovered untrusted repos cannot execute writes/hooks | Not yet tested | M2 |
| SEC-002 | User can approve trust for one or explicit set; no inherited grant | Not yet tested | M2/M3 |
| SEC-003 | Hook source/effective directory/executable status visible read-only | Not yet tested; minimum inspection prerequisite to M2 | M2/M8 |
| SEC-004 | Hook introduces extra staged file → reject before publishing | POC-tested | M2 |
| SEC-005 | Hook rewrites `prepare-commit-msg` → reject before publishing | POC-tested | M2 |
| SEC-006 | Hook rewrites `commit-msg` → reject before publishing | POC-tested | M2 |
| SEC-007 | Hook changes selected worktree content → reject | POC-tested | M2 |
| SEC-008 | Hook modifies only excluded worktree file → preserve and report | POC-tested | M2 |
| SEC-009 | Hook rejection preserves real Git state; record error | M0 Rust nonzero rejection preserves refs/index/worktree; durable failure audit still missing | M2 |
| SEC-010 | Saved execute/bypass-all policy displayed in final review | Not yet tested | M2/M3 |
| SEC-011 | Explicit bypass-all for applicable local hooks without changing original scripts | Not yet tested | M2 |
| SEC-012 | Unsupported hook managers/configurations block writes if needed | Not yet tested | M2/M8 |
| SEC-013 | Crash before reference update doesn't publish commit | POC-tested with real SIGKILL | M2 |
| SEC-014 | Crash after reference update recovers without duplicate commit | POC-tested with real SIGKILL | M2 |
| SEC-015 | Crash after reconciliation recognizes completion | Partial: SIGKILL is after finalization/complete journal; intermediate window untested | M2 |
| SEC-016 | External index edit during recovery prompts manual reconciliation | POC-tested | M2 |
| SEC-017 | Durable journal tracks OIDs, phases, scope; secrets redacted | Partial | M2 |
| SEC-018 | No arbitrary command execution exposed through Tauri IPC | Not yet tested | M3 |
| SEC-019 | Repository path traversal/symlink boundary blocked | Partial: M1 literal/native paths and preview escape checks; hostile metadata races unsupported | M1/M2 |
| SEC-020 | Browser/Tauri adapter cannot bypass Rust authorization | Not yet tested | M3 |

## 4. Bulk commit, push and dependencies

| ID | Acceptance criterion | Status | Target |
|---|---|---|---|
| BULK-001 | Eligible changes proposed without automatically staging them | Not yet tested | M5 |
| BULK-002 | Unselected repos and nested child repos receive no writes | Not yet tested | M5 |
| BULK-003 | Shared commit message with per-repo overrides | Not yet tested | M5 |
| BULK-004 | Submodule changes committed before selected parent | Not yet tested | M5 |
| BULK-005 | Parent gitlink change invalidates previous approval | Not yet tested | M5 |
| BULK-006 | Partial batch failure stops pending by default; preserves success | Not yet tested | M5 |
| BULK-007 | Retry incomplete batch performs fresh validation | Not yet tested | M5 |
| PUSH-001 | Push requires separate review and cannot follow commit implicitly | Not yet tested | M5 |
| PUSH-002 | Guided upstream setup confirms remote and destination | Not yet tested | M5 |
| PUSH-003 | Diverged remote or missing upstream is correctly blocked | Not yet tested | M5 |
| PUSH-004 | Submodule pushed/verified before dependent selected parent | Not yet tested | M5 |
| PUSH-005 | No bulk force push or unexpected destination creation | Not yet tested | M5 |
| PUSH-006 | Remote changes between fetch/review/push fail safely | Not yet tested | M5 |

## 5. AI, templates and deterministic Actions

| ID | Acceptance criterion | Status | Target |
|---|---|---|---|
| AI-001 | Named connections, endpoint validation and auth stored securely | Not yet tested | M6 |
| AI-002 | Discover Ollama models and select via searchable dropdown | Not yet tested | M6 |
| AI-003 | Test connection and inference independently, no repo modification | Not yet tested | M6 |
| AI-004 | Profiles own editable system messages and generation parameters | Not yet tested | M6 |
| AI-005 | Hierarchical profile assignment with one-off override | Not yet tested | M6 |
| AI-006 | Global/per-connection concurrency limits respected | Not yet tested | M6 |
| AI-007 | Missing model causes retry/select/cancel; no silent fallback | Not yet tested | M6/M7 |
| AI-008 | Structured templates and instructions inherit with effective preview | Not yet tested | M6 |
| AI-009 | Static preview deterministic; AI test is read-only | Not yet tested | M6 |
| AI-010 | Context includes bounded diff/source/docs/relevant Git logs | Not yet tested | M7 |
| AI-011 | Hierarchical large-diff analysis reports omissions and provenance | Not yet tested | M7 |
| AI-012 | Exactly actual sent context inspectable after filtering/redaction | Not yet tested | M6/M7 |
| AI-013 | Up to 3 meaningful candidate messages; verified template compliance | Not yet tested | M7 |
| AI-014 | Explicit AI Automatic run respects bounded authorization | Not yet tested | M7 |
| AI-015 | No watcher-triggered AI commit and no automatic push | Not yet tested | M7 |
| AI-016 | Prompt injection cannot authorize Git writes/Actions | Not yet tested | M7 |
| AI-017 | AI endpoint approval prevents unintended source transfer | Not yet tested | M6 |
| ACT-001 | SemVer file action previews old/new without writing | Not yet tested | M7 |
| ACT-002 | SemVer updates file only after approved action invocation | Not yet tested | M7 |
| ACT-003 | Idempotent retry prevents double-increment | Not yet tested | M7 |
| ACT-004 | Imported Action definitions require explicit reauthorization | Not yet tested | M8 |

## 6. Persistence, activity and release

| ID | Acceptance criterion | Status | Target |
|---|---|---|---|
| STATE-001 | TOML configuration and SQLite durable state with migrations | M1-tested TOML/SQLite schema 1 and restart/failure paths; M3 UI state pending | M1/M3 |
| STATE-002 | Git-derived activity includes local branches and dedupes per repo | Not yet tested | M4 |
| STATE-003 | Timeline/lanes seven-day default; heatmap twelve-month default | Not yet tested | M4 |
| STATE-004 | Persistent Operation History records real per-repo results | Not yet tested | M2/M4 |
| STATE-005 | Config import/export validates archive and strips credentials | Not yet tested | M8 |
| STATE-006 | Automatic rotating local config backups, no background daemon | Not yet tested | M8 |
| STATE-007 | Consistent SQLite backup and safe restore with preview | Not yet tested | M8 |
| RELEASE-001 | Presentation can be replaced without recompiling domain logic | Architecture POC supports separation; adapter not tested | M3 |
| RELEASE-002 | macOS development and packaging flows tested end-to-end | Not yet tested | M8 |
| RELEASE-003 | Supported Linux platform integration passes | Not yet tested | M8 |
| RELEASE-004 | Windows compatibility validated before claiming support | Not yet tested | Post-Mac release / M8 |
| RELEASE-005 | Default test run never writes to actual developer repositories | M0/M1-tested: only new disposable Git write fixtures | All |

## 7. Test commands and evidence

Existing v0.1.1 Mac evidence:

```bash
cargo build --workspace --all-targets
cargo test --workspace --all-targets
cargo run --bin gitfleet-lab -- --output results.json
```

Reported: 24/24 acceptance tests passed; 6/6 core behavior tests passed; 23 diagnostic scenarios passed with positive GPG test skipped. Test outcomes are limited to the experimental fixture coverage.

## 8. Signoff procedure

A milestone is accepted only when its required test IDs have runnable reproducible tests and objective evidence (CI output, integration logs and verified artifact versions), its documented unsupported cases fail safely, and its interfaces do not erode the authorized Git operation boundary.

Production release additionally requires a security review of Git command construction, trust identity, hook execution, commit journal/recovery and AI context transfer. Screenshots and unit tests alone are insufficient evidence.


## 9. M0 evidence and identifier reconciliation

Current exact validation is in [implementation status](IMPLEMENTATION_STATUS.md).
The v0.1.1 evidence in sections 1–7 is historical. The 24-case lab is retained;
Cargo now explicitly ignores optional positive GPG rather than counting a runtime
skip as a pass. An explicitly requested positive GPG Cargo test fails if skipped;
key-generation failure is an error. `gitfleet-lab` still reports missing-GPG skips.
All committed lab outcomes now compare actual full tree/blob contents with an
independent HEAD-plus-selected-source oracle, and check excluded index entries.
This strengthens evidence without promoting POC labels to production readiness.

| M0 ID | Criterion | Runnable evidence / status |
|---|---|---|
| M0-001 | Normal inspection has no object/index/ref/worktree writes | `inspection_and_cli_preserve_dirty_repository_bytes`; full fixture bytes/modes/entries unchanged |
| M0-002 | Configured helpers do not execute during supported inspection | `configured_helper_sentinels_are_not_executed`; fsmonitor/hooks/external diff/textconv/SSH/credentials/signing sentinels |
| M0-003 | Content filters blocked, including includes/worktree config | `content_filters_are_rejected_without_mutation`, `included_filter_configuration_is_rejected`, `worktree_filter_configuration_is_rejected` |
| M0-004 | Inspection handles unborn/detached/unmerged without write-tree | `unborn_and_detached_head_can_be_inspected`, `unmerged_index_can_be_inspected_without_write_tree` |
| M0-005 | Repo/config environment cannot redirect CLI; no lazy fetch helper | `cli_does_not_inherit_git_repository_or_config_overrides`, `missing_promisor_object_cannot_launch_a_remote_helper` |
| M0-006 | Branch switch at same OID invalidates plan | `same_oid_branch_switch_invalidates_prepared_plan`; exact refs/index/worktree assertions |
| M0-007 | Selected filenames are literal | `selected_pathspec_magic_is_a_literal_filename`; selected blob, excluded blobs/staging, exact changed-path set |
| M0-008 | Nonzero hook fails without publication or work loss | `nonzero_hook_preserves_refs_index_and_worktree`; typed HookRejected and exact bytes |
| M0-009 | Independent actual blob/worktree/message assertions | Original lab oracle plus `partial_staging_and_binary_content_have_independent_oracles` |
| M0-010 | Default build excludes experimental writes | Cargo feature manifest and `verify-scaffold.py`; default/opt-in builds tested separately |
| M0-011 | Typed journal phases preserve history and reject unknowns | `journal_phase_serialization_preserves_legacy_spelling`, `unknown_journal_phase_is_rejected` |
| M0-012 | Unusual filenames and root boundaries | newline fixture and `nested_directory_is_not_accepted_as_worktree_root` pass; non-UTF-8 fixture explicitly ignored on macOS after host EPERM |

Status vocabulary: **passed** requires executed assertions; **ignored/skipped**
is excluded from coverage; **unsupported** is an explicit boundary such as configured
content filters; **untested** includes Linux/Windows, advanced indexes, SSH signing
and production security gates. Unsupported/untested scenarios are not counted as
passing tests. See [supported envelope](architecture/M0_SAFETY_BOUNDARIES.md).

Specification section 16.2 now uses `INV-001`–`INV-015` for release invariants.
Its old `SEC-*` labels were ambiguous; matrix `SEC-001`–`SEC-020` retain their
original meanings. Crosswalk (relationships, not claims of complete coverage):

| Release invariant | Acceptance scenarios |
|---|---|
| INV-001 trust | SEC-001, SEC-002, SEC-020 |
| INV-002 exact tree | GIT-014, SEC-004 |
| INV-003 exact message | GIT-015, SEC-005, SEC-006 |
| INV-004 stale state | GIT-016–GIT-018, M0-006 |
| INV-005 preservation | GIT-004–GIT-010, GIT-019–GIT-020 |
| INV-006 unsupported hook/config rejection | SEC-012 |
| INV-007 no uncertain replay | SEC-013–SEC-017 |
| INV-008 reviewed bypass | SEC-010–SEC-011 |
| INV-009 separate push | PUSH-001, AI-015 |
| INV-010 AI authority isolation | AI-016 |
| INV-011 bounded Actions | ACT-001–ACT-003 |
| INV-012 imported Action authorization | ACT-004 |
| INV-013 endpoint approval | AI-001, AI-012, AI-017 |
| INV-014 dependency safety | BULK-002, BULK-004–BULK-007, PUSH-004–PUSH-006 |
| INV-015 replaceable presentation | RELEASE-001, SEC-018, SEC-020 |

M2's gate references INV-001–INV-008; M7 references INV-009–INV-013. Production
trust, approval, hook bypass and durability gates remain open despite M0 passing.

## 10. M1 acceptance gates — completed local implementation

M1 was explicitly authorized with its architectural decisions and gates. Exact
commands/results/skips are in [implementation status](IMPLEMENTATION_STATUS.md);
contracts and unsupported cases are in [M1 architecture](architecture/M1_READ_ONLY_WORKSPACES.md).
M0 evidence remains under `validation/m0/`, M1 evidence under `validation/m1/`.
The following are executed local macOS gates, not Linux/Windows support claims.

| Gate | Acceptance evidence | Status |
|---|---|---|
| M1-A01 | `saved_workspaces_restart_activate_rename_update_remove_and_lock`; CLI create/list equivalence | Passed |
| M1-A02 | `mixed_inventory_classifies_nested_submodule_and_linked_worktrees`; staged gitlink and child dirt fixture | Passed |
| M1-A03 | Existing M0 symlink/root fixtures; `partial_permission_scan_and_glob_exclusions_preserve_known_identity` | Passed |
| M1-A04 | `identities_survive_restart_and_verified_move_but_not_copies_or_replacement`; `clones_with_identical_head_have_distinct_identities` | Passed for observed filesystem bindings; no cryptographic trust claim |
| M1-A05 | `exclusions_partial_scans_and_unavailable_entries_are_explicit`; permission/root-change/refresh failures | Passed |
| M1-A06 | Structured partial/rename status; conflict/merge state; mode/deletion/binary; staged submodule gitlinks; M0 unborn/detached | Passed; non-UTF-8 filename integration still excluded on macOS |
| M1-A07 | Local-only upstream comparison, missing upstream and shallow fixtures | Passed |
| M1-A08 | Merge parent/tag/paging/stale cursor and shallow-boundary fixtures | Passed |
| M1-A09 | Independent blob/patch/worktree oracles; merge/root comparisons; literal filenames; binary/modes/gitlinks | Passed |
| M1-A10 | M0 exact repository snapshots plus all history/diff/helper/failure and CLI equivalence fixtures | Passed for stable supported configurations |
| M1-A11 | Byte parser/serialization, literal/newline paths, symlink/FIFO/path escape checks | Passed; native non-UTF-8 filesystem test ignored on macOS |
| M1-A12 | Four-process permits, same-worktree serialization, real timeout/midflight cancellation/output limits, ref/config mutation checks | Passed for owned Unix processes and responsive local filesystems |
| M1-A13 | Restart reconciliation; failed atomic config save; invalid/newer config/schema preservation; unknown v0 refusal and known-empty v0 backup | Passed for implemented schema 0→1; future destructive migrations require new tests |
| M1-A14 | Default feature boundary/static checks; CLI/library status/history/commit/diff/preview/workspace equivalence | Passed |
| M1-A15 | Complete default/all-features validation and original diagnostic runner | Passed with separately reported GPG and non-UTF-8 exclusions |

Read-only ordinary/unmerged/split-index and SHA-256 fixtures passed. Sparse-index
configuration is explicitly rejected without repository writes. Detailed submodule
working-tree dirt is derived from separately inspected child entries; parent counts
do not recursively include child changes. No forceful cleanup, index repair, remote
fetch, trust grant or production commit path is provided.

The initial M1 tests exposed setup-triggered background Git maintenance. Test Git
commands now disable it; repository snapshots are compared without ignoring lock or
object changes. A rejected diff option was corrected after reproducing it in a
fresh fixture. M0's original lab scenarios and their independent content oracles
remain intact.

## 11. M2 acceptance gates — controlled writes

M2 is assessed only for local macOS/APFS/Git 2.56+ and the explicit supported states
in [the M2 contract](architecture/M2_CONTROLLED_WRITES.md). All named tests below
are in `app/core/tests/m2_writes.rs` unless otherwise stated. The
[evidence index](validation/m2/README.md) distinguishes passed, ignored,
unsupported and untested cases. POC evidence above remains historical; this ledger
supersedes its production-pending labels only for the listed scope.

| Gate | Acceptance criterion | Executed evidence / disposition |
|---|---|---|
| M2-A01 | Preserve architecture and M0/M1 boundaries | Layout/manifest check; default, experimental-only, controlled-only and all-feature builds/tests/Clippy; unchanged original tests. |
| M2-A02 | Untrusted writes blocked, no inherited authority | `untrusted_proposals_and_forged_trust_reviews_are_rejected_without_repo_writes`, `linked_worktrees_require_independent_trust_and_preserve_other_checkout`. |
| M2-A03 | Binding changes/revocation require authorization | `trust_revocation_and_changed_hook_or_configuration_block_before_publication`, `moved_replaced_and_permission_changed_metadata_never_inherit_write_authority`. |
| M2-A04 | Exact Core-owned one-use expiring approvals | `approvals_reject_tampering_expiry_restart_and_same_oid_branch_switch`, exact-commit idempotence test, `changed_selection_index_and_ref_invalidate_approval_and_noop_is_rejected`. |
| M2-A05 | Proposal/approval do not mutate managed Git state | Complete byte/mode snapshots in untrusted and exact-commit tests, unsupported/stale-state tests. |
| M2-A06 | Stage/unstage and initial commits preserve unrelated content | `stage_unstage_and_commit_preserve_partial_and_excluded_staging`, `supported_sha256_unborn_stage_unstage_and_commit`. |
| M2-A07 | Mixed sources/renames/literal/binary/mode/symlink/gitlinks exact | `mixed_sources_rename_gitlinks_and_worktree_config_preserve_exact_selection`, `literal_binary_symlink_executable_deletion_and_initial_commit_have_exact_trees`. Non-UTF-8 filesystem case excluded. |
| M2-A08 | Supported-state preflight fails closed before writes | `unsupported_indexes_operation_states_filters_and_foreign_locks_do_not_write`, `unsupported_sparse_conflicts_detached_and_conversions_never_mutate_git`. |
| M2-A09 | Hook inspection and effective configuration are accurate | `effective_global_hook_path_and_prepare_message_rewrites_are_reviewed`, mixed-source worktree-config test, trust invalidation tests. |
| M2-A10 | Preparatory hooks cannot expand/alter approved content | `hooks_reject_tree_message_and_selected_content_changes_without_publishing`, `preparatory_hooks_can_read_selected_private_blobs_and_rejected_adds_stay_private`, prepare-message rewrite test; independent actual index/ref/file/object assertions. |
| M2-A11 | Complete reviewed bypass; unsupported managers blocked | `hook_bypass_is_complete_and_managers_and_reference_hooks_fail_closed`; six hook sentinels, remembered-policy assertion. |
| M2-A12 | Post effects/errors separate; bounded process and cancellation | `post_hook_failure_preserves_published_commit_and_excluded_side_effects_are_reported`, `hook_timeout_is_bounded_and_shared_repository_lock_prevents_execution`; M1 process cancellation/output tests and new bounded-input unit regression. |
| M2-A13 | Configured signing succeeds and verifies; no unsigned fallback | `ssh_signing_is_verified_against_approved_key_without_unsigned_fallback`, mandatory `openpgp_signing_positive_gate_requires_isolated_tool_and_keyring`, `wrong_key_and_timed_out_signers_never_publish_or_fall_back_to_unsigned`. |
| M2-A14 | Exact commit and index publication, safe concurrency | Exact-message/tree/file assertions, same-OID branch-switch test, mixed/index-only tests, shared repository lock test and linked-worktree test. |
| M2-A15 | Durable journal/migration and inspectable operation results | `m2_schema_upgrade_backs_up_m1_and_default_services_can_reopen_schema_two`; persisted outcomes/reopen assertions in crash tests; SQLite phases include scope/OIDs. No automatic retention pruning. |
| M2-A16 | Process-crash recovery at all recorded mutation boundaries | `sigkill_matrix_recovers_from_actual_ref_index_and_owned_locks_without_replay`: 13 actual signal-9 phases, including ref-updated-before-published receipt and renamed-index-before-lock release. OS/power-loss behavior untested. |
| M2-A17 | Storage/artifact/foreign-state failures cannot force unsafe recovery | `storage_failure_and_corrupt_candidate_cannot_publish_unapproved_content`, `recovery_refuses_foreign_index_lock_ref_and_partial_reflog_without_writes`, `recovery_review_is_session_owned_stale_and_post_attempt_uncertainty_survives_retry`, explicit re-trust regression. |
| M2-A18 | Evidence, unsupported cases and next boundary documented | M2 architecture, current status/handoff, decision disposition and separate raw validation evidence. No M3 implementation, commit or push. |

Requirement mapping: GIT-004–GIT-010 and GIT-014–GIT-019 have M2 service coverage;
GIT-011 remains partial because of the non-UTF-8 filesystem exclusion. GIT-020 has
fail-closed advanced-index coverage and supported linked-worktree execution, not
advanced-index write support. GIT-021 has positive OpenPGP/SSH coverage for the
stated envelope; GIT-022 has explicit post-processing outcomes. SEC-001–SEC-017
have the bounded Core coverage above; SEC-002 explicit bulk/set trust and SEC-010
desktop presentation still await later adapters. SEC-019 retains the hostile-race
limitation. STATE-004 has Core persistent operation records, not a desktop history
screen. No future bulk, remote, UI, AI or Actions acceptance is implied.

## 12. M3 acceptance gates — implementation with native gates open

**M3 is not complete.** Ordinary desktop writes remain disabled in Rust and UI;
`desktop-writes` is non-default. The November 1 agent policy denial is historical.
[The November 2 checkpoint](M3_NATIVE_ACCEPTANCE_HANDOFF_2026-11-02.md) records later
user-observed native results supported by terminal output/screenshots, without
independent reproduction by the documenting agent or exact test-time binary attestation.
Current November 2 agent-operated follow-ups additionally establish actual Cancel,
close during read/write, guarded abort/no replay and repository preservation against
hashed development artifacts: [foreground reads](reports/M3_FOREGROUND_CANCEL_2026-11-02.md),
[native lifecycle](reports/M3_NATIVE_LIFECYCLE_2026-11-02.md),
[polling and inert review text](reports/M3_POLLING_2026-11-02.md).
The table combines these bounded results with the historical
[M3 automated evidence](validation/m3/README.md) and [IPC contracts](architecture/M3_DESKTOP_AND_IPC.md).
“Pass” remains bounded to the identified evidence; partial gates are not closed.

November 7 G01 follow-up: [report](reports/G01_REVIEW_AUTHORITY_2026-11-07.md)
adds actual native hostile-text exact publication, native decline/close, same-OID
branch rejection, review replacement and workspace-change cancellation evidence,
plus two Core regressions and expanded origin tests. A12–A15 remain partial;
outstanding-confirmation/reload/live-caller/unusual-filename and policy criteria
remain explicit in the report and current G01 tracker.


November 7 continuation: [G01/G02 lifecycle report](reports/G01_G02_CONFIRMATION_LIFECYCLE_2026-11-07.md)
adds unusual-filename SSH/direct-hook publication and fixes/retests outstanding
confirmation Quit, workspace invalidation/dialog reuse, main close and ordered
review replacement. A12–A17 remain partial for the remaining native renderer-reset,
live caller, compatibility and recovery criteria; full gate closure is not inferred.

Latest November 7 disposition: [G01 closure / G02 recovery](reports/G01_NATIVE_SUPPORT_G02_RECOVERY_2026-11-07.md)
adds real-webview denial/navigation probes, actual pending/issued-confirmation
reload, close-delivery fallback and native before/after-ref SIGKILL with separately
approved recovery. **G01 accepted within envelope**; overall M3 and G02 remain
partial. Earlier paragraphs describe historical checkpoints. G03/G06 criteria are
not waived by G01 closure.

| Gate | Criterion | Status and evidence |
|---|---|---|
| M3-A01 | Independent Core and thin desktop adapter | Pass: Core feature builds without Tauri/React; separate native adapter path dependency, no Core split/layout reorganization. |
| M3-A02 | Safe ordinary feature graph | Pass: normal graph excludes POC/crash features; default native write handlers disabled; no automation server or backend-evaluation endpoint. |
| M3-A03 | Single service/storage/approval owner | Pass at Core/adapter level: one `Application`/`ControlledWrites` owner, app-data lock contention/reopen tests, stable session and per-operation cancellation. Native lifecycle remains A17. |
| M3-A04 | Workspace/picker lifecycle | Partial: picker/scope criteria remain open. Active definition removal defect fixed and natively retested with repository preservation; see the M3 desktop progress report. |
| M3-A05 | Shared Overview selection | Pass in renderer: Table/Tree continuity, no descendant selection, tab close preservation and 100-entry filtering tests. Native visual acceptance remains A19. |
| M3-A06 | Tabs, drafts and restoration | Partial: tabs/drafts and geometry survived native restart. Shared bulk form state also survives view/restart. Immediate close/Quit paired draft persistence and save-failure recovery now passed ([close report](reports/M3_NATIVE_CLOSE_QUIT_2026-11-02.md)); mixed-DPI geometry/minimum restoration passed ([geometry report](reports/M3_MIXED_DPI_WINDOW_RESTORE_2026-11-02.md)). Full focus and display-reconfiguration criteria remain open. |
| M3-A07 | Changes/History fidelity | Selected native retests passed: Changes/History isolation and exact parent comparison. Stale-response regressions and actual foreground graph Cancel/reload pass. Full scale/accessibility acceptance remains open. |
| M3-A08 | Automatic monitoring and reconciliation | Pass at Core level with existing monitoring tests. Later user observations exercised edits, rename/move/missing restoration and inactive-workspace isolation, generally within about four seconds; no formal performance acceptance follows. |
| M3-A09 | Read-only Git boundary | Pass: M0 helper sentinels/state snapshots retained; scoped M3 reads, proposals, declined/replaced reviews preserve repository bytes. |
| M3-A10 | Actual command ACL separation | Pass under Tauri mock runtime with actual generated context: allowed main calls, denied main→review, review→dispatch and unprivileged calls; platform-specific origin checks. Not a live-OS approval test. |
| M3-A11 | Strict bounded input/scope | Pass: unknown/nested approval fields, oversize/version inputs, foreign IDs, invalid paths/picker/state, queues and dialog concurrency rejected. |
| M3-A12 | Independent human authorization | Accepted within G01 envelope: actual independent native approval, decline, expiry/context invalidation and pending/issued-confirmation reload passed across the November 7 reports. Actual main/review/unprivileged webviews rejected forbidden commands; wrong-page/origin navigations denied. Broader lifecycle/accessibility remains A17/G03. |
| M3-A13 | Exact trusted review | Accepted within G01 envelope: exact hostile-text message/index bytes, escaped unusual UTF-8 filename and signed publication independently verified; actual reload/replacement/context changes invalidate stale authority. See [closure report](reports/G01_NATIVE_SUPPORT_G02_RECOVERY_2026-11-07.md). Non-UTF-8 filesystem acceptance remains unproven. |
| M3-A14 | Trust, hooks and signing-policy presentation | Partial overall (G06): G01 presentation cases accepted using exact trust/hook/signer review and selected SSH/direct-hook evidence plus dated OpenPGP results. Broader compatibility and unsupported manager/interactive signer exclusions remain; no general support inferred. |
| M3-A15 | Desktop-controlled stage/unstage/commit/signing | Partial overall: selected native stage/unstage/index-only exact publication and isolated SSH/OpenPGP evidence retained; G01 fidelity accepted. Broader native compatibility, accessibility and ordinary artifact enablement remain G06/G03/G04. |
| M3-A16 | Results and guarded recovery UI | Partial: earlier hooks-running abort plus actual native SIGKILL at publication_pending/reference_updated, restart without replay, separately approved abort/index reconciliation passed. Published outcome freshly observed despite stale journal flag. See [recovery report](reports/G01_NATIVE_SUPPORT_G02_RECOVERY_2026-11-07.md). Remaining required phase-specific coverage stays G02. |
| M3-A17 | Last-window shutdown and cancellation | Partial: prior Cancel/close/read/write/draft retry and outstanding-sheet Quit/main-close passed. One suppressed close delivery exercised native fallback, Keep app open and successful retry with draft preserved; real reload cancelled issued confirmation. Actual main renderer-process loss now passed for auto-reload with pending/issued confirmation, saved draft, dialog reuse and restart/no replay. Direct AppKit termination now uses guarded shutdown; native fallback/Keep app open/retry, forwarding and restart passed ([fix report](reports/G02_GUARDED_APPKIT_TERMINATION_2026-11-07.md)). Active-sheet actual AppKit delivery now verifies safe deferral with no implicit approval, followed by native Cancel/fresh termination/restart ([report](reports/G02_ACTIVE_SHEET_TERMINATION_2026-11-07.md)); immediate sheet cancellation/exit is not accepted. Dock Quit remains policy blocked and session termination remains open. See [latest report](reports/G02_ACTUAL_RENDERER_LOSS_2026-11-07.md). |
| M3-A18 | State/migration/locking preservation | Pass at Core level: schema-2 backup/schema-3 migration, restore/newer-invalid state checks, known schema-3 reopen, atomic config bounds and independent duplicated-descriptor lock regression. |
| M3-A19 | Native accessibility/visual/lifecycle quality | Partial: selected native geometry/layout/focus observations and keyboard Return on initially focused Cancel review passed. Selected declared CSS contrast checks and native keyboard/AX/Light rendering now passed; see the [accessibility report](reports/M3_ACCESSIBILITY_2026-11-02.md). Interactive Light/Dark switching and Dark restart passed ([theme report](reports/M3_NATIVE_THEME_2026-11-02.md)); actual 1×/2× window restoration/minimum repair passed ([geometry report](reports/M3_MIXED_DPI_WINDOW_RESTORE_2026-11-02.md)). Full VoiceOver and remaining layout/scale/display-reconfiguration checks are open. |
| M3-A20 | Complete evidence and handoff | Partial: new reports contain durable scrubbed evidence, source/feature/binary hashes and fixture identities. Older observations remain historical; final release artifact and unfinished gates are not accepted. |

These gates address WS-002/003/011–015, UI-001–005, GIT-012, SEC-002/010/018/020,
STATE-001 and RELEASE-001 only at the boundaries stated above. Future bulk Git,
remote, graph/activity, AI/Actions and release requirements are tracked separately
in [execution status](EXECUTION_STATUS.md); M4 and selected M5 foundations now have
implementation evidence, while their remaining gates stay explicit.
An explicit set of trust grants does not implement M5 bulk Git execution.

M0/M1/M2 historical evidence and completion snapshots remain unchanged. The current
matrix includes the additional Core lock-ownership regression, monitor tests and
application integration tests. Skips remain explicit; none are counted as passes.
