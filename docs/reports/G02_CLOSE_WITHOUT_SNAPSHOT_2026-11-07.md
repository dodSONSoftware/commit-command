# G02 explicit Close without saving after missing snapshot — November 7, 2026

**Selected native fallback-exit branch passed within the disposable fixture envelope.**
This complements the previously accepted Keep app open branch; it does not establish
actual renderer loss or recovery of unsaved keystrokes. G02/G06 remain partial.

## Fixture and build

Fresh marked `gitfleet-m3-native-*` fixture, one dirty repository, no remotes/hooks/
credentials/trust. Staged, unstaged and untracked content is included in the file
SHA256/mode baseline. Checkout `286448c`; product source unchanged from validated
`e9eeacd`. Built the existing `native-acceptance-support` feature into a separate local
artifact, **without desktop-writes**. Startup enforced the existing marked-fixture and
workspace bounds. No new test hook, IPC, permission or crash capability was added.
Artifact hashes/features/paths are recorded in `fixture.json`; support build succeeded.
The previous debug executable was restored after copying; ordinary release/normal
user bundle were not replaced. Full automated suites were not rerun for unchanged code.

## Native actions and results

1. Entered `G02 fallback close — already saved draft` through the native composer.
   Independently verified it had persisted and recorded the presentation payload hash.
2. Used the existing fixed test menu **Drop next close acknowledgement**, then Cmd+Q.
   Audit records exactly one arming and one suppression. The hook actually suppresses
   close-request delivery, exercising the production three-second snapshot timeout;
   this is not proof of an actual renderer crash or failed filesystem write.
3. Native unsaved-drafts sheet visibly explained failed latest-state saving and showed
   **Keep app open** and **Close without saving**. Explicitly selected the latter.
   Fixture host exited normally with code 0, without another confirmation or hang.
4. Independent after-close assertions found the entire saved presentation payload hash
   unchanged, exact managed file bytes/modes preserved and SQLite integrity `ok`.
5. First restart used a separate **ordinary default-feature** artifact on the same
   fixture state. It rendered without reload, displayed the exact saved draft, disabled
   write controls and had no native acceptance menu. Normal Cmd+Q exited 0 without a
   fallback sheet. CLI then acquired the writer lock and read the one-entry inventory.

## Preservation / disposition

Final reusable read-only oracle passed: exact repository file bytes/modes, SQLite
integrity, saved draft and coherent scan/membership count; zero trust/write-operation
rows and zero remote-operation state records. No publication, hook or network action
was requested. Existing records were retained; no manual database result/approval was
seeded. The fixture is closed and the user's normal app/state were untouched.

This case deliberately used an already saved draft. It proves explicit fallback exit
and preservation of saved state, **not** that latest unsaved edits survive the chosen
discard path or that edits were actually discarded. No exhaustive data-loss or timeout
latency guarantee is inferred. The suppression fault is not included in the ordinary
restart. Broader startup, Dock/operator session, remaining lifecycle phases and G06
remote/auth/topology criteria keep their open dispositions. No product patch needed.

[Evidence](../validation/g02-close-without-snapshot-2026-11-07/README.md).
