# Commit Command completion decisions

These decisions are inputs to [the implementation runbook](COMPLETION_RUNBOOK.md). Every entry starts **pending**. Suggested values below make the choice concrete; they are engineering proposals, not approved requirements or claims that the current app meets them. An implementer must record the selected values, owner authorization and date before exposing dependent behavior. Independently useful code/tests can proceed against explicit injected policies while runtime defaults remain unchanged.

Do not ask the owner to decide implementation details already specified by the runbook. Resolve only the product/support/authority choices below. Preserve approved decisions D-040–D-048 and existing accepted write support restrictions.

## Decision record format

For each DCP entry replace `Pending` with the actual selected disposition and record:

```text
Status: Pending
Decision owner:
Authorization and date:
Selected values:
Differences from the proposal and reason:
Affected source and case IDs:
Unresolved fields that still block execution:
```

A proposal is rejected or superseded explicitly; it is not deleted from history. A partial choice unlocks only the tasks whose required fields are complete.

## DCP01 Measured performance and support sizes

**Status: Pending.** Product owner accepts final values after P03 measurements.

Proposed target envelope for evaluation, not a current capacity claim:

| Metric | Proposed acceptance target | Measurement |
|---|---|---|
| First usable Overview | <=2 seconds after first partial inventory is available | Main-thread ready marker to usable selected rows, excluding initial OS launch |
| Bounded scan slice | Keep current 15-second cap and report progress after each slice | Traversal plus inspection phase timings, no deadline removal |
| Complete 100 repositories | <=15 seconds warm | Release fixture, exact complete count, >=3 repetitions |
| Complete 1,000 repositories | <=120 seconds warm with useful partial progress | Full generation time, never same-prefix starvation |
| 10,000 repositories | Stress/partial-progress test until separately accepted as support size | No full-support claim from a partial result |
| Targeted status refresh | <=2 seconds warm on fixture repo <=1,000 tracked files | No whole-workspace scan; measure event to UI update |
| UI filter/sort/navigation | <=250 ms on accepted supported inventory | Input to next usable frame, steady state |
| Cancel acknowledgement | <=500 ms; graceful owned process disappearance <=2 seconds | Actual host/client phases and process inspection |
| Main close/Quit | <=5 seconds for bounded cancellable fixture operations, explicit fallback if saving fails | Native action to host exit/lock reopen |
| Total resident memory | <=512 MiB for 1,000-repo representative accepted workload | Host plus renderer, state the OS/tool calculation |
| Single ordinary IPC payload | <=existing request/result safety caps; page rather than grow caps for large inventory | Record serialized bytes and highest observed value |

Choose the reference machine/minimum supported hardware, representative history size/file count, first-pass versus warm policy and whether target limits change. If rejected by measurements, document the revised support scope or implementation work rather than increasing safety limits without analysis.

Outputs: exact supported inventory/history sizes; hardware/OS/build/architectures; each numeric limit; measure boundary; repetition policy; explicit 10,000-repo disposition. Cases PERF01–PERF06 and SCAN01–SCAN12.

## DCP02 macOS distribution identity and tools

**Status: Pending.** Product owner selects each field before P16 packaging.

Choose and record:

- Minimum supported macOS version and tested versions. Current native evidence on macOS 27.2 arm64 does not establish a minimum.
- arm64-only first release versus arm64+x86_64. Proposed smallest scope: arm64-only until Intel native evidence exists.
- Local APFS write envelope and explicit exclusions for network/removable/other filesystem cases. Preserve current accepted envelope unless new evidence supports expansion.
- Git delivery: bundled pinned Git versus explicitly installed Git >=2.56.0 prerequisite. Proposed first candidate uses an explicit supported prerequisite with clear startup diagnostics; bundling requires complete helper/notice/tool provenance work.
- Installer form/channel: signed/notarized `.dmg` containing `.app`, or another explicitly chosen package. No silent updater or login service.
- Keep `dev.gitfleet.desktop` and existing data path versus select a final identifier/path. Proposed least migration work: retain current identifiers unless product policy requires a change; visible Commit Command branding already exists.
- Signing/notarization team identity and authorized credential access procedure. Never write actual credentials to this document.
- Manual-update policy versus a separately scoped updater. Proposed first release: manual install/update.

Outputs: all fields, migration/coexistence behavior if identifier changes, and authorization for signing-service use when ready. Cases RELEASE01–RELEASE05.

## DCP03 Backup format defaults and restore policy

**Status: Pending.** Product owner approves user behavior; engineer finalizes the bounded format and tests before accepting files.

Concrete proposed v1:

