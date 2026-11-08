# Native batch host-crash acceptance — November 8, 2026

Prepared corrected fixture `/private/tmp/gitfleet-m3-native-g05-batch-w8awkopg`.
Repositories are below workspace/gitfleet-batch, satisfying Core's immediate-parent
prefix requirement. Each has an ignored .gitfleet-m2-fixture marker; native host's
exact root marker and workspace confinement remain required. Only file.txt is dirty
and eligible. The app is launched with process-local TMPDIR=/private/tmp, matching
Core's canonical temporary-root guard. No guard was changed or relaxed.

The first corrected preparation attempt discovered empty-template Git init creates
no .git/info directory; the script now creates it before writing the marker exclusion.
That untrusted unused fixture was not executed. The recorded successful fixture is
new; neither earlier failed-attempt paths nor prior grants were reused.

Exact native binary was rebuilt from fa77898 with custom-protocol, desktop-writes,
native-acceptance-support. Ordinary companion is the latest validated release-mode
binary without write/test features, copied into a disposable development bundle.
These are acceptance bundles, not distributable signed/notarized artifacts.

Current preflight: exact repository baseline, zero write operations/trust rows,
canonical paths below declared process temp root, required parent names, repository
markers and exact native root marker. Authoritative Rust review names only alpha
and beta; explicit authorization requested for those exact new paths. The unselected
control remains untrusted. No commit or fault execution has occurred in this fixture.

Next, once specifically authorized: grant exact pair, fresh two-item native batch,
fixed Native acceptance SIGKILL-after-ref-update menu, exact native batch approval.
Require host process exit by SIGKILL, first commit present, successor/control unchanged.
Ordinary restart must identify the interrupted partial batch without replay, show
publication uncertainty honestly, and expose fresh guarded recovery. Reconcile only
proven owned state with separate native approval; then review only unfinished work.
Record exact bytes/modes/refs/journal and no repeated first commit across restarts.
A preparation pass is not an actual crash acceptance pass.

Earlier failed fault attempt and safe index-only recovery are preserved separately
in [the uncertainty report](G05_UNCONFIRMED_PUBLICATION_2026-11-08.md). Current
[evidence](../validation/g05-native-batch-host-crash-2026-11-08/) records the corrected
preparation, successful build/guard preflight, scan and baseline/binary identities.
