# Linked-worktree bulk acceptance — November 8, 2026

The new Core regression exercises two selected checkouts sharing a repository ID
but having distinct worktree IDs. Trusting only the original checkout blocks the
whole batch without repository changes. After independently trusting the linked
checkout in the test harness, the batch publishes two independent branch commits
with their exact selected content. The dirty unselected checkout stays byte-for-byte
unchanged. Focused application-services regression passed (1 test). No product patch.

The native disposable workspace has been rescanned with a new sibling linked
checkout at `/private/tmp/gitfleet-g05-submodule-fzz5ffky/workspace/linked-parent`.
Both selected checkouts start at `9312e26aab1068954be779b91313322544aa7ce4` with
independent dirty file contents. The original parent and submodule retain their
previously authorized trust; the new linked checkout and unselected control have
no trust. The authoritative review names only the new checkout. Native approval,
execution, preservation and restart checks remain pending specific authorization.
Core test trust setup does not authorize a native grant.

[Evidence](../validation/g05-native-linked-2026-11-08/) contains the test output,
preparation script, complete scan, exact byte/mode baseline and fixture identities.
Preparation initially used an incorrect JSON root field (`display`); recording was
corrected to `value` without recreating the worktree. The preparation script verifies
the branch when resuming an existing fixture. It must not be rerun after native
execution because it deliberately changes fixture working content.

G05 remains partial. This checkpoint proves Core linked-checkout batch isolation;
it does not prove native execution or complete the gate.

## Native completion

Specific authorization was received for the new linked checkout only (and separately
for the named G06 fixture). The native untrusted review showed two selected/one
blocked; Cancel preserved exact fixture bytes/modes and the original two trusts.
Then authoritative and OS-native confirmation granted only linked-parent trust.
Fresh batch `fb0e724f-5d0f-4bbf-a805-72903c3e29a4` reviewed the two exact file patches,
independent messages, expected common starting HEAD, and zero blocked items.
Native exact approval published main `f369e757e416e34e80d381d33b8340ebb13f5928`
and linked branch `e6802d8ab7dcc19c6a6054972f75830390ab2c9f`. Both operations
completed with reconciled indexes. One commit per selected branch, only file.txt
changed, exact messages/content, parent gitlink unchanged. Control, child checkout
and submodule metadata retained exact bytes/modes; trust set is exactly the three
authorized worktrees. No push or signing/hook execution configured.

Ordinary restart visibly retained complete/cancelled history and disabled native
bulk preparation. Post-restart repository bytes/modes and all operation/batch records
matched the post-execution baseline exactly: no replay. SQLite integrity and CLI
writer-lock reopen passed. See verify-before-restart.json, verify-after-restart.json,
untrusted-cancel.json and lock-reopen.json. Earlier preparation-pending statements
above describe the first checkpoint; this follow-up closes this native case only.
