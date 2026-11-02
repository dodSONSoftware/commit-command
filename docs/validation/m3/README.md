# M3 evidence — implementation delivered, native acceptance incomplete

Recorded on 2026-11-01, macOS 27.2 arm64, Git 2.56.0, Rust/Cargo 1.99.1,
Node 22.22.3 and npm 10.9.8. Full versions are in `environment.json` and
`core/environment.json`. The source hash manifest identifies the validated sources.
No project commit, push, remote CI, deployment or write to an existing user Git
repository was performed. Historical M0–M2 evidence remains separate.

## Automated results

Both validation scripts exited **0**. Counts overlap across feature configurations;
these are not distinct aggregate test counts. No listed automatic check failed.

| Core configuration | Passed | Failed | Ignored |
|---|---:|---:|---:|
| default | 48 | 0 | 1 |
| experimental-writes | 83 | 0 | 2 |
| controlled-writes | 71 | 0 | 1 |
| application-services | 85 | 0 | 1 |
| all features | 125 | 0 | 3 |

Core layout/manifest validation, `cargo fmt --all -- --check`, builds and
`cargo clippy --workspace --all-targets … -- -D warnings` passed for all five
configurations. Historical lab: **23 passed, 1 skipped**. The M2 positive isolated
OpenPGP and SSH tests passed; no personal keyring was used. Real SIGKILL recovery
coverage includes thirteen phases; it does not establish OS/power-loss durability.

Desktop: **14 passed** in four renderer test files. Native host **6 passed** in
each of default and all-feature configurations, zero failures/ignores. These tests
use Tauri's non-UI mock runtime, including the actual generated command ACLs.
TypeScript, zero-warning ESLint, generated-contract equality, Prettier, Vite build,
native formatting, debug/release builds and strict Clippy in both configurations
passed. The ordinary debug and release binaries have desktop writes disabled.
A development `.app` was assembled from the final ordinary debug binary; it is
not a signed/notarized release and was not relaunched after the policy block.

Commands, run from the project root:

```sh
GITFLEET_TEST_GPG=/private/tmp/gitfleet-m2-tools/install/bin/gpg CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=4 sh app/core/scripts/validate.sh ../../../docs/validation/m3/core
CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=4 sh app/desktop/scripts/validate.sh
```

Raw stdout/stderr was captured in `core/validation.txt` and `desktop-validation.txt`.
The Core output directory is resolved after the script changes into `app/core/`.
GPG-agent sockets required the approved local unsandboxed run; all keys, Git writes
and destructive/crash fixtures were disposable. Other checks used the normal
sandbox. Dependencies were already cached for the final offline Cargo runs.

## All ignores, skips, unsupported and untested cases

- Every Core configuration: `non_utf8_filename_remains_git_quoted_without_mutation`
  is ignored because this host filesystem rejected non-UTF-8 filename creation
  (EPERM). Native byte parsing/schema tests pass; filesystem acceptance is unproven.
- Experimental/all-feature configurations: historical POC `successful_gpg_signing`
  is harness-ignored, explicitly optional. Its lab counterpart is skipped because
  GPG is not on the lab's PATH. This is separate from the mandatory M2 positive
  signer test, which passed using `GITFLEET_TEST_GPG`.
- All features: `crash_worker` is harness-ignored for ordinary discovery because
  it is a subprocess entry point. The passing crash matrix explicitly invokes it
  with `--ignored --exact` and asserts actual SIGKILL outcomes; it is not an
  omitted acceptance scenario.
- No desktop automated test was skipped. Native UI gates were **not executed**,
  not silently skipped or counted as passes. Linux/Windows desktop, other Git/Rust
  versions, advanced-index writes, unsupported hooks/managers, nonlocal filesystem
  safety, OS/power-loss durability, native performance and release packaging remain
  unsupported or untested as described in M0–M3 contracts.

## Native acceptance block

Computer Use returned exactly:

> Computer Use is blocked from using the app 'dev.gitfleet.desktop' by your organization's policy.

`native-acceptance.json` records this restriction. No native screenshot, actual
native confirmation click, VoiceOver/focus/contrast inspection, or lifecycle
acceptance occurred. No alternate app identity, OS automation or browser-based
substitution was used to bypass it. A final scoped process inventory found zero
fixture application processes. `native-fixture.json` identifies the disposable
fixture; it may be removed by normal temporary-directory cleanup.

**M3 is not complete.** Ordinary Rust handlers and renderer controls keep desktop
trust/write execution disabled. `desktop-writes` is a non-default acceptance opt-in,
not an approved production default. Read-only recovery inspection remains available.
The [desktop README](../../../app/desktop/README.md) supplies the required permitted
human/native workflow. Actual picker/restart/diff flows, trusted review/OS approval,
staging/signing/recovery and close/cancel/forced interruption remain native gates.
See the [gate-by-gate ledger](../../ACCEPTANCE_MATRIX.md#12-m3-acceptance-gates--implementation-with-native-gates-open).

## Additional reliability finding

Concurrent baseline tests reproduced a workspace lock that could outlive its owner
when closing left an advisory lock attached to a duplicated descriptor inherited
during subprocess spawning. The private `ExclusiveLock` guard explicitly unlocks
before close and preserves typed Busy/I/O errors. Its unit test independently proves
the close-only behavior and successful explicit release with a duplicate alive.
The existing 15 workspace tests passed in three consecutive runs after the fix;
the final five-configuration matrix passed. No sleep, retry or weaker assertion
was added to conceal contention. This guard also owns M2 common-directory locks.

## Evidence index

- `core/validation.txt`, `core/lab-results.json`, `core/environment.json`: final Core run.
- `desktop-validation.txt`: final desktop run.
- `summary.json`: machine-readable counts and native acceptance status.
- `environment.json`: desktop tool versions/host.
- `desktop-dependency-features.txt`: ordinary native dependency graph excludes
  experimental POC/crash features; Tauri test runtime is a dev-only dependency.
- `npm-production-audit.json`: npm production audit, zero known advisories at run
  time; no cargo audit or exhaustive supply-chain audit is claimed.
- `native-acceptance.json`, `native-fixture.json`: policy block and disposable scope.
- `source-sha256.json`: Core/native/renderer/configuration/script source hashes.
- `document-checks.txt`: current local link and diff/format checks.
- `../m2/status-at-completion.txt`, `../m2/handoff-at-completion.txt`: preserved
  pre-M3 progress; original POC evidence remains historical with its limitations.

Updated CI definitions were inspected and validated locally; no hosted CI run
occurred. M4 and later remain unauthorized. No workaround or automated test can
close the explicitly open native gates.
