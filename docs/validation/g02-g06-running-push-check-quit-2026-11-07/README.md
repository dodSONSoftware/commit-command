# Native Quit during running pre-push check — November 7, 2026

Disposition: [acceptance report](../../reports/G02_G06_RUNNING_PUSH_CHECK_QUIT_2026-11-07.md).
Native controls used Computer Use; read-only independent scripts did not drive UI
or inject journal state. No product source or approval-policy change.

- `native-fixture.py`: fresh-fixture factory copied from the preceding Cancel review
  case. Creates real repositories/bare remotes, installs the sleeping hook before
  trust and starts a read-only TLS upload-pack service. Run from repository root
  with a built gitfleet-cli and Git/Python/OpenSSL available. Each run creates a
  new fixture; recorded results below belong only to the exercised instance.
- `native-fixture.json`, `fixture-preparation.txt`, `native-baseline.json`: exact
  bundle/state/worktree/remote IDs and hashes/modes after setup. Nested snapshots
  overlap; count three working repositories and three bare remotes, not each
  duplicate subtree as a separate fixture. No generated private TLS key is stored here.
- `artifact-provenance.json`: exact reused artifact SHA-256, source revision and
  matched source provenance. Prior full desktop gate and focused Core nine-case
  logs are linked as earlier evidence, not claimed as rerun.
- `prove-running.py`, `native-running-proof.json`: actual persisted running phase,
  one invocation and hook/child PIDs/timestamps collected before native Cmd+Q;
  saved draft already persisted before Quit. No precancelled or synthetic operation.
- `native-process-cleanup.json`: transcribed post-Quit process check, exit 1/no
  matching exact PIDs, host exit 0, no external hook/child kill.
- `native-oracle.py`, `native-closed.json`, `native-restarted.json`, `native-final.json`:
  read-only exact preservation of repository/remote files, cancelled/unpublished
  journal, draft and network log. Restart records identical to post-Quit baseline.
  Example final check:
  `python3 docs/validation/g02-g06-running-push-check-quit-2026-11-07/native-oracle.py final`.
- `hook-invocations.txt`, `hook-pid.txt`, `child-pid.txt`: preserved fixture hook effects
  outside the managed repositories. No hook completion marker appeared.
- `requests.jsonl`: exactly one GET `/grand.git/info/refs`, from the independently
  approved observation. No subsequent restart/history read contacted a server.
- `native-events.jsonl`: fixture startup audit; no test fault action used.
- `validation-summary.json`, `cleanup.json`: native exits, timing versus natural
  hook finish, exact scope and owned server/private-key cleanup.

Native restart showed the exact draft and cancelled hook history without retry
or publication authority. No screenshot, host crash, alternate termination entry,
Dock/session, native bypass/manager or release acceptance is inferred. Generated
server key removed and fixture service/app stopped; managed repositories preserved.
