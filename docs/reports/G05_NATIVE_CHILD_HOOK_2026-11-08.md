# Native child-hook failure and running cancellation — November 8, 2026

Prepared new disposable parent/submodule fixture under
/private/tmp/gitfleet-m3-native-g05-child-29iydypi/workspace/gitfleet-batch/gitfleet-parent
and its module worktree. Third independent dirty control is unselected/untrusted.
Both selected identities have signing off; parent has no hooks. Child has exactly one
fixed executable pre-commit hook; exported reviewed-pre-commit.sh is byte-identical
and its SHA-256 is 1ca3a0ce3c36c49cb9baa005b2b44c8d3571f1b212e8981b53a42c2ceafa1014.
It appends a run marker only in the disposable outer fixture root, exits 23 when
reject-child exists, otherwise creates a running marker and waits for release-child.
No network commands, source/config mutation or arbitrary command inputs. Core's
normal bounded execution/cancellation remains responsible for stopping/reaping it.
Hooks run unsandboxed as the user after explicit trust and native operation approval.

Native host and Core fault markers/layout are prearranged without weakening guards.
Exact binaries/features and source 4c52595 recorded; these reuse the already validated
read-only-diff build bytes. No user repository, global helper/keyring or app state used.
Initial preparation checked write_trust before the native schema was initialized;
the unused untrusted fixture was never executed. Preparation now accepts absent schema-1
write tables only before native startup, then post-start preflight requires initialized
schema with zero trusts/operations. Failed first preparation output retained.

Current native authoritative trust review names only the parent and child. User
has been asked specifically to authorize this hook-enabled pair; earlier hook-free
trust grants do not cover it. Post-start preflight confirms exact repository bytes/
modes, fixed hook hash, zero trust rows, zero operations and no hook run marker.
No hook or local commit executed by the app. Control stays untrusted.

Next after exact authorization: actual child hook rejection must stop the parent
without publication; then remove only the fixture rejection marker, prepare fresh
native review, prove child hook running from its marker/journal and cancel through
native UI. Verify group cleanup, unchanged refs/index/parent/control and no replay.
A separately reviewed successful retry can then prove supported bulk hook execution
and fresh parent gitlink reauthorization while preserving unselected content. Hook
bytes/config remain constant throughout; test markers are the only mode inputs.
This is prepared, not a native acceptance pass or G05 closure.

[Evidence](../validation/g05-native-child-hook-2026-11-08/) contains preparation, scan,
exact baseline, reviewed script/hash and no-execution preflight. No push requested.

## Actual native child-hook rejection — passed

User authorized this exact parent/child hook fixture. Native authoritative and OS
confirmation granted only the pair. Fresh batch 1d172d07-2ca9-4fc0-ae55-48bb444205fb
showed child-first order, fixed executable pre-commit manifest, one file per worktree,
exact shared message and zero blockers. Separate native approval ran child hook once.
Child operation 892338d5-7a92-408c-be61-1440638a17c7 blocked with HookRejected; parent
7345051b-e229-4345-946f-18aaf56a0501 cancelled_before_publication without starting.
Native results show partial_failure with zero published items, not successful commits.

Every repository byte/mode matched preparation exactly (including refs/index/config,
child, parent and independent control). Hook SHA unchanged; one marker run, exact two
authorized trusts, control untrusted, SQLite integrity ok. See verify-rejection.json
and records-rejection.json. This closes the actual native child-hook rejection/parent
stop case only; proven-running cancellation and separately approved retry remain next.

## Proven-running native cancellation and ordinary restart — passed

Removed only the outer reject-child marker; hook bytes/config/trust were unchanged.
Fresh exact batch 100b545d-71f1-49bd-b672-23536a590381 separately approved. Before Cancel,
the child marker existed with run count two, durable operation
e56ba4ec-2f34-4c0c-adcc-af0f69cc2958 was hooks_running, actual /bin/sh hook PID 8782
and sleep descendant were present in its owned process group 8782. The release-child
marker did not exist. See proven-running.json; this was execution, not pending review.

