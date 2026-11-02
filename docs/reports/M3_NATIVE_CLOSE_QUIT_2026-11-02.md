# Native close and Quit acceptance

November 2, 2026. Completed the pending close-draft checkpoint using disposable fixtures only. Ordinary desktop writes remain disabled. No push.

## Defect and fix

Real native testing found that immediate window close retained paired drafts but Cmd+Q lost the latest edit. The pinned Tauri default menu creates a predefined Quit item; its pinned muda implementation maps directly to AppKit `terminate:`, bypassing the existing ExitRequested shutdown handler. Replaced only the macOS application submenu with standard About/Services/Hide items and an app-owned Cmd+Q item that invokes the same shutdown handshake as primary-window close. The other default menus remain intact. No Core, renderer, feature-default or authority changes.

## Actual native evidence

- Original artifact hash matched the previous checkpoint. Window close was requested 183 ms after input, before the 500 ms debounce. It exited without a warning and restored exact individual and bulk drafts on restart.
- The original immediate Cmd+Q trial (198 ms) lost the latest bulk draft. This failure prompted the fix; it is not counted as passing evidence.
- Rebuilt ordinary artifact: Cmd+Q requested 199 ms after input saved the exact individual draft; restart visibly restored it. Paired bulk state remained intact.
- Fixture-only SQLite INSERT failure trigger caused the native Unsaved drafts dialog on both close (original artifact) and Quit (fixed artifact). Keep app open retained the latest draft, cleared inert editing, and allowed another edit. Removed only the injected trigger, retried close and verified the exact recovered bulk value in SQLite. The existing automated Core trigger test remains the atomic rollback oracle; the native run does not claim to isolate rollback from earlier debounce saves.
- Fixed ordinary artifact closed during a proven graph rev-list wrapper PID 13930 and sleep PID 13931. Both exited. CLI reopened the state writer lock; all four repositories, including Git metadata, retained identical file hashes and modes.
- Separate current acceptance artifact with desktop-writes and fresh fixture: explicitly reviewed/trusted engine, reviewed an index-only README commit, approved the native dialog, and proved pre-commit PID 16086/sleep 16087 running. Cmd+Q from the review window cancelled the owned work and exited. Durable operation ec676ecc-2705-4a33-a3ad-6ff7f395adea is cancelled_before_publication, published false. Every repository file/hash/mode matched baseline. Restart visibly restored the exact draft and cancellation result; the journal still contained one operation, with no replay.

AX and native dialogs were inspected inline; no saved screenshot is claimed. Provenance, exact persisted snapshots, process records, preservation snapshots and summary are under [validation](../validation/m3-close-native-run/). No database, trust capability or credential material is committed.

## Validation and remaining gates

Full desktop gate passed: 42 renderer tests, 10 native host tests in each default/all-feature configuration, typecheck, lint, generated contracts, format, production frontend/native builds, strict Clippy in both configurations and release build. Separate acceptance build passed. Core behavior did not change; its full signing/crash matrix was not rerun for this native-menu fix.

This accepts the selected immediate close/Quit, failure/retry, held-read and pre-publication-write shutdown cases. Missing-renderer fallback, Dock/system terminate behavior, broader publication crash phases, full VoiceOver/layout/scale, final packaged artifact and ordinary-write default decision remain open. M3 and the overall application remain incomplete. User .gitignore and docs/graphics work was preserved and excluded.
