# Native owned-config-lock recovery evidence

- `worker-build.txt`, `worker.json`: existing all-feature subprocess worker build.
- `prepare.py`: actual checkpoint SIGKILL fixture generator; no fabricated receipts.
- Per-case `*-worker.txt`, `*-marker.json`, `*-fixture.json`, `*-upstream-before.json`:
  real phase/receipt provenance, exact artifacts and worker -9 exit.
- Per-case repository/remote/network baselines: exact file SHA256/modes after crash.

Preparation passed; native package outcomes initially pending. See the report for
current results as subsequent validated steps are committed.
