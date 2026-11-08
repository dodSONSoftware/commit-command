# G06 native configuration drift after observation review — November 7, 2026

**Selected native stale-configuration/trust barrier passed.** A previously reviewed
observation cannot execute after the worktree's effective configuration changes.
G02/G06 remain partial. No product fix or ordinary write-default change was needed.

## Scope and isolation

Product code unchanged from validated `e9eeacd`; checkout `2d24b2d`. Reused the exact
explicitly authorized disposable repository
`/private/tmp/gitfleet-g06-helper-di5z10fo/workspace/repository`, with a separate
`state-config-drift` application-state copy made while both prior fixture apps were
closed. The original case's state/journals were not changed. The existing non-default
acceptance and ordinary artifacts were reused with recorded hashes. No new access
recipient, credential, real server or user repository was introduced.

The fixture's original config bytes were archived. Deliberate fixture setup changed
only origin's URL to `https://127.0.0.1:9/reviewed.git`; its review uses credential-free
HTTPS and no managed helpers/proxy/SSH commands. No listener or service was started;
this tests blocking before transport initialization, not network success.

## Actual native sequence

1. Startup correctly showed the copied old trust as revoked/stale following the
   setup config change. A fresh authoritative trust review and independent native
   confirmation renewed trust only for the same authorized repository/path. No hook,
   signer or Git write operation was requested through the application.
2. Inspected origin, chose `refs/heads/main`, and prepared **Review remote branch
   observation**. The authoritative review explicitly named the reviewed loopback
   URL and no credentials. Read-only SQLite proof captured exactly one `proposed`
   observation with that destination; no network transport storage existed.
3. While this review was pending, deliberately changed only `.git/config` to a
   different supported URL, `https://127.0.0.1:9/drifted.git`. Independent file checks
   confirmed no other repository file/mode changed. This is fixture-driven external
   drift, not an application mutation.
4. Continued the old review to native confirmation, which still correctly displayed
   **the original reviewed URL**. Approved that exact operation. Core returned
   `blocked` / `untrusted_repository` with **Trust was revoked or its identity/configuration changed**.
   OID and observation time were null. Native Operation results showed the blocked
   outcome and original reviewed destination; worktree trust was revoked.
5. Acceptance Quit exited 0. First ordinary default-feature restart rendered without
   reload and displayed the identical blocked result in Operation results. Write
   controls were disabled. Ordinary Quit exited 0 and the writer lock reopened.

## Independent validation / cleanup

Executed read-only oracles confirmed exact bytes/modes against the intentional-drift
baseline after blocking and after ordinary restart; the complete observation payload
was unchanged. SQLite integrity `ok`; precisely one scoped trust row, revoked;
zero local write operations and zero download/import/upstream/push-check records.
No observed OID/time, helper sentinel invocation or `network` transport storage.

Source control flow in `writes/remote.rs` revalidates trust/configuration before
`initialize_transport` and the transport command. The observed untrusted-repository
failure plus absent transport storage therefore support blocking before transport
initialization. This is not a host-wide packet/process capture or independent test
of the later remote URL/config-digest guard: the earlier trust barrier prevented
reaching it. No successful observation or publication is inferred.

After both apps closed, restored the fixture's original `.git/config` bytes. The
previous helper-refusal oracle passed against its untouched original state and
baseline. A final restored-baseline oracle also passed for this isolated case,
retaining exactly the blocked journal and revoked trust; restoring config did not
renew trust or replay an observation. The fixture apps are closed, and the normal
user app/default/bundle were untouched.

No product source changed and no full automated suite was rerun. Prior full validation
remains historical; this checkpoint adds actual native approvals, a genuine proposed
and blocked observation, restart/history checks and independent file/SQLite oracles.
Broader authentication/topology, transport-specific late drift barriers, startup,
Dock/session and other G02/G06 requirements remain open.

[Evidence](../validation/g06-native-observation-config-drift-2026-11-07/README.md).
