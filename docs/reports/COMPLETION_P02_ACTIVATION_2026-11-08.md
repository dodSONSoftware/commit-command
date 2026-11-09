# Completion P02 initial native activation

The reproduced blank had a mounted, non-inert, normal-size shell and completed facade requests while WebKit reported hidden document visibility. A native menu/probe interaction made the existing content visible without a reload. Inspection of the pinned Tao 0.37.2 macOS implementation established the relevant distinction: window show makes the window key/orders it front, whereas focus additionally activates NSApplication.

The main host now explicitly focuses/activates the window after its first successful page-finished show. The existing one-time atomic guard remains: subsequent renderer reloads do not unhide or steal focus. No sleep, retry/reload loop, initialization bypass, IPC command or authorization change was added. Failure to show/focus remains an explicit error.

Bounded actual-native retest on a fresh ordinary default artifact: initial direct launch and two normal restarts showed usable visible workspace content immediately upon observation, with no native menu, fixed probe or reload intervention. Screenshot/AX observations are in the Computer Use transcript. All three Cmd+Q exits were zero. Independent repository/index/mode preservation, zero trust grants/write operations, SQLite integrity and separate state writer-lock reopen passed.

Default host tests: 20 passed/one subprocess worker ignored. All-feature host: 23 passed/one worker ignored. Native build, formatter and strict all-target/all-feature host Clippy passed. Evidence and exact artifact features/hash: `docs/validation/completion-p02-startup-2026-11-08/activation-*`.

This fixes the substantiated missing initial application activation path and adds selected ordinary startup/restart evidence. It does not claim universal startup reliability, first restart after host/renderer loss, Dock/session termination, full phase/accessibility acceptance or release readiness. Retained prior blank observations stay historical evidence; G02 remains partial for those distinct criteria. Managed Git writes remain disabled in ordinary builds.
