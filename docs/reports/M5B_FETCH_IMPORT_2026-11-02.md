# M5B separately reviewed captured branch import — November 2, 2026

This checkpoint implements offline import of one approved captured branch and publication of one exact tracking ref. It does not complete M5B or establish native execution acceptance. Ordinary desktop writes remain disabled. User-authorized local checkpoint commits only; no push.

Native follow-up after the user unlocked the Mac: [selected real native fetch/import and recovery gates passed](M5B_NATIVE_FETCH_ACCEPTANCE_2026-11-02.md), including actual credential-free HTTPS network execution and positive receipt-owned leftover-lock removal. The blocked-attempt section below describes the earlier attempt and is retained as history.

## Behavior and authority

A completed captured bundle can be proposed for import through a **new** native review. The download confirmation cannot authorize import. The review binds workspace/worktree identity, current trust/configuration/generation/tool bindings, capture ID, bundle SHA-256 and byte count, captured commit, deterministic `refs/remotes/<remote>/<branch>` destination and its exact old OID or absence. Confirmation is single-use/session/epoch-bound. Decline, cancellation, changed configuration/ref, symbolic ref, artifact tampering and lost session authority fail closed.

The supported initial publication envelope uses the existing local macOS/APFS controlled-write policy, files ref backend and an ASCII remote/branch other than branch `HEAD`. Non-ASCII publication and storage case aliases/symlinks are refused. This narrower compatibility scope needs broader native/linked-worktree and adversarial acceptance before expansion. No credentials, global settings or personal repository mutations were needed.

Import rereads the approved immutable bundle with a bounded no-follow snapshot and exact checksum/length verification. It verifies the standalone bundle/head, strictly checks its objects in isolated Git storage, applies the existing 10,000-object/64 MiB limits, unpacks into isolated loose objects and promotes complete durable immutable objects to the managed object store. Temporary artifacts have a polled 256 MiB/20,000-entry budget and process/cancellation bounds. Failed pre-publication import can leave harmless unreferenced objects; it never resets/cleans them automatically.

A prepared `git update-ref --no-deref --stdin` transaction compares the exact old value. The app rechecks identity/trust/configuration and the direct destination ref **after Git prepares and holds its lock**, then records an inode/content receipt before permitting commit. A requested abort gets a bounded 500 ms acknowledgement/cleanup interval; an unresponsive abort is terminated and recorded uncertain. A disposable probe demonstrated why an OID comparison plus `--no-deref` alone is insufficient: Git can replace a symbolic ref when its resolved value matches. The actual prepared-guard regression verifies that such a transaction aborts and preserves the symbolic ref.

Publication uses a detached isolated Git context pointed at the managed common directory. Only the selected tracking ref/reflog and imported objects are intentional repository effects. Hooks and network are disabled. Positive tree-byte oracles verify preserved HEAD, index, working files, local branch/tag refs, configuration, FETCH_HEAD and existing unrelated metadata. SHA-1 and SHA-256 tests cover existing packed tracking refs as well as an absent tracking ref.

## Durability and recovery

The journal distinguishes object import, publication pending, reference prepared, reference updated, complete and uncertain outcomes. A crash never restores confirmation authority or automatically replays Git. Startup exposes unresolved import notices and workspace scope changes are blocked. New imports and local writes/recovery are blocked for the same common directory until unresolved import inspection completes; this guard also exists in controlled-write CLI builds without application services.

Fresh native recovery review inspects the actual direct ref and locks, binds a fingerprint and either records the target as currently observed or aborts an unpublished operation. Observing the target does not attribute uncertain publication to GitFleet. Recovery never rolls back or replays publication. A lock may be removed only when its current identity/content matches the durable owned receipt. Foreign/symbolic/unexpected states require manual investigation and remain untouched. A replaced lock invalidates an outstanding recovery review.

Real SIGKILL workers terminate the application owner at object-import, reference-prepared and reference-updated boundaries. Git's prepared child receives EOF and cleans its lock in the tested case. Restart and separately reviewed recovery preserve actual repository bytes and never replay. The positive leftover-owned-lock-removal case remains a separate acceptance gap; the tested foreign-lock and stale-fingerprint cases are fail closed.

## Application and desktop

Added review/import/history/recovery contracts, typed unified audit projection, startup notices and native default gating. Branch download records offer a separately gated import review. Operation results display captured import phases, exact old/target OIDs, object-import recording, ref result, bundle checksum and guarded inspection. Download history explains that downloading alone does not update the repository and directs readers to the separate import result. Historical records never recreate execution authority.

