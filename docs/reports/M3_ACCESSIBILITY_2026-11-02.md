# Selected contrast and keyboard accessibility fixes

November 2, 2026. Scope: measurable contrast defects and selected accessibility gaps in existing main/review/graph/activity views. No cosmetic redesign, Core authority or feature-default change.

## Findings and changes

The review primary label (#e1ecff on #1674dc) measured 3.864:1, below 4.5:1. Its new fill (#1264c7) measures 4.803:1. Main primary text now has more margin (white/#1264c7, 5.717:1) and an explicit darker hover fill; the inherited generic Light-mode hover previously placed white primary text on a pale background. Placeholders have an explicit opaque muted color. Text-entry boundaries now use a separate contrast-qualified token rather than decorative separators.

Graph lanes use theme tokens instead of fixed dark-theme colors. Added a Light-mode purple. Table/Tree, Changes/History and selected graph commits expose pressed state. Heatmap cells expose image descriptions, and a disclosure provides the actual daily UTC counts in a semantic table, including zero-observed days, with bounded/partial-result wording.

Contrast uses the sRGB relative-luminance calculation from [W3C contrast minimum](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html) and [non-text contrast](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html). Tests read the actual declared CSS palette: normal/muted text and actual tinted text surfaces require 4.5:1; graph colors, offset focus outlines and input boundaries require 3:1 against relevant surfaces. Review/primary text pairs are also checked. Ratios are not rounded before threshold comparison. This is selected source-style measurement, not a complete rendered-style audit or WCAG certification; disabled controls are not treated as active text.

## Verification

Full desktop gate passed: 37 renderer tests, eight native host tests per default/all-feature configuration, TypeScript, lint, contract drift, format, frontend/native builds, strict Clippy and release build. Four new contrast tests cover Dark, Light and System-light palettes plus main/review primary text. Updated component regressions verify pressed state and the accessible daily table. Core code is unchanged; full Core signing/crash matrix not rerun.

Rebuilt/hashed ordinary development bundle on the existing disposable fixture. Native AX announced Table on/Tree off; Tab moved focus visibly to Tree and Space activated it. Selection remained explicit. Activity heatmap exposed per-day image descriptions and Daily activity counts opened an actual named table with Date/Commits columns. Ordinary write controls remained disabled. In Light mode, observed input boundary and focus ring, restored Tree state, theme-aware actual graph and pressed Changes/History/commit states. Tab advanced from a selected commit to the next commit. All four repository file/Git snapshots remained equal to the prior baseline; state lock reopened after close.

Native theme-menu selection attempts through AX/mouse/keyboard left the displayed value System. The control rejected setValue as non-settable. No application root cause was established, so interactive theme switching is **not accepted** by this run. With the app closed, fixture-only presentation state was seeded to Light (no trust/operation change), then relaunched to inspect actual Light rendering. That setup and limitation are recorded. Full native VoiceOver, broad keyboard traversal, zoom/scale/narrow layouts, heatmap visual intensity distinctions and final artifact acceptance remain open. Screenshots were inspected inline; no saved PNG claimed.

Evidence: [desktop gate](../validation/m3-accessibility/desktop-validation.txt), [contrast measurements](../validation/m3-accessibility/contrast-measurements.json), [artifact](../validation/m3-accessibility/artifact.json), [source hashes](../validation/m3-accessibility/source-sha256.json), [Light fixture setup](../validation/m3-accessibility/light-fixture-setup.json), [preservation](../validation/m3-accessibility/repository-preservation.json), [repository snapshot](../validation/m3-accessibility/repositories-after.json).

M3 remains partial and ordinary writes remain disabled. No push. The unrelated .gitignore edit and concurrently appearing docs/graphics assets are untouched and excluded from this commit. Next work: native theme-selector diagnosis, remaining review/context/accessibility gates, then unfinished completion-plan packages.
