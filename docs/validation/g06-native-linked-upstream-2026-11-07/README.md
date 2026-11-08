# Native linked-worktree upstream evidence

- `core-linked-upstream.txt`: focused owning Core topology test passed.
- `fixture.json`, `original-baseline.json`, `baseline.json`, `remote-baseline.json`:
  exact artifacts/isolated state/topology/service and pre/post-setup file SHA256/modes.
- `original-config.txt`: original disposable config restored during teardown.
- `observed.json`, `before-upstream.json`, `requests.jsonl`: genuine completed
  independently approved HTTPS observation, exact real OID, one discovery GET,
  unchanged workspace before upstream setup. No synthetic observation.
- `upstream-complete.json`, `config-appended.txt`, `after-upstream.json`: actual
  independently approved shared-config publication, original prefix plus two keys.
- `oracle.py`, `oracle-published.json`, `oracle-restarted.json`: executed publication /
  restart checks, both checkout preservation, bare remote preservation, shared keys,
  fresh-trust requirement and no replay/additional request. Those checks describe the
  published topology before teardown; the temporary linked worktree is now removed.
- `linked-identity-proof.json`: distinct worktree IDs, common repository ID, lock reopen.
- `native-summary.json`, `cleanup.json`, `prior-case-restored.json`: native sequence,
  both exits 0, exact test teardown and earlier fixture revalidation.
- `cleanup-oracle.py`, `cleanup-assertions.json`: current restored final-state checks;
  run this oracle for the fixture's present state. TLS private key was removed.

[Report](../../reports/G06_NATIVE_LINKED_UPSTREAM_2026-11-07.md). No product patch.
