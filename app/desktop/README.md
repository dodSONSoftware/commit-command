# Commit Command desktop

Tauri is a thin adapter to the presentation-independent Rust Core. Main, Settings and AI suggestions
windows have separate command permissions. The application includes
workspace inspection/monitoring, graph/activity/audit, controlled local/bulk commits,
supported authentication/fetch/upstream, exact individual publication and guided
push queues. Optional AI settings, secure authentication, reviewed source message
suggestions, read-only repository tests with unsaved editors and guarded SemVer
Actions are implemented with selected acceptance.
Hierarchical analysis, bounded automatic commits, general backup/restore and release
acceptance remain unfinished. Use the [detailed completion handoff](../../docs/implementation/README.md)
for remaining implementation steps and test procedures.

**Ordinary desktop builds enable controlled Git trust/write/remote execution.**
`desktop-writes` is a default feature; it does not itself grant repository trust or
operation approval. AI source transfers retain their separate exact native confirmation.
Build an explicitly read-only bundled host with:

```sh
cargo build --manifest-path app/desktop/src-tauri/Cargo.toml --no-default-features --features custom-protocol --locked
```

Omitting `custom-protocol` selects the development server rather than the bundled UI.
Read-only builds permit local inspection, settings persistence and separately approved
AI requests, but reject managed Git trust/write/remote preparation in Rust and UI.
Use [the current record](../../docs/EXECUTION_STATUS.md) for remaining gates and
next work. Selected native cancellation, recovery, close/Quit, theme and geometry
cases have passed; broader acceptance remains partial. Checklists below retain
dated details and must be read alongside the current record.

## Build and automated validation

Requirements: Node 22.12+, npm 10+, Rust 1.99+, Git 2.56+, macOS native command-line
tools. The executed host is macOS 27.2 arm64. No other desktop platform is accepted.
No global package installation, personal signing keyring, or global Git config
change is required. Finder launches prefer Git in `/opt/homebrew/bin` or
`/usr/local/bin` when the inherited PATH selects Apple Git or contains no Git.
An explicitly selected custom Git keeps priority. Core still enforces Git 2.56+.
Locked npm installation disables dependency install scripts;
the local `.npmrc` preserves the peer resolver used to produce the lockfile.

From the project root:

```sh
npm --prefix app/desktop ci
npm --prefix app/desktop run build
CARGO_NET_OFFLINE=true sh app/desktop/scripts/validate.sh
cargo run --manifest-path app/desktop/src-tauri/Cargo.toml -- --data-dir /outside/projects/gitfleet-state
```

Omit offline mode if the locked dependencies are not cached. The default state
location is `~/Library/Application Support/dev.gitfleet.desktop`, outside managed
repositories. `--data-dir` is a trusted process-start option, never an IPC input.
M1 refuses state directories inside checkouts. Do not target existing user
repositories for write validation.

`npm run dev` in this directory serves only `127.0.0.1:1420`. The ordinary Rust
build embeds `dist/` through its default `custom-protocol` feature. A development
server run requires an explicit Rust build without that feature; development
origins are never accepted by the release caller checks.

The dev identifier is provisional, not a release branding/signing decision. There
is no installer, updater, notarization or distribution workflow. Local test `.app`
bundles are development artifacts, not releases.

## Fixed native acceptance support

`native-acceptance-support` is a separate non-default test feature. It selects
Core fault support but does not enable desktop writes. Build with
`--features desktop-writes,native-acceptance-support` only for freshly generated,
marked disposable fixtures. Startup rejects ordinary/unmarked state and checks
initial workspace containment. The native menu provides fixed reload/caller/page
probes, actual main-renderer termination on macOS, one-shot close-delivery
suppression and two existing self-SIGKILL commit checkpoints. Renderer termination
uses the owned WKWebView's runtime-checked private PID property only in this test
feature; no caller-selected PID or general process endpoint is exposed. It adds no IPC permission or independent-approval bypass.

Use the [closure/recovery report](../../docs/reports/G01_NATIVE_SUPPORT_G02_RECOVERY_2026-11-07.md)
and linked evidence for exact artifact/fixture setup, Core's additional crash guard,
results and exclusions. A suppressed acknowledgement is not an actual renderer
crash. Actual renderer auto-reload acceptance is recorded in the
[renderer-loss report](../../docs/reports/G02_ACTUAL_RENDERER_LOSS_2026-11-07.md).
Ordinary default feature graphs/artifacts exclude this menu and Core test
support. G01 is accepted within the documented envelope; G02 remains partial.

## Core and IPC contracts

`gitfleet-core/application-services` enables the transport-neutral application
facade, state schema and monitoring; it also selects M2 `controlled-writes`.
No Tauri/React/TypeScript library is a Core dependency. Generic optional JSON Schema
derives produce the frontend contracts:

```sh
npm --prefix app/desktop run contracts
npm --prefix app/desktop run contracts -- --check
```

Do not hand-edit `src/services/*.generated.ts`. The native crate has a separate
Cargo lockfile and depends on the existing Core crate by path. Ordinary desktop
dependency graphs exclude `experimental-writes` and `write-test-support`.

Main commands include `dispatch`, `desktop_capabilities`, `job_status`,
`cancel_job` and repository folder actions. Settings uses a restricted subset of
`dispatch` plus settings persistence and shutdown acknowledgement. AI suggestions
uses its own restricted role on `settings.html`. Custom command ACLs and Rust
actual-caller/page/origin checks both apply. No generic shell, filesystem, SQL or
HTTP endpoint is exposed.