Native Cancel review stopped that active batch. Child and parent operation
7b5a0835-9372-4ddc-bc4c-f9c35ffefb14 ended cancelled_before_publication, zero publication.
The owned hook process and whole group were gone; no release marker was used to let
it finish. Every repository byte/mode remained original, fixed hook unchanged, exact
two trusts, control untrusted, integrity ok. No parent execution or owned lock residue.

Ordinary restart displayed failed and cancelled history with native preparation
disabled. Hook count stayed two; exact operation/batch payloads and all repository
bytes/modes preserved, no hook/commit replay. Acceptance and ordinary exits 0,
writer-lock reopen passed. See cancellation-restart-proof.json, cancellation/group
proof and records. Actual native child rejection and running Cancel gaps now passed;
separately approved successful retry/parent gitlink review remains next. G05 stays partial.

## Fresh supported-hook retry and stale-parent stop — passed

Added only the release-child marker, leaving hook bytes/config/trust unchanged.
Fresh native batch 4251b893-05c7-4c9a-9b22-9e24077c9b7b reviewed both exact plans and
fixed hook; separate approval ran the child hook a third time and published exactly
one child commit b270f8b451d02046f7011564b96965b69478dd8d. Correct exact message/tree,
child clean according to optional-locks-disabled status, no earlier failed/cancelled
operation replay. Batch stopped at needs_reauthorization; original parent proposal
cancelled without publication because its future gitlink was not authorized.

Parent HEAD/index/config/logs/working file and full independent control bytes/modes
remain original. Prior rejection/cancellation operation/batch payloads unchanged,
fixed hook/hash and exact two trusts unchanged, integrity ok. The first read-only
oracle used git diff --name-only with refresh disabled; it reported a stat-only ghost.
As established by the owning read-only diff fix, raw/name metadata alone is not
content evidence. Corrected to safe status --porcelain and checked the oracle itself
preserved all files; no index refresh or baseline overwrite. This was an oracle
false positive, not additional repository modification. See verify-child-success.json.
Fresh exact parent-only gitlink review/approval is still required and not yet accepted.


## Fresh parent-only approval and final ordinary restart — passed

Select unfinished worktrees selected only the parent. A fresh review contained
file.txt and the exact module gitlink from 0fb295afa2af9a6a00d8ab6d7605781b03321f4f
to b270f8b451d02046f7011564b96965b69478dd8d, with the parent's separate message
override. Independent native approval completed batch
05000b4e-4e9a-48bf-a19f-418945a003bf and operation
6702e7c9-2825-424c-8a77-3a92091ccc23, publishing parent commit
317d70a1e62c0d21aaa52f2e9e7fd8cd71180b3d with reconciled index.

The final oracle proves one commit per selected repository, exact parent message,
file and child OID, clean parent and child, unchanged child checkout and actual Git
metadata during parent retry, three hook runs total, unchanged hook and trusts,
and byte/mode-exact untrusted independent control. Prior operation/batch records
are unchanged. No child hook or completed item replayed.

Ordinary restart visibly showed clean parent/child, dirty unselected control,
parent-only selection, read-only status and disabled native preparation. Bulk results
retained complete parent approval, needs_reauthorization child batch, cancelled
batch and HookRejected failure. After native Quit the owned app process was absent;
writer-lock reopen succeeded. verify-final.py passed before and after restart,
checking exact repository bytes/modes and operation/batch payloads with SQLite
integrity ok. See verify-final-before-restart.json, verify-final-after-restart.json,
repository-final.json, records-final.json and lock-reopen-final.json. The final exit
code was not retained across tool compaction; process absence and lock reopen were
independently verified. No product change or full matrix rerun was needed for this
native-only evidence step. This closes this hook chain, not all of G05.
