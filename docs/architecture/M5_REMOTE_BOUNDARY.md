# M5 remote inspection and operation boundary

Reconciled November 7, 2026. [The current record](../EXECUTION_STATUS.md)
owns acceptance status and next work. Local inspection, reviewed observation,
authentication profiles, isolated fetch download, separate offline import/tracking-ref
publication, guarded upstream setup, child evidence, dependency planning and reviewed
pre-push hook checks, exact individual fast-forward publication and observed-outcome
recovery are implemented within documented subsets. Bulk publication is unfinished.
Ordinary desktop write/network execution remains disabled. All routine validation
uses disposable repositories and isolated services.

## Implemented: local inspection

`remote::inspect` is an application-services Core API. Its only Git commands inspect repository/configuration metadata, validate references, and resolve URLs. Protocol access, lazy fetching and terminal prompts are disabled. It does not invoke remote helpers, credential helpers, SSH commands or servers. A scoped `Remotes` application request is available even for untrusted worktrees; it grants no write/network authority. The per-repository desktop inspector loads only on explicit request and does not fabricate a journaled remote operation.

Host configuration selects the global configuration source. Tests use an isolated empty or explicit global file; the renderer cannot choose a file. Effective configuration includes origin/scope and includes. Remote names remain case-sensitive. Git itself expands `insteadOf` and `pushInsteadOf`, including explicit `pushurl` precedence. All fetch/push URL sets are retained in order; multiple destinations remain visible as ambiguous for mutation. Branch upstream configuration and branch/global push-remote precedence are separate fields. A suggested remote never authorizes or selects a publication destination. These semantics follow [Git remote get-url](https://git-scm.com/docs/git-remote).

Public results contain redacted display URLs, transport classification, configuration origins, refspec counts and policy issues. Raw configuration values/URLs are neither serialized nor persisted. Credentials in user information, query or fragment are hidden; malformed/helper and ambiguous SCP-style destinations are withheld. Credential helper, proxy, SSH command and unsafe TLS configuration are flagged without returning their command strings or secrets. Remote mirror/custom-program settings and absent/ambiguous upstreams are explicit issues. Display content remains inert text and escapes native/control/bidirectional content.

A configuration digest binds the complete effective snapshot without publishing its values. Both local repository generation and effective configuration are checked for drift before returning. The operation is cancellable and bounded by the shared read deadline/output limits, 10,000 config records, 64 remotes, 256 rewrite entries, 32 URLs per set and 1 MiB of expanded URL output. No raw URL is reconstructed from the display projection.

The URL parser is the `url` crate already used by the desktop dependency graph. It is now an optional direct Core dependency under application-services, so untrusted URL parsing does not rely on ad hoc authority/credential parsing. Default Core builds still exclude it and the remote module.

## Implemented: independently reviewed branch observation

`ReviewRemoteObservation` prepares one exact remote/full branch reference through the same native capability boundary, gated by `desktop-writes`. Trust, active workspace/worktree identity, effective configuration/generation and Git/HTTPS-helper/CA tool bindings are revalidated. Preparation performs no network activity. Decline, replacement, revocation and workspace invalidation cancel the pending proposal. Results live in versioned single-owner SQLite state, with per-worktree/workspace bounded pagination. Restart never restores confirmation authority or replays a running observation; orphaned/running records display as interrupted.

Execution creates a private temporary bare transport repository and empty HOME under app-owned state. It clears inherited environment/configuration, disables lazy network activity, hooks, recursion, maintenance, credential helpers, proxies and cookies, verifies TLS certificates/hostnames and refuses redirects. The initial credential-free path supplies no credentials. Supported authenticated paths use the explicitly assigned scoped HTTPS helper or selected SSH agent policy; managed repository helper/proxy configuration stays disabled. A separately configured CA is trusted host configuration, fingerprinted in the review; it cannot be supplied through IPC. Local file transport is restricted to a trusted host-configured disposable fixture root and is disabled in ordinary host configuration. Git and transport tool paths/content are bound to the review.

The only network command is an exact `ls-remote --refs --exit-code` for one validated full branch. Complete results contain an observed OID or explicit absence plus time; raw server diagnostics are discarded. Managed HEAD, objects, index, refs, working files and upstreams are preserved. This is a past server observation and never implies synchronization. Running cancellation terminates the bounded transport process group. The desktop requests review and reads results; it cannot supply approval or replay an observation.

Tests use actual isolated local remotes and loopback TLS advertisements, including reviewed CA success, certificate and hostname failures, redirects, helper/proxy isolation, missing credentials, CA drift and cancellation after an HTTP request begins. No external service or actual user credentials are used. Supported scoped HTTPS helpers and strict SSH-agent reads now have selected native evidence; broader compatibility remains open. See [authentication](../AUTHENTICATION.md) and the current record.

## Implemented: reviewed fetch download stage

ReviewFetchCapture selects a stored completed branch observation with an OID in the active worktree. A fresh native confirmation binds a new transport/configuration/tool/CA snapshot and expected OID; the old observation is data, not authorization. Shared transport helpers retain the observation isolation policy. Download fetches one exact branch into a new private bare repository, rejects tip drift, validates objects and produces a synced verified bundle with digest/size/count metadata. It never imports managed objects or updates managed refs, FETCH_HEAD, tags, branches, upstreams, index or working files.

The download is bounded to 10,000 objects/64 MiB uncompressed and bundle bytes, shared process output/deadlines and a polled 128 MiB/1,024-entry artifact budget. The private process check can stop/reap owned work on artifact-limit failures; polling is not a hard OS quota. Version-1 records use schema-3 state, have scoped paged history/unified audit and never restore native authority after restart. The download stage itself offers no managed ref publication or replay; separate import has its own authority below. The ordinary desktop gate remains disabled. Current artifact path invocation requires UTF-8 app-data paths; broader interoperability/native coverage and retention remain open. See [fetch download report](../reports/M5B_FETCH_CAPTURE_2026-11-02.md).

## Implemented: import, upstream and push preparation

- [Offline import](../reports/M5B_FETCH_IMPORT_2026-11-02.md) requires a fresh
  review of the exact captured bundle, expected old tracking ref and repository
  bindings. It has durable phases, guarded recovery and no restart replay.
  [Selected native chain/recovery tests](../reports/M5B_NATIVE_FETCH_ACCEPTANCE_2026-11-02.md) passed.
- [Upstream setup](../reports/M5B_GUARDED_UPSTREAM_2026-11-02.md) is a separate
  missing-only configuration review/transaction with metadata preservation and
  recovery. It is not branch creation or publication.
- [Authentication profiles](../reports/M5B_AUTHENTICATION_PROFILES_2026-11-02.md)
  explicitly bind nonsecret policies to worktree/remote destinations. Supported
  [HTTPS](../reports/M5B_AUTHENTICATION_COMPATIBILITY_2026-11-02.md) and
  [SSH](../reports/M5B_SSH_AGENT_TRANSPORT_2026-11-02.md) have selected actual
  evidence; SecurityAgent operator cases and broader compatibility remain G06.
- [Historical child mappings](../reports/M5C_CHILD_MAPPING_EVIDENCE_2026-11-02.md),
  [relative/SCP resolution](../reports/M5C_RELATIVE_CHILD_ACCEPTANCE_2026-11-02.md)
  and [dependency planning](../reports/M5C_DEPENDENCY_PLANNING_2026-11-03.md)
  provide bounded outgoing-history requirements, explicit observation/ancestry
  evidence, selected child-first order and cycle/block detection. Planning performs
  no publication and conditional child selection is not remote success.
- [Pre-push checks](../reports/M5C_REVIEWED_PUSH_CHECKS_2026-11-03.md) separately
  authorize exact hook argv/stdin/policy and already-reachable child evidence.
  Complete checks remain unpublished and cannot authorize a later push.

## Implemented individual publication — G07 acceptance partial

[Exact publication](../reports/G07_EXACT_PUBLICATION_2026-11-08.md) adds a separate
short-lived native approval and a new nested check. Only approved reachable objects
are exported through isolated configuration; inventory, pack and artifact budgets
remain enforced. Integrity and ancestry checks precede one atomic exact expected-old
update. The explicit lease is a compare-and-swap guard after ancestry proof, never a
user force option. Core preserves managed files/index/refs/config/upstream. The
existing same fetch/push destination and already-reachable child requirements remain.

A journal records `publishing` before spawning transport. Any post-boundary error is
unconfirmed. Restart cannot restore approval or replay hooks/transport. Separately
approved fresh observation plus recovery review records observed source/old/other/
missing state, without inferring which actor changed a ref. Local fixture publication
also rejects workspace/metadata overlap and non-standalone/symlink metadata. Ordinary
write/network execution remains disabled. Selected automated and native evidence is
in the linked report; broader authenticated/topology/interruption acceptance remains.

## Remaining publication acceptance and bulk contract — G07/G08

Individual push retains its own short-lived Rust-owned exact source/OID, destination
branch, authentication/tool/hook and dependency approval. Keep inspection protocols
banned and transport configuration isolated. Reject ambiguous destinations,
unsupported states, force/divergence and stale source or child evidence. Final hook
timing and transport must be validated within publication authority; an earlier
check is not an execution token.

Record durable phases before side effects and distinguish rejection, cancellation,
completed and uncertain publication. A disconnect may follow server acceptance:
reconcile using fresh approved observation and actual state, never automatic replay.
Bulk push must verify required children remotely before publishing parents, preserve
successful items, stop successors on failure and require fresh review for retry.
Do not select unselected children or push after a commit/AI run implicitly.

Validate with disposable local bare/loopback remotes: server rejection and drift,
credential/transport failure, exact hook payload, cancellation, uncertain outcomes,
dependency failure and no unexpected destination/upstream creation. Selected individual evidence does not establish full G07 or bulk executor acceptance.
