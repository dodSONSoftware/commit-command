# Interactive native theme acceptance

November 2, 2026. Continued the M3 theme-selector gate after native close/Quit acceptance. No source change was needed.

On the disposable write fixture from [close/Quit validation](../validation/m3-close-native-run/write-fixture.json), opened the actual Color theme popup. AX exposed System, Dark and Light. A direct AX option click did not change the selection; the driver reported that refreshed state was needed. After refreshing, native End then Return selected Light. The AX value became Light and the screenshot showed actual Light rendering. Reopened the popup, used Home, Down, Return, and AX became Dark. Closed through Cmd+Q and independently verified persisted desktop_state.theme was dark.

Restarted the ordinary default-feature bundle on the same disposable state. AX restored Dark and the screenshot showed the actual Dark repository/composer rendering with the saved draft intact. Ordinary write controls remained disabled. Closed normally. All repository file hashes and modes matched the write fixture baseline; no operation was replayed. Screenshot/AX observations were inline; no saved PNG claimed.

Evidence: [persisted theme](../validation/m3-theme-native/after-interactive-theme.json), [preservation](../validation/m3-theme-native/preservation.json). Binary/source hashes and full desktop validation are those in [close/Quit evidence](../validation/m3-close-native-run/). No source changes, so no duplicate automated gate was run.

This accepts selected interactive System-to-Light-to-Dark switching and Dark restart restoration; it supersedes the earlier unaccepted theme interaction result. It does not establish live system-theme change handling, every mouse/assistive interaction, full VoiceOver or layout/scale acceptance. M3 and the application remain incomplete, and ordinary writes stay disabled. No push. Unrelated .gitignore and docs/graphics work remained untouched.
