# G05 malformed submodule relationships — November 8, 2026

Disposition: eight Core cases and one actual native ancestor-path case passed.
No product runtime change. This package covers selected invalid relationship inputs,
not all G05 or arbitrary synthetic graph-cycle injection.

## Core integration

Added malformed_submodule_relationships_stop_before_batch_proposals to
bulk_application.rs. Each real .gitmodules fixture attempts one of: self path `.`,
ancestor `..`, escaped `../outside`, absolute `/outside`, normalized-traversal attempt
`child/../outside`, metadata `.git`, empty component `child//outside`, malformed Git
config syntax. Actual discovery fails complete-topology requirement before proposal
creation. Tests assert Unavailable with exact planning error, empty Batches AND
Operations pages, full repository file/directory/symlink bytes/modes unchanged.
No trust or execution capability is issued.

Self/ancestor edges are rejected at normalized repository-relative path validation;
accepted relationship paths point strictly down the directory hierarchy. This tests
cycle attempts through actual metadata, not the sorting function's synthetic cycle
fallback. Other malformed states, such as declared-but-uninitialized relationships,
retain their separate owning criteria.

Focused integration passed; final bulk suite 14 passed, 0 failed, 1 existing subprocess
worker ignored (invoked by interruption test). Format check and strict bulk-target
Clippy passed. Initial test compile referenced a private error field; corrected to
the public Display contract before running tests. Failed preflight retained. Runtime
source unchanged, so no full signing/transport matrix rerun.

## Native refusal and recovery

Reused authorized disposable fixture and instrumented acceptance build from the
completed G02 diagnostic; no new trust/config/hooks/signing changes, no commit/push.
With app stopped, saved parent .gitmodules bytes/mode, then set only working
.gitmodules to an ancestor declaration `path = ..`. Captured exact malformed setup
snapshot. Native Rescan showed Discovery incomplete / 1 scan issue; expanded details
identified parent and invalid_path: Expected a normalized repository-relative path.
Only parent selected. Prepare native bulk review returned Error: unavailable:
Complete discovery is required for batch dependency planning, with no authoritative
review/OS confirmation opened. verify.py refusal passed exact malformed bytes/modes,
zero new batches/operations, unchanged historical payloads and exact three trusts.

Native Quit exit 0. Restored only deliberate .gitmodules bytes/mode; CLI rescan
complete with four repos and zero issues. Restored full repository snapshot exactly
matches pre-test diagnostic final snapshot. Ordinary restart visibly complete
and read-only, retained results, bulk preparation disabled; Quit exit 0. Final oracle
passed all files/modes, zero new records, unchanged three trusts/control untrusted,
integrity ok; writer lock reopened. Actual refusal and recovery are demonstrated.

[Evidence](../validation/g05-invalid-relationships-2026-11-08/) retains Core checks,
original declaration, malformed snapshot, fixed binary/source provenance, trace,
restored scan, exit codes and exact preservation oracles. Existing earlier fixture
current-state oracles remain valid again after exact restoration; no baseline was
overwritten to hide writes. Broader native batch selection/revocation/accessibility
acceptance and original intermittent review-window finding remain open.
