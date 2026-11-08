# G05 native index-only partial staging — November 8, 2026

Disposition: this case passed; G05 remains partial. No product source change.
The November 2 native bulk report already recorded a selected index-only staged/
working split. This repeat strengthens actual child-metadata and restart oracles in
the current fixture; it is not a newly implemented capability or a second gate closure.
Reused the exact already authorized parent in the child-hook fixture. No new trust,
hook/config change, child execution or push. Existing child hook remains unchanged.
Source/build provenance is retained in fixture.json (source 4c52595; acceptance and
ordinary binaries from the fully validated build).

## Setup and exact native approval

After the completed hook chain, deliberately staged file.txt containing
`Reviewed staged-only content` and then appended a later working edit. Added a
separate untracked-preserve.txt. Captured every repository file hash/mode and prior
operation/batch payload before native launch. Earlier hook-chain oracles are now
historical; they must not be asserted against this intentional new setup.

Rescan showed parent 1 staged / 1 unstaged / 1 untracked, only parent selected.
Native Bulk Commit selected Index only · preserve partial staging and an explicit
message override. Stored plan selected only file.txt from source index; no hooks or
signer in the parent plan. Native authoritative review and separate OS approval
executed batch 776bd133-471a-47c1-be07-25d64cb193f4, operation
163f21cc-f4fc-47cd-af68-d4f5d0e082db. It completed with publication and reconciled
index, creating 9e08a8efcfdcfce7496c3bbc2cdec0f3ab059f5f.

## Preservation and ordinary restart

verify.py passed before/after restart: one parent commit, exact staged blob in both
HEAD and index, only file.txt changed in commit, exact message, unchanged working
file and untracked file bytes/modes. Untracked file absent from tree. Parent gitlink
unchanged. Child checkout AND actual metadata inside parent .git/modules unchanged;
full independent control unchanged and untrusted; fixed child hook still three runs.
All older operation/batch payloads and exact two trusts unchanged; integrity ok.
The oracle checks its own Git queries preserve every repository file and mode.

Acceptance native Quit exited 0. Ordinary restart showed parent 0 staged / 1 unstaged /
1 untracked, clean child, dirty unselected control, read-only status. Bulk retained
index-only mode, exact override and complete batch; Prepare native bulk review was
disabled. Native Quit exited 0; exact files/modes and payloads matched pre-restart,
no replay. Writer-lock reopen passed. Initial sandboxed launch exited -6 before UI;
normal authorized native launch then succeeded. This was a launch environment
failure, not an acceptance result; no guard was weakened.

[Evidence](../validation/g05-native-partial-staging-2026-11-08/) includes fixed fixture,
before/after file hashes/modes, prepared operation, exact durable records, verifier,
checks and ordinary exit code. No full matrix rerun for this native-only case.
Remaining dependency/selection/accessibility criteria retain their owning tests.
