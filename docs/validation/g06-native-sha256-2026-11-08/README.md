# Native SHA-256 acceptance evidence

prepare.py creates only new disposable SHA-256 repositories and uses the existing
read-only HTTPS test server. fixture.json records paths, real OID, CA and exact
opt-in/ordinary artifact hashes. Baselines store file SHA256/modes, not general
xattr/ownership guarantees. prepared.json and oracle-untrusted.json prove no initial
trust/network effects. User separately authorized this exact fixture trust grant.

oracle-cancelled.json records Cancel/no network; oracle-observed.json records real
protocol-v2 observation (GET + ls-refs POST). oracle-captured.json and oracle.py
assert SHA-256 OID, bundle digest/length/verify/list-heads, no import/local write,
exact repo/remote preservation and five total requests. network-baseline.json hashes
isolated artifacts. oracle-restarted.json/oracle-final.json assert unchanged records,
artifacts and requests after ordinary restart. native-summary.json records UI checks;
exit files, cleanup.json and final-inventory.json record clean exits and state reopen.
The server is stopped and its private TLS key removed; preparation/transport is not
currently live. Historical prepared/pending report statements are dated checkpoints.
No private key or binary is committed. Full automated suites were not rerun here.

Follow-up: this retained fixture subsequently underwent an authorized offline native
SHA256 import. The no-import/final repository baseline assertions here describe the
pre-import checkpoint and no longer apply to its current state. Use the current
[import oracle](../g06-native-sha256-import-2026-11-08/oracle.py) and its exact approved
tracking-ref/reflog delta. Original evidence remains historical and unchanged.
