# Commit Command — current implementation and acceptance record

Updated November 14, 2026. Current disposition incorporates the dated reports and
validated checkpoints below; the original reconciliation baseline was `db29ba6`. This is the single maintained record of **current implementation,
acceptance gaps and next work**. The specification and decision register define
requirements; the crosswalk routes requirement IDs to code/tests; dated reports
and validation artifacts preserve evidence. Older status statements do not override
this record. Reconciliation does not accept new gates or change product scope.

The [detailed completion handoff](implementation/README.md) supplies the executable
remaining-work packages, decision worksheet, fixture/test procedures and complete
requirement mapping. It does not accept gates or replace this live evidence ledger.

### November 14 · F22 state directory aliases

The disposable CLI reproduction returned `invalid_path` yet created
`repo/subdir/new/state` through an alias to `repo/subdir`. The regression failed
on those new directories before the fix. Startup now canonicalizes the nearest
existing ancestor and checks its checkout ancestry before creating descendants;
the final canonical check remains in place. Canonicalization errors other than
missing descendants fail before creation.

All 24 workspace tests passed, including alias/direct paths, parent components,
valid external aliases and unchanged repository bytes. A rebuilt CLI independently
refused the same alias request without creating descendants. Default all-targets
Clippy, formatting and diff checks passed.

### November 14 · F24/F25 desktop authority and validation matrix

Reproduced the documentation mismatch against the manifest: default builds select
`desktop-writes`; the entry READMEs and a later current-disposition paragraph still
promised read-only operation. Current docs now describe the default and the explicit
bundled read-only invocation. Historical dated reports retain their attribution.

The host regression submits one eligible Stage through the generated IPC handler
and real worker in a disposable repository. Default capabilities report writes enabled
and the index receives the working bytes. A no-default-features/custom-protocol host
reports writes disabled and rejects Stage before queuing it. Both preserve HEAD and
the working tree. Settings and an unauthorized caller reject the same request for
their respective role/ACL reasons; Settings capabilities provide a positive control.

Validation now builds, tests and lints the bundled read-only host alongside default
and all-feature hosts. Native acceptance menu probes use live Settings/main commands
and the existing Settings page. Positive-control failures and unknown commands cannot
count as denial passes. The generated probe JavaScript was exercised in a Node VM
against expected denial, missing-command, failed-positive-control and unexpected-success
outcomes. Live native menu interaction was not repeated in this milestone.

Frontend 254 tests, typecheck, lint, contract check, format check and production build
passed. Host suites passed: default 28, all features 31, read-only 23 (two intentional
ignored tests per configuration). Format, Clippy in all three configurations, explicit
read-only build and default release build passed. A duplicate fixture-module Clippy
failure was corrected by sharing the import; affected gate/ACL tests were rerun.
Private Keychain tests require permitted local access; the sandboxed attempt failed
fixture creation, while the permitted runs passed. The existing F01–F21 fixes and
product UI changes are included in this checkpoint rather than left uncommitted.

### November 14 · F21 empty model option

The rendered Settings regression reproduced an empty dropdown while the saved
profile still contained the previously chosen model. The empty option is now a
disabled placeholder. A configured model remains selected until the user chooses
another installed model. First-time connection-only setup remains supported.
[Model selector](../app/desktop/src/views/SingleAiSetup.tsx)

The UI test selects and saves a model, attempts the placeholder, flushes and
reopens Settings, and confirms the visible and saved models agree. Core
verification saves a connection before its first model, prepares provider
discovery, saves a model, reopens the application state, and verifies both the
effective configuration and the prepared Fill payload use that saved model.
All fourteen SingleAiSetup tests passed, together with the Core regression,
frontend production build, typecheck and lint.
[UI regression](../app/desktop/src/views/SingleAiSetup.test.tsx) ·
[Persistence and Fill regression](../app/core/tests/m3_application.rs)

### November 14 · F20 completed AI jobs and request admission

A loopback-provider regression completed 32 fast jobs, then reproduced a Busy
error on request 33 despite zero active provider work.

Admission now evicts the oldest finished result when the bounded 32-entry
registry is full. Unfinished workers retain their slots even when their displayed
phase is Cancelled. Finished-result retention is measured from completion, so a
long-running request can still expose its newly completed result.
[Registry implementation](../app/core/src/ai/jobs.rs)

Forty sequential loopback requests completed in the provider regression and
another forty completed through the desktop provider-test request path. Each
latest result was retrievable immediately, retained entries stayed at or below
32, and no ten-minute wait was needed. The AI suite passed all 66 tests, including
active global/connection concurrency and cancellation checks. Core Clippy passed.
[Provider regression](../app/core/src/ai/provider/tests.rs) ·
[Desktop regression](../app/desktop/src-tauri/src/ai_transfer_tests.rs)

### November 14 · F19 final AI Settings edit on application Quit

Native reproduction used the ordinary development bundle with isolated app data
and a synthetic loopback model-list provider. Editing Temperature from 0.2 to 0.3
and immediately pressing Command-Q exited with 0.2 still persisted.

Quit now requests a one-use Settings flush and waits for its result alongside
the main presentation save. Only the Settings caller's AI saves are admitted
during that handshake. In-flight settings saves are allowed to finish, and the
form prevents further edits while closing. Failure uses the existing save-failure
dialog. Cancelling Quit restores interaction.
[Shutdown handshake](../app/desktop/src-tauri/src/shutdown.rs) ·
[Settings listener](../app/desktop/src/settings.tsx)

The rebuilt bundle saved 0.4 on immediate Command-Q and displayed it after
relaunch. Ordinary Settings close saved 0.5. An edit left past debounce saved 0.6.
Making only the isolated AI directory unwritable and immediately quitting after
editing 0.7 displayed the native save-failure dialog. Keep app open retained 0.7
in the editable form while disk still contained 0.6. Restoring write access and
retrying Quit saved 0.7 before exit.

Bridge/form tests passed for flush ordering and failure reporting. Native tests
cover caller scope, pending-close admission, token generation and one-use
acknowledgements. Frontend build/lint and desktop Clippy passed.

### November 12 · F18 composer ownership during delayed callbacks

Rendered tests reproduced both failures. A delayed successful commit of A erased
a newer draft B. An AI stream callback restored its captured navigation snapshot,
reopening a sidebar hidden after generation began.

Draft updates now use functional worktree-scoped changes and advance a revision.
A successful commit clears its draft only if that revision and workspace context
still match. Each AI generation receives a scoped updater. A manual edit takes
ownership, causing subsequent chunks to be rejected and the obsolete job to be
cancelled. Clearing a committed draft also advances its revision.
[Composer implementation](../app/desktop/src/views/App.tsx) ·
[AI stream implementation](../app/desktop/src/views/AiCommitFill.tsx)

Tests with delayed mock responses verify untouched-draft clearing, preservation
of B and A-to-B-to-A edits through presentation save, navigation during streaming,
manual ownership, a newer generation, workspace replacement, cancellation and
retention of partial text on failure. All 64 App and AI Fill tests passed, along
with the production frontend build and lint checks.
[Composer tests](../app/desktop/src/views/App.test.tsx) ·
[AI Fill tests](../app/desktop/src/views/AiCommitFill.test.tsx)

### November 12 · F17 batched history and activity commit reads

In a disposable 500-commit repository, the original history page and activity
interval started 1,000 object-reader Git processes combined. The requests took
6.51 and 6.43 seconds in the fixture. Batching reduced this to sixteen processes
combined, with measured requests around 0.30 and 0.25 seconds. Timings use debug
builds and include the recording Git wrapper.

Both readers now request groups of up to 64 objects through `cat-file --batch`.
The existing commit parser handles each length-framed body. The reader validates
returned IDs, object types, lengths, delimiters and trailing data. Each group uses
the existing bounded process runner and remaining inspection deadline.
[History implementation](../app/core/src/history.rs) ·
[Activity implementation](../app/core/src/activity.rs)

Tests cover ordered output, identity/message equivalence with individual reads,
merge parents, paging, SHA-256 IDs, activity boundaries and completeness, missing
objects, output limits, and cancellation of a running batch child. Repository
bytes remained unchanged. The workspace activity regression covers linked
worktree deduplication through the application request workflow.
[Process-count regression](../app/core/tests/history_batch.rs)

### November 12 · F16 slow discovery units and queued siblings

A disposable workspace contained an always-slow repository followed by a healthy
sibling. A Git wrapper delayed commands in the slow repository beyond the entire
two-second scan slice. Before the fix, the initial scan and two manual
continuations all retried the slow repository and the healthy sibling remained
absent.

A timed-out directory now moves behind the other queued directories. Cancellation
retains the original pending position. The timeout report records the deferred
directory's path, and progress remains incomplete while that unit is outstanding.
[Cursor implementation](../app/core/src/discovery/cursor.rs)

The regression exercises both Continue Scan requests and scheduled monitor
reconciliation. Both discovered the healthy sibling while reporting the slow
repository's timeout and pending progress. After releasing the delay, continuation
completed, retained the healthy identity, and matched a fresh scan's repository
roots. Cancellation, automatic retry scheduling and cursor invalidation tests
passed. Core Clippy and the desktop build check passed.
[Application regression](../app/core/src/application/monitor.rs)

### November 12 · F15 workspace monitoring inventory work

Disposable 20- and 40-repository fixtures compared the original per-entry refresh
loop with a monitoring pass. Test counters measured inventory loads and decoded
rows. The original loop performed 21 loads / 420 rows and 41 loads / 1,640 rows.
The fixed pass performed one load / 20 rows and one load / 40 rows.

Inventory now joins supersession data in its main query. Monitoring retains the
pass's entries and observed heads, updates observations after refresh, and refreshes
one worktree per worker turn. Workspace invalidation and monitor restart discard
the pending pass. Targeted foreground refresh uses the same refresh implementation.
[Workspace refresh](../app/core/src/workspace.rs) ·
[Monitoring scheduler](../app/core/src/application/mod.rs)

The scale test completed foreground requests between every refresh and observed
the changed status in every repository after the pass. External-edit notification,
parent/child Gitlink and historical-identity regressions passed. Core Clippy and
the default-feature build check passed.
[Scale regression](../app/core/src/application/monitor.rs)

### November 12 · F14 repository-wide unstaged diff memory

On a disposable repository of changed 2 MiB binary files, the baseline CLI's peak
resident memory measured 47,136,768 bytes for 16 files and 283,787,264 bytes for
128 files. Patch output was 1,744 and 13,952 bytes respectively. After retaining
only paths and streaming fingerprints, peak resident memory measured 12,517,376
and 12,632,064 bytes. Measurements used macOS `time -l` and debug CLI builds.

The serialized mode, file metadata, patch bytes and completeness flag matched
the baseline at both sizes. Fingerprints preserve the existing kind/content
SHA-256 contract for regular files, symlinks and missing files. A 64 KiB buffer
checks cancellation and the remaining request deadline during hashing. The
existing per-file limit and request deadline bound aggregate work.

Tests exercise the CLI workflow, a file changed while Git builds the patch,
active hashing cancellation, deadline expiry, size limits and preservation of
Git state during successful inspection.
[Diff implementation](../app/core/src/diff.rs) ·
[CLI consistency regression](../app/core/tests/diff_consistency.rs)

### November 12 · F13 interrupted scratch and loose-object promotion

A rejected no-change proposal left an unjournaled scratch directory. A transfer
limit reached after the first 64 KiB left a partial file in the Git object
fanout. Both failures were reproduced in disposable repositories.

Preparation and promotion now persist an ownership receipt before writing
payloads. Cleanup checks the recorded filesystem identities. The first durable
proposal and the release of its scratch receipt share a SQLite transaction.
Reopening the writer cleans interrupted work that still matches its receipt.
Unreceipted artifacts from older runs are left alone, as is an empty entry if a
process dies between creating it and recording its identity.
[Ownership and restart cleanup](../app/core/src/writes/temporary.rs)

Regression fixtures cover no-change rejection, cancellation, injected write and
sync failures, process death during preparation and process exit after publishing
an object's final hard link. The foreign directory, foreign fanout file, exact-name
replacement and final object survived. A separate operation's recovery evidence
remained byte-identical and its recovery and the subsequent Stage completed.
[Preparation and recovery tests](../app/core/tests/m2_writes.rs) ·
[Promotion tests](../app/core/src/writes/transaction.rs)

### November 12 · F09 Open Folder in Finder

The original Open Folder invocation launched a disposable application bundle
containing a Git repository. Its executable appended a line to a launch counter.
The desktop now sends workspace and worktree IDs. The host resolves the active
workspace or validates the registered worktree before invoking Finder. Ordinary
folders use Finder's directory viewer. Packages and explicit Reveal actions use
macOS's reveal operation. Copy Path still reads the registered path of a missing
worktree.

The native regression inspected Finder itself. Its front window targeted the
selected ordinary repository. For a workspace rooted at an applet, Finder showed
the parent folder with the applet selected, and the applet's launch marker was
absent. Core tests cover unknown and inactive IDs, a registered symlink alias,
a missing worktree and a canonical root replaced by a symlink.
[Finder action and native regression](../app/desktop/src-tauri/src/repository_folder.rs)

### November 12 · F08 Settings and provider transfer approval

Before the fix, the Tauri Settings dispatcher accepted a valid editor-test
request for unsaved loopback provider B while provider A was saved. B received
the selected file's sentinel from a disposable repository.

Ordinary Settings now has a configuration and model-discovery allowlist. The
saved-profile suggestions window has its own host-assigned role, and all desktop
roles reject source transfers using an editor-supplied connection. Saved-profile
Fill asks for native approval for each frozen source request. The dialog shows the
destination, model, included sources and request size.

The desktop IPC regression observed only a source-free model-list GET at B.
Declining the decision sent nothing to A. Editing the selected file during
approval returned `StaleReview` and also sent nothing. A fresh, accepted Fill
sent the selected change once to A and completed. The test uses real loopback
HTTP and supplies the decision at the native dialog boundary.
[Dispatcher and provider regression](../app/desktop/src-tauri/src/ai_transfer_tests.rs)

### November 12 · F07 private-key redaction in selected diffs

A disposable repository contained a synthetic PEM block in `fixture.txt`. Editing
one interior line left both delimiters outside the three-line diff context. Before
the fix, a loopback Ollama recorder received key-body text in the approved request.
Capture now identifies protected lines in both complete file versions before
forming the diff. It carries that classification into the selected diff and
full-content fragments, including previously protected lines retained after
delimiter removal. [Capture implementation](../app/core/src/ai/capture.rs)

The regression inspected eight HTTP request bodies across Ollama and the
OpenAI-compatible protocol, saved-profile and editor-test requests, and 2 KiB and
16 KiB context budgets. Every body contained the ordinary edit and excluded all
synthetic key lines and delimiters. Accounting recorded eight redacted diff lines
and eleven redacted full-content lines. The smaller budget omitted lines and the
larger budget included all lines after screening.
[Provider regression](../app/core/tests/m3_application.rs)

### November 12 · F06 reviewed bytes and staged blobs

In a disposable repository, a Git wrapper changed a selected file from A to B
immediately before `hash-object` read it, then restored A before revalidation.
Before the fix the operation recorded A's mode/content SHA-256 but staged B
(`5663091be8ca`). With the fix, the same interleaving staged A (`c86626638e0b`).

Preparation streams each selected regular file into a private, unlinked scratch
file while hashing the same chunks. Git receives the rewound descriptor. Later
checks still hash the live working file. The scratch copy exists only during
materialization and uses temporary disk space proportional to one file at a time.
[Implementation](../app/core/src/writes/transaction.rs)

Stage and Commit both accepted A after a temporary edit, and both refused a
persistent B with `StaleReview`, preserving HEAD, the index and the user's edit.
The restored-content case also passed with a 17 MiB binary. In a separate 18 MiB
fixture, cancellation while the object command was active preserved HEAD, the
original index bytes and the concurrent working edit.
[Controlled-write regression](../app/core/tests/m2_writes.rs)

### November 12 · F05 desktop startup after hard discovery failure

A disposable workspace with 100,001 child directories in one subtree hit the
production discovery limit. Before the fix, Create reported an error after saving
and activating the definition. Opening that same state failed with the same error.

The desktop now records a failed scan as incomplete, exposes its cause in the
workspace alert and keeps the saved definition available. A hard failure writes
the scan report without updating membership. Database and schema errors still
propagate. The opt-in regression exercises the production directory limit and
checks a previously available worktree's identity and availability after a
failed rescan. [Core regression](../app/core/tests/m3_application.rs)

A macOS acceptance build reopened the original failed state, displayed the error,
switched to a healthy workspace and returned to the affected one. Adding
`generated` to its exclusions completed discovery. On a second launch the repaired
workspace was active, its repository was visible and both original workspace IDs
were intact.

### November 12 · F04 workspace changes during recovery

Before the fix, the CLI removed a disposable workspace with an unconfirmed push.
The workspace list became empty while SQLite retained the publication under the
deleted workspace ID. The shared workspace service now checks the recovery
predicates before it saves a removal, root replacement or changed exclusions.
Name-only edits and administration of other workspaces remain available.
[Scope guard](../app/core/src/workspace/recovery.rs)

Desktop request and CLI tests exercised an interrupted local write, an interrupted
push queue after restart, and an unconfirmed push after server rejection. All
three scope mutations were refused. Local recovery, explicit queue closure and
publication reconciliation against a fresh observation each released the
corresponding guard. Queue closure preserves the interrupted history and leaves
each publication's recovery independent.
[Cross-adapter tests](../app/core/tests/support/scope_recovery.rs)

In a macOS acceptance build, **Remove saved definition** failed while the queue
needed attention, then succeeded after **Close interrupted push queue**. The
default-feature CLI refused all three mutations before closure and accepted all
three on a snapshot taken after closure.

### November 12 · F02 and F03 single-repository selection

The macOS composer previously committed a staged rename with both names in HEAD.
Unstaging that row left neither name in the index. With A in HEAD, B in the
index and C on disk, Stage from the index row overwrote B with C, and Unstage
from the working row overwrote B with A.

The composer now keeps the checked side and both rename paths. Stage uses checked
working-tree rows, Unstage uses checked index rows, and Commit uses the checked
content source for each path. A rename counts as one displayed file. A copy
includes only its destination unless its source is separately selected.
[Selection logic](../app/desktop/src/stores/changeSelection.ts) · [Composer regressions](../app/desktop/src/views/App.test.tsx)

The native acceptance build exercised 21 cases in fresh disposable repositories.
The host recorded 19 write requests and zero for the two disabled actions.
Independent reads of HEAD, index blobs and working files gave these results.

| Checked row and action | HEAD | Index | Working tree |
|---|---|---|---|
| Index row, Stage (disabled) or working row, Unstage (disabled) | A | B | C |
| Working row or both rows, Stage | A | C | C |
| Index row or both rows, Unstage | A | A | C |
| Index row, Commit | B | B | C |
| Working row or both rows, Commit | C | C | C |

Committing the staged rename left only the new name in HEAD and the index.
Unstaging it restored the old name in the index and kept the new file on disk.
Staging the resulting deletion and addition together restored the rename in the
index. Mixed selections, Select all, a separately edited copy source, and a
renamed file with further working edits also matched the expected trees and blobs.

### November 12 · F01 bulk commit and push review

Bulk Commit, exact push and guided push items now display the frozen Core plan. The
executing control names the operation and submits the plan ID and operation kind.
Ordinary single-repository commits still execute on the original click.

Before the fix, the desktop worker received the push planner's review request and
sent a receive-pack POST to a disposable HTTPS remote. Its main branch advanced
from `ff16c88` to `78c9dc3`. After the fix, the same fixture recorded zero
receive-pack requests through Review and Cancel, one POST on Push, and no further
update after replay or source drift. [Host regressions](../app/desktop/src-tauri/src/publication_tests.rs)

A macOS acceptance build exercised the displayed controls against a separate
fixture. The selected worktree and bare remote both started at `e33356ce10f1`.
Bulk Review and Cancel preserved both refs. Explicit Commit created
`4ce8570d60a8` with only the selected README, preserving the excluded file and
three unselected worktrees. Push Review and Cancel preserved the remote at
`e33356ce10f1`, then explicit Push advanced it to `4ce8570d60a8`.

