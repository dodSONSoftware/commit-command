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

**Expected:** all acceptance tests and all six core behavior tests pass; the GPG-positive case may skip if `gpg` is unavailable. This expected result has **not** yet been verified by compiling v0.1.1 in this environment.

No production files or repository content are modified by the test suite; Git fixtures are disposable.
