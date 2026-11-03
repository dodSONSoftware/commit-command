# Commit Command branding and About dialog

November 3, 2026. Implemented the user's supplied Commit Command artwork and reference About layout. The visible application name is now **Commit Command**. The header's old GitFleet glyph/text/edition mark is replaced by a transparent theme-aware banner; it has a pointer cursor and opens a modal About dialog. The macOS About menu opens the same dialog.

## Appearance and behavior

The About box follows the reference structure: large supplied banner, centered “A dodson labs application” credit, divider, application name/current version, concise description, developer/publisher/copyright/license rows, and View license / GitHub repository / OK footer controls. It uses existing light/dark theme colors and remains scrollable within short windows. Light mode uses the supplied dark-letter artwork; dark mode uses the supplied light-letter artwork. System mode follows the OS preference. No artwork was redrawn or regenerated.

The dialog uses the browser's native modal behavior, including background isolation, keyboard focus containment and Escape dismissal. OK also dismisses it. Mouse-opened dialogs explicitly restore focus to the banner, accounting for macOS WebKit not necessarily focusing mouse-clicked buttons. Menu-opened dialogs restore the previously active control. About subscriptions are removed on unmount, including late subscription resolution. Version is read from the desktop package rather than a separate hardcoded About value.

The license view displays a packaged copy of the user's current MIT license as plain text, with Back to return to the metadata. The original untracked root `LICENSE` remains untouched and excluded from the commit. The repository button calls one main-window-only native action with the fixed project URL `https://github.com/dodSONSoftware/commit-command`; IPC accepts no URL or shell command. Review/unprivileged windows are denied this action. The renderer handles OS launch failure visibly. No arbitrary navigation or opener plugin permission was added.

## Application icons and compatibility

The supplied macOS `.icns` is configured in Tauri and copied into the development bundle's Resources with `CFBundleIconFile`. The supplied 128-pixel PNG is the native window icon and the 32-pixel PNG is the renderer favicon. The icon sync script now copies this approved artwork, so it will not regenerate the previous geometric icon. Visible window, application/menu/quit labels, review branding and README introduction use Commit Command.

The ordinary bundle is now `app/desktop/src-tauri/target/debug/bundle/macos/Commit Command.app`. The bundle builder reads product name and version from Tauri configuration. Existing `dev.gitfleet.desktop` identifier, executable/crate names, event/approval identifiers, app-data paths and saved workspace identities are preserved. Older development bundles were not deleted; use the new bundle to see this checkpoint. This is a local development build, not a notarized installer or release.

All source artwork under `docs/graphics/commit-command-brand-assets` is unchanged. Packaged SVG/PNG/icns copies were verified byte-identical. SVG inspection found only expected drawing/group/text/image elements, embedded PNG data and no scripts, event handlers or external image URLs.

## Validation

The full desktop gate passed **69 renderer tests** and **13 native host tests in each default/all-feature configuration**, plus typecheck, lint, contracts, formatting, renderer production build, native debug/release builds and strict Clippy. Three new About tests cover content/license/back/close/focus restoration, menu subscription cleanup and repository-launch failure. The existing label-injection regression now permits exactly the four static bundled banner images and still rejects event handlers, rather than assuming the entire app contains zero images. Initial typecheck/test failures are retained in the evidence directory; final complete gate passed. No Core code changed, so the full Core matrix was not rerun.

Actual native checks used only a fresh disposable four-repository workspace and the ordinary read-only build. Verified banner mouse activation, reference-style About layout/content, local MIT license and Back, OK/Escape dismissal, final banner focus restoration after Escape, macOS About menu activation, light and dark artwork appearance, renamed window/application/menu and native Cmd+Q. The exact final artifact was rebuilt after the focus refinement and retested. Native screenshots were inspected inline; selected accessibility observations are explicitly saved as transcribed excerpts, not a raw replayable trace. The pointer cursor is configured by CSS; no global system cursor setting changed. External browser launch was not invoked during native acceptance; its fixed callback/error behavior and window ACL were tested separately.

Independent file/mode/symlink snapshots of all four fixture repositories were unchanged. There were zero write or remote/fetch/pre-push-check operation records. Theme changes were scoped to disposable app state. The fixture app exited safely; no server, credentials, user repository mutation, global setting change or push was involved. The final ordinary bundle/icon hashes are saved and match the tested artifact.

Evidence: `docs/validation/commit-command-branding/validation-summary.json`, `native-artifact.json`, `asset-verification.json`, `source-manifest.json`, native excerpts, repository baseline and complete gate logs. User's original artwork and root LICENSE were preserved.
