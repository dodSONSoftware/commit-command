# G06 native observation review / same-OID branch switch — November 7, 2026

**Selected repository-generation revalidation case passed with trust still current.**
This advances native coverage beyond the preceding stale-configuration/trust case.
G02/G06 remain partial; no product fix or ordinary write-default change was needed.

## Fixture and setup

Reused the exact authorized disposable repository
`/private/tmp/gitfleet-g06-helper-di5z10fo/workspace/repository` with a separate
`state-generation-drift` copy of its original, closed application state. Original
state/journals were not modified. Product source remains validated `e9eeacd`;
checkout `e07594d` adds subsequent evidence only. Existing non-default desktop-writes
and ordinary artifacts were reused; exact hashes/features/paths are in `fixture.json`.
No real remote/listener, credentials, hooks or normal user repository was introduced.

Archived the original file baseline and exact config/HEAD/HEAD-log bytes. Deliberate
setup added `g06-observation-alt` pointing at the same OID as main, and configured
origin to the credential-free loopback destination `https://127.0.0.1:9/reviewed.git`.
A native authoritative/OS confirmation renewed trust only for the already authorized
repository. This setup preceded the reviewed baseline; subsequent drift did not
change configuration, index, hooks, signer policy or trust scope.

## Actual native sequence

1. Inspected origin and prepared an exact observation of `refs/heads/main`. Native
   authoritative review named the loopback destination and credential-free policy.
   SQLite independently contained one genuine `proposed` observation; no results
   or approvals were seeded.
2. While the review remained pending, changed symbolic HEAD to the already-created
   alternate branch using Git. Independent checks proved the identical HEAD OID,
   unchanged config/index and file changes restricted to HEAD/its log. The current
   inspection generation differed from the stored plan's generation.
3. Continued the original review to native confirmation and approved the exact
   observation. Core returned **blocked / stale_review: Remote observation review
   is stale** with null OID/time. The native main view showed the alternate branch
   and **granted / current** trust. Thus stale trust did not mask this later guard.
4. Operation results displayed the blocked observation. Acceptance Quit exited 0.
   First ordinary default-feature restart rendered without reload and displayed
   the identical blocked history, with ordinary writes disabled. Ordinary Quit
   exited 0, and the CLI subsequently reopened the writer lock.

## Independent results and cleanup

Executed read-only oracles after refusal and after ordinary restart confirmed exact
repository file bytes/modes against the intentional-switch baseline, SQLite integrity
`ok`, unchanged blocked observation payload, one scoped unrevoked trust row, zero local
write operations and zero other remote-operation records. No transport `network`
storage, observed OID/time or helper-invocation marker appeared. Native current trust
plus these facts and source control flow support generation rejection before transport
initialization; no host-wide packet capture or successful transport is claimed.

After both apps closed, restored original config/HEAD/HEAD-log bytes and removed only
the temporary branch files created by setup. The original baseline and prior helper
refusal oracle passed again against its untouched state. The isolated blocked journal
remained intact. Its raw trust row stays unrevoked; restoring the original config can
make that isolated trust stale on a future reinspection, so current eligibility after
cleanup is not claimed. During the tested refusal it was visibly current.

This tests generation drift at an identical commit OID and unchanged configuration,
not the independent URL-digest/authentication/tool-binding checks or remote success.
No product source changed; no full automated suite was rerun. Prior full validation
remains historical. Other G02/G06 startup, Dock/session, authentication/topology and
phase-specific requirements keep their open dispositions. Both fixture apps are
closed; normal user app/configuration/defaults were untouched.

[Evidence](../validation/g06-native-observation-generation-drift-2026-11-07/README.md).
