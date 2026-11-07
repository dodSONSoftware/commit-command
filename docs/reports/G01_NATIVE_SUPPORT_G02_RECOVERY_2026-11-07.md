# G01 closure and G02 native crash recovery

November 7, 2026. **G01 accepted within the recorded macOS/local-write envelope;
G02 remains partial.** This completes the live reset/caller cases left by the
[previous continuation](G01_G02_CONFIRMATION_LIFECYCLE_2026-11-07.md), then adds
selected native close-fallback and crash/recovery evidence. Historical reports
retain their original observations; current dispositions live in
[EXECUTION_STATUS](../EXECUTION_STATUS.md).

## Test-only support

Added non-default `native-acceptance-support`, selecting Core's existing
`write-test-support`. It does not select `desktop-writes`; write acceptance must
select both. Startup requires an existing canonical `state` directory under a
marked `gitfleet-m3-native-*` temporary fixture and validates the initial workspace
roots beneath that fixture. This is a test guard, not a sandbox against a hostile
same-user process. The existing, stricter Core fault-injection guard is unchanged.

A fixed native menu can reload main, attempt three forbidden IPC callers and two
forbidden navigations, suppress one close-request delivery, or arm one of two
existing commit SIGKILL checkpoints. Caller probes use fixed host-owned scripts
and arguments through real webviews. There is no new IPC command, capability,
renderer-controlled script/path, approval field, or substitute for OS confirmation.
Crash selection goes through the existing Core worker and a feature-only facade
setter. The menu records bounded diagnostic events in fixture state.

All native actions used Computer Use and separate disposable bundles/state. Native
AX results were inspected; no saved screenshots are claimed. No user repository,
credential, production service, default write policy, or installed app bundle was
changed. Project changes remain uncommitted and unpushed.

## G01 results

[G01 evidence](../validation/g01-native-support-2026-11-07/README.md) contains the
native event log, artifact hashes, baseline hashes/modes and independent verdicts.

| Remaining case | Observed native result / independent check |
|---|---|
| Main calls review confirmation | Actual main webview rejected the fixed `review_confirm` invocation; probe reported `PASS denied`. No operation capability was supplied. |
| Review calls main dispatch | Actual review webview rejected the fixed read-only `dispatch`; probe reported `PASS denied`. |
| Unprivileged webview | Separate `native-probe` label loaded review.html without review capabilities. Its normal `review_data` and fixed main-dispatch attempt were denied; dispatch probe reported `PASS denied`. |
| Wrong main page / origin | Native attempts to navigate main to review.html and a different tauri host were rejected by the production navigation policy. Main remained at its original page; audit recorded both `allowed: false`. Unit rejection cases cover additional malformed/encoded URLs. |
| Reload with unapproved trust review | Real main webview reload invalidated/closed the review. Zero trust rows/operations; all four repositories' bytes/modes matched baseline. |
| Reload with issued commit confirmation | After approving only fixture engine trust, selected index README and opened the real independent commit confirmation without approving it. Main reload cancelled the sheet/review and the operation became `cancelled_before_publication`, unpublished. One trust row, one cancelled operation; all repository bytes/modes unchanged. |

Together with the two preceding November 7 reports, this closes G01's enumerated
native authority/fidelity cases. Configured OpenPGP and other selected policies
retain their dated evidence; this run does not refresh every signer/hook variant.
G03 accessibility, G06 compatibility, unsupported managers/interactive signing,
non-UTF-8 filesystem acceptance, hostile same-user manipulation and release
platforms are not accepted through G01. Ordinary writes remain disabled under G04.

## G02 results

| Case | Observed native result / independent check |
|---|---|
| Missing close-request delivery | Armed one-shot delivery suppression, then Cmd+Q. Production timeout opened the native unsaved-drafts fallback. Selected Keep app open, verified draft retained, then ordinary Quit exited 0. Persisted draft matched exactly. This simulates missing delivery/acknowledgement; it is **not an actual renderer process crash**. |
| SIGKILL at `publication_pending` | Approved exact fixture commit through the real review and OS sheet. Native process terminated at the existing checkpoint; journal retained that phase, actual HEAD stayed at baseline (two commits), and index/worktree/excluded repositories were preserved. Restart showed needs attention and did not replay. Separate guarded `abort_unpublished` review/OS approval yielded `cancelled_before_publication`, no publication. |
| SIGKILL at `reference_updated` | Approved a different disposable fixture commit. Native process terminated after the actual ref advanced: exactly three commits, expected parent/message/staged blob, while the durable record still had `published: false`. Restart preserved that discrepancy without replay. Guarded recovery freshly observed publication and proposed `reconcile_index`; separate OS approval yielded `complete`, published/index reconciled true. Commit count and HEAD stayed unchanged; post-processing was explicitly skipped, with no hooks/commit creation replayed. |

[Crash evidence](../validation/g02-crash-boundaries-2026-11-07/README.md) preserves
before/restart/recovery verdicts and one recorded operation per fixture. All working
files, index entries and other repositories remained preserved. Approved recovery
processes exited 0. Process termination was also observed as native-app loss;
Core's selected test fault explicitly sends SIGKILL to its own process. This does
not establish power-loss durability.

The first crash fixture used the standard `/private/tmp/.../workspace` layout.
Core correctly refused injection because it lacked Core's stricter temporary-root,
`gitfleet-*` parent and `.gitfleet-m2-fixture` requirements. Its unpublished
uncertain-outcome record is retained as a refused setup, not a crash pass. Fresh
compatible system-temporary fixtures were created with those markers/layout and
a `workspace` alias for the native guard. Core's guard was not relaxed. An unused
initial after fixture is also identified explicitly in evidence.

## Validation and disposition

Final logs and a source/artifact manifest are linked from the evidence README.
The final desktop validation passed **69 renderer tests**, **13 default host /
15 all-feature host tests**, format/type/lint/contracts checks, strict Clippy and
debug/release builds. Core application tests passed **24 cases**; the existing
13-boundary aggregate SIGKILL matrix passed separately. Default debug/release
artifacts exclude the fixed acceptance menu; the ordinary feature graph excludes
Core test-write support. The support executable refused nonfixture state before
opening Application. These are overlapping targeted suites, not the full Core five-feature,
signing, remote or release matrix.

G02 remains open for **actual renderer-process loss**, **Dock/system termination**,
and any additional native phase-specific acceptance required by M3-A16/A17 beyond
these two publication boundaries and earlier hooks-running evidence. Injected
close-delivery suppression and automated crash coverage do not waive those cases.
Next work starts there; G03/G05/G06 and the G04 default-change decision remain
separate prerequisites. No push executor or release is implied.
