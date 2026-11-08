# G07 exact individual publication checkpoint

November 8, 2026. Local implementation checkpoint; gate acceptance remains partial.

An independent native publication approval binds one committed source, one existing
remote branch, its exact observed previous tip, the effective transport/authentication
policy, and a new pre-push check. A completed check is never a publication token.
Core exports only the approved reachable object closure into isolated transport
storage, verifies its integrity and fast-forward ancestry, observes the destination
again, then attempts one atomic expected-old guarded update. The lease is a compare
and swap guard after ancestry proof; divergent updates, new branches, deletions,
force options, mirror/refspec input and implicit child pushes are unsupported.
The source object inventory is bounded to 10,000 objects and 64 MiB uncompressed;
pack output is bounded to 64 MiB and isolated artifacts to 128 MiB.

The durable journal enters `publishing` before transport starts. Missing receipts,
server rejection, cancellation or process loss after that boundary leave an
unconfirmed outcome. Restart reads that journal without retrying transport or
hooks. A separate recovery approval records a fresh, separately approved destination
observation. Recovery rejects pre-attempt observations even when still recent and bound to the
unchanged source; the observation must be proposed and completed after the terminal
record, or after the lost review creation on crash. Matching source evidence records
observed state, not retrospective proof
of which actor published it. Managed refs/index/config/upstream remain unchanged by
Core; reviewed hooks retain their explicitly disclosed side effects.

Ordinary builds retain their write gate. Renderer requests cannot supply approval,
force flags or arbitrary transport policy. The new acceptance-only startup switch
`--acceptance-local-transport` is compiled only with native acceptance support and
resolves exclusively to the existing marked disposable fixture root. It is absent
from ordinary startup policy and renderer IPC. Local publication additionally refuses
workspace/metadata overlap and non-standalone or symlinked destination metadata.

## Validation disposition

- Focused Core matrix: 10 passed, one subprocess-only crash worker ignored in the
  parent runner and explicitly invoked twice by the actual SIGKILL test. SHA-1 and
  SHA-256 exact publication, unselected object exclusion, managed preservation,
  destination drift, source/hook drift, decline, precancel, server refusal, proven
  running server-check cancellation, restart projection and separately approved
  observation recovery passed.
- Focused renderer planner/history tests: 11 passed. Separate publication review,
  ordinary gate, explicit observation recovery and no implicit retry covered.
- Full desktop after the recovery fix: exit 0, 93 renderer / 14 ordinary host /
  17 all-feature host tests, type/lint/contracts/format/build/strict Clippy passed.
- The regression using a recent observation of the unchanged current source failed
  before the freshness guard and passed afterward. The earlier observation-before-
  source-commit fixture was already refused by generation revalidation; both logs
  are retained to distinguish the actual defect from that already-covered boundary.
- Final-source native exact leaf publication passed after separate fresh observation,
  Rust review and independent macOS approval. Exactly 6 objects / 685 uncompressed
  bytes; server tip equals approved `4b6e1ea1dbeabd9350368a24ad83206366dc7608`.
  Prior publication/check records, all managed files/modes, old operations/batches
  and trust unchanged. Parent remains revoked; dirty leaf content remains untouched.
  Artifacts and final source manifest are separately named and hash recorded.
- Final ordinary restart/history passed with both completed publication records,
  explicit read-only status, unchanged earlier journals/trust/repository bytes and
  no replay. Acceptance and ordinary exits 0; integrity and lock reopen passed.
- Full Core final script exit 0: default 62 / experimental 97 / controlled 87 /
  application-services 243 / all-features 288 passed, zero failures; isolated lab
  24/24 passed with no skips. Formatting/builds/strict Clippy passed in every
  applicable configuration. Ignored worker/opt-in cases remain explicitly labeled;
  the SIGKILL publication worker was invoked by its successful parent test.
- The earlier native publication and ordinary restart/history also passed; its
  immutable artifact/evidence remains historical. Neither preflight is a full-
  matrix pass. Final source hashes and artifact hashes independently verified.
- Initial full-run preflight hit two 15-second inspection setup deadlines under
  oversubscribed tests. The runner now defaults to four workers, matching Core's
  existing four-process cap; no production deadline or process guard changed. That
  run and the four-worker run stopped for the freshness correction are preserved.
  Do not infer full-matrix success from either partial preflight log.

G07 remains open for the broader authenticated publication, topology, native
interruption/recovery and usability matrix. G08 cannot be marked accepted from an
individual transport result. Ordinary writes remain disabled pending G04 closure.


[Validation artifacts](../validation/g07-publication-2026-11-08/) retain the exact
final source manifest, feature/binary hashes, independent repository/server/journal
proofs, native lifecycle trace, successful full logs and failed/stopped preflights.
The fixture server was deliberately reset to the known old tip between the first
and final-source native proofs; only this disposable bare fixture ref changed during
setup. Original server checkout and prior artifacts were preserved separately.
