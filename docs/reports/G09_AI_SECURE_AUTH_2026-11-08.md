# G09 optional AI secure authentication — November 8, 2026

Disposition: implemented; selected automated and native acceptance passed. This
checkpoint does not close G09, G02 or release acceptance.

Named saved connections can explicitly select no authentication or an endpoint-bound
bearer token. Imported definitions reset authentication to none. Settings, exports,
renderer IPC, review JSON, process arguments and temporary files contain no token.
A main-origin-only native command accepts only the saved connection ID and opens an
empty, focused macOS secure field. Cancel changes nothing; removal has its own native
confirmation. Unsaved endpoint edits cannot select a different saved credential.
Native request approval identifies the exact connection, canonical endpoint and route.
A missing/locked token refuses that request without unauthenticated fallback.

Keychain calls execute in bounded, owned helper processes using anonymous pipes.
Before touching Keychain, the helper checks its parent executable device/inode and
both live signed code identities; a shell caller is refused. Only the helper disables
its own OS credential interaction. The parent policy and user Keychain search list
remain unchanged. The helper has a five-second total deadline, job cancellation and
shutdown cleanup. Failed transport during an OS mutation reports an uncertain outcome
rather than retrying. This is an application boundary, not protection against a
compromised same-user process capable of executing the trusted binary.

## Diagnosis and retained counterevidence

Initial direct locked-Keychain lookup took 45,224 ms despite a noninteractive
LAContext. Adding per-query UI-fail policy alone still took 4,096 ms. Isolating the
call and its interaction policy in an owned helper fixed the demonstrated blocking
path. An initial 1.5-second helper deadline failed under the full concurrent suite;
the final five-second total deadline passes. Security-service fixture tests serialize
with one another to make their proven-running markers reliable; production request
limits and cancellation remain independent. Failed compile, timing and full-suite
logs are retained alongside final passes. No test or lint was disabled to pass.

The initial native authentication popup did not commit the intended choice. That
attempt is not accepted as a bearer test. Two explicit radio choices now expose the
selected mode; the final native test checked the radio, persisted TOML and trusted
review independently.

## Validation

Evidence: [secure-auth artifacts](../validation/g09-ai-secure-auth-2026-11-08/).
Final Core library: 50 passed; application integration: 32 passed; strict all-feature,
all-target Core Clippy passed. Final complete desktop runner exited zero: 119 renderer
tests, ordinary host 20 passed/1 ignored worker, all-feature host 23 passed/1 ignored
worker, typecheck, lint, contract freshness, formatting, frontend build, strict Clippy
and release build. The ignored worker is invoked explicitly by parent fixture tests.
Actual isolated macOS Keychain tests cover read/update/delete, wrong endpoint,
locked refusal, native capability decline/approval, successful bearer HTTP, proven
running timeout and cancellation/reaping. No user token was used. The complete five
configuration Core write/signing matrix is prior dated evidence and must be refreshed
at final regression; this step does not relabel it as a fresh run.

Native ordinary builds used only the explicitly named disposable three-level fixture:

- First artifact: empty secure field opened with focus; Escape cancelled. No secret
  entered, saved, removed or exposed to the webview.
- Final radio artifact: saved fresh connection `a9dc903e-d69e-4e5b-8822-061f8f0ad3db`
  to `http://127.0.0.1:61917/` with stored-bearer mode. Native review explicitly showed
  that binding and unencrypted transport. Fresh approval produced the sanitized
  missing-token message; the owned HTTP fixture received zero requests.
- Manual draft `G09 native reviewed version Action`, local refresh and disabled
  ordinary repository writes preserved. Quit exited zero. Exact artifact restart
  rendered the draft and an empty diagnostic job list; no HTTP replay, second Quit zero.
- Repeatable `verify-native.py` checks repository bytes/modes, journals, trust,
  retained original, older settings generations, saved binding, source/binary hashes,
  SQLite integrity and writer-lock reopening. Only two new nonsecret connection
  generations and their current pointer are intended settings changes. First manifests
  are preserved separately from the final source/artifact manifests.

Locked dependency inventory: 582 records; no missing third-party declarations.
The existing first-party npm license decision remains open. Inventory covers selected
locked development/all-feature graphs and is not a shipping license audit.

## Remaining work

Real-user GUI credential enrollment/removal is untested; no real credentials were
requested or changed. Actual supported-provider/TLS interoperability, repository
context filtering/coverage/exact payload inspection, assisted candidates and separately
bounded automatic authority remain open G09 work. Ordinary Git write gates remain
unchanged. No push, release signing, notarization or publication occurred.
