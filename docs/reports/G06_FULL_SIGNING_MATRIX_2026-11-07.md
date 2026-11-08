# Full Core signing matrix — November 7, 2026

## Step 1 — isolated signer restored and validated

Restored GnuPG 2.5.24/libgcrypt 1.12.4 and pinned supporting libraries under
/private/tmp/gitfleet-m2-tools/install. All six official archive SHA256 values match
the previous source manifest and the current [official integrity list](https://gnupg.org/download/integrity_check.html).
No global installation, shell configuration or personal keyring modification.
Libraries built static; libgcrypt assembly disabled per prior toolchain workaround.
This is an isolated test tool, not a shipped or distribution-qualified product.

Existing controlled-writes OpenPGP positive test passed (1 passed, 24 filtered out).
It creates a fresh private test keyring and unprotected test-only key, signs a real
fixture commit, verifies it, kills its own agent and removes its keyring. No personal
key or native trust grant is used. Rebuild script, pinned URLs/digests, tool hashes,
smoke log and build-log digest are retained in [evidence](../validation/g06-full-signing-matrix-2026-11-07/).
The large raw compiler log is retained under the temporary tool prefix, not committed.

Full five-configuration Core gate completed below. G02/G06 remain partial until their separate
native/facility/accessibility criteria close; restoring a signer is not gate closure.

## Step 2 — full matrix and remaining signing checks complete

The unchanged Core validate.sh exited 0: prerequisite/scaffold checks, formatting,
locked builds, all-target tests and strict Clippy passed for every configuration.
Cargo dependency access was offline. All tracked Core files match source-manifest.json;
no product source or test was changed or weakened.

| Configuration | Passed | Failed | Declared ignored in standard run |
|---|---:|---:|---:|
| Default | 58 | 0 | 1 |
| Experimental writes | 93 | 0 | 2 |
| Controlled writes | 83 | 0 | 1 |
| Application services | 227 | 0 | 4 |
| All features | 271 | 0 | 8 |

Production OpenPGP positive signing/verification, SSH signing/verification and
wrong-key/timeout no-unsigned-fallback tests passed in all three configurations that
enable controlled writes. Parent crash tests exercised their ignored subprocess
workers. The unsupported non-UTF-8 filename filesystem case and opt-in actual
Gitea/Keychain integration retain explicit exclusions; they are not accepted passes.

The standard script's older lab found no gpg on PATH (GITFLEET_TEST_GPG applies to
production tests), initially giving 23 passed/1 skipped. Explicit legacy signing then
exposed a fixture-path limitation: gpg-agent could not connect because the host's
long TMPDIR made its socket path too long. Independent disposable probe retained
that actual error. With process-local TMPDIR=/private/tmp and the isolated tool on
process-local PATH, the unchanged optional legacy positive test passed explicitly
in both experimental and all-feature configurations. The final complete lab rerun
passed **24 cases, zero skipped**, including signed=true and signature_verified=true.
Historical failure/skip logs remain; no failed attempt is represented as a pass.

The full signing package is complete. The isolated tool remains available at the
recorded temporary prefix and may disappear when temporary storage is cleaned.
Use the short process-local temporary path for future legacy signing runs. No personal
keys, global installation/configuration, native trust renewal or real-server publication.
The matrix does not establish native UI, hardware-token, non-macOS, release or every
filesystem acceptance. G02/G06 remain partial for their other documented criteria.
