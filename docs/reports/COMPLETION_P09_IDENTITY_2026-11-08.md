# Completion P09 stable AI editor identity

Submilestone P09.1.2, baseline `93321ed`. The editor previously resolved a saved selection by display name. The patch makes identity resolution independent of display names.

Correction during subsequent P09 source inspection: Core already rejected case-insensitive duplicate names at baseline `93321ed` (`ai/config.rs`, Settings::validate), and still does. The three duplicate-name regressions use renderer fixture responses, not valid persisted Core metadata. The earlier claim that duplicate names were valid was incorrect; this checkpoint establishes defensive renderer identity behavior, not reproduction of a currently permitted end-to-end Core save defect. No duplicate-name policy was changed.

The editor now preserves the exact edited ID. A new entity is resolved by the single new ID in the before/after saved snapshots, never its display name. Missing/ambiguous identity is an actionable local error rather than an implicit selection. No Core contract, credential binding, authority or persistence format changed.

Three new regressions (connection edit, profile edit, connection creation with duplicate names) failed before the fix and pass after it. Focused AI settings: 9 passed. Complete renderer: 126 passed across 29 files. TypeScript, ESLint, generated contracts, formatting and production frontend build all passed. Before/after/full logs are retained under `docs/validation/completion-p09-identity-2026-11-08/`.

This completes the renderer stable-ID improvement only; duplicate-name end-to-end acceptance is not claimed. P09 unsaved repository AI testing, safe durable AI audit/provider/native usability and the remaining G09 packages are still open. Native GUI acceptance was not performed for this renderer-only identity fix; no new source transfer or credential access occurred.