The main renderer presents explicit operation reviews for bulk commits and push
publication. Preparing a review never executes its operation; execution requires a
separate matching request. Ordinary direct Stage/Unstage/Commit flows execute their
prepared selection. Saved-profile AI source transfers require exact native approval;
unsaved Settings editors cannot send repository source. Reload and workspace
changes invalidate pending context. Cancellation cannot promise rollback after publication.

## Native acceptance workflow — still required

These steps require permitted native access or a human operator. They describe the
full procedure; consult the [reconciled checklist](../../docs/M3_NATIVE_ACCEPTANCE_PLAN.md#historical-results-checklist)
and November 2 checkpoint before repeating checks. User-observed cases remain distinct
from agent verification. Renderer mocks cannot close the remaining native gates.

1. Build `gitfleet-cli`, then run `python3 app/desktop/scripts/native-fixture.py`.
   It creates only a new `/private/tmp/gitfleet-m3-native-*` fixture containing
   four real repositories, nested relationships, two baseline commits, partial
   staging, clean/dirty entries and a separate application-data directory.
2. Review its printed JSON paths. Use only that new fixture for write acceptance.
3. Build/run the explicitly feature-enabled acceptance binary:

   ```sh
   cargo run --manifest-path app/desktop/src-tauri/Cargo.toml --features desktop-writes -- --data-dir /private/tmp/GENERATED_FIXTURE/state
   ```

   Writes are enabled by default; the explicit flag documents this test requirement.
   Use only the newly generated disposable repositories for acceptance.
4. Exercise Table/Tree selection, tabs, draft preservation, filters, themes,
   resizing, keyboard/focus behavior, refresh, missing entries and restart.
5. Review trust for `engine` in the dedicated view. Cancel once, verify no grant,
   then create a new review and explicitly approve its independent native dialog.
   Verify resolved hook/signing identities. Do not change global Git settings to
   force a test to pass; report unsupported configurations accurately.
6. Select the **index** version of `README.md`, enter a fixture message, review
   the exact Core plan, and approve natively. Verify independently that HEAD has
   the staged fixture text, the worktree still has its different working text,
   excluded/untracked files remain intact, and the nested child was not committed.
   Check actual blobs/index/tree, not merely the UI success label.
7. Exercise stage/unstage, stale content and same-OID branch-switch rejection,
   hook rejection/bypass, isolated configured GPG/SSH signing, post-hook failures,
   guarded recovery and publication uncertainty. Never use personal signing keys.
8. Test closing/reloading reviews, cancelling during execution, forced interruption,
   restart recovery without replay, and closing the last primary window during
   inspection and writes. Verify that the process/watchers stop and the data lock
   is released. Do not infer this from compilation or mock-runtime tests.
9. Record native evidence and all unresolved failures before considering the
   M3 gates complete or changing ordinary desktop write exposure.

Graph/activity/audit exploration is implemented with automated M4 coverage. M5A
adds an explicit Bulk Commit form and durable batch results through the same native
review gate. Select worktrees in Overview, open Bulk Commit, choose messages/sources
and exclusions, and prepare the exact native review. Refresh results after native
execution. Selecting unfinished worktrees always requires a new review; published
successes are excluded. Default builds enable controlled write preparation.
See [M5A contracts](../../docs/architecture/M5A_LOCAL_BULK_COMMITS.md).
Each repository also exposes on-demand **Remotes and upstream** inspection:
effective fetch/push destinations, URL rewrites, origins, upstream/push-remote
suggestions, redacted credential fields and policy warnings. Inspection contacts
no servers and grants no network authority. See [remote boundary](../../docs/architecture/M5_REMOTE_BOUNDARY.md).
Acceptance builds also offer separately reviewed, credential-free observation of
one exact remote branch. It uses an isolated transport repository, verifies TLS,
rejects redirects and preserves managed Git state. Results are timestamped past
observations. Default builds enable supported remote review. See the
[observation report](../../docs/reports/M5B_REMOTE_OBSERVATION_2026-11-02.md).
The dated observation procedure above does not summarize current capability.
Supported authenticated fetch/push, assisted AI and SemVer Actions now exist; their
remaining acceptance, hierarchical/automatic AI, backup/retention and release work
are in the completion handoff. Hunk staging, destructive mutations and expanded
hook-manager execution remain outside the current first-release scope.

See [M3 contracts](../../docs/architecture/M3_DESKTOP_AND_IPC.md),
[current status](../../docs/IMPLEMENTATION_STATUS.md),
[handoff](../../docs/IMPLEMENTATION_HANDOFF.md), and
[evidence](../../docs/validation/m3/README.md).

MacOS native termination also enters guarded shutdown through an application-delegate
wrapper that forwards Tao's other callbacks. Direct termination fallback/retry and
restart have selected native evidence; actual Dock/session and unanswered-sheet
termination retain separate open cases. See the
[termination fix report](../../docs/reports/G02_GUARDED_APPKIT_TERMINATION_2026-11-07.md).

With an unanswered native confirmation attached, the tested macOS AppKit terminate
call returns and leaves the sheet open. Explicitly dismiss it, then request Quit
again. The [active-sheet acceptance report](../../docs/reports/G02_ACTIVE_SHEET_TERMINATION_2026-11-07.md)
records actual delivery, no implicit approval, cancellation/retry and restart.
Immediate native sheet cancellation/exit and actual Dock/session acceptance are
not inferred. Test scheduling and delivery audit remain non-default.
