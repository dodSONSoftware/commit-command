# M5B guarded upstream setup — November 2, 2026

Accepted implementation checkpoint with selected actual native acceptance. This advances PUSH-002 and the completion plan's separately reviewed upstream policy. M5B, authenticated transport and actual push remain incomplete. Ordinary desktop writes remain disabled. No push was run.

## Scope and behavior

The user authorized autonomous completion-plan development, disposable native acceptance, reports and local commits, preserving unrelated work and prohibiting push. Inputs are one available worktree, one completed existing-branch observation from that worktree, current local branch/OID, current trust/repository identity/configuration and an independently approved exact native review. A past observation is explicitly historical and grants no execution authority.

The new **Review upstream setup** action prepares a missing-only local configuration change. It adds exactly `branch.<current>.remote` and `branch.<current>.merge` to the shared common-directory `config`. It requires the selected named remote's standard single tracking refspec, ASCII names and no existing current-branch configuration at any effective scope. Detached/unborn sources, worktree config, local includes, custom mappings and unsupported config metadata are refused. This initial compatibility envelope deliberately does not overwrite an existing upstream or branch policy. All linked worktrees share the change; no checkout, index, object, ref, reflog, FETCH_HEAD or network artifact changes.

The exact review binds current source ref/OID, effective remote/destination/merge ref, observation OID/time, repository identities, configuration identity/content/metadata and candidate checksum. Existing controlled-write trust and stale-review checks apply before publication; successful setup changes configuration and requires fresh trust. Preparation, decline and restart never restore or reuse native authority. The frontend consumes the generated typed contracts, displays typed audit results and offers separately reviewed recovery for unresolved outcomes. Both preparation and recovery are host-gated as writes in ordinary builds.

## Publication and recovery

Core preserves the original raw config bytes as an exact prefix and appends only the canonical two-key section. Git parses the original and candidate through bounded stdin and checks the ordered semantic records. Effective branch/refspec values are read transiently; they do not enter the deliberately restricted persisted policy map or journal. Raw configuration, unrelated URL userinfo and attribute values are never stored in the new plan/journal.

Publication takes the shared repository lock, journals intent before lock creation, creates `config.lock` exclusively with no symlink following, records inode/content receipts, writes and synchronizes the candidate, revalidates source/trust/configuration and atomically renames the owned lock. File and directory syncs bracket durable phases. The source must be a bounded regular user-owned single-link file with ordinary mode bits. Original ownership/group/mode and bounded extended attributes must be preserved exactly. Darwin file flags and ACLs are unsupported.

Fresh Git files on this Mac carry `com.apple.provenance`. Initial blanket rejection of attributes was too restrictive; the final path copies attributes using descriptor-based `fcopyfile(COPYFILE_XATTR)` and checks the exact attribute digest. A custom attribute and native provenance both survived actual publication. It does not remove protection metadata or change global settings.

Journal phases cover precreation intent, owned empty lock, ready candidate, pending publication, published config and terminal/uncertain results. Unresolved upstreams block local writes even in controlled-only builds, block new imports/setup in application builds and prevent workspace scope loss. Errors before publication remove only a receipt-proven lock; unknown/foreign/partial locks remain for investigation. Errors after rename preserve an explicit uncertain result. Fresh recovery fingerprints the actual config/lock state and either ends an unpublished operation or records matching configuration without attributing uncertain publication. It never replays config writes, rolls back publication or renews trust.

## Validation

Evidence: `docs/validation/m5b-upstream/validation-summary.json`, full gate logs, source/artifact manifests and independent repository/SQLite oracles.

| Core feature configuration | Passed | Documented direct ignores |
|---|---:|---:|
| Default | 54 | 1 |
| Experimental writes | 89 | 2 |
| Controlled writes | 79 | 1 |
| Application services | 156 | 2 |
| All features | 200 | 6 |

