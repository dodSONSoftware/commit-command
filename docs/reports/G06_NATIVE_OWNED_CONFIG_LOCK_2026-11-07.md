# G06 native owned-config-lock recovery — November 7, 2026

## Package disposition — passed within the recorded envelope

Completed native read-only startup, cancelled recovery, receipt-proven removal without
publication/replay, empty and ready/pre-publication locks, foreign replacement and
stale recovery fingerprint, then ordinary restart/preservation. Product code remains
validated `e9eeacd`; no new fault interface or ordinary write default is introduced.

### Step 1 — actual interrupted fixtures prepared and validated

Existing all-feature `upstream_crash_worker` was built and run directly in a subprocess
for LockCreated, LockReady, PublicationPending and a second LockReady fixture. Each
worker exited by SIGKILL (-9). Preparation is **automated Core test execution**, not
native approval or native host-crash evidence. The worker uses real production
observation/upstream proposal/publication checkpoints and a fixture-only fault; no
ownership receipt or terminal result was manually fabricated.

Each fixture retains its actual durable phase and config.lock. Independent assertions
verified receipt hash, device/inode identity prefix and original config digest; the
empty lock has zero bytes, ready/pre-publication candidates have 455 bytes. Original
config stayed at the reviewed pre-publication digest. File/mode baselines cover the
repository (including the lock), bare remote and isolated transport artifacts.

Existing non-default desktop-writes artifact copied into each isolated disposable
fixture; exact hashes/features/paths retained. Recovery will use the native review
and independent confirmation. No new native trust grant is required or attempted;
worker-created trust is automated fixture state and is not presented as user/native
trust-approval evidence. Native recovery outcomes are pending at this checkpoint.

Remaining package steps: native Cancel + ready-lock abort; empty/pre-publication
boundary recovery; foreign/stale review refusal and fresh owned recovery; ordinary
restarts, no replay, final cleanup and disposition.

[Evidence](../validation/g06-native-owned-config-lock-2026-11-07/README.md).

### Step 2 — ready candidate Cancel / native abort / ordinary restart passed

Native startup preserved the actual 455-byte owned lock and showed needs attention.
Recovery review proposed abort_unpublished. Cancel preserved repository/lock bytes,
receipt and lock_ready journal. A fresh review/independent native confirmation then
returned aborted/unchanged and removed only config.lock. Original config and every
other repository, bare-remote and transport-artifact file/mode matched baseline.
Ordinary restart rendered aborted history without a pending recovery action or replay;
both exits 0 and writer lock reopened. No native trust grant or transport requested.
Empty/pre-publication and foreign/stale cases remain in progress.

### Step 3 — empty LockCreated native recovery passed

Actual zero-byte receipt-proven lock was visible as lock_created/needs attention.
Fresh authoritative and native confirmation proposed abort_unpublished and returned
aborted/unchanged. Only owned config.lock disappeared; all other repository, bare
remote and transport-artifact file/mode baselines matched. Normal Quit 0 and writer
lock reopened. Pre-publication and foreign/stale cases continue below.

### Step 4 — PublicationPending native recovery passed

Actual pre-rename journal and ready lock inspected natively. Independent confirmation
approved only abort_unpublished; result aborted/unchanged. Candidate config was not
published or replayed; only owned config.lock removed. All other repository/remote/
transport file/mode baselines matched, normal Quit 0 and writer lock reopened.
Foreign-lock and stale-fingerprint refusal, followed by clean owned recovery, remain.

### Step 5 — foreign ownership and stale recovery refusal passed

Prepared a native abort_unpublished review against the actual owned candidate, then
externally saved that inode and replaced config.lock with a different inode containing
identical bytes/mode. Old native approval returned Not completed: Upstream configuration
review changed. Independent negative oracle verified the foreign inode/content and
original journal/receipt/repo/remote/transport remained unchanged. A fresh review
correctly proposed manual investigation with no writes/lock removal permitted; native
confirmation was refused with Manual upstream investigation is required. The negative
oracle passed again. No foreign file was removed or adopted. Clean restoration of the
actual owned inode and fresh abort/restart remain to complete the package.

### Step 6 — final owned recovery, ordinary restart and preservation passed

After quitting the negative-case app, the fixture operator preserved the foreign
replacement outside the repository and restored the actual original receipt-owned
inode. The receipt was never edited. A fresh native review recognized ownership and
returned aborted/unchanged, removing only that owned lock. The preserved foreign file
still matches its recorded inode and SHA256. Ordinary-build restart rendered without
reload, retained aborted/unchanged history, exposed no pending recovery action and
kept writes disabled. Both final native and ordinary exits were 0.

Final independent oracles passed for all four fixtures: original config and every
other repository file/mode, bare remote and transport artifacts match baseline;
original plans/receipts persist, SQLite integrity is ok, and no new write or remote
operation exists. CLI inventory reopened all four writer locks. No test app remains
running. Fixture evidence is retained, including the foreign file; it was not adopted
or deleted by recovery. These file/mode checks do not establish general xattr or
ownership preservation. No new native trust grant or product patch was needed.

The initial sandboxed ordinary launch attempts did not start (-6 on the retained
attempt); launching the same artifact with the required host permissions succeeded.
This is not recorded as an application recovery failure or a product fix.

The selected native owned-lock package is complete. G02/G06 remain partial: this
closes the named recovery gap, not all interruption, topology, accessibility or
release acceptance. Earlier pending statements above describe historical checkpoints.

Focused owning regression rerun passed: `upstream_sigkill_recovery_never_replays_and_removes_only_owned_config_lock` (1 passed, 88 filtered out, all features, locked dependencies). Full suites were not rerun because this package changes evidence and trackers only. `git diff --check` passed.
