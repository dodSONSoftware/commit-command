# M5B independently reviewed branch observation checkpoint

November 2, 2026. Local development and commits are authorized; no push. This implements a credential-free remote observation boundary. Fetch, authenticated transport, upstream mutation, push and the full M5 milestone remain unfinished.

## Delivered behavior

- An exact remote/full branch reference is prepared without network activity and requires an independent native confirmation. The existing ordinary-build gate denies this new request. Renderer JSON cannot approve or execute it.
- Execution revalidates workspace/worktree identity, trust, effective configuration/generation and reviewed Git/HTTPS-helper/CA bindings. No display URL is used to reconstruct transport authority. Decline/replacement/revocation/context invalidation cancels proposals. Session/review authority is never restored from SQLite.
- A private bare transport repository and empty HOME isolate network execution from managed configuration, URL rewrites, credential helpers, proxies, cookies, hooks, SSH commands, recursion, lazy fetching and inherited environment. HTTPS verifies certificates and hostnames and rejects redirects. A custom CA is trusted host configuration bound in the plan, not a renderer option. Actual credentials are not supplied. Local file transport is limited to an explicit disposable fixture root in trusted host options, disabled in normal host configuration.
- One bounded `ls-remote --refs --exit-code` observes one validated full branch. Results record OID or explicit absence and time. Managed commits/objects/index/working files/refs/upstreams remain unchanged. Raw server diagnostics are not persisted. Running cancellation terminates the transport process group.
- Versioned durable proposal/running/result records reuse the single SQLite owner. Results have per-worktree/workspace scope, bounded pagination and explicit interrupted/orphaned projections. No restart replay or retry execution endpoint exists.
- Desktop controls explicitly choose a remote and full reference, prepare native review and refresh timestamped results. The view distinguishes local inspection, network observation and past server knowledge. OpenSSL is now checked before macOS Core validation because loopback TLS fixtures require it; this adds no application runtime dependency.

Contract: [M5 remote boundary](../architecture/M5_REMOTE_BOUNDARY.md). No managed adapter protocol ban was relaxed, and no network operation is started by monitoring or local Refresh.

## Verification actually run

Ten new real-transport integration cases passed. They cover native-only authority and byte preservation of both client and local remote; absent refs without creation; untrusted/default local-transport rejection; decline/pre-cancel/configuration drift; restart capability loss; verified loopback HTTPS with a reviewed CA; managed helper/proxy/unsafe-TLS override isolation; untrusted certificates and hostname mismatch; redirect rejection with no second request; cancellation after an HTTP request actually begins; changed CA binding before any request; and missing credentials without helper fallback or raw server text leakage. Some cases combine related assertions. No external service or actual credentials were used. Loopback certificates/key material remain disposable fixture artifacts and are not included in this report directory.

The full Core and desktop validation gates passed:

| Core feature configuration | Passed | Failed | Ignored |
|---|---:|---:|---:|
| Default | 53 | 0 | 1 |
| Experimental writes | 88 | 0 | 2 |
| Controlled writes | 76 | 0 | 1 |
| Application services | 123 | 0 | 2 |
| All features | 163 | 0 | 4 |

Counts overlap. Each configuration includes one ignored host filesystem limitation for non-UTF-8 filename creation; that limitation is not a pass. Other ignored subprocess workers are invoked by parent crash tests. Two preceding checkpoint reports now have corrected ignored-column labels/prose; their raw counts/evidence are unchanged. The separate experimental lab passed 23 scenarios with one optional diagnostic skip. Mandatory controlled-write positive signing and all-feature crash matrices passed.

All applicable Core builds, format checks and strict Clippy passed. Desktop typecheck/lint/generated-contract drift/format, 28 renderer tests, frontend production build, eight native host tests per default/all-feature configuration, strict native Clippy and release build passed. Native host tests also verify the new network review gate; the renderer regression prepares only review/result requests with explicit remote/ref choices.

Evidence: [Core matrix](../validation/m5b-observation/core-validation.txt), [desktop gate](../validation/m5b-observation/desktop-validation.txt), [summary](../validation/m5b-observation/summary.json), [source hashes](../validation/m5b-observation/source-sha256.json). Source base is `9124210` plus the recorded changes. This is automated source/build/loopback evidence; no new native visual or accessibility acceptance is claimed.

## Remaining work and limits

Next implement explicit reviewed fetch and object/ref publication, credential/SSH policy, guarded upstream setup, separate individual/bulk push, dependency reachability and uncertain-publication recovery. Global Operation History integration for new remote records remains part of that package. This observation service does not mutate managed Git or establish current synchronization, authenticated remote access, remote SHA-256 interoperability, fetch/push durability or real service credentials.

M3/M4/M5 native acceptance remains open after the preceding locked-Mac block; native checks can resume when the facility is available. Ordinary desktop writes/network review remain acceptance-only and disabled by default. AI, Actions, backup/retention and release/platform work remain under the completion plan. The unrelated `.gitignore` edit is preserved unstaged. No external repository was pushed and no credentials, keyrings or state databases are committed.