The full Core format/build/test/strict Clippy gate and disposable lab passed. The all-feature real remote suite passed 33 cases with two subprocess workers ignored directly; parent tests exercised the workers. Upstream tests cover exact bytes/metadata, quoted branch names, secret non-persistence, linked worktrees, decline, pre-cancellation, stale config/source/trust/identity, foreign locks, unsupported policy, restart authority revocation, scope barriers and post-publication storage failure. SIGKILL at five durable boundaries verified no replay and removal only of owned config locks. Final review additionally rejected special mode bits; the focused application-services upstream suite passed five tests and final strict all-feature Clippy passed.

The full desktop gate passed contracts, types, lint, formatting, 50 renderer tests, frontend/native builds, 11 native host tests per feature configuration, strict Clippy and default release build. Native artifacts are local development bundles, not signed/notarized release installers. Exact hashes and source files are saved.

Historical failed iterations remain saved. Early tests exposed the restricted policy-map mismatch and mandatory provenance attributes, leading to the transient reader and exact metadata copy. Two fixture assertions needed correction: ACL principal lookup and `xattr` CLI trailing newline. The independent native oracle also needed a portable attribute reader on Apple's Python and comparison of its own `oracle` bookkeeping field. Final results above are from corrected checks; earlier failures are not passes.

## Actual native acceptance

CUA controlled both local bundles; shell/Python performed launch, read-only file/metadata/SQLite oracles and named disposable-fixture setup. No personal repository, real credential or global configuration was changed.

1. Existing public HTTPS fixture `/private/tmp/gitfleet-public-native-laodapb4`: used its actual prior native observation of `octocat/Hello-World` `refs/heads/master`, OID `7fd1a60b01f91b314f59955a4e4d4e80d8edf11d`. Native Cancel consumed the first review and left all repository bytes unchanged. A fresh exact native approval published the two keys for local `refs/heads/main`, returned `complete/updated`, required fresh trust and rendered completed/cancelled typed history. Only config bytes changed; mode/ownership/group/attribute hashes, every other file, network artifacts and earlier journals matched the independent baseline. Ordinary restart preserved the repository and operation records exactly and showed setup/download/import controls disabled. Native Quit completed and the state reopened.
2. Separate `/private/tmp/gitfleet-fetch-native-vthb_q8k`: seeded one **explicitly synthetic completed observation** from its previously cancelled fixture plan; this is no network evidence. A fixture-only SQLite trigger rejected the terminal `complete` insert after actual production config publication. Actual native approval returned `uncertain/updated`; all unrelated bytes and earlier records remained intact. Removed only the named fixture trigger. Native restart showed one attention notice and the durable uncertain result. A fresh independent native recovery approved `observe_configuration`, returned `observed_configuration/matches_target`, removed the blocker and rendered the non-attribution/no-trust-renewal explanation. Recovery and subsequent state reopen changed no repository byte or metadata. The terminal audit remains classified “needs attention” to preserve historical uncertainty, with no recovery action/blocker.

The recovery fixture initially had a disclosure that did not respond to mouse input; console metadata showed unlocked. Native Cmd+Q worked and a clean relaunch restored normal input. No source defect or lock-screen diagnosis was established. Both fixture apps are now closed, both state locks reopened, no injected upstream trigger remains, and neither fixture has an unresolved upstream or config lock. The ordinary dev bundle is refreshed to current default-feature source; the separate upstream acceptance bundles match the recorded opt-in hash.

Native close/Cancel during this short config rename, native leftover-owned-lock removal, VoiceOver and broader topology/scale acceptance remain open; automated crash/owned-lock coverage is separate evidence. Authentication/helpers/SSH/native secure storage, broader upstream compatibility, fresh push destination/dependency proof and actual separately reviewed push remain next. Retain remaining M3/M4 and M6–M8 work. Unrelated `.gitignore`, `docs/README.md` and `docs/APPLICATION_OVERVIEW.md` were preserved and excluded from this checkpoint.

November 7 follow-up: the selected [native linked-worktree shared-config success case](G06_NATIVE_LINKED_UPSTREAM_2026-11-07.md) now passed end to end. Native owned-config-lock recovery and other explicitly remaining topology/failure criteria stay open.