### November 11 remove local staging/commit file-size caps

Selected regular files are now fingerprinted in 64 KiB chunks and passed to Git
hash-object through an owned, no-follow regular-file descriptor. The write path
no longer reads complete selected files into a Rust byte vector or rejects files
above 16 MiB. Exact mode/content fingerprints remain compatible with existing
operation records. Local object publication copies and syncs loose objects in
chunks without the former 32 MiB compressed-object cap. Git handles blob creation
and fsck validation. Existing network-import bounds remain separate and unchanged.
Cancellation, process deadlines, symlink checks and content-change revalidation
continue to apply. Display-preview bounds are not used to gate these direct writes.

Validation: a disposable application regression staged, unstaged, staged again
and committed incompressible files of 65,929,398 and 24,754,883 bytes (matching the
reported catalog sizes). Independent git hash-object IDs, staged/committed object
IDs, cat-file sizes, unchanged working-file IDs, clean final status and git fsck
--strict all passed. Runtime: 95.69 seconds. Existing partial/excluded-staging,
binary/symlink/executable/deletion, and hook-driven content-change tests passed;
four bounded subprocess/cleanup/cancellation cases passed. Native debug build
passed. The user's universe repository was not staged or committed by these tests.

### November 11 clear the composer after a successful commit

Successful single-repository commits now clear that repository's draft in the
maintained navigation state. The raw message and structured subject/body share
this value, so both editor modes clear together and the cleared draft is saved.
Failed, uncertain or cancelled commits preserve the draft. Other repositories'
drafts and editor sizing preferences are retained.

Validation: all 39 App renderer cases passed, including raw/structured clearing
only after completion, persistence of the cleared value, and preservation on
blocked/cancelled outcomes. Renderer production build and ESLint passed.

### November 10 completed generation mislabeled invalid

The user's screenshot showed 6,431 input tokens, 14 output tokens, 146.2 tokens/s,
0.4 seconds total duration, an editable generated message and a generic invalid
message error. The exact subject was 76 characters. Current saved settings had
an internal Conventional template with subject_max_chars=72. The streamed answer
was complete but the legacy template checker marked its candidate invalid,
causing the renderer to report failure despite having already displayed the text.

Fill with AI now accepts completed plain-text drafts independently of template
style/subject-length preferences. Resource/text bounds remain enforced; structured
editor diagnostics retain their existing template checks. This path no longer
fabricates rationale/summary metadata solely to pass structured-response rules.

Read-only live /api/ps confirmed gemma4:12b loaded with context_length=16384;
current saved settings showed thinking=false and output_tokens=2048. No live
repository-source request or settings modification was performed. Four candidate
cases passed, including the exact screenshot subject. All 23 provider cases passed;
the delayed-loopback streaming case now uses that same 76-character subject.
Native debug build passed. The debug bundle was updated for the next launch.
Native GUI retry on the user's repository remains unverified.

### November 10 configurable Ollama thinking

Model settings now has a Thinking checkbox, default off, saved automatically
with model parameters. Missing values in older settings deserialize as false.
Ollama receives top-level think=false/true, never options.thinking. Switching
models or using model defaults resets it off. The compatible adapter does not
expose this provider-specific control. The generation statistics box now takes
its own full-width row below Fill with AI.

Live read-only model inspection on 10.10.10.72 reported gemma4:12b thinking
support with default=true. A synthetic streaming request with think=false,
num_ctx=16384 and num_predict=512 returned 'fix: correct typo in README' in seven
stream frames, output count 7, done_reason=stop and no thinking output. No
repository content was sent and no server configuration was changed.

Validation: 23 provider cases passed, including default-off, explicit false/true
request mapping and TOML round trips. All 26 AI setup/composer renderer cases
passed, including automatic toggle saves. Contract generation, renderer build,
ESLint and native debug build passed. Native toggle interaction remains untested.

### November 10 streamed commit messages and generation statistics

Ollama Fill with AI now sends stream=true and requests only a plain-text commit
subject/body. It does not request a JSON candidate schema, rationale, coverage IDs
or concerns. Existing configured model parameters and timeout remain in effect.
The bounded subprocess transport delivers NDJSON incrementally; AI job snapshots
expose accumulated message text and terminal performance metrics. The renderer
updates the composer as text arrives (400 ms polling), preserves partial text on
failure/cancellation, and ignores late updates after cancellation or unmount.

An information box directly below Fill with AI shows total duration, generation
speed in tokens/second, input tokens and output tokens. Duration runs locally
while waiting; completion uses Ollama total_duration. Speed is eval_count divided
by eval_duration in seconds. Unknown metrics display a dash until the server
reports them. Truncated generation remains an error; partial text is retained
and no commit is performed automatically. Non-Ollama adapters retain their
existing response transport and expose available final token counts.

Validation: all 22 provider cases passed, including a delayed loopback response
proving delivery before HTTP completion, fragmented Unicode NDJSON, metrics,
truncation, malformed streams and model changes. All 13 AiCommitFill renderer
cases passed, including live partial updates, final metrics, retained partial
output after a limit and cancellation rejecting late text. Four subprocess
cleanup/limit/cancellation tests and three existing candidate validation tests
passed. Generated contracts, renderer build, ESLint and native debug build passed.
The user server and native GUI streaming were not exercised for this change.
The existing debug app bundle was updated for the next launch.

### November 10 AI connection and configuration tabs

The simplified AI server button is now Connect & Load Models, with explanatory
text describing the live reachability check, model-list retrieval and unlocking
of Model / Model settings. It explicitly distinguishes retrieving the list from
loading an LLM into memory. Model tabs start disabled, unlock only after a
successful response, remain locked after failure, and relock when server type,
address or port changes. Returning to previous server values does not restore
an old successful check. Keyboard navigation respects disabled tabs. Connection
work displays a spinner and Connecting… while repeat clicks are disabled.

Validation: 20 AI setup/provider cases passed; renderer build, ESLint and native
debug build passed. Native interaction in the user's running app was not tested.

### November 10 preference startup race

App-wide preference reads and individual preference updates no longer require a
synchronized workspace epoch. These requests have no repository or workspace
identity, so a new main/settings webview can load them immediately. Workspace
requests retain their existing stale-context validation. This fixes the startup
"Layout preferences could not be loaded: stale_review" banner.

Validation: the focused application regression passed for epoch-zero load/save,
textarea height and timeout persistence after reopening, invalid-value rejection,
continued rejection of stale inventory requests, and unchanged repository bytes.
Native debug build passed; the existing debug bundle was updated for next launch.
The user's running app was not restarted.

### November 10 seed control

Seed uses an integer spinner from 0 to 2,147,483,647 with step 1 and remains
unset by default. A compact SVG dice button between the input and Unset assigns
an unbiased random value over that inclusive range using browser random bytes.
Direct entry and Unset continue to work. All 11 SingleAiSetup cases passed,
including both random endpoints, direct entry and clearing. Renderer build and
ESLint passed. Native visual acceptance remains unverified.

### November 10 AI token controls

Context size now increments by 1,024 tokens and offers Model default plus 4K,
8K, 16K, 32K, 64K, 128K and 256K presets. Custom numeric values remain editable.
Maximum output tokens increments by 256. Clearing either override still omits it
from the provider request. Core accepts the 256K context preset and rejects values
above 262,144. This does not verify that a particular installed model supports
that context length.

Validation: all 10 SingleAiSetup renderer cases passed, including preset/custom
synchronization, blank defaults, unset and numeric step validity. The focused Core
256K boundary test passed. Renderer production build, ESLint and native debug
build passed. Native spinner interaction was not exercised in the running app.

### November 10 explicit Ollama model loading

Fill with AI now starts a source-free model-load job before capturing repository
changes. The job sends the selected saved Ollama model to `/api/generate` with no
prompt, a ten-minute keep-alive and the configured context size if set. Loading
has a separate 120-second timeout and must return `done: true` for the exact model
before the UI captures/sends changes. The existing generation timeout remains
separate. Compatible providers bypass this Ollama-specific operation. Generation
also uses ten-minute keep-alive so it does not immediately shorten residency.

The UI shows Loading model with elapsed time and Cancel, then the existing source
capture/generation progress. Failed/cancelled loading cannot send repository data
or replace the draft. Transport failures now identify model loading, server check
or generation and distinguish timeouts from cancellation and output bounds;
the generic manual-workflows-remain-available message was removed.

Live diagnosis used only model metadata and synthetic requests: the configured
server listed `gemma4:31b`, while `/api/ps` initially had no loaded models. Ordinary
synthetic requests timed out at 30 and 120 seconds. A documented empty preload
subsequently returned HTTP 200 with `done_reason: load` in 35.432 seconds. `/api/ps`
then confirmed the model resident, and a synthetic JSON chat through the app's
curl/config/stdin transport completed in 6.36 seconds. No repository content was
sent during these checks and no server/global configuration was changed.

Validation: 19 provider tests passed, including exact empty request, load
acknowledgment, independent load timeout and existing cancellation/authentication
checks. The 48 focused renderer tests passed, followed by the added model-load
cancellation regression (10 AI-fill cases passed). Generated IPC contracts,
renderer/native builds, lint and diff checks passed. The debug app is rebuilt for
its next launch. These live checks validate preload and synthetic generation;
they do not claim that the user's entire 349-file generation has passed.

### November 10 cancellation, selection continuity and large AI selections

Stage/Unstage and AI filling now expose Cancel beside their progress indicator.
Staging cancellation uses the existing native operation cancellation; AI cancels
an active source-capture request or its exact model job. The UI shows Cancelling
until the operation settles and does not apply a model response after cancellation.
Deliberate pre-publication cancellation has a neutral completion message. Failed
or cancelled staging never transfers the file selection to another source.

After successful Stage/Unstage, checked files follow their rows into the new
source group on inventory refresh. The same selection is available to AI and
Commit without a second checkbox pass. Commit continues to include only selected
paths; other staged paths remain in the index, as verified independently with Git.

The 349-file AI failure was a local 128-selection limit, before provider access.
Capture now accepts up to the same 10,000 paths as writes. Its local fragment count
supports selected metadata/diffs/content/history, local raw capture remains bounded
at 128 MiB, and individual source reads are bounded at 4 MiB. Provider payload
budgets and binary/secret screening remain enforced. A 349-file staged fixture
with 26 binary assets larger than 1 MiB passes capture within the normal timeout;
its provider payload stays within the configured budget and its Git index/HEAD
are unchanged. The oversized-source refusal test now exercises the 4 MiB boundary.

Validation: all 46 focused renderer tests passed; seven AI capture integration
cases, eight context unit cases, the large 349-file regression and the selected
commit/retained-staging regression passed. Renderer/native builds, lint and diff
checks passed. No real repository was staged/committed or sent to an AI provider;
only a read-only Git blob-size summary was obtained from the user's design repo.
The debug bundle is rebuilt for the next launch. Native visual/cancellation timing
for these new controls was not separately exercised.

### November 10 visible work feedback

Stage and Unstage retain disabled controls during execution, change their button
labels to Staging/Unstaging, and display a spinner with the captured file count
in the fixed action toolbar. AI filling displays progress for settings lookup,
selected-change capture, request startup and waiting for the model. Long-running
operations show elapsed seconds after five seconds, without invented percentages.
Indicators clear on completion/error; existing completion/error feedback remains.
Reduced-motion preferences disable spinner animation while retaining the text.

Validation: all 39 focused App/AI-fill/work-status tests passed, including deferred
Stage/Unstage completion, deferred AI capture/model responses, elapsed time and
timer cleanup. Renderer/native builds, lint and diff checks passed. The native
debug bundle was updated for the next launch. No new native visual acceptance was
performed for this indicator-only change, and the user's running app was not restarted.

### November 10 obsolete overview selections

Inventory refresh now removes saved repository selections whose IDs are absent
from the current inventory or have been superseded. Replacement repositories
remain unchecked; valid selections, tabs and historical drafts are preserved.
This keeps the overview Selected metric, selection count and repository checkboxes
consistent. The attention metric now reads "Needs attention".

Validation: all 31 App tests passed, including obsolete/absent saved selections,
retained valid selections, persisted cleanup and replacement during rescan.
Renderer/native builds, lint and diff checks passed. The debug bundle was updated
for its next launch; the user's running app and live database were not edited.

### November 10 staging failure and direct operation feedback

The live failed Stage record was traced read-only to `refs/.DS_Store` in the
`design` repository. Its candidate index passed object validation, but the second
validation ran against the real reference namespace and Git rejected Finder's
metadata as an invalid reference. The corrected validator uses the isolated
reference namespace with the real destination object directory, preserving
object/index/commit verification without enumerating unrelated real refs. The
exact saved candidate also passed this corrected read-only validation. No files,
references or staging contents in the user's repository were changed during diagnosis.

Direct Stage, Unstage and Commit responses now inspect the returned Git outcome;
a successful request transport no longer disguises a blocked operation as success.
Success and failure appear directly at the top of the repository page. Success
feedback survives refresh and is scoped to the affected workspace/repository.
Published or uncertain commit outcomes explicitly warn against a blind retry.
Cleaned-up pre-publication failures remain in operation history but no longer
appear as pending recovery or prevent workspace removal. Interrupted or uncertain
operations still require recovery. History shows the plain failure before optional
technical details and offers no recovery button for an already cleaned-up failure.

Validation: 56 application integration tests passed with `--test-threads=2`,
including Stage/Unstage/Commit with synthetic Finder metadata and a rejected hook
whose failure remains in history without a pending-recovery notice. The initial
unbounded parallel run hit inspection timeouts; the bounded run passed all cases.
All 38 focused App/history/outcome-feedback tests passed, including preservation
of the draft and visible feedback through refresh. Renderer/native builds, lint
and diff checks passed. In the isolated native 349-file fixture, Stage completed
with synthetic `.git/refs/.DS_Store` present; independent Git blob hashes matched
all 349 source files and the metadata bytes were preserved. Native Unstage also
showed its green result banner at the top after the refresh completed. The normal
user app was not restarted; the debug bundle was updated for its next launch.

### November 10 large staging and persistent operation controls

Direct Stage, Unstage and Commit preparation no longer generates the full binary
patch used by the retired review display. The direct path still constructs and
binds the exact candidate trees, fingerprints selected content, revalidates state
and uses the existing transaction/publication path. The lower-level proposal API
retains full patch generation for callers that explicitly use its review output.
This removes an unnecessary process-output limit from normal operations rather
than increasing that limit or truncating content required for execution.

The changed-file list scrolls independently below its header and Stage/Unstage
toolbar. The composer content scrolls independently above a fixed Commit footer.
The pane height is fitted to the actual remaining native viewport and recalculated
on layout renders, window resizing and section expansion. File count does not
stretch the panels or move actions to the end of the list.

Validation: all 54 application integration tests and 28 App tests passed; renderer
and native builds, lint and diff checks passed. A small-output-budget integration
case stages, unstages and commits exact file content even when its complete diff
exceeds that budget. The updated prepared-operation case verifies metadata remains
bound without embedding an unused full patch.

Native verification used
`/private/tmp/Commit Command Large Stage Check 20261110.app`, isolated data at
`/private/tmp/commit-command-large-stage-check-20261110` and 349 synthetic SVG files
in `/private/tmp/commit-command-large-stage-fixture-20261110`. Verified action
positions before and after scrolling to fixture 349; Stage/Unstage and Commit
remained visible. Clicking Stage succeeded. Independent Git/index inspection
verified all 349 staged object IDs match the current file bytes, zero unstaged
changes and a full diff of 23,313,090 bytes, exceeding the old 16,777,216-byte output
limit. No user repository was staged or changed. The debug bundle was updated;
the running user app was left untouched and needs restart.

### November 10 sidebar autosave serialization repair

Sidebar pointer resizing now rounds to whole pixels before updating navigation
state. Webview pointer coordinates can be fractional under UI scaling; passing
that value to the unsigned-integer sidebar-width field rejected the entire
save-state request, including drafts. The warning now says Layout and drafts
were not saved and clears after a subsequent successful save without clearing
unrelated errors or applying an old workspace's response to the new workspace.

Validation: all 27 App tests passed, including a fractional pointer-coordinate
regression that preserves the draft and a failed-save/successful-save warning
recovery case. Renderer/native builds, lint and diff checks passed. The debug
bundle was updated without closing the user's running instance. Any unsaved draft
in that old instance should be copied before restarting.

### November 10 normal Git actions and removal of the trust workflow

At the user's explicit direction, the standard desktop build now enables
`desktop-writes` by default. This supersedes older ordinary-read-only disposition
statements below; it does not assert completion of unrelated release acceptance.
The application no longer requires a separate stored worktree trust grant before
executing an explicitly requested operation. The lower-level controlled-write
API retains its existing trust contract for its other callers. Application
operations retain identity/configuration revalidation, hooks/signing behavior,
locking, stale-input checks, cancellation and journal/recovery handling.

Removed Inspect trust & eligibility, the Trust & execution card, overview trust
controls/column/sort options, periodic trust polling and the extra hook-preference
save button. Stage, Unstage and Commit now name the action directly. The obsolete
authoritative/native-review explanation is removed from the composer. Commit is
enabled when the host supports Git writes, selected files and a nonblank message
are present, and no conflicting operation is active.

Validation: all 202 renderer tests and 53 application integration tests passed,
including an actual commit through Application without creating any trust grant.
Renderer/native builds, lint, generated native command ACL and diff checks passed.
Native standard-build verification used
`/private/tmp/Commit Command Commit Check 20261110.app` and isolated app data at
`/private/tmp/commit-command-commit-check-20261110`. Selected the disposable
`alpha/fixture.txt`, entered a message and clicked Commit once. No trust or review
dialog appeared. The file list became empty; independent Git inspection verified
commit `bb22aca` with subject `test: verify direct commit action` and a clean
worktree. The isolated database contained zero trust grants. No user repository
was committed or changed. The updated debug bundle needs restart; the normal user
instance was left running.

### November 10 direct AI commit-message fill

Fill with AI now captures the selected file changes, generates one message using
the saved server/model/instructions/parameters and fills the composer directly.
The normal button no longer opens the context/candidate diagnostic window or
requires Capture, Generate, candidate selection or another confirmation. It shows
Generating while active and errors beside the button; failed requests preserve
the current draft. Unmounting the component cancels its exact job and ignores late
results. Version file Action was removed from the commit composer.

Ollama source-message requests now send the explicit JSON schema in `format`,
constrain generation to one candidate and constrain attribution to actual request
chunk IDs. Existing response validation remains. Selected comparisons trim common
unchanged prefixes/suffixes instead of representing every line as replaced. The
serialized context budget reserves space across fragments so a large first file
cannot starve all later selected files. A seven-file regression verifies each
selected file is represented within the same byte bound.

Validation: all 201 renderer tests, 53 AI unit/protocol tests and 52 application
integration tests passed. Loopback provider tests ran outside the socket-restricted
sandbox. Renderer build, native build, lint and diff whitespace checks passed.
Native end-to-end verification used the isolated app
`/private/tmp/Commit Command Direct AI Check 20261110.app`, isolated data
`/private/tmp/commit-command-direct-ai-check-20261110` and the existing disposable
`alpha/fixture.txt`. Against `http://10.10.10.70:11434/` with
`qwen3.8:27b-mtp-bf16`, a single click changed Fill with AI to Generating and then
filled the composer with a subject and body about the synthetic fixture. No
suggestion dialog appeared. No user repository source was sent in this check,
and no files were staged or committed. The ordinary running user instance was
left untouched; the updated debug bundle needs restart to load the change.

### November 10 fresh startup discovery and prompt continuation

Application startup now performs a fresh scan of the active workspace before
returning its current inventory. The previous process's saved scan is not used
as the startup result. Existing repository identities and drafts are preserved.
An unfinished current scan schedules its next slice after five seconds rather
than waiting for the normal sixty-second discovery interval; stopping/switching
the monitor clears that retry. Actual inspection failures remain visible.

