# G08 guided push queue — November 8, 2026

Implemented a durable, explicitly selected child-first push queue on the accepted
individual publication boundary. Queue creation authorizes no transport. Each next
item requires a fresh independent exact native approval; this implementation does
not expose unattended aggregate approval or automatically enroll a child.

The immutable plan freezes selected identities, source refs/OIDs, destinations,
configuration/generation, child mapping identities and hook policies. Only matching
fresh observation IDs may replace older evidence. A selected child that published
requires a separately approved destination observation proposed and completed after
its successful item before its parent can receive publication authority. A source
already matching its recorded observation is explicitly skipped without a push.
The existing planner rejects cycles, incomplete child coverage and unsupported
relations. All selected trust, transport and hook prerequisites are checked before
the queue is saved.

Cancellation, failure or uncertainty stops the queue. Completed individual outcomes
remain intact. Recovery of an individual uncertain publication never advances the
old queue. A retry is a new explicit plan. Restart projects unfinished queues as
interrupted and restores no publication capability. Saved queue listing/paging and
unified operation history are read-only and available in ordinary builds.

Focused Core evidence covers SHA-1/SHA-256 child→fresh parent publication with exact
managed-file/mode preservation, wrong order and old-child-observation refusal,
decline, missing unselected child, revoked trust, frozen-source and request-field
guards, observed-current skip, later server rejection, independent recovery, fresh
parent-only retry, unfinished restart and actual SIGKILL between items. Crash workers
are explicitly invoked by their parent tests; their direct test entries stay ignored.

Native acceptance reused only the previously trusted leaf and middle worktrees in
the existing three-level disposable fixture. The top-level parent remained revoked
and the independent control untrusted. Preparation made an explicit empty leaf
source commit and a middle gitlink source commit with hooks/signing disabled, while
preserving the dirty leaf file. The middle's disposable remote was converted to a
standalone bare server without changing managed remote configuration. Preparation
deltas and the post-preparation preservation baseline are recorded separately.

The native two-item queue published the leaf first; a durable checkpoint proves the
middle server remained at its old tip and its item pending. After fresh child
observation, the middle's original observation expired during execution. Core
blocked that publication and stopped the queue while preserving the completed leaf.
This is a passing safe-stop case, **not** an uninterrupted native two-item success.
Fresh independently approved parent/child observations, a new explicit parent-only
queue and separate native push approval then published the exact middle source.
The child was not pushed again. Both outcomes survived ordinary read-only restart;
the stopped queue showed its retained child success and blocked parent, and the new
queue showed completion. Unified history displayed both queue and individual results.

Independent read-only oracles verified managed bytes/modes, old operations and
journals, unchanged trust/revocation, exact destination OIDs, fresh child observation
after publication and before parent review, unchanged servers/journals across ordinary
restart, SQLite integrity, clean exits and writer-lock reopen. Source and both native
artifact hashes are recorded. All evidence is under
[`g08-guided-push-2026-11-08`](../validation/g08-guided-push-2026-11-08/).

Full Core validation exited 0: 62/97/87/256/302 passed across default, experimental,
controlled, application-services and all-features, with 1/2/1/4/10 explicit ignored
entries, zero failures and lab 24/24. Format, builds and strict Clippy passed in each
configuration. Full desktop exited 0: 99 renderer, 14 ordinary-host and 17 all-feature
host tests, types/lint/contracts/format/builds/strict Clippy passed. Counts are recorded
in `matrix-summary.json` and the desktop log. Focused renderer tests cover explicit
queue creation, distinct observation labels, next-item ordering, default write gating
and stopped/interrupted no-continuation. The review's waiting text now accurately
covers native approval and active execution; history labels observed remote state.

G08 remains partial pending broader native uninterrupted multi-level/topology,
authenticated queue, phase-specific interruption, accessibility and usability
acceptance. This selected evidence does not close G02/G03/G04/G06/G07, enable ordinary
writes, or establish release readiness. Next functional work is G09 optional AI and
deterministic Actions, with the existing manual workflows independent of providers.
