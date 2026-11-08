# G07 authenticated loopback publication acceptance

November 8, 2026. Selected Core acceptance passed. Production Core/desktop code and
lockfiles are unchanged from `984a89b`; this checkpoint adds real Git-backed tests
and a strictly opt-in publication mode to the disposable TLS server. No personal
repository, user credential, real SSH agent, private Gitea service or native trust
grant was used. G07 remains partial; this is not authenticated native UI acceptance.

The new tests verify:

- SHA-1 and SHA-256 publication through verified loopback HTTPS with the explicitly
  selected get-only helper/account. The independent bare-server ref equals the exact
  approved source. Managed files/modes and the helper credential file remain exact.
  Helper calls are exclusively `get`; raw and Basic-encoded fixture secrets are
  absent from all persisted app state/artifacts.
- Changed wrong helper credentials fail before publication transport, leave the
  managed repository and bare server exact, and are not persisted. No store/erase
  is delegated to the selected helper and no credential is selected implicitly.
- Actual SSH-agent publication with a selected public identity that has no private
  sibling, a dedicated fixture agent, pinned host key and a fixed server command/
  destination allowlist. Independent server ref matches approved source; managed
  state remains exact. The fixture server explicitly clears inherited Git policy
  and uses a private HOME/no global configuration. A rejecting global-hook sentinel
  deliberately taints its parent environment; it never runs.
- The pre-existing `git` TLS fixture mode still refuses receive-pack even if its bare
  repository enables HTTP receive. Publication routes exist only when the test server
  is explicitly started in `publication` mode. They remain restricted to the one
  fixed fixture repository; no arbitrary path or command input was added.

Validation: complete all-features remote suite exit 0, **99 passed / 5 explicitly
ignored workers or opt-in cases**. After final fixture-server environment hardening,
all four new focused tests passed again (HTTPS positive loops over both object
formats). Core all-target/all-feature strict Clippy, Rust format, Python AST parse
and diff checks passed. The full remote log precedes only that test-local SSH
environment/sentinel refinement; its final focused validation is separately retained.
No production code changed, so unrelated signing/default/desktop/native matrices were
not repeated. The preceding full Core and desktop results retain their source scope.

[Evidence](../validation/g07-authenticated-publication-2026-11-08/) includes full and
final focused logs, strict checks, input hashes, test-source provenance and tool
versions. Broader topology, actual authenticated native transport/interruption and
usability cases remain explicit in the current tracker. G08 still needs a durable
ordered flow with fresh per-item authority and actual child-success evidence.
