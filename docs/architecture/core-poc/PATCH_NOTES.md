# v0.1.1 macOS discovery fix

The `discover` implementation canonicalizes the root directory, but the v0.1.0 test compared the returned path to the original `std::env::temp_dir()`-based path. macOS commonly provides `/var/folders/...` for temp directories while `fs::canonicalize` returns `/private/var/folders/...`. The repository is discovered correctly; the assertion was comparing different spellings of the same path.

## Changes

- Compare discovered paths against canonicalized expected paths (production discovery unchanged).
- Test canonicalized symlinked root paths.
- Test `.git` file discovery using a real linked worktree.
- Remove one unused `Command` import.
- Update README source listings and test matrix.

## macOS validation

```bash
cargo build --workspace --all-targets
cargo test --workspace --all-targets
cargo run --bin gitfleet-lab -- --output results.json
```

**Historical authoring note:** the artifact author could not compile v0.1.1.
Subsequent user-supplied macOS/Git 2.56.0 evidence verified the build and 30 Rust
test results, with the positive GPG diagnostic skipped. These commands describe
that historical artifact, before M0's opt-in feature boundary. Current validation
and explicit skip accounting are in [implementation status](../../IMPLEMENTATION_STATUS.md).

No production files or repository content are modified by the test suite; Git fixtures are disposable.
