# Completion baseline evidence

Current baseline run recorded November 2, 2026. See the [P0 report](../../reports/P0_BASELINE_2026-11-02.md) for scope, exact results, failed-attempt explanations and limitations.

- `core-validation-final.txt`: complete passing final Core gate, including five configurations, strict Clippy and the disposable lab.
- `core-validation.txt`, `core-validation-with-agent.txt`: earlier unsuccessful attempts, retained and excluded from acceptance counts.
- `desktop-validation.txt`: complete passing desktop gate.
- `core/environment.json`, `core/lab-results.json`, `summary.json`: environment and results.
- `starting-git-status.txt`, `starting-head.txt`, `starting-source-sha256.json`: dirty implementation baseline before current edits.
- `prior-native/manifest.json`: provenance/hashes of 54 selected historical native evidence files; no private key material or state databases copied.
- `recovery-before.json`, `recovery-after.json`, `recovery-inspection-result.json`, `recovery-reopened-workspaces.json`: independent repository preservation and ordinary read-only native recovery inspection.

Current tests and old user observations are separate evidence. No source hash in this directory attests future changes or proves earlier test-time binary correspondence. No native cancellation or full lifecycle acceptance follows from the recovery inspection.