Validation: all 52 application integration tests passed, including replacement
of a persisted timeout by a fresh startup result and immediate discovery of a
repository created while the app was closed. Both verify repository bytes remain
unchanged. Three monitor unit tests passed, including bounded retry scheduling
and stop cleanup. Renderer and native builds, lint and diff whitespace checks
passed. The first sandboxed application run passed 47 cases; five localhost HTTP
fixtures could not bind loopback sockets. The rerun outside that sandbox passed
all 52. The debug bundle was updated; the running user instance was untouched.

### November 10 workspace folder link

The active workspace path in the sidebar now uses the same hover/focus underline
and Finder folder-opening action as the repository header path. All 24 App tests
passed, including the workspace path regression. Renderer and native builds
passed; the debug bundle was updated without restarting the user instance.

### November 10 changed-file selection checkbox

An unlabeled master checkbox sits as a file row immediately above the first file
in the first populated group, aligned with the other checkboxes. It selects all enabled
checkboxes from an empty or partial selection, and clears them when all are
selected. Partial selection uses the native mixed indicator. Conflicted rows
remain disabled. When staged and working-tree rows refer to the same path, both
can be checked; requests include that path once using its full working-tree
content. The selection count remains a count of unique files.

Validation: all 23 App tests passed, including none/partial/all toggling, mixed
staged/working-tree rows, unique-file count and excluded conflict rows. Renderer
build, lint and native build passed. Updated the debug bundle executable; the
running user instance was left untouched and needs restart to load the change.

### November 10 background scan stability and AI dialog repair

A resumable discovery slice no longer marks previously known repositories
unavailable merely because it has not reached them yet. Inspected failures still
update availability; completed scans retain the existing missing/error behavior.
Time-slice progress is labeled as scanning rather than a discovery error. Actual
inspection issues remain prominent. The deterministic expired-scan regression
preserves identity, availability and repository bytes; all 23 workspace cases passed.

The repository header path underlines on hover/focus and opens the folder's
contents in Finder. The existing tab-menu Reveal action still selects the folder.
Fill with AI now opens an owned native movable/resizable dialog using the same
host presentation as Settings, with a fixed header and independently scrolling
content. A modal guard blocks owner input until closure; selected suggestions
return only the chosen full message or description to the original draft. The
child has no Git mutation capability. Settings and AI dialogs share their scoped
renderer and host presentation, rather than duplicating the window infrastructure.

Validation: all 198 renderer tests across 39 files passed; renderer build, lint,
native build and generated command ACL test passed. Native verification used
`/private/tmp/Commit Command AI Dialog Check 20261110.app` with isolated copied
fixture data at `/private/tmp/commit-command-ai-dialog-check-20261110`.
Verified native AI opening, scrolling, title-bar dragging, resizing, Close button,
red-window-button closing, reopening and return to the owner. Header-path click
opened the intended fixture folder's contents in Finder. No provider inference
was performed in this window-lifecycle check. The normal running user app and
its drafts were left untouched; the disposable test process was stopped.
The debug bundle executable was updated; a running instance needs restart.

### November 10 navigation, tab management and About cleanup

Discovery reports now live in Workspace troubleshooting in the sidebar. An
incomplete scan still produces a prominent main-content warning that opens the
troubleshooting section. Push dependency planning moved from Workspace Overview
into the repository's Remotes and upstream controls.

Hide sidebar has a subtle tinted button surface. Overview stays fixed with a
visual divider before repository tabs. Repository tabs support pointer dragging,
keyboard ordering (Alt+Shift+Left/Right), and the requested context menu: Close,
Close Others, Close to the Right, Copy Path, Reveal in Finder, Pin/Unpin. Pinned
tabs occupy the left group and survive bulk closing; direct Close remains
available. Closing tabs preserves drafts. Pin state has a default-false optional
serialized field, so older saved layouts load; navigation restart coverage verifies
pinned state persists. Path actions use fixed macOS tools with argument/stdin
passing and are restricted to the main renderer.

Native verification used three disposable uncommitted fixture repositories in
`/private/tmp/commit-command-tabs-fixture-20261110` and isolated app data in
`/private/tmp/commit-command-tabs-check-20261110`. Verified the menu, pinning and
pin persistence on restart; keyboard and final pointer-drag ordering; Close and
Close to the Right; disabled bulk-close actions when only other pinned tabs
remain; retained draft after closing/reopening; exact Copy Path output pasted
inside the fixture; and Finder selecting the intended fixture folder. Initial
native tests exposed premature menu dismissal on WebKit blur and unreliable
HTML drag/drop. Final code uses outside-click/Escape/Tab dismissal and pointer
capture for dragging. Temporary fixture repositories were not committed or pushed.

About now keeps its metadata visible while View license toggles a read-only text
box below the buttons; the open label is Close license. Publisher attribution is
removed. The bundled license text matches the root MIT LICENSE byte-for-byte,
including copyright to Randy Dodson, without the obsolete publishing attribution.

Final native About check: opened the dialog, observed Developer/Copyright/License
without Publisher, opened the read-only MIT License text area below the buttons,
and closed it with Close license; the button returned to View license.

Validation: all 182 renderer tests, frontend build and lint passed. Core navigation
migration/restart test and generated native command ACL test passed. Native bundle
rebuilt. Earlier status entries remain historical checkpoints.

### November 10 Settings close fix and native verification

The automatic-save close handler returned through Tauri's `onCloseRequested`,
which finishes by invoking window destruction. The Settings capability allowed
close but omitted destroy; this prevented both close controls from finishing.
Added window-destroy permission scoped to the Settings webview and removed the
recursive close request. The handler now awaits save completion and allows
Tauri's normal destruction path, preventing close only on save failure.

Native verification in `/private/tmp/Commit Command Close Check 20261110.app`
using `/private/tmp/commit-command-close-check-20261110`: opened Settings, clicked
Close settings, observed the main window with an enabled Settings button;
reopened Settings, clicked its native red close button, and again observed the
main window with focus returned to Settings. No user data or drafts were touched.
The earlier native-close verification gap is resolved for these no-pending-edit
cases; queued-save and failure boundaries retain renderer test coverage.

### November 10 automatic AI property persistence

The explicit Save AI settings action is removed. Valid edits save automatically
after a 250 ms typing pause; pending writes are serialized and intermediate edits
are coalesced. The latest returned settings fingerprint/revision is used for each
write. Server metadata can save before model selection. Complete server/model
settings use the existing atomic singleton update, without requiring a live test
to persist metadata. Live testing remains explicit.

Settings close actions flush pending changes, including the native title-bar
close route. A failed write is shown as an error and prevents closing; changing
the field or another close attempt retries pending values. Inputs remain editable
while writing. Model-list matching ignores only generated record identity within
the singleton editor, so creating the first saved server does not discard an
in-progress test or its model list.

Validation: frontend build, ESLint and all 173 renderer tests passed, including
automatic parameter/Unset persistence, serialized revision use, latest-edit flush
and write failure handling. Native bundle rebuilt. Native title-bar close behavior
has not been manually retested for this change.

### November 10 AI settings tabs

Supersedes the earlier wizard navigation: Server, Model and Model settings are
freely accessible tabs, with roving keyboard focus and preserved edits. Next/Back
and numbered setup steps are removed. Save AI settings is available on every tab.
Editing parameters/instructions for the unchanged saved server/model does not
require a new connection test; choosing a new server/model still uses live
model discovery. A running test continues when another tab is selected.

Validation: frontend build, ESLint and all 169 renderer tests passed, including
free navigation before discovery, edit preservation and saved-model editing.
Native bundle rebuilt; GUI behavior for this navigation change was not manually
retested. No user settings were changed.

### November 10 saved-server connection-test fix

A saved connection was spread into the request draft, including `revision` and
`endpoint_approval` fields. The strict desktop decoder rejected that request
before HTTP transport ran. Explicit draft projection now applies when loading,
testing and saving a connection. Decoder strictness remains intact. Settings
alerts now use bold theme-aware red text and a circular marker; local request
rejection explicitly says the app failed before contacting the server.

Validation: production frontend/native builds, ESLint and all 168 renderer tests
passed. Regression coverage rejects persistence fields in test/save requests,
checks saved-server editing and saving, and verifies each test click starts a
fresh discovery request. A live read-only GET to
`http://10.10.10.70:11434/api/tags` returned HTTP 200 and nine models. No inference
or source transfer ran. The bundle was rebuilt. Final native UI verification of
this patch remains unverified: the normal app was still the old running build,
and organization policy blocked computer use of the isolated app identity.
Both owned verification processes were stopped; the normal app was preserved.

### November 10 settings simplification

User-directed scope change: Settings now contains **Appearance** and **AI**.
AI setup is a single Server → Model → Model settings flow: editable address and
port, explicit reachability/model-list test, installed-model dropdown, system
instructions and supported generation parameters. Profile management, scoped
assignments, template editing, diagnostics, request-limit and backup pages are
removed from the settings UI. The AI message UI no longer offers a profile selector.

Saving uses one atomic `single_setup` change: one server/model, one application
binding, no old workspace/repository overrides or additive instructions. Existing
storage types remain compatible; the previous active application configuration
(or first saved configuration when none was assigned) supplies initial values.
Old configuration generations remain on disk. Saving does not send source or
run inference. The provider test executes directly and watches its returned job
ID rather than requiring an approval window.

Validation: frontend production build, ESLint, 164 renderer tests, two focused
Core singleton/migration tests, and generated native command ACL test passed.
Native host suite: 19 passed and one worker-only case ignored in the sandbox;
two disposable-Keychain cases failed there with OSStatus -50, then both passed
when rerun outside the sandbox with one test thread.
Native verification used `/private/tmp/commit-command-single-ai-check-20261110`:
entered `http://10.10.10.72`, port `11434`; Test provider returned five installed
models; selected `qwen3.5:9b-q4_K_M`; saved temperature `0.3` and Top P `0.8`;
reopened Settings and observed the saved server. Inspected the isolated saved
metadata to confirm model/parameters persisted. No inference or repository
content transfer was performed. The normal user's configuration was not changed.
This is settings/provider-discovery evidence, not inference or release acceptance.

### November 9 completion P13B Core AI import controller

[Controller checkpoint](reports/COMPLETION_P13B_CONTROLLER_2026-11-09.md): native-only
expiring file registration, frozen preview token, exact source/settings/identity
revalidation, one-use native confirmation, verified safety export and atomic AI
generation activation. Final98 library/preceding49 application/final18 AI cases,
150 renderer/20+23 host, strict Clippy and release checks pass at the documented
boundaries. Keychain deadline failures under concurrent validation are retained;
isolated and full four-worker reruns passed without changing deadlines. Native
file-picker/renderer controls are not yet wired; IMPORT05–IMPORT08 remain partial.

### November 9 completion P13B inert AI merge planner

[Merge checkpoint](reports/COMPLETION_P13B_MERGE_2026-11-09.md): explicit stable-ID
add/replace/rename/skip, dependency rewiring/overrides, inactive-by-default scopes,
explicit occupied-scope replacement and current-identity/archive/settings guards.
Templates now receive fresh imported revisions. Exact container SHA-256 detects
ZIP metadata drift that a manifest hash cannot. Final96 library/preceding44
application tests and strict Core Clippy pass. Picker/UI/confirmation/atomic apply
remain open; IMPORT05–IMPORT07 remain partial.

### November 9 completion P13A saved AI export workflow

[Export checkpoint](reports/COMPLETION_P13A_AI_EXPORT_2026-11-09.md): saved AI settings
export through a typed native directory picker into a verified private backup child;
stale generations, unsafe destinations and picker replay refuse. Core89/44 and full
desktop150 renderer/20+23 host, strict Clippy/release pass. A reproduced nonblocking
accepted-socket fixture race was fixed in both provider/Keychain tests;20 reflection
and5 Keychain repetitions pass, with original failure/probe evidence retained.
Native export UI is blocked by organization policy for the disposable fixture app;
owned process stopped. Import/full backup/restore remain open.

### November 9 completion P13A AI portability payloads

[AI-portability checkpoint](reports/COMPLETION_P13A_AI_PORTABILITY_2026-11-09.md):
verified AI-only payload publication clears authentication/endpoint approval;
bounded unactivated preview resets foreign authority and creates fresh revisions.
88 library/43 application tests and strict Core Clippy pass, including no-clobber,
source preservation, revision/component/schema/cancellation/oversize refusals.
Product UI, mapping/conflict choices and atomic apply remain open; BACKUP01 and
IMPORT06 remain partial.

### November 9 completion P04 Activity state persistence

[Activity-state checkpoint](reports/COMPLETION_P04_ACTIVITY_STATE_2026-11-09.md):
applied repository/author/reference/UTC dates and view mode persist per workspace
through existing navigation storage. Legacy defaults, bounded validation, actual
Core reopen and unavailable-ID/no-unsaved-edit behavior pass. Core86/43 and full
desktop148 renderer/20+23 host, strict Clippy/release pass. Native Quit/restart and
broader UI03/GRAPH04/GRAPH05 acceptance remain open.

### November 9 completion P04 Activity date filters

[Activity-date checkpoint](reports/COMPLETION_P04_ACTIVITY_DATES_2026-11-09.md):
explicit inclusive UTC dates combine with author/repository/reference filters;
all views use chosen dates and clearing restores seven-day/twelve-month defaults.
Six real Git graph/activity tests and complete desktop147 renderer/20+23 host,
strict Clippy and release build pass. Native date controls/VoiceOver and restart
filter persistence remain open; GRAPH04/GRAPH05 are partial.

### November 9 completion P13A migration safety adoption

[Migration checkpoint](reports/COMPLETION_P13A_MIGRATION_2026-11-09.md): the schema2→3
migration now requires a coherent owner-connection/config archive before its SQL
transaction. Exact config/WAL capture, revision/schema checks, private verified
publication and blocked-destination refusal pass;85 library/43 application tests
and strict Clippy pass. Existing raw migration copies remain. This is migration
safety only; full backup/restore, AI-generation capture, artifact policies and
native/installed acceptance remain open.

### November 9 completion P13A free-space preflight

[Space checkpoint](reports/COMPLETION_P13A_SPACE_2026-11-09.md): descriptor-based
free-space check precedes temporary creation, with declared content plus32 MiB
container overhead and16 MiB live-state reserve. Seventeen focused tests and
strict Core Clippy pass. This is advisory; real disk-full/fsync/crash acceptance,
owner capture/migration integration and full backup workflows remain open.

### November 9 completion P09 unsaved repository AI tests

[Editor-test checkpoint](reports/COMPLETION_P09_EDITOR_TEST_2026-11-09.md): explicit
read-only repository/source capture with current unsaved editors, distinct test
purpose, frozen native consent, no save/apply/Action authority. Core42 application,
82 library and strict Clippy pass; selected actual ordinary native decline/approve,
exact loopback markers/source, Quit/restart/no replay and preservation passed.
Final82 library/42 application/145 renderer/20+23 host and complete desktop gate
passed; request-fingerprint correlation excludes old-profile results. An intermittent existing
reflection-fixture transport failure is retained; the later export checkpoint above
reproduces and fixes a shared nonblocking accepted-socket fixture race. Broader
provider/CI reliability, version preview/audit/accessibility remain open.

### November 8 completion P04 subject/body composer

[Composer checkpoint](reports/COMPLETION_P04_COMPOSER_2026-11-08.md): individual,
shared bulk and override editors support explicit subject/body editing while keeping
raw drafts/separators lossless. Full desktop139/20/23 passed; actual ordinary
subject-only edit/raw view/Quit/restart matched exact draft oracle and preserved
repository bytes. Broader UI06 native combinations/accessibility remain open.

### November 8 completion P04 density and sidebar

[Density checkpoint](reports/COMPLETION_P04_DENSITY_2026-11-08.md): default-compatible
Comfortable/Compact and sidebar visibility persist per workspace, retaining widths.
Full desktop135/20/23 and selected ordinary Compact/Hide/Quit/restart/Show260px,
exact preservation and zero write/trust checks passed. Full geometry/accessibility
and remaining P04 workflows remain open.

### November 8 completion P04 filters and tree state

[Filter/collapse checkpoint](reports/COMPLETION_P04_FILTERS_2026-11-08.md): specified
status categories, shared counter predicates, typed/default-compatible filter state
and persisted tree collapse implemented. Full desktop 134/20/23 passed; selected
native child selection/collapse/Quit/restart/expand and exact preservation passed.
Native status-popup selection and broader accessibility remain unaccepted. P04 partial.

### November 8 completion P04 sortable overview

[Sorting checkpoint](reports/COMPLETION_P04_SORTING_2026-11-08.md): all specified
sort columns/directions, stable ties/unknown placement, typed backward-compatible
navigation and aria-sort implemented. Full desktop 131/20/23 passed; selected
ordinary native sort/selection/Quit/restart and exact preservation passed. P04/UI02
remain partial pending full accessibility, geometry and remaining shell work.

### November 8 completion P13A backup primitives

[Primitive checkpoint](reports/COMPLETION_P13A_PRIMITIVES_2026-11-08.md): bounded
stored-only archive codec and owner-connection WAL-consistent SQLite snapshot
implemented. No extraction or state activation. Focused hostile-container and
source-preservation tests pass; P13A remains partial pending complete durable
publication acceptance, coherent app-state capture and migration integration.
[Publication checkpoint](reports/COMPLETION_P13A_PUBLICATION_2026-11-08.md) adds
private descriptor-relative staging, validated/synced no-clobber publication and
streaming cancellation/failure cleanup; 16 focused tests and strict Clippy pass. Product backup,
restore and retention remain unfinished; no G10 acceptance claimed.

### November 8 completion P02 initial activation fix

[Activation checkpoint](reports/COMPLETION_P02_ACTIVATION_2026-11-08.md): pinned Tao
source distinguishes show from app activation. First page-finished display now
focuses/activates once; later reloads retain the no-focus-steal guard. Actual ordinary
first launch and two restarts rendered without diagnostic intervention; three Quit
exits, exact repository preservation, zero trust/writes, integrity/lock reopen passed.
Host 20/23 plus build/format/strict Clippy passed. G02 broader crash/OS/accessibility
criteria remain partial; no universal reliability or default-write change.

### November 8 completion P03 discovery continuation and P02 startup capture

[Continuation milestone](reports/COMPLETION_P03_CONTINUATION_2026-11-08.md): bounded
in-memory traversal/relationship continuation, progress UI and typed read request
implemented. Workspace 23/default Core 68/application 41/bulk 14 and full desktop
127/20/23 passed; release 100-repo runs complete in two slices (~24–26 seconds),
with exact preservation. Native 88 partial → explicit Continue → 100 complete/Quit
passed after fixed startup diagnostics. Full Core matrix still running at this
checkpoint. [Actual startup blank](reports/COMPLETION_P02_STARTUP_2026-11-08.md) remains
a G02 failure: shell mounted/non-inert but document hidden, visible after probe
without reload; no causal patch or release/default-write acceptance.

### November 8 completion P09 AI editor identity

[Focused fix](reports/COMPLETION_P09_IDENTITY_2026-11-08.md): saves now retain the
exact edited/new connection/profile ID instead of selecting by a duplicate name.
Three renderer fixture regressions pass; Core already rejects duplicate names,
so no valid persisted duplicate-name save defect was reproduced. Complete renderer
126 and type/lint/contracts/format/build passed. P09/G09 remain partial; no authority or storage change.

### November 8 completion P01 evidence tools

[Tooling report](reports/COMPLETION_P01_2026-11-08.md): reusable bounded no-follow
filesystem oracle, literal allowed deltas and hash-bound result validation added.
14 Python regressions and actual disposable-Git preservation passed; one non-UTF8
creation case remains explicitly skipped. Ordinary feature exclusions verified.
Full Core runner currently blocked by missing installed/selected GPG; no new native
or release acceptance. User-selected DCP03/04/05/06/08 implementation proposals are
recorded; final default/distribution/performance decisions remain separate.

### November 8 completion P00 reconciliation

