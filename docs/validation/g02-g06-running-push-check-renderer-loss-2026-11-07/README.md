# Actual renderer loss during running pre-push check — November 7, 2026

Disposition: [report](../../reports/G02_G06_RUNNING_PUSH_CHECK_RENDERER_LOSS_2026-11-07.md).
Automatic recovery/cancellation in the surviving host passed. Later normal restart was blank until explicit reload; startup presentation is reopened. No product source changed.

- `native-fixture.py`, `native-fixture.json`, `fixture-preparation.txt`, `native-baseline.json`: fresh marked factory/IDs and bytes/modes/symlinks after hook setup. Three nested worktrees/three bare remotes; snapshots overlap. Factory runs from repository root with built Core CLI and creates a fresh instance each time.
- `artifact-provenance.json`: exact reused artifact/source hashes, unchanged app tree and historical full desktop/build links.
- `initial-host-process.json`, `prove-running.py`, `native-running-proof.json`: unique host identity, durable running state/draft, one invocation and live host/hook/child immediately before renderer termination.
- `prove-recovery.py`, `native-process-recovery.json`: native-audited renderer target/success/reset, same host alive, original renderer/hook/child absent before natural completion. No external host/hook/child signal.
- `native-oracle.py`, `native-recovered.json`, `native-restarted.json`, `native-final.json`: read-only exact repository/draft/journal/request preservation; later states match recovered baseline. Restarted oracle proves durable preservation, not painted UI. Run stage argument while archived fixture exists.
- `native-events.jsonl`, `check-native-events.py`, `native-event-assertions.json`: actual renderer kill, reset/reload, both initial document completions, and explicit reload on the blank restart. Captured audit is not proof that startup content painted.
- `native-observations.json`: native automatic renderer recovery/draft/history and separate blank normal restart, recovered through explicit reload. No screenshot file saved.
- `hook-invocations.txt`, `hook-pid.txt`, `child-pid.txt`, `requests.jsonl`: one invocation and one observation GET; no completion marker, including beyond natural finish.
- `cleanup.json`, `validation-summary.json`: normal app exits, original hook/child absence, service/key cleanup and partial disposition.

Native UI used Computer Use. Scripts did not seed approvals/results or signal host/hook/child. Only the fixed fixture action killed main WebKit. No generated private key retained; disposable repositories preserved. No Dock/session, publication, release or write-default acceptance is inferred.
