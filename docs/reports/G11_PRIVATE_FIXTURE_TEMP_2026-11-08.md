# Private short-path fixture storage — November 8, 2026

A fresh full Core run exposed a test-environment conflict in the earlier CI setup:
short process-local TMPDIR=/private/tmp lets agent sockets fit, but real OpenSSH
StrictModes rejects authorized-key paths below a writable ancestor. Four SSH tests
failed with the server diagnostic naming /private/tmp. The signer passed after
allowing its isolated agent socket outside the execution sandbox; these are distinct
failures, not product signing/transport acceptance passes.

A short symlink to a private directory within the checkout was also rejected, correctly,
because application state cannot resolve inside a repository. No guard was relaxed.
A short alias to a private, outside-repository system-temp directory passed both the
real SSH agent observation/download/offline-import gate and isolated OpenPGP signing.
No global directory modes, StrictModes, personal keys, agent or SSH configuration changed.

The workflow now creates a fresh mode-0700 fixture directory under RUNNER_TEMP and
an atomic unique short /private/tmp symlink, exporting only process-local TMPDIR for
macOS Core validation. Removed forced writable TMPDIR from desktop validation too.
Linux keeps the runner's default temp behavior. Runner storage and aliases are ephemeral;
this does not configure an installed application. UUID collision fails symlink creation.

YAML parsed; every run block passed bash -n. The exact new macOS step executed locally
with an isolated RUNNER_TEMP stand-in, produced a 0700 target and matching alias.
Hosted runner execution remains not run. Full five-configuration Core refresh is
running separately and must be judged from its final exit/result, not this focused
checkpoint. [Validation](../validation/g02-g11-core-matrix-2026-11-08/workflow-temp-validation.json).
