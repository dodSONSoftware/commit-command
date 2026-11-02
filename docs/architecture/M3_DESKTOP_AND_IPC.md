# M3 desktop integration and IPC contracts

Implemented after explicit user authorization. **Native acceptance is partial;
M3 is not complete.** The earlier agent Computer Use policy denial remains historical.
The [November 2 checkpoint](../M3_NATIVE_ACCEPTANCE_HANDOFF_2026-11-02.md) records later
user-observed native approval/read/write/signing results, without independent agent
reproduction or exact test-time binary attestation. Actual cancellation, guarded
recovery, close/forced-interruption and accessibility criteria remain open.
Core/renderer/mock-IPC evidence cannot close those native gates.

## Architecture and feature boundaries

The existing docs/app structure remains. Core is still one Rust crate and contains
no Tauri, React or other presentation dependency. The separate native adapter crate
under the pre-existing `app/desktop/src-tauri/` is required by its Tauri/native build
and does not split Core. Core CLI and default feature commands remain independent.

Core's non-default `application-services` feature enables `application`, monitoring
and generic JSON Schema contracts. It selects M2 `controlled-writes`; it does not
select experimental POC APIs. Optional notify/schemars dependencies implement actual
monitoring/transport contracts, not rendering. Rust schemas generate TypeScript;
a deterministic check detects drift.

`Application` owns one `ControlledWrites`, which retains the existing workspace
service/data-directory lock and M2 session. The application facade never exposes
raw storage to Tauri. One bounded native service worker executes Core requests and
serialized mutations off the UI thread. Read concurrency in this initial adapter
is one; the existing four-process Core transport limit remains intact. Cancellation
and job-status handlers do not wait on the service worker. There is no second
workspace writer or per-request recreation of approval sessions.

Ordinary desktop features are `custom-protocol` only. The native crate explicitly
depends on `application-services`; its normal graph excludes `experimental-writes`
and `write-test-support`. Non-default `desktop-writes` enables the compiled native
approval path for acceptance. The ordinary binary rejects trust grants/preferences,
revocation and Git write planning in Rust and disables the matching UI controls.
Read-only recovery inspection remains available with confirmation disabled. This
fail-closed gate is deliberate while native acceptance is open, not an M3 pass.

## Desktop behavior

The main React view provides saved-workspace management through native picker
handles; Overview Table/Tree with shared explicit selection/filtering; closable
repository tabs and permanent Overview; Changes and paged History; exact Core diffs,
previews and parent comparisons; persistent per-worktree drafts and navigation;
minimal operation results and recovery notices.

Selecting a parent never selects descendants. Missing/unavailable entries and tabs
remain visible. New tabs choose Changes for observed dirt and History otherwise;
manual choices persist. Paths/content sources, commit/parent choices, scroll anchors,
filters, selected repositories, panel sizes and themes are stored as UI data only.
Invalid anchors fail visibly instead of substituting another repository. Read
responses carry workspace epochs and Core generations; old session results are
rejected and stale file-selection responses are discarded.

Status includes observation time; tracking is labeled as local tracking-ref data.
M1's historical `untrusted` inventory field does not grant or describe current M2
trust. Core projects grant/currentness/eligibility separately, revalidating policy
without executing repository-selected helpers. Explicit trust sets are limited to
20 worktrees, each independently reviewed with per-item results; no inherited grant
or batch-atomicity claim follows.

History is a commit list/details/explicit-parent diff base, not M4 graph layout or
workspace activity. Operation results expose M2 outcomes and pagination, not the
full M4 audit product. No future action depicted in a mockup is enabled implicitly.

## IPC and caller identity

There are exactly eight application commands:

| Caller | Commands |
|---|---|
| `main`, bundled root/index page | `desktop_capabilities`, `dispatch`, `pick_directory`, `job_status`, `cancel_job` |
| `review`, bundled review page | `review_data`, `review_confirm`, `review_cancel` |

`build.rs` declares custom application commands to Tauri's permission generator;
exact-webview capabilities explicitly grant their allowlists. No remote capability,
wildcard window/webview access, plugin filesystem/shell/HTTP/SQL access, or arbitrary
window-creation permission is granted. Merely registering application commands is
not assumed to restrict access. Mock-runtime tests use the actual generated context
and demonstrate positive allowed calls and denied cross-view/unprivileged calls.

