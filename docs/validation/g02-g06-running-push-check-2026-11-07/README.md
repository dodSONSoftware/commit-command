# Native running pre-push-check cancellation — November 7, 2026

Disposition: [report](../../reports/G02_G06_RUNNING_PUSH_CHECK_CANCEL_2026-11-07.md).
All native actions used Computer Use in disposable repositories/state. No production
code changes, synthesized running journal, special cancellation fault or push command.

- `native-fixture.py`: creates a new marked fixture only, starts a read-only loopback
  TLS upload-pack server, seeds real repositories and installs a sleeping pre-push
  hook before trust. Adapted from the prior M5C fixture. Run from repository root;
  requires built gitfleet-cli and cached Git/OpenSSL/Python tools. Produces a new
  state each run; recorded IDs/hashes below describe only the exercised instance.
- `native-fixture.json`, `fixture-preparation.txt`: exact paths, loopback endpoint,
  bare/working OIDs, worktree IDs and server PID. Git authentication credential-free;
  a generated private key was used only for local TLS and is not in this directory.
- `native-baseline.json`: hashes/modes and symlink bindings of working repositories
  and bare remotes after setup, including the installed hook. Nested snapshots
  overlap intentionally; do not sum them as independent repositories.
- `artifact-provenance.json`: reused acceptance artifact hash, base revision, source
  match, prior full desktop gate link, hook/CA/factory fingerprints. No new native
  build or full desktop gate is claimed in this step.
- `native-running-proof.json`: actual durable running phase and unpublished flag,
  invocation/PIDs/observation time before native Cancel review. No journal injection.
- `native-process-cleanup.json`: exact post-cancel ps result: exit 1/no matching
  hook or child. This is a transcribed tool result, not stored product state.
- `hook-invocations.txt`, `hook-pid.txt`, `child-pid.txt`: preserved fixture hook
  effects outside managed repositories. Hook finished marker never appeared.
- `native-oracle.py`, `native-cancelled.json`, `native-closed.json`,
  `native-restarted.json`, `native-final.json`: read-only exact file/journal/request
  checks. Final cancelled journal and observation arrays remained identical across
  Quit/restart; no hook replay, new network request or publication. For final state:
  `python3 docs/validation/g02-g06-running-push-check-2026-11-07/native-oracle.py final`.
- `requests.jsonl`: exactly one GET `/grand.git/info/refs`, from independently
  approved observation. The service rejects receive-pack/publication endpoints.
- `native-events.jsonl`: native fixture startup audit; no fault action armed.
- `core-push-check-tests.txt`: `cargo test --manifest-path app/core/Cargo.toml --offline --locked --features application-services --test remote_observation push_check::`;
  9 passed, 74 filtered. Automated cases do not become native bypass/crash passes.
- `cleanup.json`, `validation-summary.json`: native exits, server/key cleanup and
  precise executed/unexecuted scope. Generated server key removed after stopping
  the exact owned fixture server; other project/user processes were not touched.

Native AX history showed cancelled, hook_passed false, no publication and no
resume/retry control. Native restart history retained the same result. No screenshots
or actual host-crash, Quit-while-hook-running, native manager/bypass, remote write,
Dock/session or full Core/signing/release acceptance is claimed.
