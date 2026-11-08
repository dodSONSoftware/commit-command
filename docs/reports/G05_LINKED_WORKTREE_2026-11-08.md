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