Handlers independently verify the actual invoking webview's label, URL, page,
origin and build mode. Payload-provided labels/origins are not identity. Bundled
main paths include Tauri's canonical empty root, `/` and `/index.html`; review uses
only `/review.html`. Userinfo, ports on bundled origins, queries and fragments are
rejected. A specific loopback development origin is accepted only in debug builds
without `custom-protocol`, never by release caller checks.

`dispatch` accepts a version-1 envelope with epoch and a typed Core request. Unknown
fields are rejected, including ignored fields inside compatibility DTOs, by checking
the typed canonical structure. Core resolves workspace/worktree IDs against current
availability, membership and observed metadata bindings. Literal native paths/OIDs
are validated by the existing restricted inspection/write services. The renderer
cannot choose the data directory, executable, arbitrary file/journal path, Git argv,
shell, SQL, signing program/home, environment overrides or network destination.

Native directory results produce short-lived opaque picker handles; JSON paths
cannot fabricate them. Handles are consumed once, expire in five minutes and are
invalidated with application context. Removing a saved workspace never removes Git
files. Scope changes/removal are blocked while interrupted operation records need
resolution, preserving their recovery context. Historical operation evidence is
retained.

## Review and confirmation

Core stores one immutable pending application review: explicit trust set, M2 write
proposal or current recovery review. The host opens a dedicated local review window
with a dependency-minimal bundle independent of React. Its content is fetched from
Core and rendered through text nodes; there is no repository HTML/Markdown/script
execution, `innerHTML`, remote resource loading or iframe access.

The complete review contains the Core plan, exact message, patch, paths, sources,
branch/parent/tree, author/date, hooks, signer fingerprint and digest. Native summary
identifies the operation and canonical worktree, summarizes the message, and binds
to that full review. Control/bidi characters are escaped for display while approved
bytes remain unchanged. Invalid UTF-8 display uses explicit byte escapes.

Only the review view may request native confirmation. Core issues a private Rust
`Confirmation` value with a nonce, review ID and epoch; it is not serializable or
constructible by renderer JSON. The host displays the native dialog from stored
Core text and sends its actual result directly over a Rust-only queue. Only then
can Core obtain/use the opaque M2 approval. Native private capabilities, approval
tokens and native response booleans are never transported to the renderer.

The review ID/digest is lookup data, not authority. A frontend `approved` flag,
checkbox, event, guessed ID or altered message cannot execute a proposal. Reviews
expire after ten minutes; overlapping confirmations are rejected. Cancellation or
replacement records a non-executed write proposal as cancelled without modifying
Git. Core revalidates scope/trust/current state at confirmation and M2 revalidates
its exact snapshot before publication.

Main-page reload increments a native presentation generation, cancels prior jobs
and active mutation work, invalidates the review, and requests a Core epoch reset.
Queued confirmation callbacks check that generation before execution; native dialog
callbacks check it before and after the OS prompt. Stale jobs cannot be consumed by
a new renderer generation. Core session/storage ownership remains intact.

The Rust host and OS-native response are trusted components. This is not a claim
that a compromised native process or arbitrary trusted hook is sandboxed. M2's
supported Git states, hooks/signing restrictions, actual-state recovery and no-replay
rules remain unchanged.

## Bounds, cancellation and shutdown

- One Core application worker; queue capacity 8, at most 32 retained jobs, ten-minute
  completed-job retention, consume-on-read completed responses.
- 3 MiB incoming envelope and 16 MiB normal job-response limits. Oversize data fails
  explicitly, without silently showing a truncated review as complete.
- Complete review content cap 32 MiB; native summary cap 1 MiB. Messages remain
  subject to M2's 512 KiB limit; native summary uses a bounded subject plus digest.
- 100 tabs, 100 drafts, 10,000 selected repository IDs, 2 MiB UI-state payload;
  individual drafts at most 512 KiB. Duplicate tabs/selections, invalid paths,
  invalid dimensions/layout/theme and newer state versions are rejected.
