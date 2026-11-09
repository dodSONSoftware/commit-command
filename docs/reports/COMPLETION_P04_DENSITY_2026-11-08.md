# Completion P04 overview density and sidebar visibility

Added workspace-persisted Comfortable/Compact information density and a sidebar visibility control outside the sidebar. Compact reduces table/overview/sidebar navigation spacing without reducing text size; tree spacing is reduced without adding padding. The tab-strip Hide/Show sidebar button exposes expanded state and controls the sidebar ID. Hidden sidebar and separator are removed from display/keyboard traversal; the remembered width remains unchanged and returns when shown. Sorting/filtering/selection/drafts are unaffected.

Version-1 navigation JSON gains defaulted density and sidebar_collapsed fields; older records retain current Comfortable and visible-sidebar behavior. Core's enum rejects unknown density values, and actual Application reopen preserves Compact/hidden state with existing width/drafts/tabs. No SQL schema change.

Validation: new renderer test resizes by keyboard to260, hides, persists Compact/hidden/width260, verifies separator removal and restores width260. Legacy/default and actual navigation restart Core tests passed. Full desktop gate passed:135 renderer,20 ordinary host/23 all-feature host tests (one deliberate ignore each), type/lint/contracts/format/build/strict Clippy/release. A final CSS review corrected compact tree spacing; renderer build and ordinary native host build were rerun after that CSS-only adjustment.

Native ordinary fixture was explicitly seeded Compact/width260 (no native density-menu selection claim). Compact rendered; Hide sidebar freed overview width with Show sidebar available; Quit exited zero and stored Compact/hidden/260. Explicit restart restored Compact/hidden; Show sidebar restored width260. Second Quit exited zero. Exact repository/index/mode preservation, zero trust/write rows and SQLite integrity passed. Evidence:docs/validation/completion-p04-density-2026-11-08/.

P04 remains partial. Full density-menu keyboard/VoiceOver, narrow-window/zoom/contrast/mixed-display acceptance, composer/inspection and exact final-artifact criteria remain open. Compact rendering does not certify all workflow geometry. No default-write change or release acceptance.
