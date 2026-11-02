# M5B native fetch/import and recovery acceptance — November 2, 2026

Selected native gates now pass on source `ab3fd39`. The earlier locked-Mac attempt remains historical evidence; the user unlocked the Mac and native work resumed. This checkpoint changes evidence/status only. Ordinary desktop writes remain disabled. No push, credentials, global configuration changes or personal repository mutations occurred.

## Artifact and fixture boundaries

The acceptance binary was rebuilt from the checkpoint with `custom-protocol,desktop-writes` and copied into `/private/tmp/gitfleet-fetch-native-vthb_q8k/GitFleet Import Acceptance.app`. The ordinary default bundle stayed separate. Exact acceptance/ordinary hashes and source revision are in `docs/validation/m5b-fetch-import-native/artifact.json`. After testing, the ordinary default native binary was rebuilt and its bundle refreshed; its hash matched the original default artifact.

All native UI input used CUA. Git/files/SQLite checks were read-only independent oracles. Each fixture used an isolated HOME/XDG directory and its own explicit app data directory; no global environment or configuration was altered. Fixture trust was prepared through Core for these disposable worktrees; this run does not claim new native trust-grant acceptance. Temporary Rust preparation examples were compiled/executed, saved as evidence source and removed from application source afterward.

The manual-bundle fixture `/private/tmp/gitfleet-fetch-native-vthb_q8k` contains actual Git objects and manually prepared standalone bundles, with explicitly seeded completed capture records. It proves actual native import, cancellation and recovery, not network download provenance. A separate public HTTPS fixture `/private/tmp/gitfleet-public-native-laodapb4` started with **zero** observation/capture/import records and exercised the full actual native network path. Neither uses the older synthetic-history fixture or its fake checksum.

## Actual native HTTPS chain: passed

In the public fixture, local remote inspection showed the exact effective destination `https://github.com/octocat/Hello-World.git`. Native observation explicitly selected `refs/heads/master`. Three separate authoritative Rust reviews and independent native approvals then performed:

1. **Observation** `dc1b1474-f962-4e3b-8cd1-ab78d854475e`: completed and recorded OID `7fd1a60b01f91b314f59955a4e4d4e80d8edf11d`. The managed repository remained exactly at its byte/mode baseline.
2. **Download** `1dd33954-fa8d-4100-a3d7-40861cb61f39`: completed over HTTPS into isolated storage, with seven objects, 914 uncompressed bytes and a 784-byte standalone bundle. SHA-256: `3d40422535f604214de65a47f1f6925286b99cafdd44d0511252aeb6818d185a`. The actual bundle checksum/length and observed OID were independently checked. The managed repository still matched baseline exactly.
3. **Import** `a9864778-98d1-4a22-906a-dc695a32bf75`: a new review bound that exact checksum/OID, absent old value and `refs/remotes/origin/master`. Native approval completed object import and recorded `updated`. The direct tracking ref matches the observed commit and has one reflog entry. Every non-object/non-target-ref/non-target-reflog file and mode matched baseline, including HEAD, staged/working content, untracked content, configuration, local branch refs and FETCH_HEAD absence.

The acceptance app was quit natively. The ordinary default app reopened the same state and displayed all three actual completed records. Exact repository and journal equality after restart proved no extra operation or replay. The state lock reopened after Quit. Evidence: `public-observed.json`, `public-downloaded.json`, `public-imported.json`, `public-after-public-restart.json`, and the executed `public-oracle.py`.

This was an optional public-server native acceptance run, not a new public-service dependency for CI. HTTPS uses the existing verified-certificate/hostname/no-redirect credential-free policy. Local inspection flagged configured helper policy; transport review explicitly excluded credential/helper execution and no credential interaction was needed. Authentication/SSH/private-server compatibility is not established.

## Native import refusal, results and defaults: passed

The first manual-bundle review showed the exact 443-byte bundle checksum, target commit `49e243681e872caca55b76636b08b312996869a8`, absent old ref and `refs/remotes/origin/main`. Declining its independent native confirmation recorded `cancelled`, disabled further confirmation on the consumed review and left the repository byte-for-byte unchanged, with the target object absent.

The next approval was correctly **blocked**: the fixture preparation helper had created its manually seeded network directory with `0755` permissions. No managed mutation occurred. Only those disposable fixture storage directories were corrected to production's private `0700` mode, and a fresh proposal/review was used. The original blocked journal remains visible. This was a fixture setup correction, not a product patch.

