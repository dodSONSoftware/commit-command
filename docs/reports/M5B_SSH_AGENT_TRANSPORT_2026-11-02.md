# M5B reviewed SSH-agent transport — November 2, 2026

Implemented a narrow authenticated SSH read checkpoint, with automated gates and selected actual native acceptance. M5B remains partial; HTTPS credentials/helpers, native secure-storage management, broader SSH configuration and actual push remain unfinished. Ordinary desktop writes remain disabled. No push was run, including in fixtures.

## Supported behavior and authority

The user authorized autonomous completion-plan development, disposable native tests, reports and local commits, preserving unrelated work and prohibiting push. This checkpoint advances M5B's transport/authentication policy while retaining independent review and the existing credential-free HTTPS/local-fixture paths. No dependency was added.

SSH observation and separately reviewed branch download accept one canonical `ssh://login@host[:port]/repository` destination and one explicit host-selected policy: a known-hosts file, one plain Ed25519 public identity and a dedicated/current-user-owned private agent socket. No private key is supplied to GitFleet for authentication. Existing repository identity, configuration, trust, full branch-ref, generation and one-use native authority checks still apply. Agent selection alone grants no network authority. The normal read-only adapter's protocol bans are unchanged.

The native host accepts these optional startup arguments together; they are absent by default and cannot be supplied through renderer IPC:

```text
--ssh-known-hosts /absolute/path/to/known_hosts
--ssh-public-identity /absolute/path/to/selected.pub
--ssh-agent-socket /absolute/path/to/agent.sock
```

Duplicate, partial and nonabsolute SSH policy arguments fail early. Existing `--data-dir` behavior and the ordinary write gate remain. This is an explicit host configuration surface, not a completed GUI credential/settings workflow. Inspection now displays a plain safe SSH login as part of an explicit URL; passwords, queries, fragments and encoded/unsafe login forms remain withheld. SCP-style aliases are not enabled by this execution checkpoint.

Reviews bind the SSH executable, public-file bytes/identities and live agent-socket identity. Public files must be bounded, protected regular single-link files; private-key formats and implicit certificate siblings are refused. Validation, fingerprinting and snapshotting use the same descriptor-read bytes, closing a potential validation/hash race. Only verified public identity and known-hosts snapshots enter private isolated transport storage; raw keys, passwords and server diagnostics do not enter plans/journals. The public-file binding fingerprints are internal content/identity receipts, not OpenSSH's standard displayed key fingerprint.

The fixed system OpenSSH command ignores user/system SSH configuration, selects only the reviewed agent identity and permits plain `ssh-ed25519` authentication. It enforces strict host-key matching against the selected file and disables host-key updates, DNS trust, password/keyboard-interactive/keychain/private-default-file fallback, arbitrary helpers, proxies, local commands, forwarding and connection sharing. All Git/SSH children retain the existing bounded output/deadline/process-group cancellation. Managed SSH commands, credential helpers, hooks and submodule recursion remain disabled. Unknown or changed host keys and rejected/missing identity availability block the operation; no trust-on-first-use acceptance or automatic retry exists.

OpenSSH drops a missing `-i` argument and can otherwise select default identity files. Explicit `IdentityFile=none` prevents that fallback; an actual offline `ssh -G` test verifies the resulting identity list both with and without the selected public snapshot. Agent paths preserve their selected absolute spelling: canonicalizing `/var/...` to `/private/var/...` can exceed Darwin's Unix-socket address limit. Path/token-expansion restrictions, ownership/type checks and socket-identity revalidation still apply.

The existing reviewed download/import flow and typed operation audit are reused. Import remains offline and independently approved. Local push preview accepts the same unambiguous SSH destination after explicit object import; it does not contact the server or authorize push. Shared transport preflight currently requires the host SSH policy even for those local-only preparations; decoupling this availability requirement is a later compatibility task.

## Automated validation

Evidence is under `docs/validation/m5b-ssh/`; `validation-summary.json`, `source-manifest.json` and artifact hashes identify final scope. The first complete matrix passed, then both affected configurations and the full desktop gate were rerun after final public-file/algorithm hardening.

| Core feature configuration | Passed | Documented direct ignores |
|---|---:|---:|
| Default | 54 | 1 |
| Experimental writes | 89 | 2 |
| Controlled writes | 79 | 1 |
| Application services | 163 | 2 |
| All features | 207 | 6 |

Core format/build/test/strict Clippy and the disposable lab passed. Final affected matrix, final clean Clippy and a final clean authenticated roundtrip/preview passed. The all-feature real transport suite passed 38 cases with two subprocess workers ignored directly and exercised by parent tests. Desktop passed contracts, types, lint, formatting, 50 renderer tests, frontend/native builds, 12 native host tests per configuration, strict Clippy and the default release build. macOS prerequisites now check the SSH tools used by the non-skipped fixture suite.

