# G09 optional AI settings model — November 8, 2026

Added transport-neutral, versioned AI configuration types and validated immutable
editor changes in Core. Connections, profiles and structured message templates have
stable identities/revisions. Profiles own model/system/optional parameters/context
budget/template. Resolution chooses application → workspace → repository → explicit
one-off, adds scoped instructions, and refuses missing explicit selections without
fallback. Default settings contain no connection/profile and perform no AI work.

Limits cover entity counts, names, identifiers, finite/ranged provider-specific
parameters, exact canonical HTTP(S) destinations, settings bytes and template output.
Userinfo/query/fragment/unspecified bind destinations are refused. Static preview
requires no provider. Subject/body validation rejects control/bidi spoofing and checks
conventional syntax, permitted types/scopes and length. Natural-language rules remain
instructions; these checks do not claim semantic correctness of model output.

Editor changes use optimistic revisions and validate the entire cloned result before
returning it. Deleting referenced entities fails instead of silently selecting a
fallback. Connection edits/import clear endpoint metadata. Editor DTOs reject unknown
fields, including forged approval and automatic-commit properties. Stored approval
metadata itself is not a transport capability. There is no network, provider job,
credential store, persistence/UI, candidate generation or automatic authority in this
checkpoint; those remain G09 work. Core is independent of Tauri and manual services
never need to load AI settings.

Validation: application-services library suite 36 passed, including nine AI tests;
all-feature/all-target strict Clippy, format and diff checks passed. Initial focused
run was seven passes/one failure for body control screening; regression fixed before
the final run. JSON/TOML round-trip, no-AI defaults, hierarchy/no fallback, invalid
parameters/endpoints, reference/stale/transactional edits, import invalidation and
static message rules covered. No native acceptance or full write matrix rerun is
claimed for this pure model boundary. Evidence in
[validation](../validation/g09-ai-settings-foundation-2026-11-08/).
