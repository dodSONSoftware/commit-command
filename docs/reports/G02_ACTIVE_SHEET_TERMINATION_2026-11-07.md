# G02 direct AppKit termination with unanswered confirmation

This step resolves the missing-delivery evidence from
[the preceding native termination fix](G02_GUARDED_APPKIT_TERMINATION_2026-11-07.md).
**Selected safety/deferral behavior is now verified:** on this macOS/AppKit host,
a direct terminate call with a native trust or commit sheet attached returns
without entering the application termination delegate. The app and unanswered
sheet remain open; nothing is implicitly approved. After explicit Cancel and a
fresh termination request, guarded shutdown completes normally.

This does **not** accept immediate native shutdown/sheet cancellation on that first
request, actual Dock UI, or OS session termination. G02 overall remains open.
No production lifecycle, approval, IPC/ACL or ordinary write-default change was made.

## Test facility and actual delivery

The prior fixed test action scheduled NSApplication terminate with delay zero but
provided no dispatch audit. Its lack of delegate interception did not establish
whether AppKit received the call. Added a feature-only main-thread NSObject request
that schedules its fixed `fire:` selector in default, modal and event-tracking
run-loop modes, after Tauri releases its event callback lock. At actual delivery it
records the current run-loop mode and whether any owned native window has an attached
sheet, calls `NSApplication.terminate(None)`, then records return when applicable.
Foundation retains the scheduled target through delivery; no caller-selected
selector, argument, PID or execution string is introduced.

The small optional macOS objc2-foundation dependency uses the already locked 0.3.2
version and is selected only by `native-acceptance-support`. The fixed menus remain
available only after the marked disposable-fixture startup guard. No private
termination API, production delegate bypass or alternate UI automation was used.

Both active-sheet calls actually dispatched in `kCFRunLoopDefaultMode` with
`attached_sheet: true`. Thus run-loop starvation was an initial hypothesis, **not
a confirmed explanation** for the earlier result. The new evidence establishes
that AppKit received and returned from the call without invoking the production
termination-decision callback. No internal AppKit cause is asserted beyond that
observed boundary. The subsequent sheetless call dispatched with attached_sheet
false, entered the existing delegate and exited cleanly.

## Native observations and independent checks

All UI actions used Computer Use in a new, separate bundle/state containing four
real disposable repositories. The [evidence record](../validation/g02-active-sheet-2026-11-07/README.md)
contains exact fixture/source/artifact identifiers, audit and read-only oracles.
Native AX states were inspected; no saved screenshots are claimed.

| Case | Observed result |
|---|---|
| Unanswered trust confirmation | Opened authoritative engine trust review and actual independent OS sheet. Native terminate dispatched with attached sheet true and returned; delegate was not called, sheet remained, and UI stayed responsive. Zero trust rows/operations and every repository byte/mode unchanged. Explicit native Cancel then reported declined. |
| Separate fixture trust | Fresh review plus explicit native Approve granted only engine trust. This is independent of the earlier termination request. |
| Unanswered commit confirmation | Selected only index README and opened exact commit review/OS confirmation without approving it. Native terminate again dispatched with attached sheet true and returned without delegate interception. Commit record remained proposed/unpublished, one explicit trust, all repository bytes/modes preserved. |
| Explicit commit decline | Native Cancel recorded cancelled_before_publication, unpublished. No hook/commit ran and no extra operation was created. |
| Fresh termination after dismissal | Closed the consumed review, then issued a new native terminate request. Audit recorded sheetless dispatch, production delegate interception and return; the existing guarded shutdown exited 0. No delayed replay of the earlier active-sheet requests was observed. |
| Restart | Reopened exact app/state successfully. Saved draft restored in native UI; one cancelled unpublished operation and one explicit engine trust remained, repositories unchanged. Ordinary Cmd+Q cleanup exited 0. |

Some Computer Use actions reported a user-change interruption. They were refreshed
and retried; no approval was inferred from an interrupted call. The generated commit
review was selected through the native Window menu before confirmation; it existed
and displayed the exact expected proposal. No missing-window failure is claimed.

## Validation and gate disposition

Complete final desktop validation passed: **69 renderer tests, 13 default host /
16 all-feature host tests**, TypeScript, ESLint, generated contracts, frontend
format/build, Rust formatting, strict Clippy in both configurations, native debug/
release builds. Default dependency graph excludes Core test features; ordinary
executables exclude the test request class, dispatch audit and private WebKit PID
selector. Acceptance executable still refuses nonfixture state. No Core transaction
source changed; full Core/signing/remote/release matrices were not rerun.

The active-sheet evidence gap is closed for **verified safe deferral, explicit
decline, fresh termination retry and restart without replay**. Preserve the distinction:
the first request does not cancel the OS sheet or shut down immediately. Should
immediate exit during native confirmation become a product requirement, it needs
a separate supported design and acceptance; this test does not waive such a criterion.

Remaining G02 queue: actual Dock Quit (organization-policy blocked), operator-controlled
session logout/restart, and further active-execution interruption combinations
shared with G06 (running remote-hook checks). Earlier custom Cmd+Q/main-close,
renderer loss, missing-delivery fallback, hooks-running abort and native
publication-boundary recovery retain their exact documented bounds. No logout,
reboot, production service, user repository or external publication was involved.