[Reconciliation report](reports/COMPLETION_P00_2026-11-08.md): the detailed handoff and
execution ledger route all 109 requirements and 256 catalog cases to existing source,
tests, evidence candidates and remaining packages. Documentation/reference checks
passed. DCP01–DCP09 remain explicit policy dependencies; P01 independent tooling is
next. No new application/native acceptance or write-default change.

### November 8 G02–G11 full Core regression after AI source integration — passed locally

[Regression checkpoint](reports/G02_G11_AI_SOURCE_REGRESSION_2026-11-08.md): complete
five-configuration runner exited zero against the Core source in `d9d6954`:
66/101/92/318/367 passes, documented ignored workers/capability cases retained;
24 disposable lab cases passed, zero skipped. Locked builds, formatting and strict
Clippy passed with four test threads and isolated GPG. Source hashes match the
committed tree. This is fresh automated evidence, not acceptance of remaining native,
AI/backup or release gates. No release/default-write change or publication occurred.

### November 8 G09 Ollama real-provider diagnostics and completion limits — selected acceptance passed

[Real-provider checkpoint](reports/G09_OLLAMA_REAL_PROVIDER_2026-11-08.md): native
model discovery and synthetic generation were exercised against the user-supplied
Ollama endpoint/model. The default output budget produced a safely discarded token-limit
result; an explicitly larger profile budget completed synthetic inference. Missing-model
refusal, client cancellation/reaping and generated disposable-fixture suggestions were
also observed. The suggestions were structurally valid but overinterpreted fixture text,
so semantic correctness still requires human review. No project source was sent and no
repository writes occurred. Provider parsing now rejects partial output with actionable
errors; focused Core and complete desktop validation passed. Native host closure after
the final suggestion run was interrupted and is not claimed. G09 and G02–G11 remain open.

### November 8 G09 exact source transfer and assisted messages — implemented, selected acceptance passed

[Assisted-message checkpoint](reports/G09_AI_ASSISTED_MESSAGES_2026-11-08.md): composer
context viewer, raw/filtered snapshot revalidation, exact native source-transfer
approval and one-to-three Rust-validated text suggestions now exist. Suggestions
replace a draft only by explicit choice and grant no Git/Action/push authority.
Final library/application 61/41, strict Core Clippy and complete desktop runner
(123 renderer, ordinary/all-feature host 20/23 plus ignored workers) passed.
Native fixture decline sent zero HTTP; fresh approval sent one request whose context
hash matched the viewer. Candidate application changed only the draft, restored
before clean Quit/restart/no replay; repository/journal/trust/original/hash/integrity
and lock checks passed. A real native panel-remount finding was reproduced and fixed;
failed timing runs and the corrected four-thread results remain recorded.
Full five-configuration Core write/signing regression subsequently passed; see the
[regression checkpoint](reports/G02_G11_AI_SOURCE_REGRESSION_2026-11-08.md). Real-provider
synthetic checks are recorded above. Next G09 work is hierarchical analysis and bounded
automatic authority; G09 and the remaining G02–G11 acceptance scopes remain open.

### November 8 G09 restricted repository context — implemented, automated selected acceptance passed

[Capture checkpoint](reports/G09_AI_CONTEXT_CAPTURE_2026-11-08.md): explicit staged,
full working-tree/untracked selections, bounded related tracked source/docs and
current-branch/file history now feed the filter. Read-only descriptor traversal
refuses links/nested repositories; raw fingerprints revalidate excluded/redacted
changes too. No trust or AI setup required. Complete library/application suites
57/39 passed, strict all-feature/all-target Clippy, generated contracts and desktop
TypeScript passed. No source transfer/native context UI is claimed. Next: exact
context viewer, native-reviewed source inference and Rust-validated candidates.

### November 8 G09 exact context filtering — implemented, automated foundation passed

[Context filter checkpoint](reports/G09_AI_CONTEXT_FILTER_2026-11-08.md): deterministic
serialized-byte budgets, line/chunk provenance, explicit omission accounting,
sensitive path exclusions and conservative secret/control screening now exist in
pure Core. Seven focused tests and strict all-feature/all-target Clippy passed.
No repository capture/transfer or native acceptance is claimed; capture, exact
filtered context review and assisted candidates remain next. Screening is defense
in depth, and incomplete/redacted coverage cannot authorize an automatic commit.

### November 8 G09 endpoint-bound AI secure auth — implemented, selected acceptance passed

[Secure-auth checkpoint](reports/G09_AI_SECURE_AUTH_2026-11-08.md): optional saved
connection authentication now uses native-only macOS secure entry and endpoint-bound
Keychain storage, isolated attested helpers, bounded deadlines/cancellation and no
fallback on missing/locked credentials. Imported definitions reset authentication.
Final Core library/application 50/32 passed; complete desktop runner passed (119
renderer, ordinary host 20 + one ignored worker, all-feature host 23 + one ignored
worker). Actual private-Keychain tests cover round trip, locked/wrong endpoint,
approval, timeout and cancellation. Native ordinary missing-token request showed the
exact saved binding, failed clearly and sent zero HTTP. Secure-field Escape, manual
draft/refresh, two clean quits/restart/no replay, hashes, repository/journal/trust
preservation, integrity and lock reopen passed. Initial slow locked lookups and the
1.5-second deadline failure remain counterevidence in the report. No real user token
was entered or modified. Source context/candidates/automatic authority, real-provider
interoperability and broader G02–G11 acceptance remain open; next functional work is
bounded repository context and assisted message generation.

### November 8 G09 native-reviewed provider diagnostics — implemented, selected acceptance passed

[Provider diagnostic checkpoint](reports/G09_AI_PROVIDER_DIAGNOSTICS_2026-11-08.md):
fixed Ollama/compatible discovery and synthetic inference use current unsaved editors,
exact one-use native approval, bounded stdin transport, no redirects/fallback and
independent cancellable jobs with request limits. Ordinary provider reads are
separately classified by trusted Core; repository writes remain disabled. Final
library 48 / application 31, renderer 116 / hosts 14 / 17 and full checks passed.
Native decline zero traffic, approved discovery/inference, exact unsaved synthetic
payload, running Cancel, manual refresh during held inference, Quit/owned curl reap
and ordinary restart/no replay passed. Exact settings/repositories/journals/trust,
hashes, integrity/lock preserved. This is synthetic loopback acceptance; secure auth,
TLS/actual provider, exact source context/candidates and bounded automatic authority
remain G09 work. Selected AI-read lifecycle evidence does not close G02 facility gaps.

### November 8 G09 optional AI settings storage/editor — implemented, selected acceptance passed

[Storage/editor checkpoint](reports/G09_AI_SETTINGS_STORAGE_2026-11-08.md): ordinary
build explicitly edits optional profiles/templates/connections/instructions/limits
and inspects effective configuration. Lazy private TOML/text generations activate
through an atomic synced pointer; links/unsafe artifacts/stale editor overwrites
are refused and prior generations preserved. Missing/corrupt AI cannot block manual
startup/reads. Unsaved static preview performs no inference. Library 39, application
30, final renderer 113 / hosts 14 / 17 and full checks passed; focused final editor
4 passed. Native saves/effective settings/unsaved preview/Escape focus/manual draft/
restart/Quit and preservation oracle passed. No transport, credentials, model
verification, inference, candidates or automatic execution is claimed yet.

### November 8 G09 optional AI settings model — implemented, automated boundary passed

[Settings model checkpoint](reports/G09_AI_SETTINGS_MODEL_2026-11-08.md): Core now
has versioned connections/profiles/templates, additive scoped instructions,
most-specific selection without fallback, capability-gated optional parameters,
validated revision-guarded edits and deterministic static previews. No connection
is configured by default; models/configuration confer no Git authority. Library
36 passed (nine AI tests), strict all-feature/all-target Clippy, format/diff passed.
No persistence/UI/provider/context/candidate/automatic execution is claimed yet.
Next add lazy atomic settings persistence and editor integration.

### November 8 G09 retained version inspection — implemented, selected acceptance passed

[Read-only reader checkpoint](reports/G09_RETAINED_VERSION_INSPECTION_2026-11-08.md):
ordinary build displays the privately retained original by validated operation ID,
without mutation authority. Descriptor-relative path guards reject links/unsafe
artifacts; revoked/moved repositories do not prevent recovery inspection. Focused
11 / 1 ignored worker, path guards 2, application 29, strict Core Clippy and complete
desktop 109 / 14 / 17 passed. Native original bytes/location/fingerprint/match,
spacing, disabled writes and Quit passed; preservation oracle verified exact files,
journals/trust, hashes, integrity and lock reopen. Broader G09 remains open; optional
AI foundation is next. Earlier full write matrix remains dated evidence.

### November 8 G09 guarded SemVer Action — implemented, selected acceptance passed

[Action checkpoint](reports/G09_SEMVER_ACTION_2026-11-08.md): independent native
approval now binds exact typed definition/invocation, file/index/HEAD/config/trust.
MacOS same-filesystem atomic exchange retains the original privately and detects
last-precheck external edits without deleting them. Intent is durable; interruption
blocks writes/scope loss until fresh read-only recovery; no replay/double increment.
Staging and manual commit each require separate fresh reviews. Selected Core
SHA-1/SHA-256, real SIGKILL, refusal/recovery and external-race tests passed; native
Action → stage → exact one-file manual commit and ordinary read-only restart/history
passed with exact preservation, old journals/trust, hashes, exits/integrity/lock.
Full desktop 108 renderer / 14 ordinary-host / 17 all-feature-host passed. Full
Core 66/101/91/270/319, lab 24/24, builds/format/strict Clippy passed, runners exit 0.
Explicit ignored cases remain recorded. Retained recovery data is not pruned. Broader G09 and ordinary write/release gates remain open.

### November 8 G09 provider-free SemVer preview — implemented, selected acceptance passed

[SemVer preview checkpoint](reports/G09_SEMVER_PREVIEW_2026-11-08.md): explicit
read-only major/minor/patch preview now works in the ordinary composer without AI
or trust. Strict plain-value syntax, LF/CRLF preservation, explicit prerelease policy,
input bounds, unsafe-path/nested-repository refusal and frozen before/after comparison
are implemented. No Action execution/staging/approval is implied. Focused 4 parser
and 2 application tests, complete default 66 / application 29, strict Core Clippy,
full desktop 104 / 14 / 17 and checks passed. Native ordinary untrusted preview,
invalid-path refusal, manual draft persistence across restart, no preview replay,
exact bytes/modes/old journals/trust, exits/integrity/lock and hashes passed.
G09 remains open: durable approved invocation/execution/staging/recovery is next;
AI providers/profiles/context/candidates/automatic authority remain unimplemented.

### November 8 G08 guided push queue — implemented, selected acceptance passed

[Guided queue checkpoint](reports/G08_GUIDED_PUSH_QUEUE_2026-11-08.md): durable
explicit child-first order now composes independently approved exact publications.
Plans freeze sources/destinations/hooks/child mappings; selected published children
require fresh post-success observations before parent review. Failure/cancellation/
uncertainty stops the queue, successes persist, fresh retries use new plans, and
restart restores no execution authority. Saved paging/history is read-only.
Focused Core SHA-1/SHA-256, skip, trust/scope, partial rejection/recovery/retry and
actual between-item SIGKILL passed. Native child publication, expired-parent safe
stop, fresh parent-only queue completion and ordinary restart/history passed in the
already trusted fixture. Exact bytes/modes/old journals/trust/server preservation,
clean exits/integrity/lock reopen and source/artifact hashes passed. Full desktop
99 renderer / 14 ordinary-host / 17 all-feature-host and checks passed. Full Core
62/97/87/256/302 and lab 24/24 passed, with format/builds/strict Clippy in each
configuration. Full runners exited 0; ignored subprocess/opt-in cases remain explicit.
This is guided per-item authority; broader native uninterrupted/topology/authenticated/
interruption/accessibility coverage remains open. G09 is next functional work.

### November 8 G07 authenticated loopback publication — passed in Core

[Authenticated publication](reports/G07_AUTHENTICATED_PUBLICATION_2026-11-08.md):
actual verified HTTPS SHA-1/SHA-256 publication with explicit get-only helper/account,
wrong-credential refusal, SSH-agent/public-identity publication and independent server
ref/mode/privacy oracles passed. Test server write routes require an explicit mode;
legacy read-only mode still refuses them. SSH fixture service ignores inherited Git
configuration; rejecting global-hook sentinel stayed untouched. Complete remote suite
99 passed / 5 explicitly ignored; final focused 4, strict Clippy/format/AST/diff passed.
Production code unchanged; no real user credentials/agent/server or new native trust.
Native authenticated/topology/interruption coverage remains open; the later G08 checkpoint above adds guided coordination.

### November 8 G02/G07 native host loss during publication — passed

[Actual publication crash](reports/G02_G07_NATIVE_PUSH_HOST_CRASH_2026-11-08.md):
SIGKILL reached the exact owned native host only after a real server pre-receive
check was proven running and durable journal said publishing. After host exit 137,
the released disposable server advanced to the approved source while the app was
absent. First native restart showed unconfirmed/recovery notice with no replay.
Recent pre-attempt evidence was refused; fresh separately approved observation and
native recovery recorded approved-source-present without inventing a publication
receipt. Native and ordinary restart/Quit, exact records/files/modes/trust, parent
revocation, server preservation on ordinary restart, integrity and lock reopen passed.
Source/artifacts unchanged from the fully validated publication checkpoint. Exactly
one new publication/check; owned server hook removed. Broader G02/G07 remain open.

### November 8 G07 exact individual publication — implemented, selected acceptance passed

[Publication checkpoint](reports/G07_EXACT_PUBLICATION_2026-11-08.md): separate exact
native approval, fresh pre-push check, bounded approved-object export, isolated
ancestry-checked expected-old update, durable uncertainty and separately reviewed
observation recovery implemented. Focused SHA-1/SHA-256, object exclusion/preservation,
drift, refusal, proven running cancellation, actual pre/post-transport SIGKILL and
no-replay recovery passed (10 tests plus explicitly invoked crash workers). Native
leaf-only publication and ordinary read-only restart/history passed in the previously
trusted fixture; managed files/modes, old journals and trust exact, parent revoked.
Final source full Core matrix passed 62/97/87/243/288, lab 24/24; full desktop
93/14/17 plus checks/builds/strict Clippy passed. Final-source native publication and
ordinary restart/history, exits 0, integrity and lock reopen passed with exact hashes.
Recovery rejects pre-attempt observations (regression failed before fix, passed after).
Initial setup deadline failures and the run stopped for that fix are retained; the
runner defaults to four workers matching Core's process cap. G07 remains partial;
bulk publication is G08.

### November 8 G03/G05 changed gitlink status — fixed and natively verified

[Status fix](reports/G03_GITLINK_STATUS_2026-11-08.md): own-index gitlink OIDs now join
independently inspected current child HEADs in workspace inventory. Parent counts/
Changes rows reflect advanced child commits without recursive parent Git helpers;
refresh/reversal/staged+unstaged merge stay correct. Inspector shows exact index/
observed child OIDs, explicitly local observation data. Regression failed on old
source then passed; full Core matrix 62/97/87/234/278 and lab 24 passed, full desktop
91/13/16 plus format/types/lint/contracts/builds/strict Clippy passed. Ordinary native
Rescan, targeted refresh, OID details and restart passed, exact repository files/modes,
old journals/trusts/no replay preserved, exits 0/integrity/lock reopen. No new trust,
publication or default change. G03 scale/accessibility and remaining gates stay open.

### November 8 G07 publication primitives — isolated Git proof passed

[Primitive contract](reports/G07_PUBLICATION_PRIMITIVES_2026-11-08.md): disposable
local bare fixture proved exact source export/import into an isolated repository,
ancestry-checked publication and expected-old refusal when the server advances even
to another ancestor of the approved source. No user repository/remote involved.
Approved reachable-object export also passed: exact object set in isolated store,
unselected branch blob absent, source config not loaded, managed files/modes exact.
This is Git primitive evidence, not application push authority or G07 closure.
The separate executor checkpoint above now supplies selected application evidence;
prior hook checks remain non-publication actions. G03 ancestor-status fix passed as recorded above.

### November 8 G05 dirty unselected submodule — preservation passed; status follow-up passed

[Omitted-child package](reports/G05_NATIVE_UNSELECTED_SUBMODULE_2026-11-08.md): only
middle selected, dirty actual leaf omitted; native Cancel preserved setup, fresh
separate approval committed only middle/file.txt. Leaf checkout/actual Git metadata,
revoked ancestor metadata and independent control exact; no implicit enrollment or
new grants. Ordinary restart retained selection/dirty leaf/results, writes disabled,
exact files/records/no replay, exits 0/integrity/lock reopen passed. Expanded test-only
tracing identifies renderer Cancel/host close/native close/Destroyed; expected Cancel
signature verified, original intermittent trigger still open. Full desktop gate
91/13/16 plus checks/builds/strict Clippy passed. Native Overview ancestor false-clean finding is now fixed and verified in the
G03 gitlink-status report above. Do not infer full G05/G03 closure.

### November 8 G05 native bulk trust revocation — passed within envelope

[Revocation package](reports/G05_NATIVE_BULK_REVOCATION_2026-11-08.md): native pending
middle/parent batch, main-window parent-only Revoke trust cancelled both proposals;
old review refused confirmation without OS approval. Fresh two-item review showed
one revoked parent; native approval blocked whole batch and cancelled trusted middle
before execution. Zero publications, exact repository bytes/modes and earlier records.
Restored test file edits only; ordinary restart retained revoked parent, cancelled/
blocked histories, read-only gate and no replay; exits 0/integrity/lock reopen passed.
Middle/leaf remain trusted, control untrusted. No new grant/runtime patch. First
preparation's window disappeared before revocation; trace now confirms fresh window
Destroyed → correct cancellation, with destruction trigger still unknown. Preserve
that separate lifecycle finding. Broader G05 selection/accessibility remains open.

### November 8 G05 malformed relationship refusal — Core/native passed within envelope

[Invalid relationship package](reports/G05_INVALID_RELATIONSHIPS_2026-11-08.md): eight
real malformed .gitmodules cases (self/ancestor/escape/absolute/traversal/metadata/
empty component/syntax) refused before batch or operation creation; exact full repo
snapshots preserved. Bulk suite 14 passed / 1 subprocess worker ignored, format and
strict target Clippy passed. Native ancestor-edge case displayed incomplete discovery
and invalid_path, then refused bulk preparation with no review/grant/publication.
Restored only test declaration bytes/mode; complete discovery and ordinary read-only
restart passed, zero new records and all prior files/payloads/trusts exact, exits 0,
integrity/lock reopen passed. No runtime patch. Arbitrary synthetic graph-cycle
fallback is not independently accepted; remaining bulk selection/revocation and
accessibility criteria stay open.

### November 8 G02/G05 completed-batch review transition — traced repeat passed, finding open

[Instrumented diagnostic](reports/G02_REVIEW_TRANSITION_2026-11-08.md): fixture-only
host tracing records request kinds and review IDs, not repository content/credentials.
Repeated completed-child batch → close → select unfinished parent → fresh review,
with separately approved local commits in existing authorized paths. Trace shows old
consumed review cancellation rejected; fresh review built once and completed. No
self-cancellation reproduced. Exact old records/leaf metadata/control preserved;
rebuilt ordinary restart read-only/no replay, both exits 0/integrity/lock reopen passed.
Full desktop validation 91 renderer / 13 default-host / 16 all-feature host, type/lint/
contracts/format/build/strict Clippy passed. Original unexpected cancellation stays
open; no speculative runtime fix. Selected malformed/self/ancestor relationship refusal now recorded above; native bulk revocation now recorded above; remaining native selection/accessibility
and synthetic graph fallback criteria stay open.

### November 8 G05 native three-level chain — selected execution/restart passed

