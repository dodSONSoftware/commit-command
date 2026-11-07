# G01 — review authority and exact-content acceptance

November 7, 2026. **G01 in progress; selected cases passed, gate remains open.**
This run adds two Core regressions and expands native origin-rejection tests.
Production behavior, Cargo feature defaults and ordinary desktop write exposure
are unchanged. Existing uncommitted documentation reconciliation is preserved.
No commit or push was made in this development repository. One approved local
commit was made only in a newly created disposable acceptance repository.

## Scope, artifact and evidence

Base revision is `db29ba6`; [source manifest](../validation/g01-2026-11-07/source-manifest.json)
records native build inputs and final test inputs. The native acceptance binary
was built with `custom-protocol,desktop-writes` and copied into a separate temporary
bundle retaining `dev.gitfleet.desktop`. Its SHA-256 and exact fixture identifiers
are in [fixture metadata](../validation/g01-2026-11-07/native-fixture.json).
Only test additions differ between the native build and final source: the Core
integration tests and `security.rs` test block. No production source changed.
The ordinary development bundle was not rebuilt or replaced. After native testing,
the standard target/debug executable was rebuilt with default features to remove
the acceptance feature from that build path; final artifact hashes are recorded. The fixture app
used its own state directory, not the user's saved workspace data.

[Environment](../validation/g01-2026-11-07/environment.json), commands, logs,
repository hashes/modes and independent oracle verdicts are preserved in the
[evidence directory](../validation/g01-2026-11-07/README.md). Native controls were
operated through Computer Use. Native AX results and an inline screenshot were
inspected; excerpts below are transcribed observations, not saved screenshot files.
No credentials, private keys, raw user app state or network services were used.

## Native results

| Case | Observed result and independent check |
|---|---|
| Close unapproved trust review | Closed the native authoritative window before confirmation. No trust grant, no local operation and all four repositories' bytes/modes unchanged. |
| Decline native trust confirmation | Separate OS dialog showed the exact fixture engine path. Cancel consumed the review, disabled confirmation and showed `Native confirmation was declined`. Zero grants/operations; repository snapshot unchanged. |
| Approve explicit fixture trust | Fresh review and OS approval granted only the engine worktree. Exactly one trust row and zero operations; all repository bytes/modes unchanged. |
| Review hostile text | Message and staged blob contained harmless HTML-like strings, bidi/isolate characters and ESC. The authoritative window displayed literal tags and visible `\u{202e}`, `\u{2066}`, `\u{2069}`, `\u001b`/`\u{1b}` escapes. No rendered HTML/image was observed. Index source, unsigned policy, tree and digest were visible. |
| Decline exact commit | Native Cancel resulted in one `cancelled_before_publication` record; all repository bytes/modes matched the baseline. Confirmation could not be reused. |
| Approve fresh exact commit | One new independent proposal/OS approval produced `complete`, `published: true`, `index_reconciled: true`. Git's raw commit object matched the original message bytes exactly; the selected staged blob hash matched. Unapproved working README content, all other worktree content and all three other repositories' metadata were preserved. |
| Same-OID branch switch | Reviewed working-tree content on `main`, then explicitly switched only the disposable engine to a new branch at the same OID. Native approval returned `needs_reauthorization`, `published: false`, reason `StaleReview: Checked-out branch changed`. Both branches still reference the one previously approved commit. |
| Replace pending review | Returned to main through the native Window menu and requested another review without approving the old one. The abandoned commit proposal was durably cancelled. A subsequent fresh commit review displayed a different review/operation ID; no additional commit was published. No native stale callback was injected in this case. |
| Change workspace with pending review | From main, used the real directory picker to open the disposable tooling repository as a second workspace. New workspace Overview showed one untrusted repository; Window menu no longer listed the obsolete authoritative review. Its pending commit record became cancelled. No extra trust or publication resulted. |
| End of run | Cmd+Q exited the exact fixture process with exit code 0. Final read-only oracle passed: five operation records (one complete/published, one needs reauthorization, three cancelled), one trust row, exactly one new commit, and preserved excluded repository content. |

The staged README differed from the working README throughout the positive test.
The native hostile-text case used an ordinary UTF-8 filename; hostile filename
presentation/publication remains a separate native criterion. Native review
replacement was exercised before native confirmation, not while a confirmation
callback was already outstanding.

## Automated validation

- Core `m3_application`: **23 passed**, including two new G01 cases. One verifies
  escaped review text with a bidi/newline filename, literal HTML/ESC content and
  exact approved message/index bytes using Git inspection; it also preserves
  excluded staging and working bytes. The other rejects already-issued native
  capabilities after review replacement/close, preserves the replacement review
  after an old callback, and verifies no trust or repository mutation.
- Native host default and all features: **13 passed per configuration**. Expanded
  origin tests reject userinfo, ports, fragments, queries, encoded/lookalike paths,
  wrong hosts and the development HTTP origin in ordinary custom-protocol builds.
  Existing generated ACL, envelope, reload/reset and dialog exclusion tests pass.
- Review/bridge renderer: **10 passed** across two files; inert text, no renderer
  approval boolean and review lifetime behavior remain covered.
- Core focused strict Clippy, host all-target/all-feature strict Clippy, and both
  Rust format checks passed. Native acceptance build passed.

The first new-test compile used a nonexistent `TrustState.trusted` field. It was
corrected to the actual `grant == "untrusted"` contract; the failed log is retained.
No expectation was weakened and no production change was needed. Initial full
application/host results, before the added assertions, are retained separately.
This was not the full five-configuration/signing/crash or release matrix.

## Remaining G01 work

1. Native main-renderer reload/reset with pending review and, separately, an issued
   native confirmation; stale callback must not execute. Automated reload/context
   regressions pass. The inspected ordinary View menu exposes no reload control;
   this run did not inject browser evaluation or add a privileged test endpoint.
2. Native close/replacement/context changes while an OS confirmation callback is
   outstanding, including interaction with main-window shutdown (coordinate G02).
3. Native caller/page/origin adversarial acceptance where a permitted test facility
   exists. Generated ACL and actual caller checks currently have mock-runtime and
   unit evidence; no live webview attack injection is claimed.
4. Native unusual filename/control-byte review and exact publication within the
   supported filesystem envelope. Core bidi/newline filename case passed here;
   non-UTF-8 filesystem support remains an explicit host limitation, not a pass.
5. Reconcile remaining trust/hook/signing presentation criteria with the existing
   dated native reports and the required G02/G06 compatibility cases. This run's
   positive native commit was explicitly unsigned and had no executable hooks;
   it does not refresh configured signer/hook policy acceptance.

Keep G01 open and ordinary writes disabled. The next acceptance package is the
outstanding native context/confirmation lifecycle cases, coordinated with G02;
no push executor, AI, default enablement or release work is implied by this run.
