# Commit Command completion requirement coverage

Baseline `17ea585`, November 8, 2026. This is the execution routing appendix to [the runbook](COMPLETION_RUNBOOK.md) and [test catalog](COMPLETION_TEST_CATALOG.md). It covers every original requirement without marking new acceptance. Source/code ownership and historical evidence are also routed by [the existing crosswalk](../REQUIREMENT_EXECUTION_CROSSWALK.md).

Package `P13` means both P13A and P13B where applicable. Case ranges are inclusive. “Implemented foundation” requires retaining its current regressions and closing the named remaining criteria; it is not a request to reimplement accepted services. At execution, add exact test names/evidence and disposition to the live status record. Final installed acceptance is tied to the candidate hash, not this table's baseline.

## Original requirements

| Requirement | Criterion | Remaining packages | Required cases | Baseline and instruction |
|---|---|---|---|---|
| WS-001 | Save named workspaces with exactly one root each | P04/P13/P16 | UI03, RESTORE04, RELEASE03 | Implemented foundation; retain applicable evidence and close listed acceptance |
| WS-002 | Switch between multiple saved workspaces while only one monitors | P02/P03 | SCAN11, LIFE11 | Implemented foundation; retain applicable evidence and close listed acceptance |
| WS-003 | Restore last active workspace and tabs after restart | P02/P04 | LIFE01, LIFE03, UI03 | Implemented foundation; retain applicable evidence and close listed acceptance |
| WS-004 | Discover normal Git repositories recursively | P03 | SCAN01, SCAN05, SCAN08 | Implemented foundation; retain applicable evidence and close listed acceptance |
| WS-005 | Discover independent repositories nested under parent repos | P03 | SCAN01, SCAN05, SCAN08 | Implemented foundation; retain applicable evidence and close listed acceptance |
| WS-006 | Recognize `.git` file / linked worktree marker | P03 | SCAN01, SCAN05, SCAN08 | Implemented foundation; retain applicable evidence and close listed acceptance |
| WS-007 | Classify submodule vs nested independent repository | P03 | SCAN01, SCAN05, SCAN08 | Implemented foundation; retain applicable evidence and close listed acceptance |
| WS-008 | Canonicalize symlinked root without duplicate path or false failure | P03 | SCAN01, SCAN05, SCAN08 | Implemented foundation; retain applicable evidence and close listed acceptance |
| WS-009 | Avoid traversing symlinked directories by default | P03 | SCAN03, SCAN06 | Implemented foundation; retain applicable evidence and close listed acceptance |
| WS-010 | Respect configurable excluded directory patterns | P03 | SCAN03, SCAN06 | Implemented foundation; retain applicable evidence and close listed acceptance |
| WS-011 | Detect add/remove/move/rename while active | P03 | SCAN04, SCAN07, SCAN08, SCAN10 | Implemented foundation; retain applicable evidence and close listed acceptance |
| WS-012 | Report inaccessible/missing repo without treating it as empty | P03 | SCAN04, SCAN07, SCAN08, SCAN10 | Implemented foundation; retain applicable evidence and close listed acceptance |
| WS-013 | Monitor status changes from external tools without resetting UI state | P03/P04 | SCAN09, UI03 | Implemented foundation; retain applicable evidence and close listed acceptance |
| WS-014 | Quit and stop monitoring on last window close | P02 | LIFE03–LIFE10, RETAIN08 | Implemented foundation; retain applicable evidence and close listed acceptance |
| WS-015 | Compare canonical repo identity while showing friendly relative path | P03/P04 | SCAN05, SCAN08, UI01 | Implemented foundation; retain applicable evidence and close listed acceptance |
| UI-001 | Table/Tree share selection state, filters, and repository identity | P04/P05 | UI01, UI02, BULK03 | Implemented foundation; retain applicable evidence and close listed acceptance |
| UI-002 | Selecting parent repository does not select children | P04/P05 | UI01, UI02, BULK03 | Implemented foundation; retain applicable evidence and close listed acceptance |
| UI-003 | Overview permanent, repo tabs closable/restorable | P04 | UI03, GRAPH03 | Implemented foundation; retain applicable evidence and close listed acceptance |
| UI-004 | Adaptive Changes/History selection and manual view persistence | P04 | UI03, GRAPH03 | Implemented foundation; retain applicable evidence and close listed acceptance |
| UI-005 | Responsive resizable panels; light/dark themes and accessible controls | P03/P04 | UI08–UI12, PERF01–PERF06 | Partial; concrete layout/accessibility/scale implementation and acceptance remain |
| GIT-001 | Read-only status returns staged/unstaged/untracked accurately | P03/P04 | UI04, SCAN09, INF02 | Implemented foundation; retain applicable evidence and close listed acceptance |
| GIT-002 | Stage full selected paths only (no unrelated paths) | P04/P14/P16 | UI04–UI06, SEC04, SEC05, RELEASE06 | Controlled implementation exists in accepted support subset; preserve exact content regressions and validate enabled installed artifact |
| GIT-003 | Distinguish staged/unstaged versions of same file | P04/P14/P16 | UI04–UI06, SEC04, SEC05, RELEASE06 | Controlled implementation exists in accepted support subset; preserve exact content regressions and validate enabled installed artifact |
| GIT-004 | Commit selected index content, preserving unrelated staged paths | P04/P14/P16 | UI04–UI06, SEC04, SEC05, RELEASE06 | Controlled implementation exists in accepted support subset; preserve exact content regressions and validate enabled installed artifact |
| GIT-005 | Commit selected worktree content, preserving unrelated staging | P04/P14/P16 | UI04–UI06, SEC04, SEC05, RELEASE06 | Controlled implementation exists in accepted support subset; preserve exact content regressions and validate enabled installed artifact |
| GIT-006 | Preserve partially staged file after index-only commit | P04/P14/P16 | UI04–UI06, SEC04, SEC05, RELEASE06 | Controlled implementation exists in accepted support subset; preserve exact content regressions and validate enabled installed artifact |
| GIT-007 | Handle selected worktree version of partially staged path | P04/P14/P16 | UI04–UI06, SEC04, SEC05, RELEASE06 | Controlled implementation exists in accepted support subset; preserve exact content regressions and validate enabled installed artifact |
| GIT-008 | Handle mixed index/worktree selections | P04/P14/P16 | UI04–UI06, SEC04, SEC05, RELEASE06 | Controlled implementation exists in accepted support subset; preserve exact content regressions and validate enabled installed artifact |
| GIT-009 | Handle untracked selected path and preserve excluded untracked | P04/P14/P16 | UI04–UI06, SEC04, SEC05, RELEASE06 | Controlled implementation exists in accepted support subset; preserve exact content regressions and validate enabled installed artifact |
| GIT-010 | Handle deletions, rename pairs and spaces in filenames | P04/P14/P16 | UI04–UI06, SEC04, SEC05, RELEASE06 | Controlled implementation exists in accepted support subset; preserve exact content regressions and validate enabled installed artifact |
| GIT-011 | Handle unusual filenames/non-UTF-8, binary and mode changes | P04/P14/P16 | UI04–UI06, SEC04, SEC05, RELEASE06 | Controlled implementation exists in accepted support subset; preserve exact content regressions and validate enabled installed artifact |
| GIT-012 | Require user review for individual commit | P14 | SEC03, ENABLE03, RELEASE06 | Implemented foundation; retain applicable evidence and close listed acceptance |
| GIT-013 | Require user review and explicit batch selection for bulk commit | P05 | BULK01–BULK12 | Implemented foundation; retain applicable evidence and close listed acceptance |
| GIT-014 | Actual committed tree equals authorized tree | P02/P12/P14 | REC01–REC05, AUTO03, AUTO07, AUTO13, RELEASE06 | Implemented foundation; retain applicable evidence and close listed acceptance |
| GIT-015 | Actual commit message equals authorized message | P02/P12/P14 | REC01–REC05, AUTO03, AUTO07, AUTO13, RELEASE06 | Implemented foundation; retain applicable evidence and close listed acceptance |
| GIT-016 | External selected-worktree change invalidates approval | P02/P12/P14 | REC01–REC05, AUTO03, AUTO07, AUTO13, RELEASE06 | Implemented foundation; retain applicable evidence and close listed acceptance |
| GIT-017 | External index entry change invalidates approval | P02/P12/P14 | REC01–REC05, AUTO03, AUTO07, AUTO13, RELEASE06 | Implemented foundation; retain applicable evidence and close listed acceptance |
| GIT-018 | Stale HEAD prevents unsafe ref update | P02/P12/P14 | REC01–REC05, AUTO03, AUTO07, AUTO13, RELEASE06 | Implemented foundation; retain applicable evidence and close listed acceptance |
| GIT-019 | Actual index reconciling preserves unaffected staged/unstaged work | P02/P12/P14 | REC01–REC05, AUTO03, AUTO07, AUTO13, RELEASE06 | Implemented foundation; retain applicable evidence and close listed acceptance |
| GIT-020 | Unmerged/split/sparse indexes and linked worktree conflicts fail safe | P06/P14 | SEC05, ENABLE04, REMOTE11 | Keep bounded index/ref/config envelope and explicit refusal; no implicit advanced-index expansion |
| GIT-021 | Successful SSH and GPG signing on supported local config | P06/P16 | AUTO14, RELEASE06 | Positive signing exists; retain isolated positives and exact native/installed evidence |
| GIT-022 | Post-commit side effects are separate, visible and not misattributed | P02/P16 | REC03, RELEASE06 | Implemented foundation; retain applicable evidence and close listed acceptance |
| GIT-023 | Git log, diff, branch, tag and graph reflects real DAG | P04 | GRAPH01–GRAPH05 | Implemented foundation; retain applicable evidence and close listed acceptance |
| SEC-001 | Newly discovered untrusted repos cannot execute writes/hooks | P05/P14 | SEC03, BULK05, BULK12, ENABLE03 | Implemented foundation; retain applicable evidence and close listed acceptance |
| SEC-002 | User can approve trust for one or explicit set; no inherited grant | P05/P14 | SEC03, BULK05, BULK12, ENABLE03 | Implemented foundation; retain applicable evidence and close listed acceptance |
| SEC-003 | Hook source/effective directory/executable status visible read-only | P04/P06 | UI07, REMOTE12 | Implemented foundation; retain applicable evidence and close listed acceptance |
| SEC-004 | Hook introduces extra staged file → reject before publishing | P12/P16 | AUTO13, RELEASE06 | Existing controlled hook mutation refusals retained; compose through new automatic and installed paths |
| SEC-005 | Hook rewrites `prepare-commit-msg` → reject before publishing | P12/P16 | AUTO13, RELEASE06 | Existing controlled hook mutation refusals retained; compose through new automatic and installed paths |
| SEC-006 | Hook rewrites `commit-msg` → reject before publishing | P12/P16 | AUTO13, RELEASE06 | Existing controlled hook mutation refusals retained; compose through new automatic and installed paths |
| SEC-007 | Hook changes selected worktree content → reject | P12/P16 | AUTO13, RELEASE06 | Existing controlled hook mutation refusals retained; compose through new automatic and installed paths |
| SEC-008 | Hook modifies only excluded worktree file → preserve and report | P02/P05 | BULK06, REC03, REC06 | Implemented foundation; retain applicable evidence and close listed acceptance |
| SEC-009 | Hook rejection preserves real Git state; record error | P02/P05 | BULK06, REC03, REC06 | Implemented foundation; retain applicable evidence and close listed acceptance |
| SEC-010 | Saved execute/bypass-all policy displayed in final review | P04/P06 | UI07, REMOTE12, ENABLE04 | Implemented foundation; retain applicable evidence and close listed acceptance |
| SEC-011 | Explicit bypass-all for applicable local hooks without changing original scripts | P04/P06 | UI07, REMOTE12, ENABLE04 | Implemented foundation; retain applicable evidence and close listed acceptance |
| SEC-012 | Unsupported hook managers/configurations block writes if needed | P04/P06 | UI07, REMOTE12, ENABLE04 | Implemented foundation; retain applicable evidence and close listed acceptance |
| SEC-013 | Crash before reference update doesn't publish commit | P02 | REC01–REC03, LIFE10 | Implemented foundation; retain applicable evidence and close listed acceptance |
| SEC-014 | Crash after reference update recovers without duplicate commit | P02 | REC01–REC03, LIFE10 | Implemented foundation; retain applicable evidence and close listed acceptance |
| SEC-015 | Crash after reconciliation recognizes completion | P02 | REC01–REC03, LIFE10 | Implemented foundation; retain applicable evidence and close listed acceptance |
| SEC-016 | External index edit during recovery prompts manual reconciliation | P02 | REC04, REC05 | Implemented foundation; retain applicable evidence and close listed acceptance |
| SEC-017 | Durable journal tracks OIDs, phases, scope; secrets redacted | P09/P12/P13 | AUDIT01–AUDIT05, SEC07, REC08 | Implemented foundation; retain applicable evidence and close listed acceptance |
| SEC-018 | No arbitrary command execution exposed through Tauri IPC | P01/P15 | SEC01–SEC03, SEC06, SEC08 | G01 accepted within envelope; preserve/rerun relevant authority boundary after changes |
| SEC-019 | Repository path traversal/symlink boundary blocked | P01/P11/P13 | SEC05, ACT03, IMPORT01–IMPORT04 | Implemented foundation; retain applicable evidence and close listed acceptance |
| SEC-020 | Browser/Tauri adapter cannot bypass Rust authorization | P01/P15 | SEC01–SEC03, SEC06, SEC08 | G01 accepted within envelope; preserve/rerun relevant authority boundary after changes |
| BULK-001 | Eligible changes proposed without automatically staging them | P05 | BULK01, BULK09 | Implemented foundation; retain applicable evidence and close listed acceptance |
| BULK-002 | Unselected repos and nested child repos receive no writes | P05 | BULK01, BULK03, BULK05 | Implemented foundation; retain applicable evidence and close listed acceptance |
| BULK-003 | Shared commit message with per-repo overrides | P05 | BULK02 | Implemented foundation; retain applicable evidence and close listed acceptance |
| BULK-004 | Submodule changes committed before selected parent | P05 | BULK04, BULK10 | Implemented foundation; retain applicable evidence and close listed acceptance |
| BULK-005 | Parent gitlink change invalidates previous approval | P05 | BULK04, BULK10 | Implemented foundation; retain applicable evidence and close listed acceptance |
| BULK-006 | Partial batch failure stops pending by default; preserves success | P05 | BULK06–BULK08, BULK11, BULK12 | Implemented foundation; retain applicable evidence and close listed acceptance |
| BULK-007 | Retry incomplete batch performs fresh validation | P05 | BULK06–BULK08, BULK11, BULK12 | Implemented foundation; retain applicable evidence and close listed acceptance |
| PUSH-001 | Push requires separate review and cannot follow commit implicitly | P07/P12 | PUSH01, AUTO19, ENABLE06 | Implemented foundation; retain applicable evidence and close listed acceptance |
| PUSH-002 | Guided upstream setup confirms remote and destination | P06 | REMOTE10, REMOTE11 | Implemented foundation; retain applicable evidence and close listed acceptance |
| PUSH-003 | Diverged remote or missing upstream is correctly blocked | P06/P07 | REMOTE01, PUSH03, PUSH04 | Implemented foundation; retain applicable evidence and close listed acceptance |
| PUSH-004 | Submodule pushed/verified before dependent selected parent | P08 | QUEUE01–QUEUE05 | Implemented foundation; retain applicable evidence and close listed acceptance |
| PUSH-005 | No bulk force push or unexpected destination creation | P07/P08 | PUSH03–PUSH09, QUEUE06–QUEUE08 | Implemented foundation; retain applicable evidence and close listed acceptance |
| PUSH-006 | Remote changes between fetch/review/push fail safely | P07/P08 | PUSH03–PUSH09, QUEUE06–QUEUE08 | Implemented foundation; retain applicable evidence and close listed acceptance |
| AI-001 | Named connections, endpoint validation and auth stored securely | P09/P13 | AI15, AI16, IMPORT06 | Implemented foundation; retain applicable evidence and close listed acceptance |
| AI-002 | Discover Ollama models and select via searchable dropdown | P09 | AI05, AI06, AI17 | Implemented foundation; retain applicable evidence and close listed acceptance |
| AI-003 | Test connection and inference independently, no repo modification | P09 | AI06, LIFE12 | Implemented foundation; retain applicable evidence and close listed acceptance |
| AI-004 | Profiles own editable system messages and generation parameters | P09 | AI02, AI03, AI08 | Implemented foundation; retain applicable evidence and close listed acceptance |
| AI-005 | Hierarchical profile assignment with one-off override | P09 | AI04 | Implemented foundation; retain applicable evidence and close listed acceptance |
| AI-006 | Global/per-connection concurrency limits respected | P09/P10 | AI18, HIER09 | Implemented foundation; retain applicable evidence and close listed acceptance |
| AI-007 | Missing model causes retry/select/cancel; no silent fallback | P09/P12 | AI09, AI10, AUTO06 | Implemented foundation; retain applicable evidence and close listed acceptance |
| AI-008 | Structured templates and instructions inherit with effective preview | P09 | AI02–AI04, AI07 | Implemented foundation; retain applicable evidence and close listed acceptance |
| AI-009 | Static preview deterministic; AI test is read-only | P09/P11 | AI06, AI07, ACT11 | Read-only unsaved-editor repository test implemented with selected native evidence; simulated version preview and complete acceptance remain |
| AI-010 | Context includes bounded diff/source/docs/relevant Git logs | P09/P10 | AI11, AI12, HIER01–HIER06 | Implemented foundation; retain applicable evidence and close listed acceptance |
| AI-011 | Hierarchical large-diff analysis reports omissions and provenance | P10 | HIER01–HIER14 | New hierarchical multi-request implementation required |
| AI-012 | Exactly actual sent context inspectable after filtering/redaction | P09/P10 | AI11, HIER05, HIER07, HIER08 | Implemented foundation; retain applicable evidence and close listed acceptance |
| AI-013 | Up to 3 meaningful candidate messages; verified template compliance | P09/P10 | AI13, AI14, AI17, HIER14 | Assisted structure exists; semantic corpus and meaningful distinction/hierarchy acceptance remain |
| AI-014 | Explicit AI Automatic run respects bounded authorization | P12 | AUTO01–AUTO20 | New bounded automatic authority/coordinator required |
| AI-015 | No watcher-triggered AI commit and no automatic push | P12 | AUTO19, ENABLE06 | Implemented foundation; retain applicable evidence and close listed acceptance |
| AI-016 | Prompt injection cannot authorize Git writes/Actions | P09/P10/P12 | AI13, HIER13, AUTO02, AUTO18 | Implemented foundation; retain applicable evidence and close listed acceptance |
| AI-017 | AI endpoint approval prevents unintended source transfer | P09/P10/P12 | AI11, AI15, HIER07, HIER08, AUTO08 | Implemented foundation; retain applicable evidence and close listed acceptance |
| ACT-001 | SemVer file action previews old/new without writing | P11 | ACT01–ACT03, ACT11 | Implemented foundation; retain applicable evidence and close listed acceptance |
| ACT-002 | SemVer updates file only after approved action invocation | P11/P12 | ACT04–ACT06, ACT09–ACT11, AUTO11 | Implemented foundation; retain applicable evidence and close listed acceptance |
| ACT-003 | Idempotent retry prevents double-increment | P11/P12 | ACT07, ACT08, AUTO12, AUTO17 | Implemented foundation; retain applicable evidence and close listed acceptance |
| ACT-004 | Imported Action definitions require explicit reauthorization | P11/P13 | ACT05, IMPORT06 | New persistent definition/import reauthorization integration required |
| STATE-001 | TOML configuration and SQLite durable state with migrations | P13/P16 | BACKUP03, BACKUP11, RESTORE05–RESTORE08, RELEASE03 | Implemented foundation; retain applicable evidence and close listed acceptance |
| STATE-002 | Git-derived activity includes local branches and dedupes per repo | P03/P04 | GRAPH04, GRAPH05, PERF04 | Implemented foundation; retain applicable evidence and close listed acceptance |
| STATE-003 | Timeline/lanes seven-day default; heatmap twelve-month default | P03/P04 | GRAPH04, GRAPH05, PERF04 | Implemented foundation; retain applicable evidence and close listed acceptance |
| STATE-004 | Persistent Operation History records real per-repo results | P09/P12/P13 | AUDIT01–AUDIT05, RETAIN05–RETAIN07 | Existing Git/Action audit; persistent safe AI/run producers and retention remain |
| STATE-005 | Config import/export validates archive and strips credentials | P13 | BACKUP01, IMPORT01–IMPORT08 | General portability implementation required |
| STATE-006 | Automatic rotating local config backups, no background daemon | P13 | RETAIN01–RETAIN04, RETAIN08 | General automatic backup implementation required |
| STATE-007 | Consistent SQLite backup and safe restore with preview | P13 | BACKUP02–BACKUP12, RESTORE01–RESTORE12 | General consistent backup/previewed restore implementation required |
| RELEASE-001 | Presentation can be replaced without recompiling domain logic | P15/P16 | SEC01, SEC08, CI05 | Preserve Core independence and typed adapters; final artifact validation required |
| RELEASE-002 | macOS development and packaging flows tested end-to-end | P14/P15/P16 | ENABLE01–ENABLE06, CI01–CI06, LICENSE01–LICENSE04, RELEASE01–RELEASE12 | Distribution implementation and exact artifact acceptance required |
| RELEASE-003 | Supported Linux platform integration passes | P00/P16 | DOC03, RELEASE12 | Explicitly deferred by D-040 for macOS first release; do not claim support or implement implicitly |
| RELEASE-004 | Windows compatibility validated before claiming support | P00/P16 | DOC03, RELEASE12 | Explicitly deferred by D-040 for macOS first release; do not claim support or implement implicitly |
| RELEASE-005 | Default test run never writes to actual developer repositories | P01/P15 | INF01–INF08, CI01–CI05 | Preserve disposable-only fixture invariant across every new suite |

