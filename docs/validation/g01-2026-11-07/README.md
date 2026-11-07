# G01 acceptance evidence — November 7, 2026

See [the report](../../reports/G01_REVIEW_AUTHORITY_2026-11-07.md) for native
procedures, results, limitations and open cases. This directory contains fixture
metadata and scrubbed test evidence, not credentials or user application data.

## Commands run from the repository root

```sh
cargo test --manifest-path app/core/Cargo.toml --locked --offline --features application-services --test m3_application
cargo test --manifest-path app/core/Cargo.toml --locked --offline --features application-services --test m3_application g01_
cargo clippy --manifest-path app/core/Cargo.toml --locked --offline --features application-services --test m3_application -- -D warnings
cargo test --manifest-path app/desktop/src-tauri/Cargo.toml --locked --offline
cargo test --manifest-path app/desktop/src-tauri/Cargo.toml --locked --offline --all-features
cargo build --manifest-path app/desktop/src-tauri/Cargo.toml --locked --offline --features desktop-writes
# Restore the default target/debug executable after the isolated native run:
cargo build --manifest-path app/desktop/src-tauri/Cargo.toml --locked --offline
cargo clippy --manifest-path app/desktop/src-tauri/Cargo.toml --locked --offline --all-targets --all-features -- -D warnings
cargo fmt --manifest-path app/core/Cargo.toml -- --check
cargo fmt --manifest-path app/desktop/src-tauri/Cargo.toml -- --check
npm --prefix app/desktop test -- src/review.test.ts src/services/bridge.test.ts
```

The native fixture was created with `app/desktop/scripts/native-fixture.py`, then
prepared with `prepare-native-used.py`. That file is the exact historical preparation
script: it modifies only the recorded disposable fixture, copies a built binary,
and attempts launch. **Do not replay it against the historical state.** A new run
must create new fixture/evidence identities. Its initial background launch did not
supply the controllable native instance; the recorded executable/data-dir was then
launched in a dedicated PTY with permitted desktop access. The process exited with
code 0 after native Cmd+Q. Initial CUA app selection was ambiguous among installed
bundles; an exact path resolved the fixture without changing its bundle identifier.

`native-oracle.py` is a read-only verifier. Its stage JSONs attest their respective
times: early stage expectations intentionally no longer match the final five-record
state. `context-final` verifies the final state without replaying UI actions.
`native-before.json` records preapproval repository file hashes/modes, including
Git metadata. The final positive verdict additionally inspects raw commit bytes,
blob bytes, branch OIDs, commit count and journal outcomes independently of UI.
`source-manifest.json` distinguishes native build inputs from final test inputs.
Empty format logs mean those checks exited successfully. Test configurations overlap;
counts must not be summed as unique coverage. `core-g01-first.txt` retains the initial
new-test compile failure; corrected and final logs are separate.
