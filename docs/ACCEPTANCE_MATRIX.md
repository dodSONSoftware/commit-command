# GitFleet — Acceptance Matrix and Evidence Ledger

**Version:** 1.0 · **Date:** November 1, 2026  
**Companion:** `GITFLEET_SPECIFICATION.md`

## How to read the status

- **POC-tested:** Behavior exercised by the existing Python labs / Rust v0.1.1 *in a constrained experimental subset*. Does not mean production ready.
- **Not yet tested:** Requirement accepted, but the production implementation and acceptance test remain outstanding.
- **Partial:** One necessary mechanism was tested; broader behavior remains unproven.

These status labels are assessment of known evidence, not promises of implementation. GitFleet Rust Core v0.1.1 was reported to compile on macOS/Git 2.56.0 with **24 acceptance tests + 6 core tests passing** and **23/24 diagnostic scenarios passing (one positive GPG test skipped)**. Each production test must independently validate its own outcome.

## 1. Workspace, discovery, monitoring and navigation

| ID | Acceptance criterion | Status | Target |
|---|---|---|---|
| WS-001 | Save named workspaces with exactly one root each | Not yet tested | M1 |
| WS-002 | Switch between multiple saved workspaces while only one monitors | Not yet tested | M1/M3 |
| WS-003 | Restore last active workspace and tabs after restart | Not yet tested | M3 |
| WS-004 | Discover normal Git repositories recursively | POC-tested | M1 |
| WS-005 | Discover independent repositories nested under parent repos | POC-tested | M1 |
| WS-006 | Recognize `.git` file / linked worktree marker | POC-tested | M1 |
| WS-007 | Classify submodule vs nested independent repository | Not yet tested | M1 |
| WS-008 | Canonicalize symlinked root without duplicate path or false failure | POC-tested | M1 |
| WS-009 | Avoid traversing symlinked directories by default | POC-tested | M1 |
| WS-010 | Respect configurable excluded directory patterns | Not yet tested | M1 |
| WS-011 | Detect add/remove/move/rename while active | Not yet tested | M3 |
| WS-012 | Report inaccessible/missing repo without treating it as empty | Not yet tested | M1/M3 |
| WS-013 | Monitor status changes from external tools without resetting UI state | Not yet tested | M3 |
| WS-014 | Quit and stop monitoring on last window close | Not yet tested | M3 |
| WS-015 | Compare canonical repo identity while showing friendly relative path | Partial | M1/M3 |
| UI-001 | Table/Tree share selection state, filters, and repository identity | Not yet tested | M3 |
| UI-002 | Selecting parent repository does not select children | Not yet tested | M3 |
| UI-003 | Overview permanent, repo tabs closable/restorable | Not yet tested | M3 |
| UI-004 | Adaptive Changes/History selection and manual view persistence | Not yet tested | M3 |
| UI-005 | Responsive resizable panels; light/dark themes and accessible controls | Not yet tested | M3/M8 |

## 2. Status, staging and controlled commits

| ID | Acceptance criterion | Status | Target |
|---|---|---|---|
| GIT-001 | Read-only status returns staged/unstaged/untracked accurately | POC-tested | M1 |
| GIT-002 | Stage full selected paths only (no unrelated paths) | Partial | M2 |
| GIT-003 | Distinguish staged/unstaged versions of same file | POC-tested | M2 |
| GIT-004 | Commit selected index content, preserving unrelated staged paths | POC-tested | M2 |
| GIT-005 | Commit selected worktree content, preserving unrelated staging | POC-tested | M2 |
| GIT-006 | Preserve partially staged file after index-only commit | POC-tested | M2 |
| GIT-007 | Handle selected worktree version of partially staged path | POC-tested | M2 |
| GIT-008 | Handle mixed index/worktree selections | POC-tested | M2 |
| GIT-009 | Handle untracked selected path and preserve excluded untracked | POC-tested | M2 |
| GIT-010 | Handle deletions, rename pairs and spaces in filenames | POC-tested | M2 |
| GIT-011 | Handle unusual filenames/non-UTF-8, binary and mode changes | Not yet tested | M2 |
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
| GIT-023 | Git log, diff, branch, tag and graph reflects real DAG | Not yet tested | M1/M4 |

## 3. Hooks, trust, transaction and crash recovery

| ID | Acceptance criterion | Status | Target |
|---|---|---|---|
| SEC-001 | Newly discovered untrusted repos cannot execute writes/hooks | Not yet tested | M2 |
| SEC-002 | User can approve trust for one or explicit set; no inherited grant | Not yet tested | M2/M3 |
| SEC-003 | Hook source/effective directory/executable status visible read-only | Not yet tested | M8 |
| SEC-004 | Hook introduces extra staged file → reject before publishing | POC-tested | M2 |
| SEC-005 | Hook rewrites `prepare-commit-msg` → reject before publishing | POC-tested | M2 |
| SEC-006 | Hook rewrites `commit-msg` → reject before publishing | POC-tested | M2 |
| SEC-007 | Hook changes selected worktree content → reject | POC-tested | M2 |
| SEC-008 | Hook modifies only excluded worktree file → preserve and report | POC-tested | M2 |
| SEC-009 | Hook rejection preserves real Git state; record error | POC-tested in Python v2; production needs Rust coverage | M2 |
| SEC-010 | Saved execute/bypass-all policy displayed in final review | Not yet tested | M2/M3 |
| SEC-011 | Bypass selected hook execution without changing original scripts | Not yet tested | M2 |
| SEC-012 | Unsupported hook managers/configurations block writes if needed | Not yet tested | M2/M8 |
| SEC-013 | Crash before reference update doesn't publish commit | POC-tested with real SIGKILL | M2 |
| SEC-014 | Crash after reference update recovers without duplicate commit | POC-tested with real SIGKILL | M2 |
| SEC-015 | Crash after reconciliation recognizes completion | POC-tested with real SIGKILL | M2 |
| SEC-016 | External index edit during recovery prompts manual reconciliation | POC-tested | M2 |
| SEC-017 | Durable journal tracks OIDs, phases, scope; secrets redacted | Partial | M2 |
| SEC-018 | No arbitrary command execution exposed through Tauri IPC | Not yet tested | M3 |
| SEC-019 | Repository path traversal/symlink boundary blocked | Partial (parent escape + discovery symlink tested) | M1/M2 |
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
| STATE-001 | TOML configuration and SQLite durable state with migrations | Not yet tested | M1/M3 |
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
| RELEASE-005 | Default test run never writes to actual developer repositories | POC-tested and required | All |

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
