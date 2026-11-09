# Completion P02 actual blank startup capture

During P03 native testing, a fresh ordinary default build on a generated 100-repository workspace showed a blank window. Native AX and screenshot were captured before reload through Computer Use. The ordinary host subsequently handled Cmd+Q and exited zero. No render-ready or causal-fix claim follows from document loading alone.

A separate build with only `custom-protocol,native-acceptance-support` (desktop writes still disabled) reproduced the initial blank appearance on the same sealed fixture. The existing fixed Inspect main startup state menu recorded:

```json
{"ready":"complete","visibility":"hidden","focused":true,"rootChildren":1,"shell":true,"inert":false,"width":1440,"height":908,"shellWidth":1440,"shellHeight":908,"display":"flex","opacity":"1","tauri":true}
```

The shell became visible after the fixed probe without reload. Workspaces/load_state/load_bulk_composer/inventory had already completed successfully in the native event audit. The evidence rules out an absent React shell, inert loading state, zero-size geometry and an unfinished initial facade request in this specific observation. It does not distinguish WebKit visibility/paint scheduling, app activation/occlusion or an observation side effect. Do not treat the probe/title update as a production workaround or add sleeps/reload loops.

The now-visible app completed native partial scan → Continue discovery → 100 complete, with zero managed Git writes. Native Quit exited zero, repository bytes/index/modes remained exact, SQLite integrity passed and a separate CLI reopened the state writer lock. Details and event audit: `docs/validation/completion-p02-startup-2026-11-08/`. Screenshots remain in the Computer Use transcript; no local screenshot artifact is claimed.

Next diagnostic criterion: correlate the already-mounted/hidden document with actual native activation, key/occlusion/visibility events and first paint, before any probe/reload, on a freshly attested fixture artifact. Fix only a reproduced causal transition, then test ordinary first startup/first restart after renderer/host loss without a diagnostic intervention. G02 remains open, and permitted Dock/session and remaining phase-specific criteria retain their own blockers/requirements.
