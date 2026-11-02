# Native completion and durable result refresh

November 2, 2026. Addresses presentation findings from [native bulk acceptance](M5A_NATIVE_BULK_2026-11-02.md).

## Change

Native host exposes a monotonic completion revision through the existing main-only native-write job-status response. Confirmation success/decline/failure and abandoned stored reviews advance the revision; it carries no operation capability or authority. Core and generated Core contracts are unchanged. The renderer observes revision changes while idle, refreshes workspace/recovery inventory, and refreshes bulk, remote-observation and active applied-filter audit results. This also handles operations that begin/end between polling ticks. Result pages reset to the first page on an actual native lifecycle change, rather than on every inventory refresh. Revision/input errors remain visible, and scoped effect cleanup prevents late result application.

Bulk results load on entry/restart. Fully published batches disable Select unfinished worktrees instead of clearing selection with an empty retry list. Fresh-review selection for partially published batches remains covered by the existing regression.

## Automated validation

Full desktop validator passed: 33 renderer tests, eight native host tests per default/all-feature configuration, typecheck, lint, generated-contract drift, formatting, production frontend/native builds, strict Clippy and release build. New tests verify idle-to-idle native revision changes and malformed revisions, main-only revision IPC, and an actual App polling cycle replacing proposed batch results with complete/published without another review request. Core Git/signing/crash matrix was not rerun because Core is unchanged.

## Native validation

New disposable fixture `/private/tmp/gitfleet-m3-native-_4f0iu0_`, separate hashed acceptance artifact. Selected/trusted only engine/tooling and prepared a shared multiline message, engine Index only and tooling Review all. Approved fast batch `7f4df6b7-c253-4cfa-bea5-96d69656f11d` through the independent native dialog. On returning to the primary window, both results were already complete/Published without clicking Refresh. Retry selection was disabled. Independent Git checks confirmed one commit each, exact message bytes, engine's staged blob/index preservation, all working-file preservation and unchanged unselected audio/documentation.

Rebuilt/hashed the ordinary default bundle and reopened that already-trusted fixture. Bulk results loaded automatically; trust/write preparation stayed disabled and the read-only footer was visible. Closed the app. Repository snapshots were unchanged across the ordinary restart/session. Screenshots/AX inspected inline; no saved PNG claimed.

Evidence: [full gate](../validation/m3-native-results/desktop-validation.txt), [acceptance artifact](../validation/m3-native-results/artifact.json), [ordinary artifact](../validation/m3-native-results/ordinary-artifact.json), [source hashes](../validation/m3-native-results/source-sha256.json), [independent assertions](../validation/m3-native-results/independent-checks.json), [journal](../validation/m3-native-results/events.json), [snapshots and fixture](../validation/m3-native-results/).

Remaining: broader M3 review/context/accessibility and native recovery cases, remaining M5 workflows and M6–M8. Automatic refresh of remote/audit/recovery views has automated/static coverage here; the new native positive exercised Bulk specifically. No push or ordinary-write default change. The unrelated .gitignore edit remains unstaged.
