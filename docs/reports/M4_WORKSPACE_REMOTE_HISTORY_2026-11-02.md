# Workspace remote observation history

November 2, 2026. Continued the completion plan after the native close/Quit and theme checkpoints.

Operation results previously queried only local stage/unstage/commit audit records. Existing remote observations were visible solely within each repository inspector. Added a workspace-wide Remote observations section using the existing bounded, scoped read-only Core endpoint. The local Git filters stay explicitly scoped to local records. Remote history displays its own phases, stable record ID, repository path, preparation time, redacted destination, observation time/OID or explicit absence and failure details. It states that these are past server observations and cannot establish synchronization.

Extracted shared observation rendering so repository and workspace views show the same outcomes. Workspace history loads on entry and native completion revision, supports existing Core pagination and explicit refresh, replaces prior pages on refresh and ignores superseded/unmounted responses. No new execution, retry or recovery endpoint, authority, network command, schema or Core policy was added. Epoch-zero timestamps render correctly rather than unknown time.

## Validation

Full desktop gate passed: 44 renderer tests, 10 native host tests under each default/all-feature configuration, typecheck, lint, generated contracts, formatting, production frontend/native build, strict Clippy in both configurations and release build. Focused tests verify workspace/null-worktree scope, pagination/refresh, inert hostile-looking text, phase/absence/error distinctions and superseded completion response rejection. App integration verifies the workspace-wide history request. Core code is unchanged, so its full signing/crash/transport matrix was not repeated.

Rebuilt and hashed ordinary bundle; native fixture received three explicitly synthetic historical records only (complete/absent, blocked/error, and orphaned running). Actual AX and screenshot showed all three in Operation results with correct interrupted projection, path/ID/times and failure details. Refresh and vertical scrolling worked. The previous real cancelled local operation remained separate; ordinary writes were disabled. Closed through native Quit. All four repositories retained identical file hashes/modes, one local operation remained, and the three synthetic stored records were not executed or rewritten. This is native presentation evidence, not actual native network observation acceptance. No server was contacted by these checks. Screenshots were inspected inline; no saved PNG is claimed.

Evidence: [full gate](../validation/m4-workspace-remote-history/desktop-validation.txt), [source/artifact](../validation/m4-workspace-remote-history/artifact.json), [synthetic fixture](../validation/m4-workspace-remote-history/native-history-fixture.json), [native summary](../validation/m4-workspace-remote-history/native-summary.json).

## Remaining scope

This closes workspace-wide visibility of existing remote observation records. It does not unify all event types into one filtered chronology or add full remote phase-event retention. Fetch/object/ref publication, authenticated transports, upstream changes, push/dependencies and native network execution remain unfinished. M3 broad accessibility/recovery/default-write and M6–M8 gates remain open. No push. Unrelated .gitignore and docs/graphics work remained unstaged.
