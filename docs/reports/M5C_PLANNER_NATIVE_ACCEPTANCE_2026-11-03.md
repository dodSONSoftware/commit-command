# M5C native three-level dependency planner acceptance

November 3, 2026. Selected actual native acceptance passed for source checkpoint `13c5896`: parent-only unselected-child blocking, explicit grandchild → child → parent conditional ordering, selection/observation invalidation, native Quit and restart/no replay. The ordinary default build was restored and its empty-planner restart/read-only gates verified. No application source patch or push was needed.

## Fixture and provenance

A fresh isolated workspace contained nested parent, child and grandchild repositories, with three separate bare remotes. Each local source had one pending commit beyond its remote baseline. The parent committed `../child.git` and the child's pending source OID; the child committed `../grand.git` and the grandchild's pending source OID. All three retained staged README content, different working README content and an untracked file. The fixture server bound only loopback and exposed upload-pack routes; it had no push route. GitFleet verified the generated TLS CA supplied only to that launch. No credentials, private hosting server, system trust installation, SSH/Keychain/global Git changes or synthetic observation records were used.

The native acceptance bundle was copied from the separately built `desktop-writes` debug binary before launch. `native-artifact.json` records its SHA-256 and bundle path, and `native-source-manifest.json` records source hashes and commit. The ordinary default-feature binary was then rebuilt, copied into the normal development bundle, tested separately and hash-verified. These are local development artifacts, not signed/notarized release claims. No code changed between the recorded builds and acceptance.

## Actual native results

| Check | Observed result |
|---|---|
| Separate observations | Parent, child and grandchild each completed a stored Rust review plus independent native exact-operation approval against their exact loopback HTTPS destination and `refs/heads/main` |
| Parent only | Explicitly checked and adopted the child binding. Required child commit was unavailable remotely. Plan blocked; child/grandchild planning boxes stayed unchecked; no order returned |
| Scope invalidation | Selecting the child removed the parent preview, adopted binding and blocked result; Build became disabled pending new inspection |
| Freshness refusal | Initial three-level attempt exceeded the parent observation's five-minute age. Native UI showed `stale_review`; no order returned. All three observations were refreshed through new native approvals, without editing timestamps or limits |
| Three-level selected graph | Explicitly selected all three roots, chose their refreshed observations, inspected sources, checked/adopted parent→child and child→grandchild bindings. Native result was conditional grandchild → child → parent with exactly two selected-child publication conditions |
| Native Quit | Cmd+Q from the successful conditional result exited owned PID 32288; independent process inspection confirmed exit |
| Same-bundle restart | Workspace/tabs restored. Expanded planner had all selections empty, no sources/bindings/result and both action buttons disabled; no operation resumed |
| Observation invalidation | After restart, inspected parent with its newer observation, then chose the older record with identical destination/ref/OID. Settled UI removed the source/result and disabled Build, preserving explicit parent-only selection |
| Ordinary restart/default gates | Normal default build showed the read-only desktop status and empty planner. With inspected origin and complete branch input, observation review stayed disabled; upstream/download reviews for existing records also stayed disabled. Quit completed |

The native three-level result explicitly said remote availability remained unproven until each selected child successfully publishes and that the plan grants no push authority. No actual push or inferred publication was performed. Selection invalidation was exercised on a blocked plan with an adopted binding; native restart was exercised directly from a successful conditional plan. This distinction is intentional evidence scope, not a claim that every lifecycle transition was tested.

Early combined native-menu actions sometimes left placeholders unchanged. Opening and observing a settled menu before keyboard selection worked. One immediate accessibility sample after changing the observation preceded React's state update; subsequent fresh snapshots confirmed complete invalidation. The acceptance outcome uses settled states. No source defect, weakened freshness check or approval bypass was introduced to make the run pass.

## Independent preservation and no replay

`native-oracle.py` compares every fixture worktree/remotes file's SHA-256 and mode, plus symlink targets, against the prelaunch baseline. Parent snapshot includes both nested repositories. It reads fixture SQLite in read-only mode and compares exact observation/download/import records and server request hashes across planning, Quit, both restarts and invalidation.

All three worktrees and bare remotes were unchanged. There were exactly six completed real observations (initial plus refreshed for each worktree), zero write operations and zero fetch capture/import records. The server recorded exactly six read-only requests for the six separately approved observations. No planning or restart added requests or changed those records. State keys were also audited; the planner created no durable push/dependency plan. Existing UI/navigation persistence is outside the operation-record equality claim.

Evidence is in `docs/validation/m5c-planner-native/`: `native-verdict.json`, exact baseline/oracle stages, request/state audit, artifact/source hashes, build logs and selected accessibility excerpts. The excerpts are explicitly transcribed from actual CUA observations, not a raw replayable automation trace.

## Validation and cleanup

Both acceptance and ordinary native builds passed. The existing three planner renderer tests were rerun; their result is in `renderer-focused.txt`. Both Python fixture/oracle scripts passed syntax parsing and the read-only oracles passed before launch, after observations/planning, on Quit, after invalidation and on both restarts. The full Core/desktop suites were not rerun for this evidence-only checkpoint; their prior passing results belong to unchanged source commit `13c5896` and its implementation report.

Both fixture apps exited. The exact owned server PID 31827 was verified against its fixture command before termination; its loopback listener closed. Its generated TLS private key was removed. The repository/state/public certificate evidence remains disposable historical data; the stopped fixture cannot be replayed without fresh provisioning and approvals. The ordinary development bundle is restored and its final hash matches the tested artifact. User's unrelated untracked `LICENSE` remains untouched and unstaged. No branch change or push.

## Remaining work

These selected planner acceptance gates are closed. Native cycle presentation, broader accessibility/scale/interruption scenarios and other unrun M3–M5 acceptance remain distinct. One usability follow-up: the planned-observation picker shows destination/ref/OID without time or record ID, so repeated observations with identical values have identical labels. This run selected newest-first and verified the actual preview timestamps; Core freshness enforcement remained effective. Actual separately approved individual/bulk non-force push still needs exact publication authority, pre-push hook policy/stdin, source/destination revalidation, child-success gating, cancellation, server rejection and uncertain-publication recovery/no replay. Preserve the explicit no-push boundary. Operator-only SecurityAgent Deny/Allow/unlock and OS-blocked direct SCP port-22 acceptance remain open. Ordinary desktop writes stay disabled; the whole project is not complete.
