# G09 restricted repository context capture — November 8, 2026

Disposition: implemented with automated selected acceptance. Capture is read-only,
requires neither AI configuration nor worktree trust, and cannot transfer content.
Native context inspection, provider integration and candidates are next G09 work.

`ai/capture.rs` and `PreviewAiContext` capture explicit selected index/working-tree
paths, up to eight explicitly chosen tracked nearby-source files and eight tracked
documentation files, current-HEAD history (0–20 commits), and selected-path history
(0–10 commits each). There is no recursive source scrape or all-branches history
fallback. Selected changes are bounded to 128 distinct normalized UTF-8 paths.
Each blob/file/history response is bounded to 1 MiB, aggregate fragments to 16 MiB,
and the filtered serialized context uses the independent configured byte budget.

Index selection captures immutable staged content. Working-tree selection compares
HEAD with the entire chosen file, including staged plus unstaged changes; untracked
selection captures full file content. Unborn branches and selected staged/working
file deletions are supported. Whole-file comparisons are built from captured bytes
and immutable HEAD blobs, without invoking diff/text conversion or rereading mutable
working content through Git. These deliberately verbose comparisons favor explicit
coverage; budget omissions remain visible rather than implying complete analysis.

Read-only descriptor traversal pins each ancestor, refuses symlinks, hardlinks,
nonregular/nonowned/special files and nested repository markers. Related source/docs
must be tracked regular paths; ignored untracked files are refused. Non-Unix capture
fails closed; it is not cross-platform acceptance. Raw-content fingerprints and
repository generation cover staged/config/ref/identity changes plus even excluded
or redacted material. A second file pass detects changed observations; `revalidate`
re-captures the exact specification and checks raw and filtered fingerprints. No
filesystem snapshot or protection against arbitrary same-user hostile ABA changes
is claimed. No source-bearing diagnostics are persisted by this service.

Validation: complete application-services library 57 passed, application integration
39 passed, including seven new capture cases: exact staged/untracked selection and
no trust/AI setup, redacted change invalidation, HEAD-to-full-working comparison,
unborn and deletions, links/nested/ignored refusal, bounds/cancellation, and explicitly
selected related source/docs/current-branch history with secret screening. Fixtures
verify exact repository bytes/modes before/after reads. Strict all-feature/all-target
Core Clippy, generated contracts, desktop TypeScript check, formatting and diff checks
passed. Evidence: `docs/validation/g09-ai-context-capture-2026-11-08/`.

Unsupported selections (including gitlinks, unmerged index entries, non-UTF-8 labels
and oversized source objects) fail the AI preview rather than falling back to a
broader capture. Broader selection/native usability and hierarchical model analysis
remain open. No provider, real credentials, Git writes or publication were used.