Renderer regressions exercise import gating/request scope and uncertain/observed-target rendering without HTML execution. Native host tests cover both new review requests under the existing write feature gate; ordinary history reads remain available.

## Validation

Final full Core matrix passed: 54/89/78/146/188 tests across default/experimental/controlled/application/all-features, with 1/2/1/2/5 documented ignores. The all-features real transport/import suite passed 22 cases plus an ignored subprocess-only crash worker that is exercised by its parent test. Four prepared-ref runner regressions passed. The complete desktop gate passed 47 renderer tests, 11 native host tests per configuration, generated contracts, type/lint/format, native builds, strict Clippy and release build. The lab reported 23 passed/one skipped within its existing research-only scope.

Final gate results and source/artifact hashes are recorded in `docs/validation/m5b-fetch-import/validation-summary.json`, `source-manifest.json` and `artifact.json`. Commands:

- Full Core matrix: `GITFLEET_TEST_GPG=/private/tmp/gitfleet-m2-tools/install/bin/gpg CARGO_NET_OFFLINE=true sh app/core/scripts/validate.sh <absolute evidence directory>`.
- Full desktop gate: `CARGO_NET_OFFLINE=true sh app/desktop/scripts/validate.sh`.
- Separate controlled-only import guard regression, focused real import/crash cases and prepared-ref transaction regressions, with exact logs alongside the full gates.

The first full Core attempt caught a real regression: a broadened shared guard treated a blocked pre-publication local operation as unresolved and prevented an existing batch retry. The guard was narrowed to unresolved imports for local operations, and the separate local-write check remains scoped to importing a branch. The batch retry suite subsequently passed and the final full gate was rerun. The early preservation-test oracle was also corrected to allow the newly created tracking-ref/reflog ancestor directories; source behavior was already correct in that focused test. Earlier failed logs are retained and are not acceptance evidence. A later full run had one crash worker exit without the required SIGKILL; the parent assertion omitted stderr, so that historical run does not establish a cause. The assertion now includes phase/stdout/stderr and the worker reports its returned outcome. A complete real transport/import diagnostic suite passed afterward; the final clean matrix and separate crash repeat are recorded independently. The initial intermittent worker exit remains an unexplained validation observation, not a silently retried pass. A subsequent matrix also exposed a real cancellation race: the transaction runner killed Git immediately after sending abort, potentially leaving the prepared lock behind. The runner now permits a bounded 500 ms abort cleanup interval and retains uncertain-outcome handling if Git does not respond. Focused actual-Git cancellation and unresponsive-abort regressions pass; the full gates were rerun after this source fix.

## Native attempt: BLOCKED

The ordinary development bundle was launched explicitly with disposable state `/private/tmp/gitfleet-m3-native-5zxerugm/state`. The CUA tool reported: “The Mac is locked and automatic unlock could not unlock it.” No native UI assertions or import execution occurred. The user was notified; automated work continued. Only the exact launched fixture process was sent SIGTERM, after which the Core state lock reopened and all 68 fixture repository file/symlink entries and file modes matched baseline. SIGTERM is not native close/Quit acceptance.

Two explicitly **synthetic** import-history records (complete/uncertain) were seeded solely for native startup/history presentation. The fixture has no actual bundle/import/trust authority. Their fake checksum and phases are not evidence of download or publication. IDs and fixture purpose are in `native-fixture.json`; the tool block/owned process and preservation evidence are in `native-blocked.json` and `native-preservation.json`. Native rendering/default-gate/import/recovery acceptance remains **NOT RUN/BLOCKED**, independently of automated success. The ordinary bundle is refreshed after validation; older separate write-acceptance packages remain stale.

## Next package and remaining limits

Finish native captured-import/recovery acceptance with a real fixture bundle and a rebuilt write-acceptance artifact, recording exact source/features/hashes. Exercise decline, approved import, immediate close/Cancel, uncertain recovery, receipt-owned leftover lock and restart/no replay. Do not turn synthetic history into publication evidence. Broader cancellation/output/size/repository-topology stress, VoiceOver/focus and remaining M3/M4 gates remain open.

Then continue M5B transport/credential policy and upstream setup, M5C separately reviewed push/dependency proof, and M6–M8 under the completion plan. Authentication/SSH and push are unimplemented; no implicit merge/pull/branch mutation, force publication, version bump or release was introduced. Preserve the user's unrelated `.gitignore`, `docs/README.md`, `docs/APPLICATION_OVERVIEW.md` and `docs/graphics/` work.
