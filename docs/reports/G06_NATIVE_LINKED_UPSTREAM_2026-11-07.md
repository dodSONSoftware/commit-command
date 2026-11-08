# G06 native linked-worktree upstream setup — November 7, 2026

**Selected end-to-end native linked-worktree upstream setup case passed.**
This closes the specific native shared-config topology gap previously left as broader
topology in the guarded-upstream report; it is not another stale-review variant and
does not close G06 as a whole. No product patch or ordinary write-default change.

## Criteria exercised

Real independently approved HTTPS observation → separately reviewed/approved upstream
setup → exactly two shared-config keys → visibility from both linked worktrees →
checkout/remote preservation → fresh-trust requirement → ordinary restart/no replay.
Owning implementation/test: `writes/upstream.rs` and
`upstream_shared_config_is_visible_to_linked_worktrees_without_touching_either_checkout`
in `remote_observation.rs`. The existing focused Core test passed (one test).

## Fixture and artifact

Product source remains validated `e9eeacd`; checkout `a3198c2` adds evidence only.
Reused the already authorized disposable main repository with isolated
`state-linked-upstream` copied while prior apps were closed. Original state was not
modified. A temporary linked worktree on `g06-linked-upstream` shared the common Git
repository. Native discovery showed two distinct worktree IDs sharing one repository
ID. Only the authorized main worktree received trust; no linked-worktree trust grant.

A separate ephemeral loopback HTTPS upload-pack service served a disposable bare clone.
The existing bounded fixture handler was reused; it exposes no receive-pack/push route.
The app used the fixture CA via an explicit startup option, credential-free transport,
and no managed helper/proxy/SSH execution. No global TLS/security policy was changed.
Exact acceptance/ordinary hashes, paths and server PID are in `fixture.json`.
Baseline file SHA256/modes cover both checkouts/common metadata and the bare remote.

## Actual native chain

1. Separate authoritative/native trust approval covered only the previously authorized
   main path. Inspected origin, chose `refs/heads/main`, and independently approved
   observation of the exact loopback HTTPS destination. It completed with real OID
   `1008391da64d7debe65ed29a8f63bb1604f9a367`, matching the bare ref. Server recorded
   exactly one GET to `/grand.git/info/refs`. No synthetic observation/result was used.
2. Before upstream setup, all workspace bytes/modes still matched baseline. Selected
   **Review upstream setup** for that completed observation. Both authoritative and
   native confirmation identified source main/OID, origin/merge ref, before/candidate
   config hashes, shared linked-worktree scope, required fresh trust and no push/retry.
3. Approved that exact missing-only configuration change. Result was **complete /
   updated**. Independent checks proved the existing config prefix retained its exact
   baseline hash, followed only by the canonical `branch.main.remote=origin` and
   `branch.main.merge=refs/heads/main` section, with original config mode preserved.
4. Those main-branch settings were readable through Git from **both** worktrees.
   The linked worktree's current `g06-linked-upstream` branch acquired no branch
   settings. No HEAD/index/worktree/ref/object/reflog/other file bytes or modes changed
   in either checkout/common directory. Bare remote snapshot stayed identical.
5. Native trust display became revoked as required; linked worktree remained untrusted.
   Operation results showed one completed observation and one completed upstream setup,
   explicitly described as shared by linked worktrees and requiring fresh trust.
6. Acceptance Quit exited 0. Ordinary default-feature restart rendered without reload,
   displayed both records, and kept write controls disabled. Ordinary Quit exited 0;
   writer lock reopened. Exact published config and journals matched before/after
   restart, and request count stayed **one**: no contact during setup or replay on restart.

## Independent checks / cleanup

Executed publication and restart oracles: only shared config changed; exact original
prefix/two-key suffix; other files/modes in both checkouts and bare remote preserved;
matching durable completed records; main trust revoked with no linked grant; no
stage/unstage/commit operation rows, download/import or push-check records; SQLite
integrity `ok`. Upstream config publication is an actual controlled write and is
recorded separately in the upstream journal, not the local-commit operations table.

After recording restart evidence, removed only the generated linked worktree/branch
and restored the fixture's original config. This is explicit test teardown, not product
rollback. Original repository baseline and prior helper-refusal oracle passed again.
Isolated completed journals are retained as historical observations of the test;
they do not claim that teardown left upstream configured. Stopped exact owned server,
removed generated TLS private key, retained request log/public certificate, and verified
original file baseline/bare remote/journals with the final cleanup oracle. Both apps
closed; normal user app and system trust/configuration were untouched.

## Limits / next substantive work

This accepts the selected native linked-worktree shared-config success case, not all
upstream topology, SHA-256 fetch chains, config-lock interruption/recovery or metadata/
platform compatibility. Ownership/xattrs beyond recorded modes are not new independent
claims here; their existing automated/native evidence remains separate. Existing
full-suite results were not rerun for unchanged code. One focused Core test plus the
real native chain and independent oracles passed.

Next prioritize the already documented **native owned-config-lock recovery** gap and
other named package-level criteria. Do not treat additional read-only guard variations
as broad gate closure. G02 Dock/operator-session and broader startup findings retain
their dispositions; G06 OS credential choices and other compatibility gaps remain.

[Evidence](../validation/g06-native-linked-upstream-2026-11-07/README.md).
