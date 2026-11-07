# G01/G02 — native confirmation lifecycle continuation

November 7, 2026. **Selected G01/G02 cases passed after three related native
lifecycle fixes. Both gates remain open.** This follows the
[first G01 pass](G01_REVIEW_AUTHORITY_2026-11-07.md). Existing uncommitted work is
preserved. Ordinary writes remain disabled, with no commit/push/release in the
project repository. One local commit was published only in a fresh disposable
fixture, never pushed.

## Findings and changes

1. **Quit was blocked by synchronous confirmation.** With an unanswered native
   commit sheet, Cmd+Q and the app's Quit menu left the dialog open; switching to
   main also returned to the sheet. The proposal stayed unpublished and the hook
   did not run. After Cancel, a separate ordinary Quit exited. Inspection of pinned
   rfd 0.15.4's macOS backend showed the synchronous parented dialog enters
   `NSAlert.runModal()`. Replaced only operation confirmation with rfd's existing
   parented `AsyncMessageDialog`. Actual independent native approval remains the
   only source of the accepted result; Core's one-use capability is unchanged.
2. **Closing the async sheet's parent did not resolve its completion.** The first
   async build allowed workspace switching with confirmation outstanding. It
   cancelled the Core proposal and closed the review, but a subsequent picker
   failed with `A native dialog is already open`. Added a native-adapter helper
   that explicitly ends the attached macOS sheet with Cancel, orders it out, then
   closes the review. Review cancel/replacement, workspace invalidation, renderer
   reload and shutdown use this helper. It runs on the AppKit main thread; the
   single raw handle cast is documented and scoped to the owned live Tauri window.
   `objc2-app-kit` 0.3.2 was already a locked transitive dependency; it is now an
   explicit macOS-only dependency with limited features. No dependency version changed.
3. **Review replacement raced asynchronous label teardown.** Requesting a new
   trust review while an old native confirmation was outstanding closed the old
   window but produced no replacement. Tauri close requests return before the
   window label is released. The host now remembers the desired review, waits
   for the review `Destroyed` run event, and asks the existing Core worker to show
   that review only if its ID/session remains current. Old destruction callbacks
   cannot cancel the new review. This uses an internal Rust task, not a new IPC,
   renderer permission, automation endpoint or approval field.

The read-only inspection policy, Git transaction engine, signing/hook support
scope, independent confirmation, caller/ACL checks and feature defaults are unchanged.
The old app-modal sheet had hidden these reachable main-window lifecycle cases;
retaining responsive async sheets requires both explicit sheet cancellation and
ordered window replacement.

## Native acceptance and independent oracles

The [evidence directory](../validation/g01-g02-2026-11-07/README.md) records fixture,
artifact and source hashes, raw automated logs and per-stage verdicts. All native
runs used a separate disposable app-state directory and bundle, retaining the
original bundle identifier. Native UI/AX and inline screenshots were inspected;
no saved screenshot or live webview attack injection is claimed.

| Case | Result |
|---|---|
| Unusual filename | Main lists and authoritative review displayed a UTF-8 filename containing bidi U+202E and newline with visible escapes; Git patch paths were quoted/escaped. Staged and working versions remained distinct. |
| Hook/signer presentation | Trust review showed the fixture pre-commit path, executable flag/fingerprint and SSH signer/tool identity. Commit review and native summary showed Execute and the exact signer fingerprint; full review showed selected Index source, message, hook manifest and literal patch. |
| Signed exact publication | On the original acceptance artifact, native approval published the selected staged blob and exact message. Git independently verified the ED25519 signature against the disposable public identity. Exactly one hook marker line was written. Excluded README staging, working content, three other repositories and all non-Git working bytes/modes were preserved. This does not attest every hook/signing configuration. |
| Async positive approval | A fresh native trust approval on the first async build succeeded only after the explicit OS button. No extra repository commit or hook call resulted. |
| Quit with unanswered commit confirmation | Original synchronous build failed responsiveness; first async build exited with code 0 on Cmd+Q without answering the sheet. The proposed operation became cancelled before publication. All repository bytes/modes equalled the pre-Quit snapshot; hook count stayed one. |
| Workspace switch with outstanding commit confirmation | First async build cancelled safely but leaked the dialog lease. After explicit sheet cancellation was added, switching through the real main-window selector cancelled the old proposal. A new tooling trust review opened a real native confirmation, proving the lease was released. No tooling grant was approved. |
| Main close with outstanding trust confirmation | Selected main via the native Window menu while its sibling review had an unanswered tooling trust sheet, then selected Close Window. The fixture exited with code 0; tooling remained untrusted, and no hook/commit replay occurred. |
| Replace outstanding native trust confirmation | Before the ordered replacement fix, old window disappeared and no new authoritative window was listed. After the fix, replacement displayed a different review ID, opened its own fresh OS confirmation, and Cancel consumed only that replacement. Old sheet disappeared and granted no trust. |
| Restart/no replay and cleanup | Repeated fixture restarts restored drafts with no approval capability or extra publication. Final oracle: five local operation records, one complete/published and four cancelled, one engine trust row, one hook run, exact signed commit/blob, unchanged repository snapshot after the sole approved publication. Fixture processes exited with code 0. Generated private signing key removed; public verification evidence retained. |

Some native selector actions required state refresh/keyboard selection when the
Computer Use surface reported a user-change interruption. No action or pass was
inferred from those interrupted calls; final UI state and independent checks were
observed after successful actions.

## Automated validation

- Core application suite: **24 passed**. Added a regression that issues a commit
  confirmation, then resets or shuts down Core, rejects the old capability, verifies
  no hook/repository mutation, and reopens state to verify durable cancellation/no replay.
- Full final desktop validation: **69 renderer tests**, **13 native host tests per
  default/all-feature configuration**, typecheck, ESLint, generated contracts,
  Prettier, production frontend build, Rust formatting, native debug/release builds
  and strict Clippy in both configurations passed. Configurations overlap.
- Focused Core strict Clippy and formatting passed. Native acceptance builds passed
  at each patch stage. Earlier desktop runs are retained; use
  `desktop-validation-accepted.txt` for the final replacement implementation.

The first fixture preparation attempt lacked a hooks directory because templates
are intentionally empty. Preparation was corrected to create that fixture-only
directory; no product code was changed for that setup error. The three actual
native failures above remain recorded, not relabeled as initial passes. This is
not a full Core five-feature/signing/crash matrix or release acceptance. Default
CLI/desktop artifacts were restored by the final desktop validation build; the
ordinary development bundle was not replaced.

## Remaining acceptance queue

G01 still needs live native main-renderer reload/reset with pending and issued
confirmation, and live caller/page/origin adversarial cases where a permitted test
facility exists. Core and host mock-runtime coverage for these paths passes; the
normal app exposes no reload or script-evaluation control. No privileged test
endpoint or alternate automation driver was added. Configured OpenPGP and broader
hook/signing presentation criteria retain their existing historical evidence and
supported exclusions; this run refreshes only the explicit SSH/direct-hook case.

G02 still needs missing-renderer close fallback, Dock/system termination, and
remaining before/after-publication interruption/recovery phases. Outstanding-sheet
Quit, workspace invalidation/dialog reuse, main close and native replacement now
have selected actual evidence. No power-loss durability or universal callback-race
coverage is inferred. Ordinary write enablement remains G04 and is unchanged.
