# Native write shutdown and guarded recovery checkpoint

November 2, 2026. Source `00ba0e9`, separate hashed development acceptance artifact with `desktop-writes`; ordinary defaults unchanged. All operations used fresh disposable fixture `/private/tmp/gitfleet-m3-native-21cg777_`. No user repositories, real credentials or remote activity.

## Native observations

- The independent trust review initially focused **Cancel review**. Pressing Return dismissed it without granting trust. A fresh review and separate native approval granted trust only to engine.
- An exact one-path index-only commit used the installed pre-commit hook. The hook wrote its PID outside the repositories and slept 120 seconds before intended exit 23. Process inspection proved app PID 48455, hook 50537 and sleep 50538 alive. Switched from the review window to the primary window through the Window menu and closed the primary window while the hook ran. All three processes exited; the journal recorded `cancelled_before_publication`, published false, for `4833408b-5666-465a-9591-cfd76c99d165`. All four repository snapshots were identical. After restart the native Operation results showed that exact durable outcome and no replay.
- A separate fresh review/native approval started `2a5fbbd6-0375-4f3d-8461-56e9718e36fd`. After the second hook sentinel, abruptly killed only fixture app PID 50601, then its isolated hook/sleep process group 52321/52322 before restart. The journal stopped at `hooks_running`, published false. Repository snapshots still matched.
- Restart displayed the attention notice and exact interrupted operation. Inspect guarded recovery opened an independent review specifying `abort_unpublished`, no publication evidence and no hook/commit replay. Separately approved that recovery through the native dialog. Core returned `cancelled_before_publication`, published false, with `Aborted without replay; unreferenced objects/artifacts retained`. Refresh showed the terminal outcome. Hook sentinel remained exactly two executions total, one per explicitly started trial. All four repository snapshots still matched. Closed the app and reopened its data lock with the CLI.

Screenshots/AX were inspected inline; no saved PNG is claimed. Evidence in [m3-native-lifecycle](../validation/m3-native-lifecycle/) includes fixture/artifact provenance, acceptance build output, process records, before/after repository hashes/modes/symlinks, exact journal events and verification summary. No application state database or key material is committed. An initial fixture setup needed creation of the empty hooks directory; no write trial ran before the final hashed artifact/setup.

## Findings and limits

Background renderer polling continued queueing inventory/trust requests during the long write, displaying `Application busy; request queue is bounded`. Cancellation and orderly close still worked. Next patch will serialize polling and skip repository refresh while a native write is running.

These tests cover close during pre-publication hook execution and abrupt interruption at that same actual phase. They do not establish every native publication/reconciliation crash phase; existing automated phase coverage remains separate. Full VoiceOver, measured contrast, hostile-content/context/expiration and release artifact acceptance remain open. M3 is incomplete; ordinary writes stay disabled. No push occurred.
