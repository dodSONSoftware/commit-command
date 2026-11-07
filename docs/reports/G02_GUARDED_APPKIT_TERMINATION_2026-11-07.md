# G02 guarded AppKit termination fix and native retest

The [preceding diagnostic](G02_APPKIT_TERMINATION_2026-11-07.md), committed as
`904a8e5`, established that direct AppKit termination bypassed the custom Quit
handshake. The earlier documentation/G01/renderer-loss checkpoint was committed
as `aad07d2`. This step fixes the selected direct native termination path; **G02
remains open** for the explicit cases below. No push or write-default change.

## Implementation

Added a main-thread-owned NSApplicationDelegate wrapper. It intercepts the public
`applicationShouldTerminate:` decision, cancels that native request and calls the
existing phase-aware shutdown handshake. Save failure can therefore keep the app
open; successful state/Core shutdown exits through the existing path. A completed
shutdown may allow AppKit termination immediately. `applicationWillTerminate` is
not used to initiate an asynchronous final save because that notification is too late.

The wrapper retains Tao's original delegate, advertises its optional methods and
forwards unhandled selectors to it. NSApplication holds its delegate weakly; a
main-thread-local retained owner keeps the wrapper alive without marking Cocoa
objects Send/Sync or replacing Tao's implementation. A fixed native test probe
called the original and wrapped `applicationSupportsSecureRestorableState:` and
observed identical true results, demonstrating optional-selector forwarding. This
is representative forwarding evidence, not exhaustive testing of every delegate method.

`objc2` 0.6.5 is now a direct macOS production dependency, already present transitively;
objc2-app-kit adds its NSResponder feature for the public delegate method. Versions
and lock resolution are unchanged. Private WebKit PID lookup and native test menus
remain feature-only. No Core transaction/IPC/ACL/renderer-authority change was made.

## Native results and preservation

The existing marked four-repository disposable fixture was reused. All actions used
Computer Use. Exact source/artifact chronology and logs are in the
[evidence record](../validation/g02-guarded-termination-2026-11-07/README.md).
Native AX results were inspected; no saved screenshots are claimed.

1. **Direct termination with missing final delivery:** armed the existing one-shot
   close-delivery suppression, then requested deferred AppKit terminate. Audit now
   recorded `appkit_termination_intercepted` and `close_ack_suppressed`. The real
   unsaved-drafts fallback appeared. **Keep app open** restored a running app with
   the exact saved draft. Repeating direct terminate without that one-shot fault
   completed shutdown with exit code 0.
2. **Restart and forwarding:** reopened the same state, releasing/reacquiring the
   data lock successfully. Exact draft restored; fixed native forwarding probe
   passed. Direct AppKit terminate again intercepted and exited 0.
3. **Unanswered confirmation context remains unproven:** reopened a fresh engine
   trust review and real OS confirmation without approving it. Requested direct
   terminate; sheet remained open and no new interception event appeared. The
   action did not reach the tested decision callback in that context. Do not
   relabel it as a successful sheet cancellation or infer a cause from this alone.
   Explicit native Cancel declined trust; closing review and ordinary Cmd+Q exited 0.
4. **Independent final oracle:** all four repositories' complete bytes/modes
   matched the original diagnostic baseline, with zero trust grants and zero local
   operations. Exact persisted draft retained. No hooks/commits/replay occurred.

The first guarded fallback artifact preceded only the addition of the feature-only
forwarding probe. Final artifact exercised forwarding, clean direct termination,
restart and the unanswered-sheet exclusion. Both hashes are preserved; no claim
that those two binaries are bit-identical is made.

## Validation and next steps

Full final desktop script passed: **69 renderer tests, 13 default host / 16
all-feature host tests**, TypeScript, ESLint, generated contracts, frontend format/
build, Rust formatting, strict Clippy in both configurations and native debug/release
builds. Default feature graph excludes Core test-write support; default binaries
exclude test menu and private PID selector. The native delegate protection is
included in ordinary macOS builds. Ordinary write execution remains disabled.
This step does not rerun the full Core/signing/remote or release matrix.

Remaining G02 work is concrete:

- Actual Dock Quit: organization policy blocks Computer Use access to `com.apple.dock`.
  A fixed own-app termination entry point is not a substitute Dock pass.
- Direct native termination while an unanswered OS confirmation is active: obtain
  a permitted facility that demonstrates actual callback delivery in that context;
  current scripted native request did not do so. Custom Cmd+Q/main-close cases retain
  their earlier passes and are not relabeled as system termination.
- OS session logout/restart termination: use a dedicated operator-controlled test
  session. No logout/reboot of the user's active machine was attempted or implied.
- Remaining active-execution native interruption combinations: preserve the prior
  running-hook and before/after-ref evidence; coordinate running remote-hook checks
  with G06. Automated thirteen-checkpoint Core coverage is not thirteen native cases.

G02 is not closed by this step. G01's recorded envelope and G03–G11 gates are
unchanged. Local step commits are authorized; no external publication is authorized.
