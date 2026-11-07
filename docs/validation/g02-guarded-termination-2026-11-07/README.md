# Guarded native termination evidence

See [report](../../reports/G02_GUARDED_APPKIT_TERMINATION_2026-11-07.md).

- `fixture.json`: reused disposable fixture and final native artifact hash. Original
  baseline lives in `../g02-appkit-termination-2026-11-07/repositories-before.json`.
- `manifest.json`: source hashes relative to diagnostic commit `904a8e5`, final
  native artifact and earlier guarded fallback artifact (before feature-only probe).
- `initial-fixed-events.jsonl`: cumulative audit through first guarded fallback/
  Keep app open/retry. Includes earlier failed diagnostic runs, which are historical.
- `native-events.jsonl`: cumulative audit, distinguished by startup boundaries.
  Final fixed runs record termination interception and actual fault consumption;
  forwarding probe passed. Last request with unanswered sheet has no corresponding
  interception, and remains an exclusion. Final cleanup used custom Cmd+Q.
- `final-verdict.json`: actual exit results, exact draft, zero trust/operations and
  full repository baseline preservation. This is a transcribed combined native/
  persisted-state oracle, not an imported product record.
- `support-build.txt`: final acceptance-feature build.
- `desktop-validation.txt`: complete final validation script, exit 0; 69 renderer,
  13 default / 16 all-feature host tests and all static/Clippy/build checks.
- `default-isolation.json`: final default executable hashes and absence of fixed
  acceptance menu/private PID selector. Ordinary feature graph excludes Core test
  features; objc2 itself is now an ordinary macOS adapter dependency.

No native screenshot, Dock action, machine logout/reboot, every-selector forwarding
pass or universal active-sheet cancellation is claimed. No user repositories or
private credentials were used. Native fixture processes exited; installed app bundle
was not replaced. Test menus and crash/PID controls remain non-default.
