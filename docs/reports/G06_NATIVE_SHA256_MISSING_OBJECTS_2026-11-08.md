# Native SHA-256 missing-object import — November 8, 2026

This case uses a new disposable SHA-256 repository, separately advanced remote and
packed `refs/remotes/origin/main` at the old commit. Three incoming objects (commit,
tree and blob) were proven absent locally before observation and after download.
User specifically authorized trust only for the fixture repository, granted through
authoritative Rust review and OS-native confirmation. No hooks/signing configured.
Preparation and exact binary hashes/features/source are recorded in fixture.json;
product binaries are the previously validated exclusion-UI build from c95378e.

Native observation `de54b5c5-81fa-4b2f-a59f-8a0e7971a6fb` returned the exact 64-digit
remote commit. Separate native capture `88bdea26-df20-432b-9b4f-4a146ca4295d` completed:
6 objects, 733 uncompressed bytes, 685 bundle bytes, SHA-256
`de08d8d8d2de010b8b8d1d7ed5a1b77ed53e0d3bc2d69b3553c86c54b20eab29`.
Repository and remote bytes/modes stayed exact; all three new objects still absent.
Five credential-free HTTPS protocol requests recorded. Server was identified by
its PID and exact fixture command, stopped, and temporary private TLS key removed
before preparing either import. No global certificate change or user credential use.

First offline import `065e6a07-444d-4a7c-8920-3850222ad75f` was cancelled; exact repository
bytes/modes and request log preserved. Fresh review
`cfa91e36-f2ef-43f7-8c72-acd08d1747d8` named exact packed old value and captured new value.
Native approval completed offline with objects_imported=true and ref updated.
Three previously absent objects are now readable. Only object files and the new loose
tracking ref were added, with exactly one old-to-new reflog line appended to the
existing reflog. All other original bytes/modes (including packed-refs, HEAD, index,
working/untracked files and config) remained exact. Remote and request log unchanged.

Ordinary read-only restart displayed complete import, cancelled import, capture and
observation chronology with exact full OIDs/digest; native write controls disabled.
Post-restart repository, network artifact files and operation records matched the
post-import baseline exactly, no replay or extra request. SQLite integrity, process
exits 0 and CLI writer-lock reopen passed. Exactly one authorized trust row.

Oracle corrections: the preparation already created a tracking reflog, so verification
checks one appended line and its unchanged baseline prefix, rather than one total line.
Missing OIDs are lexically sorted, not sorted by object type. Stored remote records
are versioned envelopes; verification reads their value. The initial artifact snapshot
looked in an unused artifacts directory and was empty; it establishes no preservation
claim. The immutable capture bundle was instead verified against the recorded native
digest and byte count; complete network snapshots establish restart preservation.
No product fix was needed. These corrections changed only read-only evidence checks.
A post-Quit UI query surfaced the separate personal app window; no action was taken
there. Subsequent ordinary acceptance explicitly reopened the fixture bundle/state.

This closes the selected native missing-object and packed-expected-old SHA-256 import
case, not all G06. Native SHA-256 upstream/check topology and other gate criteria
remain open. [Evidence](../validation/g06-native-sha256-missing-2026-11-08/).
