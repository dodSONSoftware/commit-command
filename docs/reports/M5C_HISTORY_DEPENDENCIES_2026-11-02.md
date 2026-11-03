# M5C outgoing-history child requirements

November 2, 2026. Push previews now enumerate unique path/child-commit pairs from **every outgoing commit tree**, independently of the 100-commit presentation limit. This extends the source-only inventory in checkpoint `e63d56c`. Requirements remain local evidence, with no child checkout access, network operation, destination selection, credential lookup, push authorization or actual push.

## Behavior

The captured observed destination OID and source OID define the existing outgoing range. A bounded `rev-list` returns every outgoing commit/tree pair, including merge-side history, and its count must match the independently computed outgoing count. Each distinct tree is inspected with the existing byte-safe `ls-tree` reader. Identical trees are scanned once. Unique raw-path/child-OID pairs retain an example parent commit showing one occurrence. Removed or replaced gitlinks remain visible even when the final source contains no gitlinks. Sorting is deterministic by raw path and child OID. Source-tree requirements remain separately visible.

The UI clearly states that these requirements have no proven availability on intended child remotes. An example parent is one occurrence, not an exhaustive provenance list. No `.gitmodules` URL is resolved, followed or promoted into authority. Future destination proof must account for historical mapping changes and cannot treat this deduplicated list or its single example as complete destination evidence.

Limits: the existing 10,000-commit ancestry bound, session deadline/cancellation, 2 MiB per-command output limit and 1,024 gitlinks per tree remain. The history inventory additionally refuses more than **256 distinct outgoing trees** or **1,024 unique path/child requirements**. Exceeding either bound returns an error, never a partial successful inventory. This deliberately narrows successful local preview scope for histories with many distinct trees; large histories that previously produced a count-only preview may now require a smaller range/future scalable inspection work. No fallback silently skips dependencies. All context/generation/configuration checks and final transport revalidation remain.

## Validation

Focused application preview tests passed **nine** cases. Added fixtures cover SHA-1/SHA-256, introduced/replaced/removed links, a unique dependency reachable only through a merge side parent, requirements older than the 100 displayed commits, raw tab/newline and non-UTF-8 paths, identical-tree deduplication, exactly 1,024 accepted requirements, 1,025 requirements rejected even when the final source fits, and 257 distinct trees rejected. Before/after complete repository and fixture-remote snapshots verify no effects. Existing cancellation/refusal cases remain exercised. All fixtures are disposable local repositories; no Git push is used to seed or test them.

Renderer coverage adds a removed historical requirement with an empty source inventory and hostile-looking path text. It verifies separate historical/source counts, literal text rendering, explicit unproven remote availability, example provenance and absent execution controls. Generated response contracts and the existing inspector test fixture were updated. No persistence schema, dependency or lockfile changes.

Final affected Core transport suite passed **61 tests/four documented ignores**; strict all-target/all-feature Core Clippy and formatting passed. The complete desktop gate passed **58 renderer tests** and **13 native host tests per configuration**, generated contracts, type/lint/format/build, strict Clippy and default release build. Exact results are recorded in `docs/validation/m5c-history-dependencies/validation-summary.json`, alongside command logs and source/artifact hashes. The whole five-feature Core matrix is not claimed as rerun. The ordinary default development bundle is refreshed; live native presentation of the new history section is not claimed.

## Next work and boundaries

Fresh explicit intended child/destination selection and server reachability evidence remain open, including mapping changes across outgoing history, selected/unselected dependency ordering and safe interruption/recovery. The push preview retains its existing single-identical-fetch/push-destination restriction. This checkpoint does not implement push execution, pre-push hooks or individual/bulk push authority. Ordinary desktop writes remain disabled; no push.

The M5B SecurityAgent Deny/Allow/unlock and lingering prompt-dismissal tests still require operator assistance because organizational computer-use policy blocks SecurityAgent. No new Keychain material or credential/store access occurred here. Preserve unrelated `.gitignore`, `docs/README.md`, `docs/APPLICATION_OVERVIEW.md` and `docs/graphics/` changes.