[Native package](reports/G05_NATIVE_THREE_LEVEL_2026-11-08.md): exact authorized
root/middle/leaf, correct child-first order and preparation without staging. Separate
native approvals published leaf, then fresh middle gitlink, then fresh root gitlink;
one commit each, with needs_reauthorization barriers at both ancestors. Descendant
checkout/actual Git metadata, working bytes/modes, full untrusted control and prior
records preserved. Ordinary restart showed three clean selected-chain repos, retained
all outcomes, disabled writes and replayed nothing. Exact files/records, exits 0,
integrity and lock reopen passed. One middle preparation self-cancelled before any
approval/writes. Five pending-trust openings/four close-reopen transitions did not
reproduce it; exact final files/records unchanged, native exit 0. The original
completed-batch transition remains an open lifecycle finding. No product patch;
broader G05 remains partial for invalid/cyclic/selection/accessibility criteria.

### November 8 G05 three-level dependency chain — Core passed

[Three-level package](reports/G05_THREE_LEVEL_2026-11-08.md): new real root/middle/leaf
submodules test proves leaf-first exact dependency order, no staging in preparation,
one leaf commit followed by a fresh middle review, then one middle commit followed
by a fresh root review. Final exact gitlinks, one commit per worktree, preserved child
checkouts and actual Git metadata verified. Bulk suite 13 passed / 1 subprocess worker
ignored, format and strict target Clippy passed. Test-only change; this adds Core
evidence; actual native three-level execution/restart is now recorded above.
Invalid/cyclic input and remaining selection/accessibility cases remain explicit work.

### November 8 G05 native partial staging — passed

[Partial-staging package](reports/G05_NATIVE_PARTIAL_STAGING_2026-11-08.md): existing
authorized parent, index-only bulk review, one exact staged blob committed while
later working edits and an untracked file retained exact bytes/modes. Child checkout
and actual Git metadata, independent control and earlier operation/batch payloads
unchanged; no extra hook runs. Ordinary restart retained index-only mode/message,
showed 0 staged / 1 unstaged / 1 untracked, complete batch and disabled preparation.
Acceptance/ordinary exits 0, exact restart preservation, integrity and writer-lock
reopen passed. No product patch. Earlier fixture current-state oracles are historical
after deliberate new setup. G05 remains partial for other owning criteria.

### November 8 G05 child-hook failure, cancellation and fresh retry — passed

[Native child-hook chain](reports/G05_NATIVE_CHILD_HOOK_2026-11-08.md): exact authorized
pair; hook rejected once and stopped parent, then fresh approved batch reached proven
hooks_running with marker and actual hook/sleep process group. Native Cancel reaped
the group and cancelled both items before publication. Every repository byte/mode,
hook hash and trust scope preserved. Ordinary restart retained exact operation/batch
payloads, hook count two, no replay; read-only gate, exits 0, integrity and lock reopen
passed. Fresh separately approved retry ran the hook a third time, made one child
commit and stopped at needs_reauthorization. A separate parent-only review approved
the exact new gitlink and file; one parent commit completed. Final ordinary restart
showed both clean, retained all four batch outcomes, disabled native preparation and
preserved exact files/records without replay. Child checkout/metadata unchanged by
parent retry; unselected control remained byte/mode exact and untrusted. Final oracle,
integrity and writer-lock reopen passed. No product patch. Broader G05 remains partial.

### November 8 G05 pre-publication crash/cleanup/restart — passed

[Pre-publication package](reports/G05_NATIVE_BATCH_PREPUBLICATION_2026-11-08.md):
same authorized alpha/beta, fresh native exact batch reached publication_pending
then SIGKILL -9. No heads advanced; successor/control exact. First ordinary restart
preserved crash bytes/records without replay. Separate native abort_unpublished
removed only proven owned index/HEAD locks, retained three unreferenced approved
objects and every original file/mode. Final ordinary restart preserved all files and
operation/batch payloads; gates read-only, exits 0, scoped trusts/integrity/lock reopen
passed. No new trust/commit/push. This distinct native case is complete; broader G05
remains partial. Earlier fixture oracles are historical after intentional setup.

### November 8 G05 corrected final preservation — passed

[Corrected final native repeat](reports/G05_NATIVE_BATCH_HOST_CRASH_2026-11-08.md):
original failed baseline preserved; explicitly named post-oracle baseline passed
self-preserving queries and actual ordinary restart. Exact repository bytes/modes,
operation/batch payloads, one commit per selected repository, alpha snapshot, control,
trusts/integrity/lock reopen all passed; Quit 0 and preparation disabled. Earlier
1fef386 overstatement remains withdrawn. Selected actual crash/recovery/beta-only
retry chain is complete within this recorded envelope; broader G05 remains partial.

### November 8 read-only diff index refresh — fixed and fully validated

[Read-only diff fix](reports/G03_READONLY_DIFF_INDEX_2026-11-08.md): Core regression
reproduced index mutation/ConcurrentChange on clean unstaged diff with empty stat
cache. Restricted commands disable automatic index refresh; content-aware numstat
filtering removes raw stat-only false positives. Read suite 17, full Core
61/96/86/231/275, lab 24/0, strict Clippy and full desktop 91/13/16 passed. Exact
source hashes match the separately verified ordinary restart artifact. The oracle
failure and 1fef386 overstatement remain preserved/withdrawn; corrected repeat passed
from an explicitly named post-oracle baseline. Broader G03/native/release gates open.

### November 8 G05 native partial-batch crash/recovery/retry — final preservation pending

[Actual crash package](reports/G05_NATIVE_BATCH_HOST_CRASH_2026-11-08.md): authorized
alpha/beta exact batch reached first ref update then SIGKILL -9. First ordinary
restart displayed interrupted/unconfirmed history without replay; beta/control exact.
Separate native guarded index-only recovery confirmed alpha publication. Fresh
unfinished-only selection/review/approval committed beta alone; alpha byte/mode
snapshot untouched, one commit per selected repository, control exact/untrusted.
Final ordinary restart retained both histories with preparation disabled and exact
operation/batch payloads; beta index differed after the oracle diff refreshed it.
Exact final file preservation is withdrawn pending corrected repeat. Exits 0,
integrity and lock reopen passed. New labels
natively verified. No guard/product patch/push/default change; broader G05 remains open.

### November 8 G05 uncertain publication display — fixed, crash case pending

[Publication checkpoint](reports/G05_UNCONFIRMED_PUBLICATION_2026-11-08.md): actual
post-ref uncertain outcome exposed false certainty in Not published/false labels.
Shared display now distinguishes unconfirmed publication; full desktop 91/13/16 and
all checks passed. Native fault attempt was refused by Core fixture guard (preparation
error, no host crash); fresh native index-only recovery proved the first commit and
preserved beta/control without replay. Actual crash/restart and new-label native
verification remain pending a correctly marked fixture. No safety guard relaxed.

### November 8 full Core matrix — refreshed and passed

[Fresh matrix](reports/G02_G11_CORE_MATRIX_2026-11-08.md): script exit 0, configurations
60/95/85/230/274 passed, exact ignored reasons retained, lab 24/0. Builds, formatter
and strict Clippy passed. Isolated signer restored from checksum-pinned sources;
private outside-repository short temp path resolved agent/SSH fixture restrictions
without weaker guards. Earlier failed attempts retained. No native/release gate waiver.

### November 8 G11 locked license declarations — inventoried

[Inventory checkpoint](reports/G11_LICENSE_INVENTORY_2026-11-08.md): read-only offline
macOS arm64/all-feature Rust plus complete npm lock inventory has 580 records;
repeat byte-identical, all third-party declarations present. npm first-party root
metadata lacks a declaration; no license guessed/changed. Shipping closure, actual
texts/notices and product distribution policy remain open, as do hosted/release gates.

### November 8 G11 private fixture temp — corrected, hosted run pending

[Temp checkpoint](reports/G11_PRIVATE_FIXTURE_TEMP_2026-11-08.md): real SSH rejected
writable /private/tmp ancestry; an in-checkout trial correctly rejected app state.
Short alias to private outside-repository storage passed real SSH and isolated GPG
focused gates. CI now prepares mode-0700 RUNNER_TEMP storage and short macOS alias;
YAML, all shell syntax and actual local workflow-step execution passed. No permission
or product guard weakened. Hosted CI remains unrun; full Core matrix subsequently passed ([report](reports/G02_G11_CORE_MATRIX_2026-11-08.md)).

### November 8 G06 SHA-256 missing objects / packed tracking ref — native passed

[Missing-object package](reports/G06_NATIVE_SHA256_MISSING_OBJECTS_2026-11-08.md):
real native observation and separately approved six-object capture preserved the
repository with three incoming objects still absent. Server stopped before offline
import. Cancel preserved bytes/modes; fresh exact approval imported those objects
and advanced the packed old tracking value via a loose ref plus one appended reflog
line. HEAD/index/work/config/packed-refs unchanged. Ordinary restart retained exact
records/artifacts/files with no replay/network; one scoped trust, exits 0, integrity
and writer-lock reopen passed. No product patch. G06 remains partial for other cases.

### November 8 G05 linked-worktree batch — Core and native passed

[Linked-worktree checkpoint](reports/G05_LINKED_WORKTREE_2026-11-08.md): independent
trust, native untrusted review/Cancel preservation, two exact branch commits,
unselected control/child preservation and unchanged excluded gitlink passed. Specific
new-checkout trust granted after authorization; exact three trusted identities only.
Ordinary restart retained history with preparation disabled and exact files/records,
no replay; integrity and writer-lock reopen passed. G05 remains partial for other
fault/cancellation cases. Fresh full Core matrix also passed; broader native gates remain separate.

### November 8 G05 explicit gitlink exclusions — implemented and verified

[Exclusion checkpoint](reports/G05_BULK_EXCLUSIONS_2026-11-08.md): exact relative-path
entry makes fresh-review-only gitlinks excludable; added paths stay visible/removable.
Enter does not submit review, whitespace preserved, Core eligibility unchanged.
Full desktop 89/13/16 passed. Native add/remove/Enter, exact file-only proposal + Cancel,
byte/mode/prior-record preservation and ordinary persistence/default gate passed;
no product commit/push/new trust. This closes the identified exclusion UI gap, not
all G05. Shared fixture follow-up baseline is in this package, earlier oracles historical.

### November 8 G11 CI prerequisites — implemented, hosted run pending

[Provisioning checkpoint](reports/G11_CI_PREREQUISITES_2026-11-08.md): workflow now
provisions checksum-pinned Git into runner-temporary prefixes and macOS GPG with
explicit test signer/short TMPDIR. Actual isolated local Git source build, SHA256
commit/blob smoke, helper/prerequisite checks, four archive-layout tests, YAML and
shell syntax passed. Hosted CI/Linux/Brew steps and distribution acceptance unrun;
no push/release. G11 is partial. This independent prerequisite work ran while native
G05 exact fixture-trust authorization was pending; remaining gates are not skipped.

### November 8 G05 child/parent native case — complete within envelope

[Final native package](reports/G05_NATIVE_SUBMODULE_2026-11-08.md): child-first commit,
stale-parent stop, fixed fresh gitlink review, parent-only retry and exact message
override passed. Incomplete pre-fix parent proposal cancelled before publication.
Bulk regression failed before/passed after fix; Core suites 11/11, strict Clippy and
full desktop 87/13/16 passed. Repeated ordinary restart retained history/default gate
and exact files/batches with no replay; one child/one parent commit, control untouched,
all exits 0 and lock reopened. First oracle's unsafe git status refresh was reproduced
in a separate copy; corrected optional-locks-disabled oracle passed from explicit
baseline. G05 remains partial for other cases; G04 remains disabled.

### November 8 G03 measured scale / partial scan recovery

[Measured correctness fix](reports/G03_SCALE_PARTIAL_INVENTORY_2026-11-08.md): all six
100/1,000/10,000-repository scans previously discarded results on deadline. Discovery
now returns inspected entries with explicit incomplete/timeout issue; cancellation
contract preserved. Repeated runs retain 88–93 entries near 15 seconds. Workspace 20,
application 27, strict Core Clippy and full desktop gate (87/13/16) passed. Fresh
ordinary native startup/live Rescan showed timeout details with retained inventory;
exact repository preservation, no trust, Quit 0 and lock reopen passed. This closes
the partial-result loss defect, not scale usability: incremental progress and broader
G03 criteria remain open. G02 facility blocks retained; G04 still disabled.

### November 8 G02–G11 continuous execution

User authorized proceeding through G02–G11 and local add/commit after each completed,
validated step without advance prompts. Gate closure still requires its owning
criteria; no blocked case becomes a pass. G04 depends on G02/G03/G05/G06, and G08
on G07, so work follows dependencies when numeric order cannot close a gate.
Live user services/credentials, publication/release and security-sensitive approvals
retain their existing boundaries. AI stays optional under D-048.

G02 disposition audited against later reports: deterministic initial-commit defect
already fixed, bounded restarts/interruption evidence retained, broad reliability
unproven. Current Dock access returned: “Computer Use is blocked from using the app
'com.apple.dock' by your organization's policy.” No alternate route used. Operator
session termination is separate and requires coordinated operator execution.
Continue unblocked G03 coverage while keeping G02 partial; do not repeat accepted
interruption variants solely to accumulate passes.

### November 8 optional AI requirement confirmed

D-048 makes AI an optional enhancement: supported manual workflows require no AI
configuration/service; provider/model failures do not block manual work or discard
drafts. Deterministic user-selected SemVer increments work without AI. Specification
and M6/M7 plan now require no-AI/offline-provider acceptance. This is a requirement
clarification, not implemented AI functionality or a new gate pass.

### November 8 native SHA-256 offline import — complete

[Final import package](reports/G06_NATIVE_SHA256_IMPORT_2026-11-08.md): retained
capture and existing authorized fixture trust reused. Cancel preserved repository;
fresh native exact import approval completed offline, adding only the full SHA256
tracking ref and one reflog entry. Ordinary restart retained complete/cancelled
chronology, writes disabled, no replay/extra request. Exact file/mode, artifact and
journal oracles passed; both exits 0 and writer lock reopened. Server remains stopped.
Objects already existed locally, so missing-object transfer is not accepted by this
case. No product patch/new trust. Native SHA256 missing-object/upstream/check and
other G02/G06 criteria remain open.

### November 8 native SHA-256 observation/download — complete

[Final native format package](reports/G06_NATIVE_SHA256_2026-11-08.md): exact fixture
trust explicitly authorized, untrusted refusal and Cancel/no-network passed. Fresh
native HTTPS observation returned the real full SHA-256 OID; separately approved
isolated download produced a verified standalone bundle (3 objects, 428 bytes).
Repository/remote file bytes/modes remained exact. Ordinary restart retained full
OID/history with writes disabled; no replay/additional request, both Quit exits 0,
writer lock reopened. Test service stopped/key removed. No import/commit/push or
product patch. Selected native SHA-256 observation/download gap closed; native
import/upstream/check topology and other G02/G06 criteria remain partial/open.

### November 7 full Core signing matrix — complete

[Final signing package](reports/G06_FULL_SIGNING_MATRIX_2026-11-07.md): isolated pinned
GPG restored; unchanged full five-configuration Core format/build/test/strict-Clippy
script passed. Tests: default 58, experimental 93, controlled 83, application 227,
all features 271 passed, zero failures. Production OpenPGP/SSH verification and
wrong-key/timeout refusal passed wherever controlled writes are enabled. Declared
worker/facility exclusions remain explicit. Legacy long-TMPDIR agent failure was
reproduced and resolved using process-local short TMPDIR; optional signing passed
explicitly in both configurations and final lab passed 24/24, zero skips. No product
patch or global install. This closes the missing-signer/full Core matrix gap;
G02/G06 remain partial for native/facility/accessibility criteria.

### November 7 current-source automated transport subset — complete

[Final compatibility package](reports/G06_CURRENT_TRANSPORT_MATRIX_2026-11-07.md):
application-services inspection/execution 10/81 passed (2 declared ignored);
all-feature inspection/execution 10/85 passed (4 declared ignored, including parent-
driven subprocess workers). Format, strict target Clippy and source-manifest check
passed. Live disposable HTTPS/SSH, isolated credential state, SHA-256, crash/storage
recovery and check-only hook/dependency coverage refreshed without a product patch.
This closes the selected automated remote-target refresh only. Full five-configuration
signing matrix still needs isolated GPG; native SHA-256/topology and facility-dependent
criteria remain open. G02/G06 remain partial.

### November 7 repository remote-status indicators

[Remote-status report](reports/REPOSITORY_REMOTE_STATUS_2026-11-07.md): the overview
table now shows no remote, missing upstream, local up-to-date/ahead/behind/diverged
counts and unknown/unavailable states, with expandable configuration/comparison/time
details. No automatic remote contact. Legacy snapshots remain unknown until local
rescan. Full desktop checks passed (79 / 13 / 16 tests); Core inspection 12 passed,
one existing filesystem ignore. Selected ordinary native presentation verified.
This is inspection/UI work; G02/G06 and ordinary write enablement remain open.

### November 7 post-load startup follow-up

[Diagnosis, patch and bounded retests](reports/G02_POST_LOAD_STARTUP_2026-11-07.md):
a deterministic initial React commit ordering defect is fixed. Full checks passed
(70 renderer / 13 default-host / 16 all-feature host tests); three native launches,
including first restarts after renderer and host loss, recovered without manual
reload and preserved exact draft/history/repository state with no replay. Diagnostic
launches did not recapture the intermittent blank, so broader startup reliability
remains open; earlier blank evidence is retained. G02/G06 stay partial.

### November 7 repository identity recovery

[Recovery implementation and acceptance](reports/REPOSITORY_IDENTITY_RECOVERY_2026-11-07.md):
local Rescan and review separates verified same-path replacements from historical
identities, retains drafts/history, blocks historical execution and invalidates pending
reviews without transferring trust. Missing/failed inspection remains unresolved.
17 workspace / 26 application tests and full desktop checks (83 / 13 / 16) passed;
selected ordinary native recovery presentation verified. G02/G06 remain partial;
broader lifecycle/accessibility/default-write gates are unchanged.

### November 7 live-workspace identity recovery verification