Five real SSH integration cases cover the authenticated observation/download/offline-import chain; two loaded fixture identities with only the selected key offered; unknown/mismatched host rejection; wrong selected identity without fallback; missing policy/private-key refusal; known-host/public-key/agent replacement and private-key swap after review; decline/pre-cancellation; cancellation after proven authenticated observation and download; server diagnostic non-persistence; exact repository/remote preservation; and local-only preview after import. Paths with spaces and apostrophes are exercised. An actual `ssh -G` unit check verifies effective strict/fallback/quoting behavior.

Historical failed iterations are retained. Initial integration compilation used the wrong local helper name; the first authenticated test exposed the socket-canonicalization problem, substantiated by independent fixture SSH diagnostics and corrected without relaxing verification. One later full application retry omitted `GITFLEET_TEST_GPG`, correctly failed its mandatory signing gate, and was rerun with the explicit isolated signer. No tests or guards were weakened.

**Open finding:** one focused roundtrip returned `blocked` with `Unsafe fetch artifact type` before preview/import. The unidentified entry was not captured by the old generic diagnostic. Ten serial and eight concurrent instrumented roundtrips passed without reproducing it. The temporary fixture-only diagnostic patch/logs are saved, and the patch was removed from product source. The artifact guard is byte-for-byte unchanged and remains strict. This is not a diagnosed/fixed race or a pass for the failed attempt. Investigate exact entry/type on recurrence before broad fetch acceptance or changing this guard.

## Actual native acceptance

A new fixture `/private/tmp/gitfleet-ssh-native-dufk8c3i` used only generated keys, a dedicated two-key agent, an unprivileged loopback-only forced-upload-pack server and isolated app HOME/state. Zero observation/download/import records were seeded. CUA performed native UI interaction; independent read-only file/Git/SQLite/process oracles verified actual effects.

- Native decline consumed its review, made zero authenticated requests and preserved repository/remote bytes.
- The first native approval correctly blocked because the server's authorized-key path traversed writable `/private/tmp`. Its result and server diagnostics are retained. Moved only the fixture public authorized-key file into protected temporary storage and reloaded that exact fixture listener, keeping `StrictModes yes`; a fresh approval was required.
- Native SSH observation returned the exact remote-only commit `9406de64dd52f46ec03b721d3eb82e3131f5d522`. Separately approved download captured four objects, 533 uncompressed bytes and a 396-byte bundle. Separately approved offline import published exactly `refs/remotes/origin/main`, its reflog and objects. Before import all managed files matched baseline; afterward every original entry remained unchanged. Remote bytes and selected known-host/public files stayed identical. Two authenticated requests occurred; offline import added none.
- A fresh observation was held only after actual server authentication and durable `running` phase. Native Cmd+Q cancelled it, terminated the captured owned client/read process identities, persisted `cancelled` and released the state lock. Imported contents remained intact. Ordinary restart displayed real complete/blocked/cancelled history with network/write actions disabled; all repository, policy, journal and artifact snapshots matched, with no replay or new request.
- After final public-file/algorithm hardening, rebuilt the opt-in artifact, restarted only the fixture agent/listener and approved one new observation using the new socket binding. It authenticated successfully, preserved all previous journals and repository/artifact contents and returned the same exact OID. The final ordinary default artifact restarted without SSH policy, made no new request and preserved the final snapshot exactly. Initial and hardened artifact hashes/evidence are separate; the final source's full download/import chain is automated evidence, while the complete native chain/Quit test predates only that final hardening.

Pointer selection in the native remote popup stalled once while the console was unlocked. Native Quit/relaunch and keyboard selection worked. No source cause was established; do not classify it as a fixed product defect. All fixture apps are now closed; both rounds of dedicated agent/listener processes were stopped by verified exact PID, the hold flag is removed and the state lock reopens. No user keys, user agent, credential store, global settings or personal repository changed. These are local development bundles, not signed/notarized release installers.

Next retain the open artifact-type finding, add HTTPS/native secure-storage or explicitly approved helper interaction and broader SSH/settings compatibility, then fresh push destination/dependency proof and separately reviewed push. M3/M4 accessibility/topology/scale and M6–M8 remain open. Unrelated `.gitignore`, `docs/README.md` and `docs/APPLICATION_OVERVIEW.md` are excluded from this checkpoint.

Implementation references: [OpenSSH client options](https://man.openbsd.org/ssh_config) describe identity/agent selection and strict host verification; [Git SSH command configuration](https://git-scm.com/docs/git-config/2.42.4.html) documents SSH variant selection. Actual installed OpenSSH 10.5p1 behavior was verified locally, including shell interpretation and effective options; no credential provider or public server was used for this checkpoint.
