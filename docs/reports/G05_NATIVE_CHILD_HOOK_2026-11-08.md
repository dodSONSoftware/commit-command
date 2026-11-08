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
