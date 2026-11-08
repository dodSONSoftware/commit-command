# Full signing matrix evidence

restore-gpg.py rebuilds pinned upstream sources only in /private/tmp/gitfleet-m2-tools.
verified-sources.json records SHA256-verified official archives. tool.json fingerprints
the installed signer/agent/config tools. tool-build-summary.txt records raw log digest
and local location; signer-smoke.txt records the actual production sign/verify test.
No private keys or binaries are committed. Full matrix uses the existing validate.sh
unchanged, locked/offline Cargo dependencies and this explicit test signer.

Final full script command (exit 0; standard lab initially 23/1):

```
GITFLEET_TEST_GPG=/private/tmp/gitfleet-m2-tools/install/bin/gpg CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=4 sh app/core/scripts/validate.sh /Users/randydodson/Documents/_code/git-fleet/docs/validation/g06-full-signing-matrix-2026-11-07/core
```

For final legacy signing, process-local PATH prepends the same install/bin and
TMPDIR=/private/tmp avoids the reproduced macOS agent socket limit. Explicit
`cargo test --features experimental-writes --test acceptance successful_gpg_signing -- --ignored --exact --test-threads=1`
and the same command with `--all-features` both passed. The full experimental lab
was then rerun with `--output lab-with-signing.json`: 24 passed, zero skipped.

full-matrix.txt preserves all five configurations and original lab output;
legacy-signing.txt retains the failed long-path attempt; legacy-path-probe.json
records the independent actual error; legacy-signing-short-path.txt and
legacy-signing-experimental.txt are passing retries. lab-with-signing.json is final
lab evidence. summary.json separates original and final results and ignored reasons.
source-manifest.json verifies unchanged tracked Core sources after all checks.
