# G03/G05 changed gitlink status — November 8, 2026

Disposition: demonstrated ancestor false-clean defect fixed; Core/desktop/native
validation passed. Broader G03/G05 and ordinary write enablement remain open.

## Cause and change

Restricted parent status deliberately uses ignore-submodules=all so parent Git
cannot recurse into child configuration/helpers. Staged gitlinks were recovered
from parent index/tree comparison, but child HEAD advances were not represented in
workspace counts/Changes. Independent Git showed M middle while Overview said Clean.

Snapshot now retains optional structured own-index gitlink path/OID metadata from
bounded ls-files. Older persisted snapshots decode without this field; refresh fills
it. Discovery reuses it only when its generation matches the current read session,
avoiding duplicate normal-scan index commands. Workspace inventory derives changed
unstaged gitlinks using separately inspected available/current child HEADs at exact
tracked paths, excluding historical/unavailable identities. No recursive parent Git
status, new child subprocess, network call, staging or managed write in this join.
Targeted refresh applies the same derivation without rediscovery. Derived changes
are not persisted over raw inspection status, preventing stale/double-counted rows;
staged and unstaged changes merge on the same path.

Application unstaged diff inspection also reports exact index and observed child
OIDs rather than an empty patch. Parent generation must match fresh inspection;
child identity checked independently, comparison labelled as last local inspection,
output budget retained. Staged/historical comparisons retain their existing path.
Raw standalone parent inspection remains own-repository data; workspace-level joins
supply the child comparison. Child worktree dirt without a HEAD change stays in the
child's own row, not an implicit child selection or invented gitlink commit.

## Regression and full validation

New real-submodule workspace regression failed on unchanged product source with
parent unstaged count 0 instead of 1. Candidate passes: child refresh makes parent
changed; repeated parent refresh does not erase/double count; observed child HEAD
reversal clears change; staging current gitlink clears unstaged count; later child
reversal merges staged+unstaged into one row. Rescan consistent, exact repository
bytes/modes preserved, child filesystem-monitor sentinel never executed. Existing
bulk submodule integration now checks exact old/new OIDs, explicit observed-data
text and repository preservation through application diff request.

Full five-configuration Core validate.sh exit 0 with isolated GPG/private temp and
loopback fixtures: default 62 passed/1 ignored; experimental-writes 97/2;
controlled-writes 87/1; application-services 234/4; all-features 278/8. Ignored cases
retain their existing declared worker/facility reasons. Lab 24 passed/0 skipped.
All builds/format/strict Clippy included. Full desktop gate exit 0: 91 renderer,
13 default-host, 16 all-feature host tests plus type/lint/generated contracts/format,
frontend/native debug+release builds and strict default/all-feature Clippy.

Initial full-matrix preflight exposed the new test comparing lexical temp paths to
canonical metadata under the required private TMPDIR symlink. Corrected test to use
canonical paths, preserved failed log and reran entire matrix successfully. No
production guard or expected result weakened. Native/source hashes and raw results
retained; matrix summary uses actual test-result lines, not expected negative-test
GPG diagnostics in lab JSON.

## Native ordinary build and restart

Rebuilt ordinary release (custom-protocol only) used the existing disposable state;
parent revoked, middle/leaf trusted, control untrusted. No commit/grant/network action.
After explicit Rescan, ancestor Overview showed 1 unstaged / Changes. Parent Changes
showed middle; inspector showed exact index 949253cc316c4811271f4eb272f1196a8027d10e
and observed child 662bd57b94051be630d74febfb365564b36687a3, with local-observation
wording. Targeted refresh retained Changes 1 and exact inspector details. Native Quit
exit 0. Restart restored Changes 1/details before another Rescan; Overview retained
ancestor Changes with one unstaged. Writes disabled; Quit exit 0.

Every managed file hash/mode, all historical operation/batch payloads and trust
scope remained exact, no replay/new operations; SQLite integrity ok, writer lock
reopened. The deliberately dirty leaf and uncommitted ancestor gitlink remain intact.

[Evidence](../validation/g03-gitlink-status-2026-11-08/) includes failing/passing
regressions, source hashes and exact ordinary binary, generated contract, full logs,
environment/lab/matrix summary, native before/proof/exit records and verifier. This
accepts the observed status/inspection defect fix, not full scale/usability/release.
