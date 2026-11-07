# G02 — actual main renderer-process loss

November 7, 2026. **Selected actual renderer-loss cases passed; G02 remains open.**
This follows the [publication-boundary recovery checkpoint](G01_NATIVE_SUPPORT_G02_RECOVERY_2026-11-07.md).
The current authoritative gate disposition remains [EXECUTION_STATUS](../EXECUTION_STATUS.md).

## Scope and test support

Extended only the non-default `native-acceptance-support` facility with a fixed
native menu action, **Terminate actual main renderer process**. It obtains the
live owned main WKWebView's renderer PID on its main thread and sends SIGKILL to
that PID through fixed `/bin/kill` arguments. The renderer supplies no PID, command,
selector or signal. PID zero, negative/group/system PID and host PID are refused;
a focused regression covers that guard. The existing marked disposable-fixture
startup/workspace guard still applies. No new IPC/ACL or approval capability exists.

The macOS-only optional `objc2` dependency pins the already locked transitive
0.6.5 version. WebKit's private `_webProcessIdentifier` selector is checked at
runtime, with signature verified against the [WebKit header](https://github.com/WebKit/WebKit/blob/main/Source/WebKit/UIProcess/API/Cocoa/WKWebViewPrivate.h).
This private lookup belongs only to test builds, not a supported production API
or release entitlement. Unavailable selectors or unsafe PIDs record a refusal.

Inspection of pinned `tauri-runtime-wry` showed its default content-process
termination callback reloads the affected webview. The product's existing
main-page reset path then invalidates Core presentation authority and closes the
obsolete native review/sheet. No production lifecycle change was needed here.
The tests do not suppress acknowledgements or call that invalidation path manually.

## Actual native results

All UI actions used Computer Use in the exact separate fixture bundle/state from
[fixture metadata](../validation/g02-renderer-loss-2026-11-07/fixture.json).
Native AX observations were inspected; no saved screenshots are claimed.

| Case | Native observation and independent evidence |
|---|---|
| Persist draft before fault | Entered `G02 actual renderer loss preserves saved draft`; independently read the exact persisted desktop-state draft before termination. No claim covers keystrokes not yet persisted. |
| Pending unapproved trust review | Opened authoritative engine trust review without confirming it. Fixed native action killed renderer PID 17908; native host PID 17901 survived. Independent `ps` found the renderer absent while the exact host remained. Tauri reloaded main and the obsolete review closed. Saved draft reappeared. Zero trust rows, zero operations and all four repositories' bytes/modes unchanged. |
| Fresh native trust | New review/OS confirmation after recovery explicitly granted only the fixture engine. No commit was approved. |
| Issued commit confirmation | Selected only index README and opened the real OS commit confirmation without approving it. Native action killed the replacement main renderer PID 18478. Host remained responsive, main reloaded, and the old review/sheet disappeared. The one proposed commit became `cancelled_before_publication`, unpublished. Saved draft reappeared; all four repositories remained byte/mode identical to baseline. |
| Dialog lease reuse | A new trust review opened a real independent OS confirmation after the second crash. Native Cancel produced `Native confirmation was declined`; no additional grant or operation resulted. |
| Quit/restart/no replay | Cmd+Q exited 0. Reopened the exact fixture/state successfully, demonstrating app-data lock release. The draft reappeared and independent inspection still showed one explicit grant and one cancelled unpublished operation. Final Cmd+Q also exited 0. No commit/hook execution or repository mutation occurred. |

Audit records contain the two exact PID targets, successful kill results and two
subsequent real page-reset events. The [read-only oracle](../validation/g02-renderer-loss-2026-11-07/oracle.py)
checks the complete repository baseline, durable operation/trust count and saved
draft. Artifact/source hashes, before/after verdicts and native audit are in the
[evidence README](../validation/g02-renderer-loss-2026-11-07/README.md).

## Remaining G02 work and facility boundary

Actual main renderer loss with pending review and issued confirmation is now
accepted for this auto-reload path. It does not establish behavior for a persistent
renderer restart failure, renderer loss during every active execution phase,
unsaved keystroke durability, OS/power loss, or every platform.

Dock termination was attempted through Computer Use. The tool returned:
`Computer Use is blocked from using the app 'com.apple.dock' by your organization's policy.`
No alternate UI driver or signal was used as a substitute Dock action. Dock Quit
therefore remains **facility blocked**, not passed. System/session termination and
remaining required phase-specific native cases remain open. Existing direct Quit,
main-close, missing-delivery fallback and native before/after-ref recovery results
retain their previously documented bounds. No machine logout/reboot was attempted.

## Validation and repository state

Final full desktop validation passed: **69 renderer tests**, **13 default host /
16 all-feature host tests**, including the new PID guard; TypeScript, ESLint,
contracts, frontend formatting/build, Rust formatting, strict default/all-feature
Clippy and debug/release builds. Ordinary feature/artifact isolation was rechecked:
no test menu marker or private PID selector in default debug/release executables,
and no test-write feature in the ordinary dependency graph. Support executable
still refuses nonfixture state. This is not a rerun of the full Core/signing/remote
matrix; Core transaction source did not change in this checkpoint.

Existing uncommitted work is preserved. Ordinary writes stay disabled; default
executables were restored by validation. No installed app bundle, user repository,
credential, production service or other project's files were changed. No project
commit, push or release was made.
