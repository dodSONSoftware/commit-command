# G01/G02 continuation evidence — November 7, 2026

[Report](../../reports/G01_G02_CONFIRMATION_LIFECYCLE_2026-11-07.md) contains the
native procedures, three reproduced findings, patches, retests and remaining gates.

## Artifact stages

- `native-fixture.json`: fresh fixture, original binary hash, selected unusual
  filename, public signer identity and expected commit/blob. No private key content.
- `patched-artifact.json`: first async-dialog build; Quit passed, lease leak found.
- `final-artifact.json`: explicit sheet cancellation; workspace invalidation/reuse
  and main close passed, ordered replacement still missing.
- `replacement-artifact.json`: final accepted replacement implementation; actual
  old-sheet replacement, fresh independent confirmation and decline passed.
- `source-manifest.json`: original-source reference and final validation source; intermediate binary hashes remain separate.
- `native-*.json`: independent Git/SQLite/file-hash verdicts at each named stage.
  These record their original times; early record-count assertions are not intended
  to pass against later state. `accepted-final` verifies final state read-only.
- `cleanup.json`: generated private signing key removed. Verification needs only
  retained disposable public identity/allowed-signers. No keyring/agent/global
  Git configuration or system trust was changed.

`prepare-native-used.py` preserves historical fixture preparation. It targets the
recorded fixture metadata, generates a disposable signing key and copies an app;
**do not replay it against historical state**. A new run needs new fixture and
evidence identities. Native binaries were launched in dedicated PTYs against the
exact recorded data directory with permitted desktop access. All owned fixture
processes exited normally; no process kill or UI-policy bypass was used.

## Commands

```sh
python3 app/desktop/scripts/native-fixture.py
cargo test --manifest-path app/core/Cargo.toml --locked --offline --features application-services --test m3_application
cargo clippy --manifest-path app/core/Cargo.toml --locked --offline --features application-services --test m3_application -- -D warnings
cargo fmt --manifest-path app/core/Cargo.toml -- --check
cargo build --manifest-path app/desktop/src-tauri/Cargo.toml --locked --offline --features desktop-writes
CARGO_NET_OFFLINE=true sh app/desktop/scripts/validate.sh
```

The first build after adding the already-cached macOS dependency intentionally ran
without `--locked` to update only the root package dependency list in Cargo.lock;
subsequent builds and validation used the updated lock. No package version changed.
`desktop-validation.txt`, `desktop-validation-final.txt` and
`desktop-validation-accepted.txt` are separate successive runs after actual source
changes; only the last attests the final implementation. Core contains 24 cases,
including the new two-branch reset/shutdown regression. Empty format logs reflect
successful exit. Native UI observations in the report are transcribed, with inline
screenshots inspected; raw user desktop/picker contents were not saved.
