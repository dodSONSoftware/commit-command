# G07 exact publication primitives — November 8, 2026

Disposition: local Git primitive experiment passed; application push executor is
NOT implemented by this checkpoint. No application acceptance or release claim.
User authorized continued G02–G11 implementation and disposable acceptance tests.
Only brand-new /private/tmp/gitfleet-g07-primitives-* source/isolated/bare fixture
repositories were used. No personal repository, external service or trust grant.

## Verified primitive

With installed Git 2.56.0, exported one explicitly named source branch to a bundle,
imported its exact source OID into an isolated bare repository and independently
proved expected-old is an ancestor of source. Pushed that fixed OID to exactly
refs/heads/main with atomic expected-old comparison. Independent remote ref matched.

Then approved a later source descended from the expected old tip. Advanced remote
to an intermediate ancestor of the approved source before publication. Ordinary
fast-forward ancestry alone would permit that drift, but explicit expected-old
comparison rejected it and preserved the intermediate server tip. Positive/refusal
porcelain and independent OIDs retained. No deletion, tag, wildcard or ref creation
was part of the tested application-equivalent update (initial fixture seeding separate).

The Git expected-old primitive uses an explicit --force-with-lease=<ref>:<old> guard.
It must only be reachable AFTER independently proving old is an ancestor of source;
it is not a user-selectable force operation. The intended application has no force
flag, plus refspec, mirror/all/tags option, or arbitrary command input. Tests must
prove divergent/unrelated updates are refused before spawning publication, not rely
on a lease to establish ancestry. This distinction is part of the implementation
contract; this experiment alone is insufficient production safety evidence.

## Required application implementation

- A fresh immutable publication plan and independent native approval bind exact
  worktree/config/tool/auth identity, source ref/OID, destination URL/ref, expected
  old OID, outgoing history, hook manifest/policy and child reachability evidence.
  No completed pre-push check becomes a publication token.
- Run reviewed pre-push checks as part of new approved publication scope, revalidate
  after hooks, isolate source objects and destination transport, enforce object/time/
  output budgets, and verify source/config/trust immediately before publication.
- Keep managed config/refs/index/objects untouched. Do not run source export using
  repository-controlled pack helpers: copy the approved reachable object closure
  through a sanitized isolated Git configuration, then fsck before transport.
- Journal before publication starts. Cancellation/process loss after that point is
  potentially published until separately reviewed fresh remote evidence resolves
  current state. Never automatically replay, retry or infer failure from nonzero exit.
- Check exact remote old OID before sending and enforce the expected-old comparison
  atomically at update. Verify actual destination afterward; retain uncertainty if
  verification fails. Recovery evidence describes observed state, not invented
  causal proof of which actor changed a ref.
- Expose exact refusal/uncertainty in durable operation history and preserve default
  ordinary write gate. Implement G08 only on verified individual outcomes and
  separately bound selected-child success; unselected children never enrolled.

[Reproducible evidence](../validation/g07-publication-primitives-2026-11-08/) contains
probe.py, Git version, exact fixture/commit IDs and expected-old success/refusal.
These requirements are implementation work, not completed application capabilities.


## Approved object closure export — passed as a Git primitive

A second disposable fixture has an approved main commit and a distinct unselected
branch containing an extra blob. Initialized an isolated bare transport repository.
With sanitized isolated configuration, GIT_OBJECT_DIRECTORY points to source objects
for rev-list/cat-file/pack-objects reads only; no source refs/config are copied.
Enumerated only approved OID reachability, checked object count/uncompressed budget,
exported a full pack through stdout, removed the managed object-directory override,
and indexed/fscked it into the isolated repository's own object store.

Actual isolated object set exactly equalled the three approved reachable objects
(265 uncompressed bytes, 246-byte pack); unselected branch blob absent. Source-only
config key absent from isolated configuration; every managed repository file hash
and mode unchanged. No network or trust grant. pack-probe.py / pack-result.json
retain exact commands and independent object-set proof. Production implementation
still needs fixed tool bindings, strict process/artifact budgets, revalidation and
durable uncertain-publication handling. No application capability inferred.
