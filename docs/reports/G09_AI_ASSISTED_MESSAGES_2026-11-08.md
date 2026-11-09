# G09 exact source review and assisted messages — November 8, 2026

Disposition: implemented; selected automated and native fixture acceptance passed.
G09 remains open for hierarchical model analysis, bounded automatic authority and
broader real-provider/TLS/usability acceptance. This checkpoint grants no Git writes.

The commit composer now offers optional AI message suggestions for explicitly selected
index/working-tree/untracked paths. Capture is independent of AI setup or trust and
shows the exact filtered context, snapshot fingerprint, line accounting, omissions
and warnings in a keyboard-dismissible modal. Related source/docs, exclusions,
history counts and byte budget are explicit. Changing inputs invalidates the preview.
Profiles resolve through the saved hierarchy or a deliberate one-off choice; additive
instructions do not broaden authority. Configuration containing recognizable credentials
or unsafe control text is refused for source inference before transfer.

Each source inference re-captures the selection, compares raw/filtered fingerprints,
resolves current settings, and freezes an exact provider plan. The authoritative
review identifies source transfer, endpoint/model/auth, profile/template revisions,
coverage and actual request payload. Native approval is one-use. Settings and source
are revalidated at confirmation; decline/drift sends nothing. The filtered context
is embedded as its exact original JSON string, so its fingerprint matches the text
actually received by the provider. Source-bearing request data/results remain
ephemeral; no source capture is serialized into application state or operation journals.
The existing bounded jobs/credentials/transport enforce limits, cancellation, shutdown,
redirect refusal and no model/auth fallback. Known bearer material reflected in decoded
provider output is refused before results reach the renderer.

Rust accepts one to three structured suggestions with subject/body, rationale, summary,
concerns and known chunk IDs. Unknown fields/commands, duplicate messages or invented
provenance are refused. Template violations are displayed as invalid and cannot be
applied through the UI. A deliberate button replaces only the draft; generation cannot
stage, commit, execute an Action, select extra paths or push. Semantic accuracy and
meaningful differences still require human review. Coverage IDs are model reports,
not independent proof of analysis; no confidence score is invented. Partial/redacted
coverage can support a clearly warned manual suggestion, never automatic authority.

## Native finding and correction

The first ordinary artifact correctly showed 1,024-byte omissions and invalidated
its preview on budget changes. Native decline sent zero HTTP, but the parent view's
completion revision remounted the suggestions panel and lost its context/results.
`panel-lifetime-before.txt` reproduces that failure. The panel now keeps state across
review completion and is scoped to repository/selection identity. Core still checks
fresh source before any transfer; displayed suggestions explicitly describe their
captured snapshot. The new App regression test proves the modal/context/draft survive
a completion revision with no recapture, generation replay or automatic draft change.
First source/artifact manifests are retained separately from the corrected build.

## Validation and native sequence

Evidence: [assisted-message artifacts](../validation/g09-ai-assisted-messages-2026-11-08/).
Final application-services library 61 passed, application integration 41 passed,
strict all-feature/all-target Core Clippy passed. Complete desktop runner passed:
123 renderer tests, ordinary host 20 passed/one ignored worker, all-feature host
23 passed/one ignored worker, contracts/typecheck/lint/format/build/Clippy/release.
After a settings-hint text correction, renderer/typecheck/lint/format/build and the
ordinary release binary passed again. The final native artifact uses those sources.
The ignored worker is invoked explicitly by parent Keychain tests.

Actual owned HTTP fixtures cover both provider adapters: exact byte-preserved context,
pre-approval zero traffic, native-capability decline, source drift, missing assignment,
partial/redacted coverage, valid versus invalid messages and unchanged repository bytes.
Pure parser tests cover command injection, duplicates, bad provenance and unsafe controls.
The existing cancellation/concurrency tests exercise the shared bounded transport.
Two initial complete-library runs at unrestricted test parallelism failed timing
assumptions while queued calls consumed the global process permit. Their logs remain.
The held overlap fixture now outlives its two-second observation window; final runs
use the repository validation script's existing four-thread setting. Production limits
and cancellation checks were not relaxed. The complete five-configuration write/signing
matrix subsequently completed; see the separately recorded [full regression](G02_G11_AI_SOURCE_REGRESSION_2026-11-08.md).

Native ordinary builds used only the previously authorized disposable three-level
workspace; no trust grants or real credentials were requested:

1. Native settings changed only the nonsecret offline-fixture connection endpoint to
   the owned loopback provider. Existing profiles/assignments/older generations remain.
2. The corrected artifact captured the dirty leaf `file.txt` with complete accounting.
   Native decline left the context panel intact and sent zero HTTP.
3. A fresh exact approval sent one POST `/api/chat`, 3,304 request bytes, five chunks.
   Provider-recorded context SHA-256 matched the viewer's
   `ea156f56007fa914d34966594b80d297def7e9df5031360ef68ed21fe490d7c8`.
4. A template-valid suggestion appeared with rationale/summary, model-reported chunks,
   warnings, 42 input/12 output fixture tokens and latency. Selecting it changed only
   the draft. Ordinary stage/commit/trust controls remained disabled. The original
   `G09 native reviewed version Action` draft was restored through the UI.
5. Local refresh and Quit succeeded. Exact artifact restart preserved the original
   draft; fresh read-only capture plus result polling showed no saved generation job
   or candidate. Escape returned focus to the suggestions trigger. Second Quit exited
   zero; HTTP count remained one. The owned server was stopped by its recorded PID.
6. `verify-native.py` passed: repository bytes/modes, execution journals, trust (parent
   still revoked), retained original, older AI generations, source/binary hashes,
   SQLite integrity and writer-lock reopening. Provider metadata stores hashes/counts,
   not source bodies or credentials. No repository or publication operation occurred.

The mock's model name is deliberately `Fixture-model-unavailable`; this proves the
adapter/native boundaries, not real inference or that such a model is installed.
The newly supplied Ollama endpoint/model will receive separate real-provider synthetic
acceptance. No repository content has been sent to that endpoint by this checkpoint.
