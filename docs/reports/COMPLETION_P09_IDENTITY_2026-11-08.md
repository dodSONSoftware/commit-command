# Completion P09 stable AI editor identity

Submilestone P09.1.2, baseline `93321ed`. Duplicate connection/profile names are valid metadata. After saving, the editor previously selected the first matching name, potentially directing the next save or native token action at a different saved entity.

The editor now preserves the exact edited ID. A new entity is resolved by the single new ID in the before/after saved snapshots, never its display name. Missing/ambiguous identity is an actionable local error rather than an implicit selection. No Core contract, credential binding, authority or persistence format changed.

Three new regressions (connection edit, profile edit, connection creation with duplicate names) failed before the fix and pass after it. Focused AI settings: 9 passed. Complete renderer: 126 passed across 29 files. TypeScript, ESLint, generated contracts, formatting and production frontend build all passed. Before/after/full logs are retained under `docs/validation/completion-p09-identity-2026-11-08/`.

This closes the specific duplicate-name selection defect only. P09 unsaved repository AI testing, safe durable AI audit/provider/native usability and the remaining G09 packages are still open. Native GUI acceptance was not performed for this renderer-only identity fix; no new source transfer or credential access occurred.
