# G01 live native support acceptance — November 7, 2026

Disposition and observations: [report](../../reports/G01_NATIVE_SUPPORT_G02_RECOVERY_2026-11-07.md).
Native controls were operated through Computer Use in the exact separate bundle
in `fixture.json`, selecting `custom-protocol,desktop-writes,native-acceptance-support`.
No saved native screenshot is claimed. These fixtures contain no credentials.

- `source-artifact-manifest.json`: final changed source hashes, base revision and
  independently rechecked native bundle hashes. This describes uncommitted source,
  not a new revision. The initial G01 artifact predates the later system-temp-root
  allowance and equivalent Clippy-only borrow/condition cleanup. G02's artifact
  includes those changes. Native support build logs preserve this chronology;
  final automated validation covers the final source. Do not describe the initial
  G01 binary as a bit-identical final-source rebuild.
- `repositories-before.json`: original bytes/modes, including metadata, for all
  four disposable repositories. `pending-reload-verdict.json` verifies zero trust/
  operations after actual reload; `issued-reload-verdict.json` verifies one explicit
  engine trust and one cancelled unpublished operation, with all bytes unchanged.
- `native-events.jsonl`: actual fixed native menu requests, two rejected navigation
  attempts, three IPC `PASS denied` titles, reload resets and one suppressed close
  delivery. Native trust/confirmation outcomes were inspected through AX; independent
  persisted-state/Git verdicts substantiate their effects.
- `fallback-verdict.json`: exact persisted-draft preservation and normal retry exit
  0 after native Keep app open on the fallback sheet. One delivery was suppressed;
  no renderer process was killed in this case.
- `desktop-validation.txt`: final full `CARGO_NET_OFFLINE=true sh app/desktop/scripts/validate.sh`.
  69 renderer tests, 13 default host and 15 all-feature host tests, TypeScript,
  ESLint, generated contracts, Prettier, frontend build, Rust formatting, strict
  default/all-feature Clippy, debug and release builds passed.
- `core-application-tests.txt`: final locked/offline `m3_application` suite with
  `application-services`: 24 passed.
- `initial-check.txt`, `support-build*.txt`, `host-support-*.txt`: development checks
  including initial compile/Clippy failures and successful corrections. The first
  compile used `.items` rather than the actual `.workspaces` contract. Initial
  Clippy rejected three needless eval borrows and one collapsible conditional;
  all were corrected without changing expectations.
- `default-feature-verdict.json` and `support-refuses-nonfixture.txt`: final ordinary
  artifact hashes/string absence and default feature graph; support executable
  rejects a nonfixture state directory before Application opens.

The standard default debug/release executables are restored by final desktop
validation. The installed/ordinary `.app` bundle was never replaced. Native G01
fixture exited 0. Only the selected local macOS envelope is accepted; G02/G03/G06,
write enablement and release remain distinct.
