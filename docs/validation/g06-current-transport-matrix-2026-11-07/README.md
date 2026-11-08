# Current-source G06 automated transport matrix

Scope: remote_inspection and remote_observation integration targets, with
application-services and all-features, locked dependencies. Sources are fingerprinted
in source-manifest.json; environment.json records the tested checkout and tools.
Test fixtures use disposable repositories, isolated authentication material and
loopback servers. This is automated Core acceptance, not native UI approval evidence.

Commands:

```
cargo test --manifest-path app/core/Cargo.toml --locked --features application-services --test remote_inspection --test remote_observation
cargo test --manifest-path app/core/Cargo.toml --locked --all-features --test remote_inspection --test remote_observation
cargo fmt --manifest-path app/core/Cargo.toml --all -- --check
cargo clippy --manifest-path app/core/Cargo.toml --locked --all-features --test remote_inspection --test remote_observation -- -D warnings
```

Complete signing/full five-configuration validation remains open: the previous
/private/tmp/gitfleet-m2-tools/install/bin/gpg is gone and GPG is absent from PATH.
No signer was installed, no user keyring used and no global configuration changed.
Opt-in actual Gitea/Keychain and subprocess-only workers retain their declared ignore
policy; all-feature parent tests execute their crash workers.
