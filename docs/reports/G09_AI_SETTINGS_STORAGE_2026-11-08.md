# G09 optional AI settings storage and editor — November 8, 2026

Implemented explicit application AI settings, independent of workspace creation and
manual composition. The ordinary build can edit connection metadata, profile-owned
model/system/optional parameters/budget/template, structured templates, scoped additive
instructions/assignments and concurrency limits. Duplicate-profile/default-system
controls and effective configuration inspection are available. Static template
preview uses current unsaved editor input. No provider discovery, inference, credential
store, source transfer or Git/Action authority is implemented by this checkpoint.
Those capabilities remain visibly unavailable until their separate integration.

Core lazily reads `state/ai/current.toml` only for explicit AI requests. Versioned,
private generation directories hold metadata TOML and separate UTF-8 Markdown system,
template and instruction text. Files are exclusively created/synced, generation and
parent directories synced, then a synced pointer is atomically renamed. Existing
and orphan generations remain preserved for later G10 retention/backup work. Native
Unix descriptor-relative openat operations refuse symlink components, hardlinked/
nonregular/nonowned files and bounded-read overflows. Missing configuration yields
empty optional AI settings; corruption affects only AI requests. No permissive reset
or automatic activation of an incomplete generation.

Whole-value fingerprints include externally edited prompts. Saves check the loaded
fingerprint/revision, validate the cloned result, and recheck disk immediately before
pointer activation. Stale saves refuse without discarding editor text or old files.
The application owns the existing exclusive state writer lock. This does not claim
filesystem transactions against an arbitrary concurrent external editor; preserved
old generations remain available. Imported metadata never supplies endpoint authority.

Validation: final library suite 39 passed (12 AI tests), application suite 30 passed,
strict all-feature/all-target Core Clippy, format and diff passed. Final desktop
runner exited zero: 113 renderer, 14 ordinary host, 17 all-feature host and all
checks/builds/Clippy. Final four focused renderer tests cover lazy opening, exact
nonsecret saves, current unsaved preview including typed comma-separated rules,
local errors, draft preservation and focus return. Final type/lint/format/frontend
build and release rebuild ran after that small rules-input fix. Initial renderer
failures were incorrect mock response envelopes; fixed rather than weakening tests.
Initial strict Clippy identified the large effective-response enum variant; boxed
that variant. Failed logs remain alongside final logs.

Application integration proves an unreachable configured endpoint does not impede
manual reads. Corrupt AI pointer data then survived unchanged across actual Core
startup, workspace/inventory/refresh/manual-state loading. Repository bytes exact.
Storage tests cover external edits/stale refusal, readable external prompts, previous
and abandoned generations, symlink roots/pointers/prompts, hardlinks and oversize.
This is selected foundation acceptance, not full M6/M7 or D-048 inference acceptance.

Native ordinary artifact: initial empty settings; saved one synthetic loopback-port-1
connection, unavailable model profile and application assignment without contacting a
provider. Effective inspection showed exact destination/model/template/system text.
Unsaved subject limit 48 produced a deterministic static sample; persisted limit
remained 72. Escape returned focus to AI settings; the existing manual draft stayed
exact. Cmd+Q and exact-artifact restart/Quit exited zero. Restart effective inspection
showed the saved profile without generation or write replay. Three settings generations
correspond to the three explicit saves. Native text entry autocorrect capitalized
the synthetic model ID to `Fixture-model-unavailable`; that exact value is recorded.
No real model installation or inference success is claimed.

[Evidence](../validation/g09-ai-settings-storage-2026-11-08/) includes exact source/
artifact manifests, settings hashes and preservation oracle. All repository files/
modes, execution journals, prior trust (parent revoked) and retained version original
remain exact; SQLite integrity ok, exclusive writer lock reopened. The first shell
settings-proof attempt lacked Python tomllib; equivalent bounded literal fixture
assertions then passed, while the real Rust loader was independently exercised by
the UI and automated tests. No packages were installed for that helper.

Next: approved provider transport/secure auth, responsive independent job scheduling,
exact screened context and assisted candidates, then bounded automatic authority.
G09 remains open. G10 must protect recovery artifacts and retained settings generations;
G11 exact release/default writes remain gated.