## Release invariants

All invariant cases also apply to the exact installed artifact in P16. Invariants are required properties, not optional features.

| Invariant | Required property | Packages | Required cases |
|---|---|---|---|
| INV-001 | Untrusted repository cannot write or execute hooks | P05/P14/P16 | BULK05, BULK12, ENABLE03, RELEASE06 |
| INV-002 | Actual published commit tree equals exact authorized tree | P02/P05/P12/P16 | REC02, BULK01, AUTO03, AUTO13, RELEASE06 |
| INV-003 | Actual published message equals exact authorized message | P05/P12/P16 | BULK02, AUTO03, AUTO13, RELEASE06 |
| INV-004 | Selected content/index/HEAD drift invalidates approval | P02/P07/P12 | REC05, PUSH03, AUTO07, AUTO08 |
| INV-005 | Partial staging/exclusions/untracked content survive | P04/P05/P12/P16 | UI04, BULK01, BULK09, AUTO03, RELEASE06 |
| INV-006 | Unsupported hooks/config fail closed before unsafe publication | P06/P12/P14 | REMOTE12, AUTO13, ENABLE04 |
| INV-007 | Uncertain operation is never silently replayed | P02/P07/P08/P12/P13 | LIFE10, REC01–REC08, PUSH07–PUSH09, QUEUE08, AUTO17, RESTORE10 |
| INV-008 | Hook bypass is explicit and audited | P04/P06/P16 | UI07, REMOTE12, RELEASE06 |
| INV-009 | Commit/AI does not implicitly push | P07/P08/P12/P16 | PUSH01, AUTO18, AUTO19, RELEASE08 |
| INV-010 | AI text has no filesystem/shell/Git authority | P09/P10/P12 | AI13, AI14, HIER13, AUTO02, AUTO18 |
| INV-011 | Action modifications are previewed and bounded | P11/P12 | ACT01–ACT11, AUTO11, AUTO12 |
| INV-012 | Imported Actions disabled until local reauthorization | P11/P13 | ACT05, IMPORT06, RESTORE10 |
| INV-013 | Outbound source only to an approved AI destination | P09/P10/P12 | AI11, AI15, HIER07, HIER08, AUTO08 |
| INV-014 | Bulk dependencies cannot be violated unsafely | P05/P08/P12 | BULK03–BULK05, BULK10, QUEUE01–QUEUE08, AUTO15 |
| INV-015 | Presentation replaceable without changing domain logic | P15/P16 | SEC01, SEC08, CI05, RELEASE12; inspect Core dependency graph and build headless Core without Tauri |

