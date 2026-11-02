# M5B remote inspection implementation checkpoint

November 2, 2026. Local development/commits are authorized; no push. This is the read-only foundation of M5B, not completion of network fetch/push or the overall M5 milestone.

## Delivered

Core now inspects effective fetch/push URL sets using Git's URL-rewrite semantics, configuration scopes/origins, upstreams and branch/global push-remote precedence. Names remain case-sensitive. Suggestions do not select or authorize destinations. Multiple URLs, mirrors, custom programs, missing/ambiguous upstreams and unsupported/helper destinations remain explicit policy issues. The inspector never contacts a server or invokes a credential/remote helper or SSH command.

Public data redacts embedded URL credentials/query/fragment and withholds malformed/helper/ambiguous SCP-style destinations. Raw configuration, proxy credentials, helper commands and raw effective URL values are not serialized or persisted. The result exposes relevant configuration flags and a digest, with both effective-config and local-generation drift checks. Inspection uses bounded subprocesses, cancellation and explicit record/remote/rewrite/URL/payload limits.

A scoped read-only `Remotes` request is available for untrusted worktrees without granting write authority. A repository's **Remotes and upstream** panel loads only on explicit request. It displays destinations, configuration origins, policy issues and inspection time, and states that cached tracking refs do not establish current server state. There are no new network execution controls or capabilities.

One optional direct Core dependency, `url` 2.5.8, provides standards-based authority/credential parsing. It was already in the desktop graph; no package versions were upgraded there. Default Core builds exclude this dependency/module. The generated-contract tool now explicitly uses the project's 80-column formatter setting; the first desktop gate exposed its previous 120-column default, which conflicted on the new transport union. Regenerated contracts pass both drift and format checks.

Contract and next network boundary: [M5 remote boundary](../architecture/M5_REMOTE_BOUNDARY.md). URL resolution follows [Git remote documentation](https://git-scm.com/docs/git-remote); network implementation remains separately gated.

## Verification actually run

Eight real-Git inspection tests cover distinct fetch/push rewrites, explicit-pushurl precedence, multiple destinations, case-sensitive names, origins, helper non-execution, embedded secret redaction, unsupported/helper/control-containing destinations, missing upstream/detached HEAD/cancellation, push-remote precedence, malformed names/limits/case-insensitive TLS flags and malformed SCP password redaction. Repository-byte snapshots and an external sentinel prove no managed-state mutation or configured helper execution in the tested cases. A ninth application test verifies untrusted/scoped access and the host-config source. A renderer test verifies on-demand scoped requests, inert text and absence of fetch/push controls.

The complete Core and final desktop validation scripts passed:

| Core feature configuration | Passed | Failed | Ignored (workers / host limitation) |
|---|---:|---:|---:|
| Default | 53 | 0 | 1 |
| Experimental writes | 88 | 0 | 2 |
| Controlled writes | 76 | 0 | 1 |
| Application services | 113 | 0 | 2 |
| All features | 153 | 0 | 4 |

Counts overlap. Ignored totals include one host filesystem limitation for non-UTF-8 filename creation in each configuration; remaining ignored subprocess workers are invoked by parent acceptance tests. That host limitation is not a pass. All applicable Core builds, formatting and strict Clippy passed, including mandatory isolated positive signing and crash matrices. The experimental lab passed 23 with one optional diagnostic skip. The desktop passed typecheck/lint/contract drift/format, 27 renderer tests, frontend production build, eight default and eight all-feature native host tests, strict native Clippy and release build.

Evidence: [Core matrix](../validation/m5b-inspection/core-validation.txt), [first desktop gate/format failure](../validation/m5b-inspection/desktop-validation.txt), [final desktop gate](../validation/m5b-inspection/desktop-validation-final.txt), [summary](../validation/m5b-inspection/summary.json), [source hashes](../validation/m5b-inspection/source-sha256.json). Base source revision `b924587` plus the recorded changes. No new native visual/accessibility session was run; the preceding native session was blocked by the locked Mac and remaining acceptance is deferred under the user's instruction.

## Remaining work

M5B/C still needs the separately authorized network service, transport/credential/hook policy, explicit fetch, guarded upstream setup, individual/bulk push, reachability proofs and uncertain-publication recovery. The read-only adapter's protocol ban is unchanged. The proposed separate transport repository boundary is documented for the next package; it is not yet implemented. SSH/Keychain/platform acceptance is neither proven nor excluded from the product by this checkpoint.

M3/M4/M5A native acceptance, AI, Actions, backups/retention and release/platform gates remain open. Ordinary desktop writes remain disabled by default. The unrelated `.gitignore` edit is preserved unstaged, and no credentials/keyrings/state databases enter this commit.
