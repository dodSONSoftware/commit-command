# G09 deterministic SemVer preview — November 8, 2026

The commit composer now has an explicit **Preview version update** panel that works
in ordinary read-only builds, in an untrusted repository, with no AI settings,
provider, model, credentials or service. Nothing runs on mount. The user chooses a
repository-relative file and major/minor/patch increment, then explicitly reads a
preview. Core returns the exact old/new version and byte fingerprints. This does
not write, stage, approve an Action, or authorize a commit.

The initial file format is exactly one SemVer 2.0 value in 1–1024 UTF-8 bytes,
optionally terminated by LF or CRLF. Leading whitespace, prefixes, multiple values,
binary data and numeric overflow are refused. The selected numeric increment clears
build metadata. Prerelease input is rejected unless the user explicitly chooses
**Discard prerelease and increment its numeric base**; the exact result remains
visible before any future mutation. This is a specific supported format and explicit
per-preview policy, not an inference that all project version formats or prerelease
conventions are supported.

Parsing uses pinned `semver` 1.0.28 rather than a new handwritten grammar. The same
version was already locked by the desktop toolchain. Its
[documented parser and numeric bounds](https://docs.rs/semver/1.0.28/semver/struct.Version.html)
and the [SemVer 2.0 specification](https://semver.org/) were checked during implementation.
No unrelated dependency versions changed. Preview validation recomputes the exact
result; the pure recovery comparison distinguishes frozen-before, frozen-after and
conflicting bytes without recomputing another increment. This helper is **not yet a
durable invocation journal or implemented execution recovery**.

The application verifies current workspace/repository identity and uses restricted
inspection. Targets must be regular files, never symlinks; traversal, metadata paths,
symlinked ancestors and crossing into a nested repository are refused. Existing
untrusted-inspection restrictions remain intact. UI input changes invalidate old
results, late superseded responses are ignored, and manual message state remains
independent of previews/errors.

Validation:

- Four focused parser/recovery-comparison tests passed, covering all increments,
  line endings, build metadata, explicit prerelease handling, invalid input,
  overflow, tampering and repeated comparison without double increment.
- Two application regressions passed with exact repository byte/mode oracles,
  untrusted use, no AI configuration, preserved manual drafts and unsafe targets.
- Complete default Core suite: 66 passed / 1 explicit ignored entry. Complete
  application suite: 29 passed. All-feature strict Core Clippy passed.
- Full desktop runner exited 0: 104 renderer, 14 ordinary-host and 17 all-feature
  host tests; types/lint/contracts/format/builds/strict Clippy passed. Host regression
  explicitly keeps previews outside the write gate.
- Exact ordinary native artifact showed Patch `1.4.2 → 1.4.3` and Minor
  `1.4.2 → 1.5.0` on the still-untrusted disposable control, with writes disabled.
  Traversal was refused without losing the manual message; a fresh valid preview
  worked afterward. Ordinary restart preserved the message and restored no preview
  result or execution. Both Quit exits were 0.
- Native independent oracle verified unchanged managed bytes/modes (including
  `1.4.2\r\n`), old operations/remote journals/trust, no control trust grant,
  manual draft persistence, source/artifact hashes, SQLite integrity and writer-lock
  reopen. One new untracked version file was explicit fixture preparation, before
  the preservation baseline; no managed ref/index/config was changed in preparation.

Evidence: [`g09-semver-preview-2026-11-08`](../validation/g09-semver-preview-2026-11-08/).
The preceding G08 full write/signing/remote matrix remains the current evidence for
unchanged execution engines; it was not rerun for this read-only preview boundary.

G09 is still open. Next: approved typed Action invocation, guarded file replacement,
exact staging and durable interrupted/idempotent recovery. AI connections/profiles,
provider transport, exact bounded context, assisted candidates and separately bounded
automatic authority remain unimplemented. No AI configuration is required by this
feature or the existing supported manual workflows.
