# Actual renderer-loss evidence — November 7, 2026

Disposition: [G02 report](../../reports/G02_ACTUAL_RENDERER_LOSS_2026-11-07.md).
Native actions used Computer Use against only the marked disposable fixture.
No saved screenshot, actual Dock Quit or session termination is claimed.

- `fixture.json`: exact fixture/state/bundle and native binary SHA-256. Features:
  `custom-protocol,desktop-writes,native-acceptance-support`.
- `source-artifact-manifest.json`: exercised source/lock hashes and base revision;
  native artifact matches those changed inputs. Work remains uncommitted.
- `build.txt`: successful support build. Root lockfile changes only dependency
  edges to already locked versions; no transitive version upgrade.
- `repositories-before.json`: complete baseline bytes/modes including Git metadata
  for all four real repositories. No keys/credentials are included.
- `pending-review-verdict.json`: historical zero-grant/zero-operation verdict,
  full byte/mode preservation and independent process-check observation. `ps`
  returned no entry for PID 17908 and the exact fixture executable for host 17901
  while the host was still running. This is a transcribed observation, not raw output.
- `issued-confirmation-verdict.json`, `closed-verdict.json`, `restarted-verdict.json`,
  `final-verdict.json`: one approved engine trust and one cancelled unpublished
  operation, exact draft and full repository preservation. Historical pending
  verdict predates explicit trust; it cannot be regenerated from final state.
- `oracle.py`: read-only Git-file/state oracle; run from any directory with a stage
  label, e.g. `python3 docs/validation/g02-renderer-loss-2026-11-07/oracle.py final`.
- `native-events.jsonl`: two real renderer kill requests/results and subsequent
  page reset callbacks. Native host survived both. No dropped-ack fault was armed.
- `desktop-validation.txt`: final complete desktop validation script, exit 0.
  69 renderer tests, 13 default / 16 all-feature host tests plus all static,
  Clippy and build checks. Suites overlap; these are not unique-case totals.
- `default-isolation-verdict.json`: final default graph/artifact/private-selector
  isolation and support nonfixture refusal, plus default executable hashes.
- `validation-summary.json`: recorded validation and native exit results.

Actual WebKit main renderer termination used the live owned webview's checked
private PID property, not process-name matching, synthetic IPC, window destruction
or journal injection. Automatic Tauri reload is part of the observed recovery.
Persisted drafts were checked before fault and after recovery; no guarantee is
made for text not yet saved. The test guard is not a hostile same-user sandbox.

Dock access was denied by organization policy on `com.apple.dock`; no bypass
was attempted. The remaining G02 exclusions are enumerated in the report/tracker.
