# Explicit bulk path exclusions — November 8, 2026

Fresh Core review can include a separately discovered submodule gitlink absent from
the restricted status checklist. Added an exact relative-path exclusion entry per
worktree. Added/stored exclusions remain visible as removable checkboxes even when
not in cached status. Enter adds only the exclusion and cannot submit the outer
review form. No trimming or path normalization; Core's existing relative-path and
fresh-eligibility checks remain authoritative. Unknown/ineligible exclusions block
preparation, rather than grant any new file or execution authority.

Two focused renderer regressions cover Enter/no request, exact path payload including
leading/trailing spaces, visibility and removal. Full desktop gate passed 89 renderer,
13 default-host and 16 all-feature host tests, type/lint/contracts/format, builds and
strict Clippy. Initial whitespace-label test relied on unnormalized label text; fixed
the test to assert actual request bytes while using normalized accessible labels.
No Core implementation change in this checkpoint; its gitlink/exclusion regression
passed in the preceding package.

Native follow-up reused the same explicitly authorized parent/child identities. An
operator-created child fixture commit and dirty parent file established another
unstaged gitlink; this is setup, not native product publication evidence. Native
Enter added module without any proposal/batch row. Removal/re-addition worked. Fresh
native review visibly contained only file.txt and no module patch; inspected stored
proposal confirms exact selection. Cancelled without confirmation or product commit.
Repository byte/mode baseline and previous operations/batches unchanged, one new
cancelled proposal only, original two trust rows, SQLite integrity ok. Ordinary
restart retained selected Exclude module and disabled Prepare native bulk review.
Both exits 0, state lock reopened, no network/push or new trust grant.

Selected explicit gitlink exclusion UI gap is closed. Broader G05 fault/cancellation/
linked-worktree and G03 accessibility/scale criteria remain open. Earlier submodule
case's current-state oracle has a one-child-commit precondition and is now historical
after this intentional setup; use this package's verify.py against the retained
fixture. [Evidence](../validation/g05-bulk-exclusions-2026-11-08/).