[Current-workspace follow-up](reports/REPOSITORY_IDENTITY_RECOVERY_2026-11-07.md#current-workspace-follow-up):
explicit native Rescan and review preserved separation of 11 unique available current
repositories and 10 historical identities. Table/Tree/sidebar and read-only database
assertions agree; no duplicate-separation code fix needed. **Discovery remains
incomplete:** metadata inspection failed in two `.tools/cache/sdists-v10` locations.
Zero repository-attention count does not establish scan completeness. Cache traversal
and scan-level issue presentation remain open. No new automated suite; G02/G06 partial.

### November 7 cache discovery / scan-status closure

[Diagnosis and verification](reports/DISCOVERY_CACHE_SCAN_STATUS_2026-11-07.md):
the two cache errors came from empty `.git` markers within valid tagged cache subtrees.
Discovery now skips and lists verified tagged caches, preserving real malformed-marker
errors. Native current-workspace rescan reports complete: 11 repositories, zero issues,
six tagged caches skipped; 10 historical identities remain separated. Last-scan summary,
error paths and exclusions are now visible independently of repository attention counts.
18 Core workspace / 27 application tests passed; full desktop gate passed (86 renderer /
13 / 16), followed by 87 renderer tests and static checks after final integration coverage.
The previously recorded cache/scan-presentation findings are closed within this envelope;
broader G02/G06/accessibility/scale remain open.

### November 7 G02 active-scan main-window Close

[Native case report](reports/G02_CLOSE_DURING_SCAN_2026-11-07.md): fresh ordinary
80-repository fixture; Rescan in progress, live Git children sampled, then native
main-window Cmd+W entered safe shutdown and host exited 0. First restart rendered
without reload and recovered the exact draft; final Quit 0 and state writer lock
reopened. Exact repository file bytes/modes preserved, zero trust/write/remote records.
Selected M3 plan section 12 step 1 passed within this envelope. No source change or
new automated suite. G02 remains partial; G06 remote compatibility is unchanged.

### November 7 G02 active-scan native host loss

[Forced read-interruption report](reports/G02_HOST_LOSS_DURING_SCAN_2026-11-07.md):
fresh ordinary 80-repository fixture, native scan request active and live Git child
proven before exact host-only SIGKILL. SQLite integrity and prior one-entry completed
inventory preserved; first restart rendered and recovered saved draft without reload.
Explicit fresh Rescan completed with 80 entries/zero issues; final Quit 0 and writer
lock reopened. Repository file bytes/modes match baseline; zero trust/write/remote
records. Selected bounded G02 case passed; no code fix or new automated suite.
G02 remains partial, including Dock/session and broader startup/phase coverage;
G06 remote compatibility is unchanged.

### November 7 G02 active-scan Cancel

[Native cancellation report](reports/G02_CANCEL_ACTIVE_SCAN_2026-11-07.md): fresh
ordinary fixture, active Rescan plus live Git child proof, then native Cancel returned
Inspection cancelled and restored controls. Prior one-entry completed inventory retained;
no partial discovery published. Explicit fresh Rescan completed with 80 entries/zero
issues, exact draft retained, normal Quit 0 and writer lock reopened. File bytes/modes
preserved, SQLite valid, zero trust/write/remote records. Selected G02 read-cancellation
case passed within this envelope; no code fix or new automated suite. G02 remains
partial; G06 remote compatibility is unchanged.

### November 7 G02 missing-snapshot explicit fallback exit

[Native fallback-exit report](reports/G02_CLOSE_WITHOUT_SNAPSHOT_2026-11-07.md): fresh
marked fixture and existing non-default test support (desktop writes disabled), one
suppressed close-request delivery, then explicit native Close without saving exited 0.
Saved presentation payload and repository file bytes/modes preserved; ordinary first
restart rendered/recovered saved draft without reload, normal Quit 0 and lock reopened.
Selected exit branch passed, complementing prior Keep app open evidence. Already saved
draft used: no unsaved-edit loss/recovery claim. No product fix or new full automated
suite. G02 remains partial; G06 remote compatibility is unchanged.

### November 7 G06 executable-helper refusal preparation — authorization blocked

[Prepared native case](reports/G06_NATIVE_REMOTE_HELPER_REFUSAL_2026-11-07.md): fresh
disposable `ext::` remote-helper sentinel fixture and acceptance build prepared.
Automatic approval review rejected the native fixture trust grant because specific
access-change authorization is missing. User authorization requested; no bypass used.
Read-only checks confirm unchanged repositories, no helper invocation and zero
trust/write/observation rows. **No remote-refusal pass claimed.** Resume only after
specific fixture authorization; G02/G06 remain partial. No product patch.

### November 7 G06 executable-helper refusal — authorized native pass

[Completed native case](reports/G06_NATIVE_REMOTE_HELPER_REFUSAL_2026-11-07.md): user
explicitly authorized only the prepared disposable worktree trust, resolving the
previous automatic-review block. Native inspection classified/withheld the executable
helper destination; origin/main observation preparation refused unsupported configuration
before transport review. Exact files/modes preserved, sentinel never invoked, exactly
one scoped trust record and zero observation/write/other remote-operation records.
Acceptance and ordinary-restart Quit 0, final oracle unchanged and writer lock reopened.
Selected unsupported-helper refusal passed; no code fix/full automated rerun. G06 remains
partial; unsupported helpers remain unsupported and other G02/G06 limits persist.

### November 7 G06 observation review / configuration drift

[Native stale-configuration case](reports/G06_NATIVE_OBSERVATION_CONFIG_DRIFT_2026-11-07.md):
same explicitly authorized disposable repository, isolated state; renewed scoped trust,
prepared exact observation review, then deliberately changed only remote config.
Old native confirmation retained reviewed URL; approval returned blocked/untrusted
before transport initialization, revoked trust, no OID/time. Ordinary restart retained
identical history without replay; both exits 0, writer lock reopened. Exact expected
repository baselines preserved; original config restored and previous case reverified.
Selected trust/config-drift barrier passed; later URL-digest guard not independently
isolated. No source fix/full-suite rerun. Other G02/G06 criteria remain open.

### November 7 G06 same-OID observation generation drift

[Native generation-barrier report](reports/G06_NATIVE_OBSERVATION_GENERATION_DRIFT_2026-11-07.md):
same authorized disposable repository/isolated state; prepared observation, switched
to an existing alternate branch at the identical OID without config/index changes,
then approved the old exact review. Core blocked stale_review while trust stayed
current, with no transport storage/OID/time. Ordinary restart retained identical
history, both exits 0 and writer lock reopened. Expected file baselines preserved;
fixture restored and earlier case reverified. Selected generation guard passed beyond
stale-trust coverage; URL/auth/tool drift checks not independently isolated. No product
fix/full-suite rerun. Other G02/G06 requirements remain open.

### November 7 G06 linked-worktree upstream native chain

[Native shared-config topology report](reports/G06_NATIVE_LINKED_UPSTREAM_2026-11-07.md):
real approved loopback HTTPS observation, separately approved missing-only upstream
setup completed/updated, exactly two shared-config keys visible from both worktrees.
Both checkouts/common metadata otherwise unchanged; linked branch policy untouched,
main trust revoked, no linked trust grant. Ordinary restart preserved both completed
records/config with only one server GET total. Focused Core topology test passed;
independent file/journal/request oracles passed, both exits 0 and lock reopened.
Selected native linked-worktree upstream success gap closed; fixture restored/service
stopped/key removed. No product patch/full-suite rerun. G06 remains partial; prioritize
named recovery/topology packages rather than additional already-covered guard variants.

### November 7 owned-config-lock recovery package — complete

[Final recovery package](reports/G06_NATIVE_OWNED_CONFIG_LOCK_2026-11-07.md): native
Cancel preserved the ready lock; empty, ready and pre-publication owned-lock recovery
returned aborted/unchanged without replay or config publication. Stale approval and
same-byte foreign-inode replacement were refused; fresh review required manual
investigation. After the fixture operator restored the original owned inode, fresh
native recovery passed. Ordinary restarts retained resolved history with writes
disabled. All four final file/mode/journal/remote oracles and writer-lock reopens
passed; foreign evidence remains intact. No native trust grant or product patch.
This selected package is closed; G02/G06 remain partial. Earlier progress entries
below/above are historical checkpoints, not remaining work in this package.

### November 7 G06 native owned-config-lock recovery — in progress

[Recovery package](reports/G06_NATIVE_OWNED_CONFIG_LOCK_2026-11-07.md): existing Core
subprocess worker created actual interrupted empty/ready/pre-publication locks and
ownership receipts. SIGKILL/phase/config/receipt assertions passed; exact baselines
and non-default artifact provenance retained. Preparation is automated Core evidence,
not native recovery acceptance. Continue Cancel, owned recovery, foreign/stale refusal
and ordinary restart preservation. No product patch; G02/G06 remain partial.

### November 7 owned-lock recovery — ready candidate checkpoint

[Recovery package step 2](reports/G06_NATIVE_OWNED_CONFIG_LOCK_2026-11-07.md): actual
ready-candidate lock preserved on startup and Cancel. Fresh native abort_unpublished
approval removed only owned config.lock, returned aborted/unchanged; config/repo/remote/
transport snapshots otherwise exact. Ordinary restart retained resolved history with
no replay/pending action; both exits 0, writer lock reopened. Empty/pre-publication
and foreign/stale cases still in progress. No product patch; G02/G06 partial.

### November 7 owned-lock recovery — boundary cases passed, negative cases ongoing

[Recovery package steps 3–4](reports/G06_NATIVE_OWNED_CONFIG_LOCK_2026-11-07.md): native
LockCreated empty-lock and PublicationPending candidate-lock recoveries both returned
aborted/unchanged, removed only receipt-proven config.lock, preserved original config
and all other repo/remote/transport files/modes. Both normal exits 0 and writer locks
reopened. Foreign-lock/stale-fingerprint negative cases continue; package not yet closed.

## Status meanings and availability

- **Accepted within envelope:** the recorded milestone criteria passed only for
  the documented environment and exclusions.
- **Implemented / acceptance partial:** source exists and has automated and/or
  selected native evidence; the full product criterion remains open.
- **Not implemented:** required behavior still needs code and acceptance.
- **Deferred / unsupported:** explicitly outside first-release scope or the
  supported execution envelope; this is not a pass.
- **Environment blocked:** a check could not exercise its intended behavior;
  historical passes remain historical and are not replaced by a new pass.

Ordinary desktop builds enable controlled managed Git trust, writes and supported
remote execution through the default `desktop-writes` feature (November 10 decision).
Explicit read-only bundled builds use `--no-default-features --features custom-protocol`.
AI source transfers retain exact native approval. Default Core builds expose no
managed-repository writes.
Local publication means a local ref advanced; it does not mean a server push.
Exact individual publication and guided per-item queues are implemented within
recorded acceptance envelopes. Neither planning nor a completed hook check grants
push authority. No default change, commit, push or release is authorized by this record.

## Current capability and milestone disposition

| Area | Current disposition | Evidence / remaining gate |
|---|---|---|
| P0 baseline | Accepted historical baseline | [P0 report](reports/P0_BASELINE_2026-11-02.md); current checks below do not replace the full baseline matrix |
| M0–M1 Core inspection/workspaces | Implemented and accepted within recorded local envelope | [M1 contracts](architecture/M1_READ_ONLY_WORKSPACES.md), [M1 ledger](ACCEPTANCE_MATRIX.md#10-m1-acceptance-gates--completed-local-implementation); desktop/scale criteria remain separate |
| M2 controlled stage/unstage/commit | Implemented and accepted within bounded macOS/APFS envelope | [M2 contracts](architecture/M2_CONTROLLED_WRITES.md), [M2 ledger](ACCEPTANCE_MATRIX.md#11-m2-acceptance-gates--controlled-writes); default desktop exposure enabled; broader G01–G04 acceptance remains partial |
| M3 desktop/lifecycle/reviews | Implemented; acceptance partial | [M3 ledger](ACCEPTANCE_MATRIX.md#12-m3-acceptance-gates--implementation-with-native-gates-open), [close/Quit](reports/M3_NATIVE_CLOSE_QUIT_2026-11-02.md), [review lifetime](reports/M3_REVIEW_LIFETIME_2026-11-02.md), [geometry](reports/M3_MIXED_DPI_WINDOW_RESTORE_2026-11-02.md); G01–G04 |
| Branding/About | Selected automated/native visual acceptance passed | [Branding report](reports/COMMIT_COMMAND_BRANDING_ABOUT_2026-11-03.md); internal storage identifier preserved; release branding/signing work remains G11 |
| M4 graph/activity/audit | Implemented; acceptance partial | [Graph/activity](reports/M4_READ_ONLY_PROGRESS_2026-11-02.md), [unified audit](reports/M4_UNIFIED_OPERATION_AUDIT_2026-11-02.md); G03, G10 |
| M5A bulk local commits | Implemented; acceptance partial | [Bulk implementation](reports/M5A_LOCAL_BULK_PROGRESS_2026-11-02.md), [native bulk](reports/M5A_NATIVE_BULK_2026-11-02.md); G05 |
| M5B inspection/authentication/fetch/upstream | Implemented for supported subset; acceptance partial | [Fetch native chain](reports/M5B_NATIVE_FETCH_ACCEPTANCE_2026-11-02.md), [authentication](reports/M5B_AUTHENTICATION_COMPATIBILITY_2026-11-02.md), [SSH](reports/M5B_SSH_AGENT_TRANSPORT_2026-11-02.md), [upstream](reports/M5B_GUARDED_UPSTREAM_2026-11-02.md); G06. Artifact initialization finding [fixed](reports/M5B_ARTIFACT_INITIALIZATION_FIX_2026-11-02.md) |
| M5C push preview/child evidence/planning | Implemented; selected native acceptance passed | [Relative child evidence](reports/M5C_RELATIVE_CHILD_ACCEPTANCE_2026-11-02.md), [multi-level planning](reports/M5C_DEPENDENCY_PLANNING_2026-11-03.md), [native planner](reports/M5C_PLANNER_NATIVE_ACCEPTANCE_2026-11-03.md); G03, G06–G08 remain; no publication |
| M5C independently reviewed pre-push hook check | Implemented; selected automated/native acceptance passed | [Check report](reports/M5C_REVIEWED_PUSH_CHECKS_2026-11-03.md); hook approval only, unpublished history/no replay; selected native interruption/manager-refusal/bypass cases passed; broader G06 compatibility remains |
| M5C individual push publication | Implemented; selected acceptance passed | Separate exact native publication, isolated ancestry/expected-old transport and observation recovery; broader G07 acceptance remains |
| M5C bulk push publication | Guided per-item queue implemented; selected automated/native acceptance passed | Durable child-first scheduling, fresh post-child evidence, partial stop/retry and no-replay restart; [G08 report](reports/G08_GUIDED_PUSH_QUEUE_2026-11-08.md); broader acceptance remains |
| M6 AI foundation | Configuration/storage/editor, secure auth and native-reviewed source suggestions implemented; acceptance partial | [Settings model](reports/G09_AI_SETTINGS_MODEL_2026-11-08.md); [storage/editor](reports/G09_AI_SETTINGS_STORAGE_2026-11-08.md); [provider diagnostics](reports/G09_AI_PROVIDER_DIAGNOSTICS_2026-11-08.md); [secure auth](reports/G09_AI_SECURE_AUTH_2026-11-08.md) implemented; [assisted source suggestions](reports/G09_AI_ASSISTED_MESSAGES_2026-11-08.md) implemented; hierarchy/automatic and real-provider acceptance remain G09 |
| M7 AI commits / SemVer Actions | Typed provider-free SemVer invocation/recovery implemented; assisted messages implemented; automatic commits pending | [G09 Action report](reports/G09_SEMVER_ACTION_2026-11-08.md); separate stage/commit reviews, selected exact native evidence; broader recovery usability/native coverage and bounded AI authority remain |
| M8 backup/retention/usability/release | Partial foundations only | Atomic storage, migrations and development bundles exist; general backup/restore, retention and release acceptance do not: G03, G10–G11 |

## Open acceptance and implementation gates

These stable gate IDs group the acceptance queue; they supplement, rather than
renumber, original requirement IDs and M3-A criteria. G01 is **accepted within its
recorded envelope**; G02–G11 remain **open**. Accepted rows remain here for traceability. Close a row only
with criterion-level evidence, exact source/artifact/features where applicable,
and explicit disposition of exclusions. A facility block is not a waiver.

| Gate | Remaining work and closure evidence | Requirement / plan ownership |
|---|---|---|
| G01 — native authority/review fidelity | **Accepted within envelope, November 7:** remaining actual-webview caller/page/origin and pending/issued-confirmation reload cases passed using non-default native test support. Combined with prior exact-content/signing/lifecycle evidence, the enumerated G01 cases are closed. [Closure report](reports/G01_NATIVE_SUPPORT_G02_RECOVERY_2026-11-07.md). Accessibility, broader compatibility and ordinary write exposure remain G03/G06/G04. | M3-A12–A15; SEC-001–012, SEC-018–020; M3A/B |
| G02 — lifecycle and recovery | **Partial; facility blocks retained.** Initial React commit ordering defect fixed; renderer/host-loss and bounded restart/no-replay cases passed ([post-load report](reports/G02_POST_LOAD_STARTUP_2026-11-07.md)). Earlier intermittent blank remains counterevidence; capture diagnostics on recurrence before reload, rather than treating document Finished as proof of content. Actual Dock access rechecked November 8: organization policy still denies com.apple.dock; no surrogate accepted. Operator session termination and specifically uncovered phase/recovery cases remain open. | M3-A16–A17; SEC-013–017; WS-014; M3B |
| G03 — accessibility, layouts and scale | Full VoiceOver/focus/review/dialog coverage, long/byte paths and large content, narrow/zoom layouts, display disconnect/reconfiguration, measured inventory/history/monitoring budgets. Selected contrast/theme/1×–2× geometry passes remain valid only for their recorded cases. | UI-001–005; M3-A04/06/07/19; M4; M8B |
| G04 — ordinary write enablement | Reconcile all applicable safety prerequisites with G01–G03/G05–G06 and support exclusions, then obtain the explicit default-change decision. Validate the exact ordinary artifact with supported writes and independent approval. No current gate completion is inferred. | D-045; M3C; RELEASE-002 |
| G05 — native bulk commit failures/dependencies | **Partial:** real native child-first publication, stale-parent stop, corrected fresh gitlink review, parent-only retry/message override and ordinary restart passed ([package](reports/G05_NATIVE_SUBMODULE_2026-11-08.md)). Native linked-worktree independent trust/two-branch execution/restart also passed ([checkpoint](reports/G05_LINKED_WORKTREE_2026-11-08.md)). Selected actual host-crash/partial-publication/index recovery/unfinished-only retry also passed ([package](reports/G05_NATIVE_BATCH_HOST_CRASH_2026-11-08.md)). Native pre-publication host crash/owned-lock cleanup/restart also passed ([package](reports/G05_NATIVE_BATCH_PREPUBLICATION_2026-11-08.md)). Actual child-hook rejection/proven-running Cancel and separately approved supported-hook child retry/fresh parent gitlink approval/ordinary restart passed ([package](reports/G05_NATIVE_CHILD_HOOK_2026-11-08.md)); Native index-only partial staging and ordinary restart also passed ([package](reports/G05_NATIVE_PARTIAL_STAGING_2026-11-08.md)); other required dependency/selection and broader accessibility criteria retain their owning evidence requirements. Explicit relative-path exclusions for fresh-review-only gitlinks are implemented and natively verified ([checkpoint](reports/G05_BULK_EXCLUSIONS_2026-11-08.md)); Core eligibility remains authoritative. Preserve completed/unselected content and never replay successful items. | GIT-013; BULK-001–007; M5A |
| G06 — supported remote/check compatibility | **In progress:** selected native executable-helper refusal after explicit fixture trust passed ([report](reports/G06_NATIVE_REMOTE_HELPER_REFUSAL_2026-11-07.md)); helpers remain unsupported.  native supported pre-push Cancel review, Cmd+Q, main-window Close and direct AppKit termination during proven running execution passed, including process cleanup and restart without replay ([AppKit report](reports/G02_G06_RUNNING_PUSH_CHECK_APPKIT_2026-11-07.md)). Actual running-check main-renderer loss now passed with host survival/cancellation/no replay; a subsequent normal restart blank reopens G02 presentation ([renderer report](reports/G02_G06_RUNNING_PUSH_CHECK_RENDERER_LOSS_2026-11-07.md)). Earlier running-check host crash and first-restart recovery passed in their fixture; original hook survival remains explicit ([startup retest](reports/G02_STARTUP_READINESS_2026-11-07.md)). Selected native detected-manager Execute refusal, OS-declined bypass and fresh approved bypass passed with zero hook execution and restart/no replay ([bypass report](reports/G06_NATIVE_MANAGER_BYPASS_2026-11-07.md)); unsupported managers remain unsupported for Execute. Operator SecurityAgent choices, broader fetch/upstream topology/recovery, planner cycle/accessibility/scale and direct SCP facility block retain their limits. | M5B/C; PUSH-002; AUTHENTICATION.md; M8B |
| G07 — individual push | Exact separate publication approval, isolated ancestry/expected-old transport, durable uncertainty and fresh-observation recovery implemented with selected automated/native evidence. Actual native host-loss/remote-completed/observation-recovery case passed ([report](reports/G02_G07_NATIVE_PUSH_HOST_CRASH_2026-11-08.md)); finish authenticated publication, broader topology/native interruption/recovery and usability acceptance. Existing hook checks never become publication tokens. | PUSH-001/003/005/006; INV-009; M5C |
| G08 — bulk push | Guided per-item queues are implemented with selected automated/native stop/retry/history evidence. Remaining acceptance includes uninterrupted native multi-level execution, authenticated topology, phase-specific interruption and accessibility. Preserve fresh per-item authority, actual child availability, no implicit selection and partial-success/fresh-retry rules. See completion packages P08 and QUEUE01–QUEUE09. | PUSH-004–006; INV-014; M5C |
| G09 — AI and Actions | Optional settings/profiles/templates, secure provider auth, exact context/source transfer, assisted candidates and guarded SemVer preview/execution/recovery are implemented with selected acceptance. Remaining implementation includes hierarchical analysis, separately bounded automatic authorization, persistent safe AI audit, unsaved-editor repository AI testing and stored definition/import integration; broader provider/native usability acceptance remains. See completion packages P09–P12. | AI-001–017; ACT-001–003; INV-010–013; M6/M7 |
| G10 — backup/portability/retention | Versioned export/import, consistent manual/rotating backups, previewed safe restore, malicious/interrupted archive tests and protection of unresolved recovery. Decide proposed 90-day history/ten-backup defaults before implementing cleanup. Migration backups alone do not meet this gate. | STATE-005–007; ACT-004; M4C/M8A |
| G11 — reproducible release | Git provisioning/macOS signing-tool workflow now implemented and locally smoke-tested ([checkpoint](reports/G11_CI_PREREQUISITES_2026-11-08.md)); actual hosted/Linux/Brew results remain pending; resolve OS/architecture/tool delivery/identifier/signing/update policy; verify licenses/notices and exact packaged install/upgrade/migration/uninstall, signing/notarization and release invariants. A development bundle is not a release. | RELEASE-001–005; INV-001–015; M8C |

### Latest G02/G06 running-hook renderer loss / startup counterevidence — November 7, 2026

[Renderer-loss report](reports/G02_G06_RUNNING_PUSH_CHECK_RENDERER_LOSS_2026-11-07.md)
and [evidence](validation/g02-g06-running-push-check-renderer-loss-2026-11-07/README.md)
record actual main WebKit process SIGKILL after independent running-hook proof.
Same host survived; original renderer/hook/child were absent 18.7 seconds after hook
start, before natural finish. Main reset/reloaded, obsolete review closed, and exact
draft/cancelled unpublished history recovered automatically with no replay.

**Reliable startup presentation is reopened.** After a later normal Quit/restart,
the window was blank across native AX/screenshot observations despite a Finished
page-load event and successful initial show. Explicit native Reload main renderer
restored draft/history in that same host without replay. Page completion/display
therefore cannot by itself establish painted application content. The earlier
readiness patch remains, but its passes do not close this broader startup symptom.
Next: investigate the post-load blank restart using this retained fixture/evidence.

Exact repository/remote/draft/journal/request assertions passed after renderer
recovery, restart and final cleanup; one hook invocation and one discovery GET.
Both app exits were 0; original hook/child remained absent past natural finish,
owned service stopped and generated TLS key removed. No product source changed;
prior automated validation is historical. G02/G06 remain open; this checkpoint
accepts running renderer-loss cancellation, not automatic restart presentation.

### Latest G06 native manager refusal / explicit bypass — November 7, 2026

[Native bypass report](reports/G06_NATIVE_MANAGER_BYPASS_2026-11-07.md) and
[evidence](validation/g06-native-manager-bypass-2026-11-07/README.md) close the
selected detected-manager refusal/decline/bypass cases. A fresh fixture hook had a
Husky signature installed before trust. Execute returned unsupported state before
proposal/approval. Explicit BypassAll was visible in authoritative and OS reviews;
OS Cancel left a cancelled record and consumed review. A fresh separately approved
BypassAll check completed with hook not passed and no publication. Neither ran the
hook. This is detector/refusal coverage, not installed-manager Execute support.

Native history and first restart preserved distinct cancelled/complete bypass
records and the exact draft without retry/publication authority or replay. All
repository/remote bytes/modes and journal/request assertions passed at refusal,
decline, approval, restart and final cleanup: zero hook markers and one observation
GET. Both app exits were 0; exact owned service stopped and generated key removed.
No product source changed; unchanged validated artifact reused. Prior automated
matrix remains historical. One early final-phase assertion was run while pending;
the unchanged oracle passed after native completion, with the timing note retained.

G06 remains open for broader authentication, fetch/upstream topology/recovery,
planner cycle/accessibility/scale and SCP facility limits. G02 dispositions are
unchanged; actual Dock/session and remaining phase-specific recovery remain open.
Ordinary writes stay disabled; no publication authority was added.

### Latest G02/G06 direct AppKit termination during running hook — November 7, 2026

[AppKit report](reports/G02_G06_RUNNING_PUSH_CHECK_APPKIT_2026-11-07.md) and
[evidence](validation/g02-g06-running-push-check-appkit-2026-11-07/README.md)
record the fixed native test action calling actual NSApplication termination from
the active review while a supported hook was independently proven running. Audit
confirms request, dispatch, delegate interception and native return in order, with
no attached confirmation sheet. The host exited 0; exact host/hook/child were absent
at 43.7 seconds after hook start, before natural finish. No external host/hook/child
signal was sent. The raw check became cancelled/not passed/unpublished.

First restart restored the exact draft and cancelled history, released the state
lock and caused no hook/request/operation replay. Exact working/bare repositories,
journal, observation and request evidence were preserved through final cleanup.
One hook invocation and one discovery GET; no completion marker beyond the natural
interval. Both app exits were 0, owned service stopped, generated TLS key removed.
No product code changed; exact artifact/source provenance matched the prior full
desktop gate. Earlier automated counts remain historical, not a new test run.

This direct AppKit running-check case passed. G02/G06 remain open for actual Dock
(policy blocked), operator-controlled session termination, remaining phase-specific
recovery requirements, native bypass/manager and broader compatibility criteria.
Ordinary writes stay disabled; hook acceptance grants no publication authority.

### Earlier G02/G06 main-window Close during running hook — November 7, 2026 (historical)

[Close acceptance report](reports/G02_G06_RUNNING_PUSH_CHECK_CLOSE_2026-11-07.md)
and [evidence](validation/g02-g06-running-push-check-close-2026-11-07/README.md)
record native Window > Close Window (`performClose:`) with the main window foreground
and the authoritative review still open. Independent proof immediately before Close
found raw running/unpublished state and exact live host/hook/child. Host exited 0;
all three PIDs were absent 33.6 seconds after hook start, before its 45-second
natural finish. No external host/hook/child signal was sent.

The check became cancelled/not passed/unpublished. First restart restored the exact
draft and cancelled history, released the state lock, and added no hook invocation,
request or operation. Exact repository/bare-remote snapshots were preserved through
final cleanup beyond the natural hook interval. The service stopped and generated
TLS key was removed. No product source changed; artifact/source hashes matched the
startup-readiness checkpoint. Prior full automated checks remain historical.

This main-window Close case passed; the subsequent direct AppKit running-check
pass is recorded above. G02/G06 remain open. Actual Dock is
policy blocked; session termination and native bypass/manager criteria remain
separate. Ordinary writes stay disabled; no publication authority was added.

### Earlier G02 startup readiness fix and native retest — November 7, 2026 (historical)

[Fix/retest report](reports/G02_STARTUP_READINESS_2026-11-07.md) and
[evidence](validation/g02-startup-readiness-2026-11-07/README.md) record the
reproduced transient startup exposure, committed baseline `a89ecc6`, and fix:
keep the initial main window hidden until its document finishes loading. An
HTML startup status covers React mounting. Later renderer navigation does not
repeat initial display; approval and write defaults are unchanged.

A fresh actual running-hook SIGKILL test passed **on the first restart**, restoring
draft and interrupted/not-passed history with no replay. A further idle-host crash
also passed first restart. Normal renderer reload and actual renderer-process loss
recovered. Three audited startups were hidden through document completion, each
with exactly one initial display. Repository/remote/draft/request/journal oracles
passed. Original hook survived and finished naturally, invocation count one.

Fresh full desktop gate passed: 69 renderer / 13 default-host / 16 all-feature-host
tests, type/contract/format/lint, strict Clippy and debug/release builds. Fixture
service/key cleaned up. Premature-display finding was closed within that envelope;
the newer blank normal restart above reopens reliable startup presentation despite
Finished/display audit. G02/G06
remain open for other criteria. The subsequent main-window Close pass is recorded
above; direct AppKit termination, Dock/session and bypass/manager remain separate.

### Earlier G02/G06 running-hook host crash — November 7, 2026 (historical)

[Crash report](reports/G02_G06_RUNNING_PUSH_CHECK_CRASH_2026-11-07.md) and
[evidence](validation/g02-g06-running-push-check-crash-2026-11-07/README.md)
record actual SIGKILL to the exact host after independent running-phase/PID proof.
Original hook and child survived, then finished naturally; no external child signal.
Exact repositories, remotes, draft and journal/request counts were preserved with
one invocation and no replay/publication. Raw check stays running/unpublished;
audit projects interrupted, not passed, with fresh approval required.

First valid restart showed a blank window; second clean restart restored draft and
interrupted history. First-restart presentation was open at that checkpoint; the
startup readiness fix/retest above supersedes that finding. Other running-check entry points,
Dock/session and G06 bypass/manager criteria remain separate. No product changes;
artifact/source hashes verified, prior automated tests historical. Fixture cleaned
up; ordinary writes stay disabled. G02/G06 remain open.

### Earlier G02/G06 Quit during running hook — November 7, 2026 (historical)

[Quit acceptance report](reports/G02_G06_RUNNING_PUSH_CHECK_QUIT_2026-11-07.md) and
[evidence](validation/g02-g06-running-push-check-quit-2026-11-07/README.md) record
actual Cmd+Q from the review window after a real pre-push hook was independently
proven running. Host exited 0, hook and child stopped without an external kill,
and the check became cancelled/unpublished/not passed before natural hook finish.
Exact draft and repository/bare-remote snapshots were preserved. Restart acquired
the state lock, restored the draft and cancelled history, and caused no hook,
request or operation replay. Fixture server stopped and generated TLS key removed.

No product code changed. Reused artifact/source hashes match the prior full desktop
gate; this step executes native/independent acceptance checks rather than claiming
a fresh automated matrix. The preceding nine Core pre-push tests remain historical.
At that checkpoint the next shared case was native host crash, now recorded above,
with hook/child lifetime, interrupted-history and no-replay assertions. Main Close,
direct AppKit/Dock/session entry points during this exact operation are not inferred.
Ordinary writes remain disabled; no push or publication authority was added.

### Earlier G02/G06 native running-hook cancellation — November 7, 2026 (historical)

[Running pre-push-check report](reports/G02_G06_RUNNING_PUSH_CHECK_CANCEL_2026-11-07.md)
and [evidence](validation/g02-g06-running-push-check-2026-11-07/README.md) establish
actual native cancellation of a supported pre-push hook after independent proof
of running phase and hook/child start. Cancel review stopped both exact processes;
durable result was cancelled, unpublished, hook not passed. Native history and
restart retained no retry authority or replay. One explicit worktree trust, one
completed loopback observation, one hook invocation and one cancelled check; every
repository/remote byte/mode and request-log byte preserved after execution/restart.
No production code or write default changed and no push ran.

Nine focused Core push-check tests passed. Reused exact prior validated artifact/
source hashes; earlier full desktop gate retained, not relabeled as rerun. Fixture
apps/server stopped and generated private TLS key removed. Next shared case is
Quit during a proven running pre-push check, followed by native host-crash cases.
Native bypass/manager interaction remains G06; Dock/session restrictions remain G02.

### Earlier G02 unanswered-sheet termination — November 7, 2026 (historical)

[Active-sheet report](reports/G02_ACTIVE_SHEET_TERMINATION_2026-11-07.md) and
[evidence](validation/g02-active-sheet-2026-11-07/README.md) resolve the delivery
uncertainty in the prior checkpoint. A fixed feature-only request now audits actual
native dispatch, attached-sheet state and return. AppKit received two calls with
unanswered trust/commit confirmations attached and returned without consulting the
termination delegate. Both sheets remained responsive/unanswered; no implicit grant
or publication occurred. The run-loop starvation hypothesis was not confirmed:
observed dispatch used default mode. No production behavior changed.

After explicit native Cancel, the commit was durably cancelled before publication.
A fresh sheetless terminate request entered guarded shutdown and exited 0. Restart
restored the exact saved draft without replay, and all four repository byte/mode
snapshots matched baseline. Full desktop checks passed: 69 renderer, 13 default /
16 all-feature host tests and all static/Clippy/build checks; default feature/
artifact isolation and nonfixture refusal passed. Ordinary writes remain disabled.

This closes the selected active-sheet **safe deferral/decline/retry** evidence gap,
not immediate termination or automatic sheet cancellation. Remaining G02 steps:
actual Dock Quit (policy blocked), operator-controlled session logout/restart, and
running-execution interruption combinations shared with G06. No Dock/session or
unrun native phase is waived. A future immediate-exit requirement needs separate
supported design/acceptance.

### Earlier G02 guarded native termination — November 7, 2026 (historical)

[Fix/retest report](reports/G02_GUARDED_APPKIT_TERMINATION_2026-11-07.md) records
public applicationShouldTerminate interception into the existing save/cancel/Core
shutdown handshake, with original Tao delegate forwarding. Native missing-delivery
fallback now supports Keep app open and a successful direct termination retry;
restart restored the draft, representative forwarding probe passed and clean direct
termination exited 0. All repository bytes/modes remained identical, zero grants/
operations. Full desktop gate passed (69 renderer, 13 default / 16 all-feature host,
static/Clippy/debug/release checks). Ordinary writes remain disabled.

G02 remains open for Dock Quit (policy blocked), unanswered-sheet direct native
termination (request did not reach the decision callback), operator-controlled
session logout/restart and remaining active-execution combinations with G06.
Do not substitute own-app fixture requests for actual Dock/session evidence.
Completed checkpoint commits: `aad07d2` (G01/renderer recovery) and `904a8e5`
(direct termination diagnostic); the validated fix is recorded by the commit
containing this checkpoint.

### G02 direct AppKit termination diagnostic — November 7, 2026 (historical)

[Diagnostic report](reports/G02_APPKIT_TERMINATION_2026-11-07.md) records a corrected
native terminate request that exited while bypassing the guarded close handshake.
An armed missing-close-delivery fault was never consumed; no fallback appeared.
No repository mutation, grant or operation occurred. This is a failed graceful
termination criterion, not acceptance. A first test-control reentrancy hang was
identified separately by stack sample and corrected through run-loop scheduling.
Next: intercept the cancellable AppKit termination decision and route it into the
existing shutdown handshake, then natively retest. Dock remains policy blocked.

### Latest G02 actual renderer-loss checkpoint — November 7, 2026

[Renderer-loss report](reports/G02_ACTUAL_RENDERER_LOSS_2026-11-07.md) and
[evidence](validation/g02-renderer-loss-2026-11-07/README.md) record two actual
WebKit main-renderer terminations through a fixed native acceptance action targeting
only the PID reported by the owned fixture WKWebView. Host survived, Tauri's normal
termination handler reloaded main, and the existing reset path invalidated review
authority. Pending trust granted nothing; an issued commit confirmation was
cancelled before publication. The saved draft returned, a fresh OS dialog worked,
and normal Quit/restart preserved state without replay. All four repositories'
bytes/modes remained unchanged. No production lifecycle behavior changed.

Full desktop validation passed: 69 renderer tests, 13 default / 16 all-feature host
tests and all static/strict-Clippy/build checks. Default feature/artifact isolation
and nonfixture refusal passed. No full Core/signing/remote matrix rerun is claimed.
Dock access via Computer Use was denied by organization policy on `com.apple.dock`;
no alternate UI driver was used. Dock remains blocked, system/session termination
and further required phase-specific native cases remain open. Actual renderer loss
is accepted only for the observed auto-reload path; persistent restart failure and
unsaved keystroke durability are not established. Ordinary writes remain disabled.

### Earlier G01 closure / G02 recovery — November 7, 2026 (historical)

[Closure/recovery report](reports/G01_NATIVE_SUPPORT_G02_RECOVERY_2026-11-07.md),
[G01 evidence](validation/g01-native-support-2026-11-07/README.md) and
[G02 evidence](validation/g02-crash-boundaries-2026-11-07/README.md) close the
remaining G01 live-native cases with fixed, non-default test support. Actual
webviews rejected cross-caller IPC; wrong main page/origin navigations were denied.
Real main reload invalidated pending trust and an issued native commit confirmation,
with no unauthorized grant/publication and all repository bytes preserved.
No IPC permission, independent-approval rule or default write policy changed.

G02 additionally passed one suppressed close-delivery fallback with Keep app open,
persisted draft and successful Quit retry, and actual native process SIGKILL at
`publication_pending` / `reference_updated`. Restart did not replay either operation.
Separately approved recovery aborted the first without publication and reconciled
the already published second commit/index without creating a second commit.
A refused incompatible crash fixture is recorded separately; Core's guard was
not relaxed. Actual renderer-process loss and Dock/system termination remain open.

Final checks passed: 69 renderer tests, 13 default / 15 all-feature host tests,
24 Core application cases, the existing aggregate 13-checkpoint Core crash matrix,
all desktop static/build/strict-Clippy checks, and default feature/artifact isolation.
Native artifacts/source chronology and exclusions are explicit in the report.
G01 acceptance is bounded; M3 overall, G02–G11 and ordinary write enablement stay open.

### Earlier G01/G02 continuation — November 7, 2026 (historical)

[Latest report](reports/G01_G02_CONFIRMATION_LIFECYCLE_2026-11-07.md) and
[evidence](validation/g01-g02-2026-11-07/README.md) record three native lifecycle
findings and fixes: synchronous confirmation blocked Quit, closing an async sheet's
parent leaked its dialog lease, and replacement raced window-label teardown.
Confirmation now uses a responsive native sheet, cancels that sheet explicitly on
review teardown, and waits for Destroyed before revalidating/showing a replacement.
No renderer authority, IPC command or write-default change was added.

Actual native tests passed unusual filename/exact SSH-signed publication with a
direct pre-commit hook, outstanding-confirmation Quit, workspace change followed
by a fresh native dialog, main close with outstanding confirmation, and direct
replacement with a different review ID and fresh native decline. Independent
checks found exactly one fixture publication/hook run, five local operation records
(one complete, four cancelled), no tooling trust grant and preserved excluded state.
Full final desktop gate passed (69 renderer, 13 host per configuration and all
static/build checks); Core application suite passed 24 cases. Generated private
key removed and fixture processes exited. Ordinary writes remain disabled.

At this earlier checkpoint G01 remained open for live renderer reset/reload and
native caller/page adversarial cases; automated coverage is not substituted for those native cases. G02 remains
open for missing-renderer fallback, Dock/system termination and additional recovery
phases. Preserve earlier historical results and support exclusions; no full signing,
crash, platform or release gate is closed by this continuation.

### First G01 execution checkpoint — November 7, 2026

[Acceptance report](reports/G01_REVIEW_AUTHORITY_2026-11-07.md) and
[durable evidence](validation/g01-2026-11-07/README.md) record the first G01 pass.
No production code or write default changed; two Core integration tests and expanded
native origin assertions were added. The separate disposable native build published
one explicitly approved local commit with exact hostile-text message/staged blob
bytes and preserved excluded content. Decline, same-OID branch drift, replaced
review and workspace change produced no additional publication. The fixture exited.

This first checkpoint remains historical. The continuation above supersedes its
then-open unusual-filename, selected signer/hook and outstanding-confirmation
close/replacement observations. Mock-runtime/Core coverage is not relabeled as
native acceptance. All gate-level dispositions are in the current rows above.

### Supported scope and explicit deferrals

First release is macOS; Linux/Windows are deferred by D-040 and are not accepted
platforms. Controlled writes retain the M2 local APFS/files-ref/ordinary-index
limits. Advanced indexes, unsupported hook managers/reference-transaction Execute,
interactive/hardware-token signing, hostile same-user manipulation and OS/power-loss
durability are not implicitly supported by process-crash tests. Non-UTF-8 filesystem
acceptance remains unproven on the current host. See the M2 contract for exact rules.
SSH uses the documented selected Ed25519 agent identity/strict known-host policy;
HTTPS uses the documented scoped helper/TLS policy. Unsupported configurations must
fail closed. Pull, branch mutations, destructive Git operations, hunk staging,
individual hook toggles and arbitrary Actions remain deferred/outside v1.
AI, SemVer Actions and backups remain in the approved product scope, not deferred.

## Next work and exit conditions

1. **Acceptance closure:** G01 is accepted within its recorded envelope. Continue
   G02 with permitted Dock/system termination and remaining required phase-specific
   cases, then G03/G05/G06; attach
   criterion-level native evidence and classify facility blocks. Restore the full
   reproducible automated matrix with isolated GPG and permitted local transport
   fixtures. Do not reopen selected historical passes merely because an old intro
   still lists them as untested. Default desktop writes are enabled; G04 remains an acceptance gate.
2. **G07 remaining acceptance:** expand exact individual publication and recovery
   acceptance to supported authenticated loopback transports, topology and native
   interruption cases. The implementation and selected fixture evidence are above;
   never use a user repository or private server as an implicit acceptance target.
3. **G08 remaining acceptance:** guided per-item coordination now has selected
   dependency/partial-stop/fresh-retry evidence. Expand native uninterrupted multi-level,
   authenticated topology, phase-specific interruption and usability coverage.
4. **G09:** retained-file inspection is implemented and selected acceptance passed;
   finish provider/TLS/editor and assisted-workflow acceptance, then hierarchical
   analysis and bounded automatic commits under separate authorization contracts.
   Existing secure auth, exact context and assisted candidates are foundations, not
   new implementation tasks. Preserve provider-free manual workflows.
5. **G10–G11 and remaining G03:** complete state portability, measured usability and
   exact macOS release acceptance. Strengthen backup primitives before later schema
   migrations; do not postpone data safety until packaging.

### G02/G06 queue audit after linked-worktree upstream acceptance

Recent guard/interruption cases are evidence of distinct barriers, not completion of
whole gates. Group completed variants; do not repeat them without a new failure,
source change or an explicitly uncovered requirement. Choose the next package using
its owning criterion and state what its completion closes before execution.

| Package | Current disposition | Concrete remaining work / handling |
|---|---|---|
| G02 shutdown/read interruption | Selected graceful Close, active scan Cancel/host loss, running-hook lifecycle and both missing-snapshot choices passed within reports | Do not add more equivalent scan/close variants merely to accumulate passes |
| G02 startup | Initial React commit ordering defect fixed; bounded native restarts passed; earlier intermittent blank retained | Capture diagnostic state on an actual recurrence before reload; bounded clean launches do not prove a universal cause or guarantee |
| G02 real OS entry points | Direct AppKit covered; actual Dock policy blocked; operator session termination distinct | Keep blocks explicit; no surrogate or destructive session termination without operator coordination |
| G06 upstream topology | Selected missing-only setup/recovery history and now native linked-worktree success passed | Native owned-config-lock recovery package now passed (Cancel, empty/ready/pre-publication abort, stale/foreign refusal and ordinary restart); broader named topology/failure/accessibility criteria remain open |
| G06 fetch/auth/format compatibility | Selected real HTTPS/SSH/auth chains have dated evidence; identified artifact initialization race already fixed | Full current-source five-configuration Core signing/transport matrix passed with restored isolated GPG. Native SHA-256 HTTPS observation/download now passed; native offline import with existing objects now passed; missing-object/packed-expected-old import now passed ([checkpoint](reports/G06_NATIVE_SHA256_MISSING_OBJECTS_2026-11-08.md)); upstream/check topology remains; fresh five-configuration Core matrix passed after private-temp correction; do not reopen the already diagnosed artifact race without new counterevidence |
| G06 OS credentials / other transport exclusions | SecurityAgent choices and direct SCP facility limits retain prior blocks | Operator/facility-dependent work is not an unattended pass; unsupported helpers stay unsupported |
| Planner accessibility/cycles/scale | Broader native criteria remain open; overlap with G03/G08 | Track as their own criteria, not repeated observation-guard tests; publication remains G07/G08 |

The latest linked-worktree native chain closes one concrete topology criterion.
Configuration-trust drift and same-OID generation drift are now grouped guard evidence;
further URL/auth/tool variants are not the default next work unless tied to an explicit
remaining criterion. G02/G06 are not closed by this queue audit.

The user has authorized proceeding through acceptance steps and making a local
Git commit after each completed, validated step. This includes committing the
completed documentation/G01/G02 renderer-loss checkpoint. The queue does not
itself change write defaults or authorize live services, pushes, release publication
or deployment. Acceptance writes use only explicitly disposable fixtures.
Organization-policy blocks remain blocks, not acceptance waivers.

## Latest review validation — November 7, 2026

These results were obtained during the preceding read-only project review at
`db29ba6`, before this documentation reconciliation. They are a focused snapshot,
not the full five-configuration/signing/crash/release gate. No native GUI acceptance
was performed. Temporary raw logs are not durable release evidence.

| Check | Observed result |
|---|---|
| Renderer tests | 69 passed across 18 files |
| TypeScript, ESLint, generated contracts, frontend production build | Passed |
| Native host tests, default configuration | 13 passed |
| Core application integration | 21 passed |
| Core units / bulk / graph / inspection / reads / workspaces | 22 / 11 / 5 / 11 / 16 / 15 passed; bulk subprocess and non-UTF-8 filesystem cases explicitly ignored |
| Controlled-write integration | 24 passed, 1 failed: mandatory positive OpenPGP gate lacked `GITFLEET_TEST_GPG`; not a signing pass |
| Remote inspection | 10 passed |
| Remote observation | 52 passed, 29 failed, 2 ignored. HTTPS fixtures failed to bind loopback sockets (`Operation not permitted`); SSH fixtures timed out at startup. Transport acceptance remains unverified here |

Historical passing matrices below remain evidence for their recorded sources and
environments; this snapshot neither erases them nor attests the current release.
Documentation-only validation for this reconciliation is recorded in the final
change summary; application tests need rerunning only for subsequent code changes
or unresolved acceptance work.

## Historical checkpoint index — November 3, 2026

The following original tracker is retained for provenance. Its counts, "next"
labels and pending observations are dated; use the current tables above for work
selection. Later reports listed here can supersede earlier checkpoint limitations.


Updated November 3, 2026. Execution resumed under the user's fresh-chat authorization. Local checkpoint/milestone commits and reports are authorized; no push. Reasonable permission/facility blocks may be deferred; an unrun test is not a pass. Latest [native close/Quit report](reports/M3_NATIVE_CLOSE_QUIT_2026-11-02.md).

| Package | State | Evidence and next action |
|---|---|---|
| P0 baseline | Accepted | Commit `7130ced`; [report](reports/P0_BASELINE_2026-11-02.md); full baseline matrix passed |
| M3 desktop fixes | Implemented with native retests partial | [Progress report](reports/M3_DESKTOP_PROGRESS_2026-11-02.md); 17 renderer, 87 application-services and 127 all-feature passes; seven native tests per configuration |
| M3 native acceptance | Partial; execution resumed | Actual running cancellation, no replay, geometry, History/removal retests plus ordinary close and proven held-read close passed. Close during a running pre-publication hook and forced interruption/guarded abort now passed; broader review/interruption/accessibility gates remain open; graph-read Cancel fixed |
| M3 native lifecycle follow-up | Selected gates passed | [Report](reports/M3_NATIVE_LIFECYCLE_2026-11-02.md); exact PID termination, durable cancellation, native guarded abort, no replay and four-repository preservation; background polling queue finding subsequently fixed and retested |
| Mixed-DPI window restore | Fixed; automated and actual 1×/2× native retests passed | [Report](reports/M3_MIXED_DPI_WINDOW_RESTORE_2026-11-02.md); target-display conversion, shared minimum, repair/restart/no shrink and exact negative-coordinate Retina restore; 45 renderer/11 native host tests per configuration; physical disconnect/full accessibility remain open |
| Commit Command branding/About | Implemented; automated and native visual acceptance passed | [Report](reports/COMMIT_COMMAND_BRANDING_ABOUT_2026-11-03.md); supplied theme banners, pointer/click About, license and native menu, macOS icons and visible rename; 69 renderer/13 host per configuration. Internal identifier/storage preserved |
| M3 ordinary writes | Disabled | No change to default features; requires remaining acceptance and explicit default decision |
| M4 graph/activity/audit | Automated gates passed; selected native retests passed | [Original report](reports/M4_READ_ONLY_PROGRESS_2026-11-02.md); timeline/lanes/heatmap and basic graph/parent navigation now observed natively; foreground Cancel retest passed; remaining accessibility/scale gates open |
| M5A local bulk commit | Implemented; automated coverage and ordinary native form retests | [Original report](reports/M5A_LOCAL_BULK_PROGRESS_2026-11-02.md); new shared-form persistence fix passed automated/native navigation and restart retests; native two-repository publication/decline/restart now passed; broader dependency/failure/accessibility gates remain open |
| M5A native bulk follow-up | Selected native gates passed | [Report](reports/M5A_NATIVE_BULK_2026-11-02.md); exact staged/working blobs and messages, exclusions, unselected preservation, native Escape decline and restart/no replay; result refresh/empty retry gaps subsequently fixed and natively retested |
| M5B local remote inspection | Automated gates and ordinary native redaction/gate retest passed | [Report](reports/M5B_REMOTE_INSPECTION_2026-11-02.md); 113 application, 153 all-feature, 27 renderer and eight native host passes per configuration; no server contact |
| M5B authentication compatibility/artifact fix | Implemented and selected real acceptance passed; manual SecurityAgent interaction open | [Compatibility report](reports/M5B_AUTHENTICATION_COMPATIBILITY_2026-11-02.md), [artifact fix](reports/M5B_ARTIFACT_INITIALIZATION_FIX_2026-11-02.md), [setup guide](AUTHENTICATION.md). Actual osxkeychain/Gitea password + approved read-only PAT, native chain/Quit/ordinary restart passed; 56 renderer/13 host, Core 54/89/79/183/227. Native Deny/Allow policy blocked; lingering dialog dismissal unverified. Ordinary writes disabled |
| M5B authentication profiles/settings | Implemented; selected actual native acceptance passed; artifact finding resolved in later checkpoint | [Report](reports/M5B_AUTHENTICATION_PROFILES_2026-11-02.md); separately reviewed nonsecret HTTPS/SSH profiles, explicit destination-bound per-remote assignments, persistence and tool/revision revalidation. Native decline/save/assign/authenticated read/unassign/ordinary restart passed without auth startup flags. Desktop 54 renderer/13 host per configuration; real OS helper/Gitea now passed in later checkpoint; direct SecurityAgent interaction open. Final Core 54/89/79/179/223 passed; initial all-feature artifact-guard failure retained; now diagnosed/fixed in later checkpoint |
| M5B HTTPS credential helper | Automated matrix and selected actual native gates passed | [Report](reports/M5B_HTTPS_CREDENTIAL_HELPER_2026-11-02.md); host-selected scoped get-only helper/account, verified TLS/Basic, no secret IPC/journal, no store/erase forwarding, inherited process-group cleanup. Core 54/89/79/173/217; desktop 51 renderer/13 host per configuration. Native decline/rejection/authenticated observation/download/offline import and held-lookup Cmd+Q passed. OS secure-store/settings compatibility remains open; ordinary writes disabled |
| M5B reviewed branch observation | Automated and selected actual native HTTPS gates passed | [Report](reports/M5B_REMOTE_OBSERVATION_2026-11-02.md); exact credential-free network read through separate native review and isolated transport; local/loopback TLS tests pass; actual native HTTPS chain now passed in [native fetch report](reports/M5B_NATIVE_FETCH_ACCEPTANCE_2026-11-02.md) |
| Workspace remote history | Implemented; automated and native presentation passed | [Report](reports/M4_WORKSPACE_REMOTE_HISTORY_2026-11-02.md); existing scoped history endpoint, shared records, pagination/refresh/stale-response handling; 44 renderer tests and full desktop gate; synthetic native history only, no network execution claim |
| Unified operation audit | Implemented; automated matrix and selected native filters passed | [Report](reports/M4_UNIFIED_OPERATION_AUDIT_2026-11-02.md); one typed read-only chronology of local/batch/remote records; SQL filters before pagination; 45 renderer tests, full Core matrix and desktop gate; retention/later producers remain open |
| M5B reviewed fetch download | Implemented; automated real transport and native presentation/default gate passed | [Report](reports/M5B_FETCH_CAPTURE_2026-11-02.md); exact observed branch to isolated verified bundle; 16 transport cases, full Core matrix, 46 renderer/11 native host cases per configuration; separate import checkpoint implemented; actual native HTTPS download/import chain passed in [native report](reports/M5B_NATIVE_FETCH_ACCEPTANCE_2026-11-02.md) |
| M5B reviewed fetch import | Automated and selected real native gates passed | [Report](reports/M5B_FETCH_IMPORT_2026-11-02.md); separate exact-bundle review, bounded offline object import, direct expected-old tracking-ref transaction, durable journal/recovery and no replay. Full Core matrix 54/89/78/146/188 passed; desktop 47 renderer/11 native host tests per configuration and complete gate passed. [Native report](reports/M5B_NATIVE_FETCH_ACCEPTANCE_2026-11-02.md): actual HTTPS observation/download/import, decline, prepared Cancel/Cmd+Q, receipt-owned lock recovery, ordinary default gating and restart/no replay passed; ordinary writes remain disabled |
| M5C reviewed pre-push checks | Implemented; automated and selected native check acceptance passed; publication open | [Report](reports/M5C_REVIEWED_PUSH_CHECKS_2026-11-03.md); one-use independent hook approval, exact argv/stdin/fingerprint, already-reachable child gate, cancelled/interrupted unpublished history and restart/no replay. No push executor or publication authority; ordinary writes disabled |
| M5C native planner acceptance | Selected actual native gates passed | [Report](reports/M5C_PLANNER_NATIVE_ACCEPTANCE_2026-11-03.md); fresh TLS three-level conditional order, unselected-child blocking, selection/observation invalidation, Cmd+Q and acceptance/ordinary restart/no replay; exact worktree/remote/record/request preservation. No source patch, push or default-write change |
| M5C multi-level dependency planning | Implemented; automated and selected native acceptance passed | [Report](reports/M5C_DEPENDENCY_PLANNING_2026-11-03.md); explicit selected graph, conditional child-first order, actual closed cycle witness and blocked unavailable unselected children. No auto-selection, partial order, push authority or new network operation; ordinary writes disabled |
| M5C relative/SCP child acceptance | Implemented; selected actual native/SSH acceptance passed | [Report](reports/M5C_RELATIVE_CHILD_ACCEPTANCE_2026-11-02.md); explicit-parent relative resolution, exact SCP syntax, Git oracle and redaction regression. Native verified-TLS relative child evidence/clearing/ordinary restart/no replay and real agent SSH evidence passed. Direct SCP port-22 network acceptance OS-blocked; supported-subset limitations retained. No push/default writes |
| M5C historical mapping/child evidence | Implemented; validation checkpoint | [Report](reports/M5C_CHILD_MAPPING_EVIDENCE_2026-11-02.md); committed mapping blobs preserve URL changes; explicit child/approved observation selection and bounded local ancestry evidence with destination/context/freshness checks. No new network action, inferred relative URL, push authority or push. Native form/authenticated chain acceptance remains open |
| M5C outgoing-history child requirements | Implemented; local verification checkpoint | [Report](reports/M5C_HISTORY_DEPENDENCIES_2026-11-02.md); unique byte-path/child-OID requirements from all outgoing commit trees, including removed/replaced and merged-side links, independently of displayed commit cap. Bounded 256 distinct trees/1,024 requirements; no remote reachability proof, destination authority or push. Native presentation not run |
| M5C source child requirements | Implemented; automated verification checkpoint | [Report](reports/M5C_SOURCE_DEPENDENCIES_2026-11-02.md); exact source-tree gitlink paths/OIDs, SHA-1/SHA-256 and byte-safe rendering, bounded refusal. No child checkout/network access, remote proof, history-wide dependency completeness or push authority. Native presentation not run |
| M5C local push preflight | Full automated matrix and ordinary native checks passed | [Report](reports/M5C_LOCAL_PUSH_PREVIEW_2026-11-02.md); current local branch vs explicit past observation, exact counts/outgoing details, divergence/refusal checks, no network or authority. Actual native raw-parent oracle and full repository/artifact/journal preservation passed |
| M5B guarded upstream setup | Automated matrix and selected actual native gates passed | [Report](reports/M5B_GUARDED_UPSTREAM_2026-11-02.md); independent missing-only config review, exact byte/metadata preservation, durable phases and fresh recovery/no replay. Core 54/89/79/156/200; desktop 50 renderer/11 native host per configuration. Native decline/publication/default restart and actual post-publication journal-failure recovery passed; broader compatibility/accessibility remains open |
| M5B reviewed SSH-agent reads | Implemented narrow policy; selected actual native gates passed; artifact finding subsequently fixed | [Report](reports/M5B_SSH_AGENT_TRANSPORT_2026-11-02.md); host-selected Ed25519 agent identity/strict known hosts, isolated snapshots, no credential fallback. Core 54/89/79/163/207 and desktop 50 renderer/12 native host cases per configuration passed; real native chain, held-read Quit/no replay and final hardened authentication passed. Prior artifact refusal now captured/diagnosed/fixed; retain original evidence |
| M5B/C fetch/push workflows | Next | Continue operator-assisted SecurityAgent interaction, explicitly supported compatibility and broader upstream acceptance; artifact initialization refusal now fixed; retain native accessibility/topology/recovery gaps. General authentication workflows, individual/bulk push and dependency proof remain unfinished |
| Previous native follow-up | Validated checkpoint | [Report](reports/M5_NATIVE_FOLLOWUP_2026-11-02.md); 18 application integration tests per configuration, 29 renderer tests, eight native host tests per configuration and native preservation checks pass |
| Foreground cancellation | Validated, automated and native | [Report](reports/M3_FOREGROUND_CANCEL_2026-11-02.md); 30 renderer tests, eight native host tests per configuration; actual graph cancellation/process termination/reload and preservation passed |
| Background polling | Validated, automated and native | [Report](reports/M3_POLLING_2026-11-02.md); 31 renderer/eight native host tests per configuration; serial refresh/no queueing during native write, native Cancel and preservation passed |
| Native completion/results | Validated, automated and native | [Report](reports/M3_NATIVE_RESULTS_2026-11-02.md); 33 renderer/eight native host tests per configuration; fast native batch automatically refreshes to Published, empty retry disabled; ordinary read-only restart preserved |
| Selected accessibility | Fixes validated; full acceptance open | [Report](reports/M3_ACCESSIBILITY_2026-11-02.md); 37 renderer/eight native host tests per configuration, measured CSS contrast, keyboard/AX and Light rendering; interactive native Light/Dark switching and ordinary Dark restart now accepted in [theme report](reports/M3_NATIVE_THEME_2026-11-02.md); VoiceOver/broader cases open |
| Review lifetime/context | Selected native gates passed; presentation timer validated | [Report](reports/M3_REVIEW_LIFETIME_2026-11-02.md); actual late native approval and removed-workspace review rejected; zero trust/write records and preserved repositories; full current Core matrix plus 40 renderer/eight native host tests per configuration |
| Obsolete review window | Fixed and natively retested | [Report](reports/M3_STALE_REVIEW_WINDOW_2026-11-02.md); epoch-changing workspace removal closes review immediately; no grants/writes or repository mutation; 40 renderer/eight native host tests per configuration |
| Final close draft snapshot | Selected native gates passed; Quit defect fixed | [Restart handoff](reports/M3_CLOSE_DRAFT_HANDOFF_2026-11-02.md); atomic paired presentation save, one-use close token and main-only listener/ack, safe unsaved choice and Quit/close coordination; 19 Core integration cases per relevant configuration, 42 renderer/10 native host cases per configuration; actual immediate close/Quit, save failure/retry, held read and pre-publication write passed; [native report](reports/M3_NATIVE_CLOSE_QUIT_2026-11-02.md); broader lifecycle/accessibility gates remain open |
| M6 through M8 | Planned | Follow dependency and acceptance packages in completion plan |

Historical logs remain immutable. Current [requirement crosswalk](REQUIREMENT_EXECUTION_CROSSWALK.md), [completion plan](PROJECT_COMPLETION_PLAN.md), reports and per-run validation directories track progress. Do not infer a full milestone or product completion from an implementation checkpoint.