## M3 desktop criteria

G01 is accepted within its recorded envelope; relevant authority tests rerun when a changed boundary requires them. This mapping closes remaining criteria without reopening every historical pass.

| Criterion | Execution disposition | Packages and cases |
|---|---|---|
| M3-A01 | Preserve independent Core/thin adapter | P15/P16; INV-015, SEC01, CI05 |
| M3-A02 | Ordinary feature graph and no test facilities | P01/P14; INF05, ENABLE02, SEC08 |
| M3-A03 | One storage/approval owner and lock continuity | P01/P02/P13; INF08, REC08, RESTORE06 |
| M3-A04 | Workspace/picker lifecycle residual native scope | P02/P04; UI03, LIFE11, SEC02 |
| M3-A05 | Shared Overview explicit selection | P04; UI01, UI02 |
| M3-A06 | State/draft/focus/display continuity | P02/P04; LIFE01, LIFE03, LIFE04, UI03, UI08, UI12 |
| M3-A07 | Changes/History exactness and stale responses | P04; UI04, UI05, GRAPH01–GRAPH03 |
| M3-A08 | Active-only monitoring and reconciliation | P03; SCAN07–SCAN11, PERF05 |
| M3-A09 | Read-only boundary remains byte preserving | P01/P03; INF02, SCAN02, REMOTE01 |
| M3-A10 | Actual command ACL/caller | P15; SEC01, SEC08; retain G01 actual-webview evidence |
| M3-A11 | Strict bounded scope/input | P15; SEC02, SEC05, SEC06 |
| M3-A12 | Independent human approval | P14/P15; SEC03, ENABLE03; accepted G01 envelope retained |
| M3-A13 | Exact hostile-content review | P14/P15; SEC03, SEC04, RELEASE06; accepted G01 envelope retained |
| M3-A14 | Trust/hooks/signing presentation/compatibility | P04/P06; UI07, REMOTE05, REMOTE12, AUTO14 |
| M3-A15 | Native stage/unstage/commit/signing supported scope | P06/P14/P16; ENABLE03–ENABLE06, RELEASE06 |
| M3-A16 | Results and guarded phase recovery | P02; REC01–REC08 |
| M3-A17 | Last-window close/cancel/OS termination | P02; LIFE01–LIFE12 |
| M3-A18 | Migration/locking/data preservation | P13; BACKUP11, RESTORE05–RESTORE08 |
| M3-A19 | Native accessibility/visual/scale | P03/P04; UI08–UI12, PERF01–PERF06 |
| M3-A20 | Durable evidence and exact artifact handoff | P00/P01/P16; DOC01–DOC03, INF04, RELEASE12 |

## Acceptance updates during implementation

For each original ID and invariant, attach: selected support envelope, implemented source, exact automated test names, exact native/installed case evidence where required, retained evidence rationale, open cases and decision/deferral authorization. Use `accepted`, `partial`, `blocked`, `not implemented`, or `approved deferral/exclusion`; never infer acceptance from a package title or total test count.
