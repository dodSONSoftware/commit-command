# Reviewed fetch download checkpoint

November 2, 2026. Base `da23197`; completion-plan M5B / preparation for PUSH-002 through PUSH-006. User authorized local development, tests/reports and checkpoint commits; no push. This checkpoint implements only the independently reviewed download stage. **Managed object import and tracking-ref publication remain unimplemented. This is not completed Fetch or M5 acceptance.**

## Inputs, boundary and behavior

A completed, scoped remote observation with an existing branch OID is the input. Preparation revalidates current worktree/trust/configuration/generation and builds a fresh transport/tool/CA binding without contacting a server. A new Rust-only native confirmation authorizes the exact remote/full branch and expected observed OID. Decline/replacement/context invalidation cancels the proposal. Source absence, foreign/unknown scope, stale configuration and expired/replayed confirmation fail closed. Restart does not restore confirmation authority or resume a download.

Execution uses a new bare repository and empty HOME in private app-owned storage. Extracted the existing transport initialization/revalidation helpers so observation and fetch share the already tested TLS, credential/proxy/helper/hook/environment isolation. The managed inspection adapter's protocol ban is unchanged. HTTPS verifies certificates/hostnames and refuses redirects; local file transport stays restricted to a trusted host-configured disposable root, disabled in ordinary configuration. No credentials are supplied.

One exact branch is fetched into a fixed isolated ref with explicit ref mapping, no tags, FETCH_HEAD, maintenance, commit-graph writes or recursion. These flags follow the current official [Git fetch contract](https://git-scm.com/docs/git-fetch). The downloaded commit must equal the observed OID; server drift stops the result. Strict fsck and the actual object inventory precede creation/verification of a standalone [Git bundle](https://git-scm.com/docs/git-bundle) containing the captured branch. The synced bundle's digest/size, object count/uncompressed bytes and capture time become durable data, never an import capability.

Bounds: 10,000 objects, 64 MiB uncompressed objects and bundle bytes, 1 MiB stdout, 64 KiB diagnostics, existing bounded process deadlines, and a polled 128 MiB/1,024-entry owned-artifact budget. The process runner gained a private trusted check callback before spawn, during execution/descendant-pipe draining and after completion; failure kills/reaps the owned process group. The artifact budget is polled, not an OS quota, and may transiently overshoot between checks. Special files/symlinks fail closed. Current bundle-path invocation requires UTF-8 app-data paths; this adds no general native-path support claim.

Version-1 capture proposal/running/complete/blocked/cancelled records reuse the existing SQLite owner/schema. Orphans project interrupted without rewriting journals. Scoped paged history and the unified audit include the new typed records. UI offers Review branch download only for completed observations with an OID, behind the existing non-default desktop-writes gate. It displays exact expected commit, checksum/limits/results and states that the repository and tracking refs have not been updated. No import, retry-execute or push endpoint was added.

## Verification actually performed

Six new real-transport integration cases join the existing ten observation cases. All 16 passed with permitted loopback access. New coverage includes:

- A remote-only commit not present in the managed object store; exact bundle head/digest, independent import into a separate disposable verification repository, exact blob contents and unchanged managed/server bytes.
- Fresh native authority, decline, pre-cancel, configuration drift, absent/forged source rejection, restart capability loss/no replay and scoped unified audit visibility.
- Actual remote branch movement after observation/review, yielding ConcurrentChange without managed import.
- Real object download over verified loopback HTTPS; authentication/redirect rejection without raw server-text retention or redirect following; cancellation after an HTTP request actually starts.

Added unit coverage for object/count/size parsing bounds, sparse oversized artifacts/symlink rejection and a running process stopped/reaped by the new trusted artifact guard. Existing observation TLS/helper/proxy/certificate/cancellation cases still pass after helper extraction. The sandboxed transport attempt failed because loopback binding was denied; it was rerun with permitted execution, not counted as passing. Earlier compile/Clippy corrections remain separate from successful final logs.

Full Core build/test/strict-Clippy matrix passed:

| Configuration | Passed | Failed | Ignored |
|---|---:|---:|---:|
| Default | 54 | 0 | 1 |
| Experimental writes | 89 | 0 | 2 |
| Controlled writes | 77 | 0 | 1 |
| Application services | 137 | 0 | 2 |
| All features | 177 | 0 | 4 |

Counts overlap; existing non-UTF-8 filesystem ignores and parent-invoked subprocess-worker ignores retain their meaning. Positive isolated signing and crash matrices passed. The separate disposable lab passed 23 cases with one explicit optional diagnostic skip. Full final desktop gate passed: 46 renderer tests, 11 native host tests under each configuration, typecheck/lint/contracts/format, production frontend/native builds, strict Clippy and release build. Native host tests cover the new write gate; renderer tests prove review-only requests and absent-branch/default gating.

Fresh native ordinary fixture `/private/tmp/gitfleet-m3-native-5zxerugm`: explicitly synthetic complete/blocked capture records and a completed observation exercised rendering only. Observed disabled Review branch download, scoped results/refresh, unified branch-download filtering, explicit non-import wording, restart restoration and safe native Quit. The final hashed artifact uses the user-facing branch download category. All four repository file hashes/modes and seeded history payloads matched; zero trust grants, zero write operations and no network artifact directory existed afterward. CLI reopened the state writer lock. AX/screenshots were inspected inline; no PNG saved. **Native network execution/approval of this new operation remains NOT RUN; synthetic history is not transport evidence.** No external service, real credentials, personal repository or global setting was used.

Evidence: [full Core gate](../validation/m5b-fetch-capture/core-validation.txt), [permitted real transport suite](../validation/m5b-fetch-capture/transport-tests-permitted.txt), [final desktop gate](../validation/m5b-fetch-capture/desktop-validation-final.txt), [final source/artifact](../validation/m5b-fetch-capture/source-artifact-final.json), [native summary](../validation/m5b-fetch-capture/native-summary.json), [counts/scope](../validation/m5b-fetch-capture/summary.json). TLS private keys, state databases and downloaded bundles are disposable artifacts and are not committed.

## Next package and limits

Implement a separate immutable import/tracking-ref review: validate the exact completed bundle/digest/OID and current worktree identity/configuration/trust, bind the explicit destination ref and expected old value, validate/import objects durably, publish only that ref with guarded compare-and-swap semantics and truthful interruption recovery. Never reuse the download confirmation as publication authority or replay uncertain updates. Include bundle tampering, ref/symref/config drift, shared-directory contention and crash phases. Updating tags, upstreams, FETCH_HEAD, local branches, index or working files must not occur implicitly.

Authenticated transport/SSH, upstream configuration, individual/bulk push and dependency reachability, broader native acceptance and SHA-256 fetch interoperability still need evidence. Failed/cancelled download artifacts currently remain for inspection; retention/backup treatment and cleanup policy belong to the remaining M4/M8 gates. Ordinary desktop writes remain disabled. Full M5 and the application remain incomplete. User .gitignore, docs/graphics, docs/README.md and docs/APPLICATION_OVERVIEW.md work was excluded. No push.