- Workspace names at most 512 UTF-8 bytes and no control characters. Exclusion
  policies have at most 1,000 entries, 1,024 bytes each. Core configuration saves
  cannot exceed the existing 2 MiB read limit.
- Core read contexts use their own cancellation tokens and a 15-second default
  session deadline in this facade. Public M1 convenience APIs retain their previous
  defaults. Cancellation of M2 execution is operation-specific and does not poison
  subsequent approvals/jobs.
- Native dialogs are serialized. User cancellation prioritizes an active native
  write over a background read. After publication it means cancellation requested,
  not rollback or proof that no commit exists.

The primary window's close handler prevents immediate disappearance, requests
cancellation, closes auxiliary review UI, drains to a service shutdown boundary,
stops monitoring and exits. The visible title indicates safe closing. Forced
termination remains an M2 recovery case. This lifecycle code compiles and its Core
parts are tested, but **actual native close-during-operation acceptance is pending**.
Kernel I/O and hostile process detachment retain M0/M1 limitations.

## Monitoring

Core owns notification filtering/debounce/reconciliation policy; Tauri only drives
the service timer. The active root and necessary per-worktree/common/ref metadata
are watched. Related external worktrees are not automatically inspected. Discovery
retains M1 exclusions and does not enumerate Git object storage; coarse object-store
notifications are discarded before scheduling inspection.

Trusted Rust-only `MonitoringOptions` allow test schedules without exposing timing
or authority overrides to IPC. The native host uses unchanged production defaults.
Defaults: 250 ms debounce, two-second maximum coalescing delay, ten-second status
fallback, sixty-second discovery fallback. Notification queue capacity is 128;
overflow requests discovery reconciliation. Long requests/writes defer background
work; these defaults are scheduling targets, not latency guarantees. No watcher
path can grant trust, approve, stage, commit, execute hooks, fetch or push.

Incomplete scans and refresh failures retain missing/unavailable/error distinctions.
Refresh errors are exposed in monitor diagnostics and do not starve remaining
entries. Switching workspace drops old watches and changes epoch. Shutdown stops
watches even if persisting a cancelled review fails. Native backend behavior and
last-window termination still need their separate permitted UI acceptance.

## Persistence and lock correction

Workspace TOML stays version 1. Application opening performs M2's backed-up schema-2
upgrade if needed, then backs up schema 2 before transactional schema 3 (desktop
state and workspace index over operation records). Known schema 3 can still be
opened by default workspace services. Newer/invalid state is rejected, never reset.
Private state files/backups retain 0600 permissions; operation directories retain
M2's 0700 protection. Drafts/messages/paths remain sensitive unencrypted local data.
No automatic pruning, export, backup UI or encryption capability was added.

Concurrent regression runs exposed a pre-existing advisory-lock lifetime issue:
closing the owner's descriptor can leave a lock held through a duplicate briefly
inherited during subprocess creation. A private `ExclusiveLock` guard now explicitly
unlocks on owner drop, including failed initialization. It protects workspace data
and M2 common-directory execution/recovery. An independent test proves close-only
retention through a duplicate and correct explicit release. The unchanged workspace
suite then passed three repeated runs and the full matrix. The actual held-owner
Busy check remains enforced; no retry or weakened assertion hides contention.

## Acceptance status

See [M3 evidence](../validation/m3/README.md) and the
[M3 acceptance ledger](../ACCEPTANCE_MATRIX.md#12-m3-acceptance-gates--implementation-with-native-gates-open).
Historical Core, renderer, actual generated-ACL mock-runtime tests, formatting,
lint, contract and native debug/release results remain in the evidence index.
The November 1 agent inspection attempt was policy-blocked. Later user-operated
screenshots, native confirmations, signing and restart observations are recorded
in the November 2 checkpoint, with no exact test-time binary attestation.
Actual cancellation, guarded recovery, close/forced-interruption, VoiceOver and
measured contrast remain untested. Workspace-removal and retained History patch
fixes have component regressions; their native retests remain open. Default desktop
write exposure stays disabled. No M4 implementation, commit, push, installation or
release publication occurred in this follow-up.
