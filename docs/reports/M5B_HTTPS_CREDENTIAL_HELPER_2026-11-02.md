# M5B reviewed HTTPS credential-helper checkpoint — November 2, 2026

Implemented a narrow host-selected HTTPS helper/account policy for separately reviewed observation and branch download. This is a completed implementation checkpoint; M5B remains partial. Local checkpoint commits are authorized. No push or ordinary-write default change.

## Behavior and authority

The native host accepts `--https-credential-helper /absolute/executable` and `--https-credential-account account` together. Optional `--https-ca-file /absolute/certificate` selects a reviewed trust source without installing a global CA. No password argument, renderer credential-policy request or secret-bearing IPC was added. The current native executable provides an internal bridge mode before Core state ownership/UI startup; a small CLI bridge supports fixture tests. Ordinary startup selects no helper.

Review binds the exact canonical HTTPS host/repository path, public account, selected helper and bridge executable paths/fingerprints, current repository/configuration/trust/generation and the existing native authority. Account metadata is deliberately visible in reviews and history. Old plans deserialize without authentication metadata. A recorded account is history, not another lookup grant. This narrow envelope rejects ambiguous/noncanonical URLs, encoded or unsupported paths, userinfo, queries/fragments and invalid account framing.

Credentials are obtained only after fresh approval and revalidation. The bridge sends only a scoped `get` frame to the selected helper. It discards Git `store` and `erase` operations without calling the helper, including rejection cleanup. There is no GitFleet credential cache, credential digest, credential-store management or password persistence. Credential bytes travel through bounded pipes; no credential appears in process argv, environment, review plans or app journals. Raw helper/server diagnostics are suppressed. Protocol parsing rejects malformed/duplicate/oversized responses and changed accounts or context. Common-exit buffers are cleared; this is not a claim of complete in-memory erasure.

HTTPS uses verified TLS, Basic authentication only, no redirects, and explicit suppression of empty/proactive ambient authentication outside this selected policy. Repository/global/system helper configuration, proxies and other managed transport commands remain isolated. The selected helper runs **unsandboxed as the user** and may have intrinsic side effects or access its credential store; get-only forwarding does not constrain arbitrary helper implementation behavior. No real user credential store or macOS Keychain was accessed during acceptance.

Network Git owns its process group. The bridge/helper inherit that group, so outer cancellation/deadline kills ordinary descendants. Successful network completion also reaps residual group descendants. Existing nontransport process semantics remain unchanged. Deliberate `setsid`/hostile helper process escape is outside this trusted-helper envelope.

## Automated evidence

Evidence: `docs/validation/m5b-https-credentials/`. The complete Core validation script exited zero: **54 / 89 / 79 / 173 / 217** passed for default/experimental/controlled/application/all features, with **1 / 2 / 1 / 2 / 6** direct ignores. Ignored subprocess workers remain exercised by parent scenarios; consult logs for exact cases. The research lab retained 23 passed/one skipped, without expanding its production-safety claim. Strict Clippy and formatter checks passed. The full matrix preceded a presentation-only cleanup of helper review text; final all-feature/all-target strict Clippy and the rebuilt native artifact cover that final change.

Eight new real TLS integration cases cover authenticated observation/download/import and non-persistence, decline/pre-cancel/helper drift/trust revocation, wrong/missing/changed credentials, cancellation during actual lookup, normal-completion background descendant cleanup, malformed/oversized/secret-bearing diagnostics, timeout cleanup, certificate-hostname and redirect refusal. Two protocol unit cases cover scoped framing/refusal and nonexecuting store/erase handling. App-state scans check both raw generated credential bytes and their Basic Base64 encoding. Existing SSH and credential-free transport cases passed in the full matrix; the strict fetch-artifact guard was not weakened.

The full desktop validation script exited zero: **51 renderer tests and 13 native host tests per configuration**, plus generated-contract, type, lint, format, frontend build, native Clippy and ordinary release build checks. No new dependencies or lockfile changes.

Initial compile-only failures for changed internal function/plan signatures were corrected and logs retained. An early native oracle ran before the asynchronous rejection completed and asserted prematurely; the terminal blocked native result and subsequent oracle passed. No retry is claimed as a product fix. The earlier SSH `Unsafe fetch artifact type` refusal remains unexplained and open; these successful cases do not diagnose it.

## Actual native acceptance

CUA operated the built macOS app and independent read-only repository/SQLite/process oracles checked effects. Fixture details and exact artifact hashes are in `native-fixture.json`, `artifact.json` and the source manifest. Only generated disposable credentials, CA/key and local repositories were used. CLI setup granted fixture trust; it seeded no network-operation records.

- Native confirmation decline left zero helper calls and zero HTTP requests and recorded cancellation.
- After preparing a fresh review, only the fixture credential-store value was changed. Execution called `get`, attempted Basic authentication, received rejection and retained a blocked result without store/erase or credential-store mutation.
- Restoring the generated fixture credential and using a new native review observed commit `f38c5c0b0211af4413951beebf03ee58374bc6b2`. The server independently recorded Basic authentication success, never credential values.
- A separate native review downloaded the exact branch into isolated storage: four objects, 577 uncompressed bytes, 447 bundle bytes; SHA-256 `000b9618409c2501581331421e7842f32746b7a36ca01b34cabe392270ff480d`. All repository/remote bytes and modes still matched baseline.
- A separate native offline import published only `refs/remotes/origin/main`, its reflog and new objects. All original entries remained unchanged. No extra helper call or HTTP request occurred during import.
- A new reviewed observation entered the actual held helper. Exact Git/bridge/helper PID and process-group evidence was captured. Cmd+Q terminated owned clients, durably recorded cancellation and allowed state reopening. No extra HTTP request, repository mutation or credential-store mutation occurred.

Ordinary restart used the refreshed default development bundle with only the isolated data-directory flag, no helper/account/CA policy. Native history showed the reviewed public account and cancelled/blocked/complete records; observation, download, import, upstream and trust write controls remained disabled and the status said read-only desktop. After native Quit, repository/remote/journal/artifact/helper/request equality matched the cancelled snapshot exactly: no replay or credential lookup. Both apps are closed, the hold flag is removed, the exact owned TLS server was sent SIGTERM and fixture state reopened. Generated credentials and TLS key remain only in the protected disposable temporary fixture, outside source/evidence. The development bundles are not signed/notarized release acceptance.

## Remaining work

Native settings/profile selection and secure-store management, real supported OS-helper compatibility (including access-denied/locked-store/prompt behavior), broader SSH/account/upstream compatibility and remaining M3/M4/accessibility/topology/scale gates remain open. The host CLI policy is a deliberately narrow development entry point, not finished credential UX. M5C still needs fresh destination/dependency proof and separately reviewed push. No push is authorized by this checkpoint; retain the user's no-push instruction. M6–M8 remain planned.

Primary protocol references: [Git credential helpers](https://git-scm.com/docs/gitcredentials), [Git HTTP configuration](https://git-scm.com/docs/git-config/2.56.0), [Git HTTP implementation](https://github.com/git/git/blob/master/http.c). These support get/store/erase framing, path-scoped selection and Basic/empty-auth controls; repository tests establish the implemented envelope.