- ZIP container with only stored, unencrypted regular-file members. Reject compressed/encrypted/link/special members in v1. This avoids needing to support arbitrary archive features. Use a maintained bounded archive implementation after checking compatibility; do not implement ad hoc ZIP parsing.
- Root `manifest.json`; content members under `config/`, `ai/`, `state/` and optional `recovery/`. SQLite snapshot at `state/state.sqlite3`. Do not embed arbitrary original absolute paths. Manifest lists every file, byte size and SHA-256; unknown members rejected.
- Archive limits: manifest <=1 MiB; <=10,000 files; per file <=256 MiB; total content <=1 GiB; path UTF-8 length <=1,024 bytes and <=32 components. Limits are checked against bytes actually read, not declarations alone. Oversized app state fails with an actionable message; never truncates recovery data to fit.
- AI-only portable exports: profiles/connections without auth/endpoint approval, templates/instructions and disabled Action definitions. Unresolvable workspace/repo assignments require explicit preview mapping or remain unassigned.
- Full backup: settings/workspaces/navigation plus SQLite snapshot. Operation history selectable for manual backup. Recovery artifact policy must be explicit per artifact type; unresolved artifacts remain in live state and are included inertly when backup claims recovery continuity.
- Automatic backup: a logical revision change schedules after 60 seconds idle; maximum 15 minutes since first pending change; only while app runs. No backup for unchanged state. Defaults proposed as last 10 successful automatic backups; manual/safety backups never automatically rotated.
- Proposed history retention: 90 days for eligible terminal unreferenced history; unresolved/in-flight/uncertain/referenced rows and artifacts pinned indefinitely until resolved. The user can disable pruning. No deletion before policy and tests are accepted.
- Backup location: private application-controlled location outside managed repositories, with explicit user choice for manual exports. Keep backup safety generations independent from active files. A backup beside the data directory is local recovery, not protection against device loss; UI should accurately identify its location.
- Before full restore: exclusive ownership, verified safety backup and no current unresolved recovery/active jobs. Proposed first version refuses merging into unresolved live recovery; it does not invent a merge strategy.
- Import/restore approval: harmless preview needs no approval. Full state replacement and authority-sensitive configuration application use an explicit concrete confirmation; reusable trust/credentials/capabilities are always reset regardless of confirmation.
- Encryption: no app-provided encrypted archive in v1; clearly disclose archive contents at export and rely on user-selected protected storage if needed. If product requires portable encryption, scope/key management and adversarial tests must be designed first.

Outputs: selected format/limits/timing/location/history/artifact rules, precise confirmation flow, encryption disposition and disk-space check policy. Cases BACKUP/IMPORT/RESTORE/RETAIN. If stored-only ZIP is selected, bomb tests assert compressed input is rejected before decompression; other size/path/count attacks still run.

## DCP04 Bounded automatic commit authority

**Status: Pending.** Product owner authorizes the exact user-visible authority scope before exposure.

Proposed first implementation:

- One active explicit run, sequential commits, at most 20 explicitly selected worktrees; no inherited selection or trust.
- Native run capability expires after 10 minutes, is session/epoch-bound and single-run, never serialized/restored. Total wall time includes queueing/analysis. Existing underlying exact-operation approvals can be shorter; never extend them automatically.
- At most 32 inference requests, 16 MiB aggregate serialized request bodies and 16 MiB aggregate response bytes per run; each request retains stricter provider-specific existing bounds/deadlines. Checked accounting includes failed requests and hierarchical stages.
- One exact selected path/source set per worktree; all current capture safety bounds remain unless deliberately changed with tests. Only explicitly previewed frozen SemVer result allowed; no AI-selected arbitrary Action/command.
- Candidate rank is validated provider order; first valid candidate with complete required coverage wins. No confidence score, model rationale or unsupported semantic claim can override hard validation. If product needs a different ranking/evaluation standard, define it before automatic use.
- Required selected-change coverage complete at capture and all hierarchy stages; redacted/omitted/failed required content blocks automatic execution. Related optional context omissions visible but do not replace required selected-change coverage.
- Human approval explicitly covers selected endpoint/auth/profile/template, exact initial source scope, all-stage derived summaries within caps, Action and hook/signing policy. Alternatively use individual stage consent; do not reinterpret existing one-request consent as all-stage approval.
- Changed parent gitlink stops for fresh parent approval. No automatic retry/fallback/push. A stopped run needs a new unfinished-only run review.

Outputs: exact caps/lifetime/coverage/ranking/endpoint consent/Action behavior and native review wording, plus approved failure/retry policy. Cases AUTO01–AUTO20. Semantic suitability must be evaluated with P09's corpus before a release support claim; structural validation alone is insufficient.

