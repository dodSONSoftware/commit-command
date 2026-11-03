# Recurring fetch-artifact refusal — diagnosis and fix

November 2, 2026. The previously unexplained `Unsafe fetch artifact type` refusal has a captured cause and deterministic before/after regression evidence. No filesystem type is newly accepted. No push or managed repository outside disposable fixtures occurred.

## Captured failure

Twenty-four parallel focused SSH roundtrips passed without reproducing the failure. A separate 30-second regular-file create/delete metadata probe found zero unknown file types (780,544 successful reads/92,526 missing entries). These negative results did not diagnose the issue. The full concurrent transport workload then failed in local push-preview preparation's prerequisite offline import and captured:

`loose.git/tiIfoO0`, symlink, mode `120755`, link count one, target length seven bytes.

Evidence: `docs/validation/m5b-authentication-compatibility/artifact-full-workload.txt`. The failure was not specific to SSH authentication. The import runs a second `git init --bare` to create `loose.git`; its generic runner polled the complete artifact tree while initialization was still in progress.

Git's [initialization implementation](https://github.com/git/git/blob/master/setup.c) creates a temporary `tXXXXXX` symlink to `testing` while probing filesystem support, then removes it. That explains the captured path/type/length and timing. The guard correctly refuses symlinks in a completed artifact, but was being applied during this trusted bootstrap probe.

## Change

Extracted the existing fixed-argument, sterile, bounded bare-repository initialization into `initialize_bare_repository`. Both initial transport setup and loose-import bootstrap use it. Initialization has no remote object/template input and remains subject to process deadline/cancellation/output limits. The import applies the **unchanged** complete artifact type/count/size guard before and after bootstrap, then polls it during object processing and validates before publication. A persistent link/special file still blocks before untrusted object unpacking or managed object/ref changes.

No path-prefix exemption, symlink following, retry-based acceptance, guard removal or larger budgets were introduced. The test-support-only exact entry/type diagnostics remain available for future different refusals.

## Deterministic validation

A subprocess-only transparent Git fixture holds the initialization probe symlink for 150 ms. Before the fix it reproducibly failed with the same strict guard error (`artifact-probe-before.txt`). After the fix it completes (`artifact-probe-after.txt`). A companion leaves the link present: initialization completes, the subsequent guard rejects it, and an independent managed-repository snapshot stays unchanged. Both cases pass in `artifact-probe-final.txt` and the final full matrix. The worker is ignored directly and exercised by its parent; it is never an IPC/test override.

Final combined authentication/fix validation passed Core **54/89/79/183/227**, with direct ignores **1/2/1/4/8**. Application/all-feature transport suites passed **52/56**, including both deterministic artifact cases. Full desktop checks and actual native real-Gitea/Keychain results are in the companion authentication compatibility report and validation summary. Final strict checks passed; earlier failures remain immutable evidence. This closes the identified initialization race, without claiming every possible artifact refusal or filesystem failure is resolved.
