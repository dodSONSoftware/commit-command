# Foreground read cancellation checkpoint

November 2, 2026. Execution resumed at the user's request. Scope: fix the native finding from the previous stop handoff without changing Core authority, ordinary write defaults or Git behavior.

## Change

App counts pending foreground actions rather than storing one boolean. Overlapping actions keep Cancel available until all settle, including failures and the cancellation request itself. Graph initial/reload/paging requests and repository preview/commit/diff effects now participate. Existing stale-response guards remain. Graph reload/paging controls stay disabled while a graph request is pending; background monitoring and debounced preference saves remain separate from foreground actions.

## Validation

The full desktop validator passed: TypeScript, lint, generated contracts, format, 30 renderer tests, frontend production build, eight native host tests under each default/all-feature configuration, native builds, strict Clippy and release build. A new regression holds graph and commit reads concurrently, resolves the graph, requests cancellation, and checks Cancel remains enabled until the remaining read settles with an error. Existing Changes/History stale-response regression passes. The full Core Git/signing matrix was not rerun because this patch changes only presentation tracking.

Native ordinary default bundle: reused isolated fixture `/private/tmp/gitfleet-m3-native-35cm8or4`, with no user repositories or real network activity. A controlled Git wrapper held an actual graph `rev-list`; Cancel was visibly enabled, then clicked. The graph reported `Inspection cancelled`; process inspection confirmed wrapper PID 47280 and sleep PID 47281 both terminated. The independently queued commit comparison completed. Removing the test delay and clicking Reload graph displayed both actual fixture commits. Screenshot and AX were inspected inline; no saved PNG is claimed. Last-window close exited the app, and the CLI reopened its state lock. All four repository snapshots matched exactly before/after (file hashes, modes and symlink targets).

Evidence: [desktop gate](../validation/m3-foreground-cancel/desktop-validation.txt), [artifact](../validation/m3-foreground-cancel/artifact.json), [source hashes](../validation/m3-foreground-cancel/source-sha256.json), [held processes](../validation/m3-foreground-cancel/processes-running.json), [after Cancel](../validation/m3-foreground-cancel/processes-after-cancel.json), [before](../validation/m3-foreground-cancel/repositories-before.json), [after](../validation/m3-foreground-cancel/repositories-after.json), [state lock reopened](../validation/m3-foreground-cancel/cli-after-close.json).

## Remaining work

This closes the specific foreground Cancel UI finding; M3 remains incomplete. Next native checks are separate review focus/decline/close, close during a controlled write, forced termination/recovery, hostile contexts and accessibility. Ordinary writes stay disabled. M5 fetch/authenticated transport/push and M6–M8 remain open. No push; the pre-existing `.gitignore` change is preserved unstaged.
