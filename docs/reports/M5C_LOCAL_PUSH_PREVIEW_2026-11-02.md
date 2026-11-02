# M5C local push preflight — November 2, 2026

This checkpoint adds a bounded, local-only push comparison against an explicitly completed past branch observation. It implements no push, upstream write, new network read, native approval or execution capability. Ordinary desktop writes remain disabled; no push command was run, including in fixtures.

## Behavior and supported scope

`PushPreview` requests are scoped to the active workspace and available worktree, with a completed observation ID from that same worktree. Preparation rechecks the existing controlled-write eligibility/trust policy, identity, effective destination/configuration and current local generation. A fetch import or local commit may legitimately have changed local refs since observation; the preview binds the **current** source ref/OID separately while retaining the original observed remote OID and observation timestamp explicitly. It never upgrades a past observation into fresh server evidence.

The initial envelope requires one identical effective fetch/push destination, HTTPS or a host-authorized local fixture, no embedded credentials/restrictions and no configured push refspecs. Changed config, different/multiple destinations, unsupported policy, absent/cancelled/foreign observations, revoked trust, detached/unborn sources and shallow/grafted ancestry are refused. Missing observed commit objects require a separately reviewed fetch/import; lazy fetching stays disabled. An absent remote branch is not used to plan destination creation.

Local ancestry comparison reports **up to date**, **fast forward**, **behind** or **diverged** relative to the observed commit. A bounded Git traversal rejects comparisons exceeding 10,000 differing commits. Up to 100 outgoing commit OIDs/subjects are displayed, with the exact total and explicit display truncation. Subjects preserve native bytes, with a separate 4 KiB subject-display limit. Repository generation is checked around the multi-command read and transport/configuration is revalidated afterward; shallow/graft markers are checked before and after ancestry inspection. Process output/cancellation/deadline bounds remain in force.

The result includes the selected destination/ref, current local source/OID, original observed OID/time, generated time, exact incoming/outgoing counts, outgoing details and whether existing upstream configuration matches the selection. It is ephemeral read data, not a durable operation journal or capability. A fresh server observation, source/destination revalidation, exact native review, transport/auth policy and dependency proof are still required before an actual push can exist.

## Desktop

Completed existing-branch observations expose **Inspect local push preview**. It remains available in ordinary read-only builds; download/import/write controls keep their existing gates. The result labels the observation as historical, says that the preview contacted no server and grants no approval, and displays divergence/behind warnings without a force action. Outgoing subjects/destinations render as inert escaped text. Stale preview responses cannot overwrite newer responses, and results for an old workspace/worktree/revision are hidden.

There is no new `Action`, native confirmation or audit mutation for this read. Native-host tests explicitly keep the request outside the mutating-request gate. No read-only adapter protocol restriction was relaxed.

## Validation

The full Core matrix passed **54/89/78/150/192** tests (default/experimental/controlled/application/all-features), with **1/2/1/2/5** documented ignores. Desktop passed **49 renderer tests**, **11 native host tests per configuration**, contracts/typecheck/lint/format, native/debug/release builds and strict Clippy. The research-only lab retained 23 passed/one skipped within its existing scope.

Exact final counts/commands, source hashes and artifact provenance are saved in `docs/validation/m5c-push-preview/validation-summary.json` and the adjacent logs/manifests. The complete Core feature matrix and desktop validation gate were run for the checkpoint.

Four new real-Git integration cases cover:

- Same-tip/up-to-date and a known one-commit fast forward, including matching upstream config and exact outgoing OID/subject, repository/remote preservation and absence of a confirmation capability.
- Missing observed objects without a hidden fetch; explicit fetch/import followed by independently constructed behind/diverged histories; exact known OIDs/counts and preserved managed/remote trees.
- Different/multiple push URLs, push refspecs, config drift, revoked trust, shallow/grafted history, detached/unborn source, cancelled observation and wrong workspace scope, with no preview effects.
- A 101-commit outgoing chain displaying exactly the first 100 while retaining the full count, plus pre-cancellation without mutation.

Renderer regressions cover historical/limited/diverged result rendering, HTML-inert byte subjects/destinations, absence of execution controls, ordinary-build preview requests and hiding results after a worktree change. Earlier focused iterations had a test-only SQLite count type error (`usize` is not a `FromSql` type), then a fixture ordering issue: observation itself correctly rejected an already ambiguous multiple-push-URL configuration. The fixture now introduces that unsupported config after a valid observation and verifies safe refusal. These failed focused logs are retained; neither required relaxing product policy.

## Actual ordinary native acceptance

The refreshed **ordinary default** development bundle was launched with the existing disposable actual-HTTPS fixture `/private/tmp/gitfleet-public-native-laodapb4/state` and its isolated HOME. The previously accepted observation/capture/import records were reused as historical data, without repeating network operations. Its exact binary hash and base revision are in `native-artifact.json`; changed source hashes are saved separately. All UI actions used CUA.

The native UI showed the read-only footer, enabled **Inspect local push preview**, and disabled download/import controls. Clicking the preview opened no approval dialog. It displayed:

- Source `refs/heads/main` at `14dda63e7e351f8e6552c17affc372c66a9b303a`.
- Destination `https://github.com/octocat/Hello-World.git`, `refs/heads/master`, observed OID `7fd1a60b01f91b314f59955a4e4d4e80d8edf11d` and the original observation timestamp.
- **Diverged: 1 outgoing / 3 incoming**, upstream match false, and an explicit blocked normal-push/no-force explanation.
- Expanded outgoing details containing the exact local OID and subject **Native HTTPS baseline**.

An independent Python oracle walked raw `parent` headers using `cat-file commit`, rather than reusing the implementation's `rev-list` comparison, and verified those exact sets/counts. It compared every managed file/symlink byte/hash/mode, every network artifact and all three remote-operation records before preview, after preview and after native Cmd+Q. They matched exactly. The state lock reopened after Quit. Evidence: `native-oracle.py`, `native-before.json`, `native-preview.json`, `native-after-quit.json`, `native-observed.json` and the final workspace-list record. The fixture app is closed; no new remote operation or push authority was created.

## Remaining completion plan

This is the local planning portion of M5B/C, not push execution or completion of M5. Authenticated HTTPS/helper/SSH/native secure-storage policy, guarded upstream configuration, actual separately reviewed individual/bulk push, server-side uncertainty and selected submodule dependency proof remain unfinished. Future push must obtain fresh destination evidence rather than accepting this DTO, its relation label, a tracking ref, or its saved observation as authority. Broader ancestry/topology/size stress and native accessibility remain open, alongside the remaining M3/M4/M6–M8 packages.

The user’s unrelated `.gitignore`, `docs/README.md` and `docs/APPLICATION_OVERVIEW.md` work was left untouched and unstaged. No credentials, personal repositories, global settings, version bump, release or external publication were changed.
