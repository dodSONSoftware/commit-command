# G02 startup readiness evidence — November 7, 2026

Disposition: [startup fix and native retest report](../../reports/G02_STARTUP_READINESS_2026-11-07.md).
Premature initial display fixed; first restarts after real running-hook and idle host crashes passed. Other G02/G06 gates remain open.

- `baseline-reproduction.json`: original artifact/retained-fixture reproduction, committed in `a89ecc6`. Empty native window later populated without relaunch; no permanent stuck-renderer claim.
- `desktop-validation.txt`, `support-build.txt`, `artifact-provenance.json`: freshly passed full desktop gate and exact support artifact/source hashes. Ordinary default debug binary restored after the separate support build.
- `native-fixture.py`, `native-fixture.json`, `fixture-preparation.txt`, `native-baseline.json`: fresh marked fixture factory, three nested worktrees/three remotes, IDs, baseline bytes/modes/symlinks and credential-free read-only TLS service. Run factory from repository root with built Core CLI; each invocation creates another fixture. No private key retained.
- `prove-running.py`, `native-running-proof.json`, `initial-host-process.json`, `crash-host.py`, `host-crash-proof.json`: actual running hook/child proof, exact host guard and host-only SIGKILL. No journal seeding or approval bypass.
- `native-oracle.py`, `native-crashed.json`, `native-restarted.json`, `native-final.json`: independent read-only repository/state/request preservation. Raw journal is still running; interrupted is its native audit projection. Run oracle with stage argument from repository root while the archived fixture exists.
- `crash-idle-host.py`, `repeat-host-crash.json`: additional host-only crash of the first recovered process, without claiming another running hook.
- `native-events.jsonl`, `events-first-restart-and-reload.json`, `check-startup-events.py`, `startup-event-assertions.json`: all three host startups hidden through document completion, exactly one initial display per process, normal renderer reload and successful actual renderer-process loss/recovery. Run `python3 docs/validation/g02-startup-readiness-2026-11-07/check-startup-events.py` against the captured audit; it does not drive UI.
- `hook-invocations.txt`, `hook-pid.txt`, `child-pid.txt`, `hook-finished.txt`, `requests.jsonl`, `cleanup.json`: original hook survived the host and finished naturally, count one, one observation GET, no external child kill; exact server stopped and generated key removed.
- `validation-summary.json`: current check scope and transcribed native outcomes. Native AX/screenshot observations used Computer Use; no screenshot file saved.

The report distinguishes the original prolonged blank observation from the reproduced transient startup exposure and does not assert every renderer failure is solved. No user repositories or installed app were modified, ordinary writes remain disabled, and no push/publication occurred.
