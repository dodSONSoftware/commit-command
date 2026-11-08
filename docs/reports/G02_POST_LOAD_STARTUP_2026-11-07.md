# G02 post-load startup ordering — November 7, 2026

## Diagnosis and patch

The previous native restart was blank despite document Finished and window display;
explicit reload recovered it. Seven diagnostic launches did not reproduce that
intermittent blank, so the added read-only startup probe did not inspect a blank
window. OS diagnostics are inconclusive and do not establish a WebKit root cause.

A deterministic bootstrap defect was established independently: React's concurrent
initial render returned before the application shell committed. The new bootstrap
test fails against the original entry point and passes when the initial render is
wrapped in React `flushSync`. The document-finished native display policy now follows
the committed initial shell rather than merely a queued render. This changes only
the initial mount; no recurring synchronous rendering, permissions or IPC were added.
The non-default native acceptance menu adds a fixed read-only shell/layout probe.

## Validation and limits

Full desktop validation passed: typecheck, lint, contract checks, formatting,
70 renderer tests, 13 default-host tests, 16 all-feature host tests, strict Clippy,
and debug/release builds. The support artifact SHA256 is
`75eba31a1a728104e02701078f474ea21358ca253dc9495eda876de1cc7ff05c`.

Three fixed native launches displayed the retained workspace and exact draft without
manual reload: initial launch, first restart after actual renderer loss, and first
restart after controlled idle host SIGKILL. Renderer replacement also recovered
within the surviving host. Final operation history visibly retained cancelled,
unpublished execution and no push authority. Both normal exits returned 0; the
intentional host crash returned 1. These restart tests did not run a new hook.

Independent oracles passed after renderer-loss and host-loss restarts: exact three
repository/three remote byte/mode preservation, one original hook invocation, no
completion marker, one original discovery GET, no additional writes/network/import
records, and exact persisted draft/cancelled history. Original support artifact and
ordinary debug executable were restored; installed application was not changed.

The demonstrated initial-commit ordering defect is fixed and bounded native retests
passed. This does not prove every intermittent macOS/WebKit paint stall has the same
cause. The earlier blank remains counterevidence; broader startup reliability and
G02/G06 acceptance remain open, including Dock/operator-session criteria.

[Evidence](../validation/g02-post-load-startup-2026-11-07/README.md).
