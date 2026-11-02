# Mixed-DPI native window restore

November 2, 2026. M3A/M3-A06/A19 follow-up. Base `c0059f6`; ordinary default-feature bundle, disposable fixture only. No Core schema, authority or write-default change.

## Diagnosis and patch

Audit native acceptance revealed a 720×470 viewport below the configured 960×640 minimum. With the fixture closed, seeded only its native placement to physical x=1200, y=309, width=1440, height=940. Temporary diagnostic logging proved the hidden window started at scale 2 while the selected saved monitor used scale 1. The existing physical setters converted coordinates and dimensions using the hidden window's initial backing scale before it moved. On close, the scale-1 window recaptured x=600, y=155, width=720, height=470. This reproduced the exact shrinking behavior; it was not inferred solely from screenshots.

Restore now carries the selected monitor's scale with its work area, clamps saved dimensions against that monitor's physical equivalent of the logical minimum, then passes explicitly converted logical position/size to the native setters. Conversion no longer depends on the hidden window's initial display. The window builder and restore share minimum-size constants. Off-screen clamping and the existing physical-coordinate storage contract remain. If a work area is smaller than the normal minimum, fit stays inside that area. Temporary diagnostics were removed before the final build and acceptance.

## Actual validation

Full desktop gate passed: 45 renderer tests, 11 native host tests under each default/all-feature configuration, typecheck/lint/contracts/format, production frontend/native builds, strict Clippy and release build. Added destination-scale minimum/work-area coverage, including negative coordinates, scale-2 physical bounds and smaller available work areas. Existing geometry tests still pass. Core source did not change after the preceding successful full five-configuration matrix, so signing/crash tests were not repeated for this native-only patch.

Exact final hashed ordinary bundle passed these native cases:

1. Previously undersized x=600, y=155, 720×470 state restored to a visible 960×640 viewport; close persisted the repaired size and same position.
2. A second restart preserved x=600, y=155, 960×640 exactly, with no repeated shrink.
3. Original physical x=1200, y=309, 1440×940 state restored visibly at 1440×940 and recaptured exactly.
4. Actual scale-2 display: physical x=-5000, y=900, 1920×1280 restored visibly at logical 960×640 (1920×1280 screenshot pixels) and recaptured the exact physical rectangle.

All fixture windows closed through the existing safe Cmd+Q path; the CLI reopened the state writer lock. The exact saved draft survived. All four repositories' file hashes/modes and every local/batch/remote journal payload/event row matched the prelaunch audit baseline. No server contact, trust grant or Git write occurred. AX/screenshots were inspected inline; no PNG is claimed. Diagnostics are evidence from a temporary diagnostic artifact, while successful retests use the separately hashed final artifact.

Evidence: [diagnostic reproduction](../validation/m3-window-dpi/native-diagnostic.txt), [final artifact](../validation/m3-window-dpi/artifact.json), [full desktop gate](../validation/m3-window-dpi/desktop-validation.txt), [native summary](../validation/m3-window-dpi/summary.json), and exact placement JSONs in that directory.

## Remaining gates

This resolves the observed cross-display scale conversion and minimum-size defects. Physical display disconnect/reconfiguration, all focus/VoiceOver/zoom/narrow-review workflows and final release-artifact acceptance remain open. M3 is still partial and ordinary writes remain disabled. No push. Concurrent .gitignore, docs/graphics, docs/README.md and docs/APPLICATION_OVERVIEW.md work is excluded.
