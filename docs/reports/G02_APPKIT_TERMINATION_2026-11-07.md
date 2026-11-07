# G02 direct AppKit termination diagnostic

Direct native termination is a separate case from custom Cmd+Q and the policy-blocked
Dock UI. Added a fixed, non-default fixture menu action to request NSApplication
`terminate:` on its own app. No machine logout/reboot, external app, renderer IPC,
write default or production lifecycle behavior was changed by this diagnostic step.

The first action invoked terminate inside Tauri's event callback. A main-thread
stack sample identified reentrant Tao callback-mutex acquisition through
`applicationWillTerminate`/`AppState::exit`. That is a test-control scheduling bug,
not evidence of an external termination hang. The hung disposable fixture was
sampled and killed for cleanup. The corrected action uses NSObject's fixed
run-loop selector scheduling to execute terminate after the callback returns.
Both artifacts and initial observations remain in the evidence directory.

On the corrected native fixture, armed the existing one-shot missing-close-delivery
fault and requested direct termination with no sheet open. The host exited 0.
The audit showed the fault armed but never consumed and no close fallback was
observed, establishing that the guarded shutdown path was bypassed. All repository
bytes/modes were unchanged; zero trust grants/operations were created. This does
not prove last-keystroke persistence: the observed draft had already been saved.
An initial request with an unanswered native sheet left that sheet open until
explicitly cancelled; it did not approve trust.

This is **a failed G02 graceful-termination criterion**, not acceptance or a Dock
pass. Next implementation step: intercept native application termination before
AppKit tears down the event loop, route through the existing guarded shutdown,
and retest missing-delivery Keep app open/retry plus ordinary direct termination.
Do not treat `applicationWillTerminate` as a cancellable callback: it is too late.

Evidence: [fixture, audit, preservation verdict, stack and support validation](../validation/g02-appkit-termination-2026-11-07/).
Support builds, all-feature host tests and strict Clippy passed for the diagnostic.
A full ordinary desktop gate will accompany the production fix. Existing G01 and
renderer/publication-boundary G02 results retain their bounds; G02 remains open.