## DCP05 Hierarchical analysis bounds

**Status: Pending.** Engineer finalizes deterministic algorithm; product owner approves automatic eligibility consequences together with DCP04.

Proposed plan: path/source/hunk stable grouping with no more than 8 leaf groups per repository, at most 2 reduction levels and 16 requests per repository, all within global run limits where present. Every exact body fits existing provider request bounds and resolved profile context policy. Output summaries retain original chunk IDs; reductions preserve transitive IDs. Use no hidden extra retry, embedding service or model fallback.

Choose exact per-stage summary schema/byte bound, group allocation, optional context allocation, reduction fan-in and handling of a chunk that cannot fit. Proposed response text cap: 16 KiB per leaf/reduction summary; larger output rejected. Unknown model tokenization uses labeled estimates and hard byte bounds. Selected-diff incomplete coverage is automatic-ineligible. Assisted output may be shown with explicit omissions but never labeled complete.

Outputs: pure deterministic grouping contract, schema version, numerical caps, exact accounting and full-coverage predicate. Cases HIER01–HIER14.

## DCP06 Action definition and verified version binding

**Status: Pending for stored definitions/integration.** Existing explicit manual SemVer syntax/invocation/recovery must remain unchanged.

Proposed definition: version 1, stable UUID/revision, workspace+worktree identity, exact native-relative path, allowed major/minor/patch values, existing explicit prerelease policy, enabled flag and local approval metadata. Edit/import changes revision; import resets enabled/approval. No custom script or broader file format.

Proposed template integration: typed optional `version` preview/result value, derived only from deterministic parser/executor, displayed as `preview` versus `applied`. If a template requires an unavailable value, fail validation explicitly; never send a guessed value or silently replace with empty text. Keep manual Action execution, stage and commit approvals separate; only a specifically approved automatic contract can compose them without repeated prompts.

Outputs: definition ownership/versioning, approval/disable semantics, available increment policy and template-variable syntax with parser/validation tests. Cases ACT01–ACT12, IMPORT06, AUTO11/AUTO12.

## DCP07 Supported Git authentication and hook envelope

**Status: Pending reconciliation of remaining native criteria; no expansion selected.** Existing accepted M2/local and remote exclusions remain.

Produce one explicit table from actual contracts with columns: configuration, read inspection, write/Execute/bypass permission, formats/topologies, Core evidence, native evidence, exclusion/refusal case, release disposition. Include every supported helper/signer, files/ref backend/index type, worktree/submodule case and graceful-versus-hard-crash cleanup guarantee. Record exact operator procedure for disposable secure-store prompts and direct SCP facility block.

Do not add installed hook-manager Execute support, interactive signer fallback, new SSH identities/transports or advanced indexes merely to eliminate a block. Supported-envelope changes need specific implementation and acceptance, not wording alone.

Outputs: table and operator-required cases, or owner-approved exclusions for remaining required blocked criteria. Cases REMOTE01–REMOTE14 and relevant SEC/LIFE.

## DCP08 Accessibility and keyboard behavior

**Status: Pending acceptance details.** Basic keyboard names/focus/error requirements already apply.

Proposed baseline: all workflows keyboard-complete; native VoiceOver on supported macOS; text contrast >=4.5:1 normal and >=3:1 large; meaningful nontext/focus contrast >=3:1; minimum accepted layout at current native enforced minimum; 100%, 150% and 200% text/zoom where the runtime supports it. Confirm exact window dimensions against `window_state.rs` rather than inventing a new minimum. No required action offscreen without usable scrolling.

Shortcut proposal: preserve OS-standard close/Quit/Escape and existing app bindings. Document a collision-free app shortcut table before adding new bindings. Return on initially focused Cancel must cancel, not approve. Screen-reader alternatives required for graphs/heatmaps; errors/progress announced without stealing focus unexpectedly.

Outputs: accepted standard/thresholds, keyboard map, zoom/runtime method and minimum geometry. Cases UI08–UI12 and RELEASE11.

## DCP09 Distribution licenses

**Status: Pending product/legal decision.** Repository MIT files and dependency declarations are inputs, not a full release decision.

Owner must confirm product license/distribution policy and whether any third-party obligation requires legal review/source offers. Engineer derives actual shipping closure, collects license texts, fixes first-party metadata only to the selected policy and packages offline notices. No inferred third-party license when metadata is missing.

Outputs: approved policy, notices location and disposition of each shipping obligation. Cases LICENSE01–LICENSE04.
