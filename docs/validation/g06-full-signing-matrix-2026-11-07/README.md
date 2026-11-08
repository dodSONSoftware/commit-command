# Full signing matrix evidence

restore-gpg.py rebuilds pinned upstream sources only in /private/tmp/gitfleet-m2-tools.
verified-sources.json records SHA256-verified official archives. tool.json fingerprints
the installed signer/agent/config tools. tool-build-summary.txt records raw log digest
and local location; signer-smoke.txt records the actual production sign/verify test.
No private keys or binaries are committed. Full matrix uses the existing validate.sh
unchanged, locked/offline Cargo dependencies and this explicit test signer.
