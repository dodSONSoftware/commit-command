# Full Core signing matrix — November 7, 2026

## Step 1 — isolated signer restored and validated

Restored GnuPG 2.5.24/libgcrypt 1.12.4 and pinned supporting libraries under
/private/tmp/gitfleet-m2-tools/install. All six official archive SHA256 values match
the previous source manifest and the current [official integrity list](https://gnupg.org/download/integrity_check.html).
No global installation, shell configuration or personal keyring modification.
Libraries built static; libgcrypt assembly disabled per prior toolchain workaround.
This is an isolated test tool, not a shipped or distribution-qualified product.

Existing controlled-writes OpenPGP positive test passed (1 passed, 24 filtered out).
It creates a fresh private test keyring and unsigned test-only key, signs a real
fixture commit, verifies it, kills its own agent and removes its keyring. No personal
key or native trust grant is used. Rebuild script, pinned URLs/digests, tool hashes,
smoke log and build-log digest are retained in [evidence](../validation/g06-full-signing-matrix-2026-11-07/).
The large raw compiler log is retained under the temporary tool prefix, not committed.

Full five-configuration Core gate is next. G02/G06 remain partial until their separate
native/facility/accessibility criteria close; restoring a signer is not gate closure.