A subsequent native approval completed import and created exactly the target ref/reflog plus objects. Native Operation results rendered complete, blocked and cancelled outcomes and their precise object/ref status. Selecting and applying the captured-import audit filter excluded download/observation records. Ordinary restart displayed the real history, kept the import control disabled and preserved the exact post-import repository state. Evidence: `declined.json`, `fixture_permissions_blocked.json`, `fixture-permission-correction.json`, `imported.json`, `after_acceptance_quit.json` and `after_ordinary_restart.json`.

## Proven prepared-transaction Cancel and owned-lock recovery: passed

A fixture-only transparent Git proxy was placed in this fixture's PATH. Other Git commands exec the real Git. Only a fixed disposable common directory's `update-ref --stdin` transaction was intercepted, holding the **commit command** after genuine Git preparation and after Core durably recorded its real lock identity/content receipt. No production adapter, IPC fault control or application source was changed.

For operation `1615af26-86c2-4574-8050-f2b41b4b18dc`, read-only evidence established phase `reference_prepared`, the correct lock contents/receipt and no destination ref. Native **Cancel review** then terminated exact owned proxy/Git PIDs 93402/93403 (process group 93402). Core recorded `uncertain`; `refs/remotes/origin/cancellation` and its reflog remained absent. The genuine prepared lock remained, owned by the saved receipt. All other repository bytes were preserved except intentionally imported unreferenced objects.

Native recovery inspection offered `abort_unpublished` with an explicit no-replay explanation. Independent native approval removed the exact receipt-owned lock, recorded `aborted / unchanged` and left **every other repository byte/mode identical** to the pre-recovery snapshot, including imported unreferenced objects. This closes the previously unrun positive owned-leftover-lock-removal case. The attention notice disappeared. Evidence: `prepared-cancel.json`, `native-cancel-outcome.json`, `cancel-before-recovery.json`, `cancel-recovered.json` and `recovery-oracle.py`.

## Cmd+Q during prepared import, restart and fresh recovery: passed

A fresh independent approval started operation `b5b0a683-3a17-4ba9-8bbf-6c05ced15fa3` against the still-absent cancellation ref. The same proof established a genuine held prepared transaction, now with owned proxy/Git PIDs 94817/94818 (group 94817). Native **Cmd+Q** terminated both owned children, recorded `uncertain` and released the Core state lock. The destination ref/reflog stayed absent and the real owned lock remained for inspection. No shell termination substituted for this native Quit.

The ordinary app restarted with one recorded-operation attention notice, displayed the uncertain journal and preserved **all repository bytes including the owned lock**. Clicking recovery preparation was rejected by the native default gate with “Desktop writes are disabled pending native acceptance”; it produced no recovery review, new operation or mutation.

A restarted acceptance app required a fresh recovery review and independent native approval. It removed only the receipt-proven lock and recorded `aborted / unchanged`. A final ordinary restart had no unresolved import notice, preserved all repository bytes and kept the five manual-fixture operation records unchanged. Evidence: `prepared-quit.json`, `native-quit-outcome.json`, `quit-before-restart.json`, `ordinary-restart.json`, `quit-recovered.json`, `final-restart.json` and `quit-oracle.py`.

Both fixture apps are closed. Both state locks reopened. Both fixtures have zero unresolved imports and zero local write-operation records. The fixture proxy hold flag was removed so it cannot hold a later run accidentally. The fixtures and exact artifacts remain available for further tests; see `summary.json` and fixture metadata.

## Validation scope and remaining plan

Application source is unchanged from the fully validated implementation checkpoint: Core matrix 54/89/78/146/188 passes with documented ignores; desktop 47 renderer and 11 native host tests per configuration plus the complete gate. Those suites were not needlessly rerun for evidence-only changes. The acceptance feature build and restored default build succeeded here; every saved oracle executed and passed against actual native outcomes. The earlier unexplained automated crash-worker exit remains documented in the implementation report; this native acceptance does not retrospectively diagnose it.

These are selected real native gates for the existing credential-free HTTPS fetch envelope. They do not complete all M3/M4/M5B gates or change the ordinary write default. Broader topology/SHA-256 native execution, malformed/large native reviews, VoiceOver/focus/contrast, broader interruption/display changes and retention/scale remain open. Authentication/helper/SSH and secure storage policy, upstream setup, separately reviewed push and submodule dependency proof remain unfinished, followed by M6–M8. No force, merge/pull, automatic replay, version bump or release was introduced.
