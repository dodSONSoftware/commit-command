# GitFleet desktop — M3 implementation, native acceptance pending

Tauri is a thin adapter to the existing, presentation-independent Core crate.
The main React/TypeScript view and the dependency-minimal authoritative review
view are separate bundles with separate command permissions. Core default builds
remain read-only with respect to managed Git repositories.

**M3 is not accepted as complete.** The earlier organization block on agent Computer
Use is preserved in historical evidence. The [November 2 checkpoint](../../docs/M3_NATIVE_ACCEPTANCE_HANDOFF_2026-11-02.md)
records later user-observed native read/write, approval, signing and restart results;
these were not independently reproduced by the documenting agent. Actual cancellation,
guarded recovery, close/forced-interruption and accessibility gates remain open.
No policy bypass or ordinary-write enablement follows from those observations.
The ordinary debug/release binary independently rejects desktop trust/write
requests in Rust. Its write controls are disabled. Read-only recovery inspection
is available, but its execution confirmation is also disabled.

## Build and automated validation

Requirements: Node 22.12+, npm 10+, Rust 1.99+, Git 2.56+, macOS native command-line
tools. The executed host is macOS 27.2 arm64. No other desktop platform is accepted.
No global package installation, personal signing keyring, or global Git config
change is required. Locked npm installation disables dependency install scripts;
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

Main-view commands: `desktop_capabilities`, `dispatch`, `pick_directory`,
`job_status`, `cancel_job`. Review-view commands: `review_data`, `review_confirm`,
`review_cancel`. Both custom-command ACLs and Rust actual-caller/page/origin checks
apply. There are no generic filesystem, shell, Git argv, SQL, HTTP, approval-token
or journal-import endpoints.

The renderer can request a stored plan. Rust opens the dedicated review page,
which loads immutable content from Core, renders text only and requests a native
confirmation. Only the actual Rust/native dialog result can supply Core's opaque
confirmation capability. The M2 approval itself never crosses IPC. A proposal ID,
frontend checkbox, JSON `approved` flag, renderer event or digest is not authority.
Renderer reload/workspace changes invalidate reviews; cancellation is phase-aware
and cannot promise rollback after publication.

## Native acceptance workflow — still required

These steps require permitted native access or a human operator. They describe the
full procedure; consult the [reconciled checklist](../../docs/M3_NATIVE_ACCEPTANCE_PLAN.md#results-checklist)
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

   This non-default feature is for finishing acceptance; it is not authorization
   to target real repositories and must not be promoted to the ordinary build
   while the native gates remain open.
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

Full graph/activity/audit exploration is M4; bulk Git operations are M5. Remote
operations, AI, Actions, hunk staging, discard/reset/amend, advanced hook support,
backup/retention UI and release packaging were not implemented.

See [M3 contracts](../../docs/architecture/M3_DESKTOP_AND_IPC.md),
[current status](../../docs/IMPLEMENTATION_STATUS.md),
[handoff](../../docs/IMPLEMENTATION_HANDOFF.md), and
[evidence](../../docs/validation/m3/README.md).
