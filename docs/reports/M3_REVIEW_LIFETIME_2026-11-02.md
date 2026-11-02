# Native review lifetime and current validation matrix

November 2, 2026. Continued execution without routine user supervision. Local checkpoint commits only; no push.

## Native expiration and workspace invalidation

Fresh disposable fixture `/private/tmp/gitfleet-m3-native-kozwasvf`, exact hashed acceptance artifact from source fcfec7c. No external services, user repositories or signing credentials used in the native cases.

Opened trust review `bc98d499-7676-44bc-a3b9-672752396e94` for engine. Its displayed expiry was 11:43:56 a.m. Central (17:43:56 UTC). Continued to independent native confirmation before expiry and left that actual dialog pending. After a recorded UTC clock check at 17:44:20, clicked Approve exact operation. Core rejected it: `Application context or review changed`; no trust was granted. The clock and Core's ten-minute lifetime were unchanged. This tests a native confirmation capability created before expiry and returned after the deadline, rather than only trying to open an already expired review.

Prepared a fresh review `35532003-f8b6-4945-8e41-e7af1d9b2df7`. Through the Window menu switched to the main window and removed only the saved fixture workspace definition while its review remained pending. The main view reset to Welcome/zero repositories. Returned to the old review and attempted Continue; Core rejected it before opening a native dialog. Closed both windows. Independent SQLite inspection after exit showed zero trust rows and zero write operations. All four full repository/Git snapshots matched the pre-session baseline. The CLI reopened the state lock; exact fixture process inspection found no surviving app sessions.

Screenshots/AX were inspected inline; no saved PNG is claimed. The context test also shows that the stale review window remains visible until used or closed. Rust rejects its authority; proactive stale-window dismissal remains a presentation improvement.

## Presentation change

The isolated review page now disables confirmation and explains expiration when an idle review's displayed deadline passes. A click guard independently rejects a deadline race before invoking native confirmation. Once confirmation begins, that presentation timer is cleared: a subsequently executing operation must not be mislabeled expired. Core still independently checks the deadline after the OS dialog returns. No Core timeout, authorization or clock change.

The new timer has automated coverage; the long native case used the earlier hashed fcfec7c artifact, before this presentation-only addition. The final ordinary bundle was rebuilt/hashed with the timer. This distinction is retained rather than claiming native acceptance of an untested UI timer.

## Actual validation

The full Core matrix passed on current unchanged Core sources:

| Configuration | Passed | Ignored |
|---|---:|---:|
| Default | 53 | 1 |
| Experimental writes | 88 | 2 |
| Controlled writes | 76 | 1 |
| Application services | 125 | 2 |
| All features | 165 | 4 |

All builds, formatting/layout and strict Clippy configurations passed. Mandatory isolated SSH/OpenPGP positives, real transport fixtures, crash/recovery and batch interruption tests passed. Historical lab separately passed 23 with one optional GPG skip. Counts overlap across configurations and must not be summed as unique cases.

Each configuration includes one real ignored non-UTF8 filename-creation case because this host filesystem returns EPERM. Experimental/all-features additionally skip the optional historical-lab GPG case; this does not skip the mandatory controlled-write signing positives. Application/all-features include a batch subprocess worker; all-features also includes the crash worker. Parent tests invoke/assert those workers. These distinctions are in core-counts.json.

The initial sandboxed matrix failed because isolated gpg-agent IPC was blocked. That unsuccessful log is retained. The permission-reviewed retry completed with exit zero; no tests were weakened or disabled. Full final desktop gate passed 40 renderer tests, eight native host tests per default/all-feature configuration, typecheck, lint, generated contracts, formatting, production frontend/native builds, strict Clippy and release build. Three added review regressions cover expiration, the click/deadline race and an in-flight operation crossing the deadline.

## Reconciliation and next work

The completion plan now labels its original code baseline/counts as historical and removes the obsolete Mac-locked/M4-next execution paragraph. The requirement crosswalk distinguishes actual native bulk/content/restart checks from still-open dependency/failure cases; selected accessibility and current audit limits are explicit. Historical raw logs/handoffs remain unchanged.

M3 remains partial: proactive stale-review dismissal, interactive theme selector diagnosis, full keyboard/VoiceOver/layout/scale, broader native crash phases and final artifact/default acceptance remain. Ordinary writes stay disabled. M5 fetch/authenticated transport/upstream/push and M6–M8 remain unfinished. The unrelated .gitignore edit and concurrent docs/graphics assets are untouched and excluded.

Evidence: [native setup](../validation/m3-review-lifetime/lifetime-setup.json), [after-deadline clock](../validation/m3-review-lifetime/after-deadline-clock.json), [native assertions](../validation/m3-review-lifetime/native-checks.json), [before](../validation/m3-review-lifetime/before.json), [after](../validation/m3-review-lifetime/after.json), [native artifact](../validation/m3-review-lifetime/artifact.json), [final ordinary artifact](../validation/m3-review-lifetime/final-ordinary-artifact.json), [source](../validation/m3-review-lifetime/source-sha256.json), [initial failed matrix](../validation/m3-review-lifetime/core-validation.txt), [successful matrix](../validation/m3-review-lifetime/core-validation-approved.txt), [counts](../validation/m3-review-lifetime/core-counts.json), [final desktop gate](../validation/m3-review-lifetime/desktop-validation-final.txt), [summary](../validation/m3-review-lifetime/validation-summary.json).
