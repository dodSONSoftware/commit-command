# GitFleet — Decision Register and Open Items

**Version:** 1.0 · **Date:** November 1, 2026

This register separates **explicitly agreed product behavior** from **proposed implementation defaults** and unresolved architectural validation. It should be updated whenever a requirement is changed.

## Approved decisions

| ID | Area | Decision |
|---|---|---|
| D-001 | Name | GitFleet |
| D-002 | Architecture | Rust Core is independent of all presentation technology; Tauri/React is an adapter |
| D-003 | Workspaces | Multiple saved, one active, one root per workspace |
| D-004 | Discovery | Recursively discover nested repositories; classify independent/submodule/worktree relationships |
| D-005 | Overview | Flat table default + hierarchical tree; shared selection/filter model |
| D-006 | Bulk selection | Parent selection never recursively selects children |
| D-007 | Tabs | Permanent Overview and closeable repository tabs; state persisted |
| D-008 | Repository view | Adaptive Changes when dirty, History when clean; manual choice persists |
| D-009 | Staging | File-level first, hunk-level later |
| D-010 | Commit review | Mandatory for normal individual commits and bulk batches |
| D-011 | Bulk staging | Review all eligible staged/unstaged/untracked content before newly staging |
| D-012 | Commit dependencies | Commit selected submodules before selected parents; re-review changed gitlink |
| D-013 | Push | Separate confirmation; dependency-aware; guided missing-upstream setup |
| D-014 | Monitoring | Active workspace only, while process running; close last window quits |
| D-015 | Discovery changes | Auto detect new/deleted/moved/renamed repos while active |
| D-016 | Activity | Timeline + activity lanes + contribution heatmap; repo/workspace filters |
| D-017 | Activity scope | All local branches; timeline/lanes default 7d, heatmap 12mo |
| D-018 | History | Persistent Operation History and read-only Git-derived Workspace Activity |
| D-019 | Storage | Hybrid human-readable TOML + SQLite |
| D-020 | AI support | Configurable local connection/server/model/profile with editable system message |
| D-021 | AI profiles | Profile-owned system message; hierarchical app/workspace/repo profile assignment |
| D-022 | AI connection | Discover installed models from configured IP/hostname and dropdown selection |
| D-023 | AI diagnostics | Connection test + model inference test |
| D-024 | AI concurrency | Configurable global and per-connection limits |
| D-025 | AI failure | Retry/select another profile/cancel; no silent fallback; auto commit blocked |
| D-026 | AI templates | Structured editor; app/workspace/repo inheritance; persistent + one-off instructions |
| D-027 | AI preview | Live static preview + optional read-only real-context AI test |
| D-028 | AI context | Exact diff + relevant context + Git logs; hierarchical analysis for large diffs |
| D-029 | AI transparency | View actual supplied AI context and analysis coverage |
| D-030 | AI candidates | Up to three independently reviewed candidate commit messages |
| D-031 | AI automation | Assisted and user-initiated bounded automatic commits; never watcher-triggered |
| D-032 | GitFleet Actions | Deterministic typed pre-commit automation, first SemVer file update |
| D-033 | GitFleet Actions authorization | AI suggests; user approves; Rust executes; no unrestricted AI writes |
| D-034 | AI config portability | Import/export profiles, prompts, templates, instructions (no credentials) |
| D-035 | Backup | Manual backups plus change-aware automatic rotating local backups |
| D-036 | Trust | Each discovered repository individually trusted before writes; no inheritance |
| D-037 | Hooks | Hook inspection and bypass-all policy v1; selective controls later |
| D-038 | Hooks policy | Per-repo remembered execute/bypass preference, always presented at review |
| D-039 | Approved content | Final commit tree and message are immutable authorization boundaries |

## Proposed defaults / not explicitly finalized

| Topic | Proposal | Condition |
|---|---|---|
| Theme | System theme, dark used in design mockups | User may change |
| AI request concurrency | One globally and one per connection | User can configure |
| AI recent current-branch logs | 20 commits | Configurable; context-bounded |
| AI recent relevant file logs | Up to 10 commits | Configurable; context-bounded |
| AI sampling defaults | Conservative temperature around 0.2 | Profile-specific, capability-gated |
| Commit title maximum | 72 chars for Conventional Commits example | Template-configurable |
| Operation history retention | 90 days | Configurable |
| Automatic backups retained | Last 10 | Configurable |
| Initial GitFleet Action | Update SemVer file | Only approved action invoked |
| Bulk execution order | Sequential, stop on first failure | Suitable v1 default |

## Open technical validations before enabling production writes

1. Controlled hook invocation compatibility (including hook managers, config locations, context/stdin/args, Git reference-transaction hooks).
2. Correct selective index reconciliation with split/sparse/unmerged indexes and with linked worktrees.
3. Crash consistency and actual durability of journal state across operating system and filesystem failure modes.
4. Signing behavior using configured GPG and SSH signing, with positive verification on macOS.
5. Safe handling of Git SHA-256 object IDs and arbitrary filenames.
6. Trust identity invalidation for moved/replaced git directories, symlink and TOCTOU attacks.
7. Snapshot authorization and normal hook lifecycle integration without unsafe commit publication.
8. Submodule dependency graph semantics, commit/ref availability checks, and partial batch recovery.
9. Provider capabilities, endpoint authorization and sensitive-content filtering of AI requests.
10. SemVer modification/retry idempotence after crash and external edits.

## Product matters to confirm during implementation planning

- Product owner/organization ID, signing identity, license and distribution approach.
- Exact UX shortcuts, accessibility targets and desktop release OS minimums.
- Default SemVer bump strategy in templates that enable the Action.
- Which advanced Git operations (branch create/checkout, pull modes) are in initial delivery vs subsequent releases.
- Supported minimum Git version and supported hook frameworks for v1.

## Change-control rule

No new product capability is implicitly authorized by a mockup, imported template, AI output, or a development convenience. Add new behavior as a numbered decision with corresponding specification amendment and acceptance tests. The main spec prevails over this register if they temporarily diverge until reconciliation.
