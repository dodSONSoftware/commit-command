# M2 evidence — macOS/APFS, 2026-11-01

The final [validation transcript](validation.txt), [environment](environment.json)
and unchanged experimental [lab result format](lab-results.json) are separate
from the historical M0/M1 evidence. [Source SHA-256 values](source-sha256.json)
identify the validated Core sources/tests/scripts/manifest and local workflow;
[document checks](document-checks.txt) record link/whitespace results. No Git write operation targeted this project
or another existing user repository. Build outputs use ignored `app/core/target/`.

Reproduce from project root (with the locked dependency cache available):

```sh
GITFLEET_TEST_GPG=/absolute/path/to/gpg CARGO_NET_OFFLINE=true sh app/core/scripts/validate.sh ../../docs/validation/m2
```

The script changes into `app/core`, so the evidence argument is relative to there.
Omit offline mode if dependency downloads are needed. GPG, SSH signing, Rust 1.99+,
Git 2.56+, Python 3 and a C toolchain are required for the full macOS matrix.
GPG-agent must be able to create a local Unix socket. The sandbox denied that in
an initial isolated test; the approved final validation ran outside that sandbox
using only new disposable repositories and keyrings. No user's signing keys,
keyring, Git configuration or globally installed tooling were changed.

| Configuration | Passed | Failed | Cargo ignored |
|---|---:|---:|---:|
| Default | 47 | 0 | 1 |
| Experimental writes only | 82 | 0 | 2 |
| Controlled writes only | 70 | 0 | 1 |
| All features, including fixture crash support | 110 | 0 | 3 |

Each configuration's all-target build and Clippy with `-D warnings` passes. Format
and layout/manifest checks pass. Counts overlap; do not sum them. The all-feature
suite has 113 harness entries: 110 ordinary passes and three explicit ignores.
M2 contributes 23 ordinary controlled-write tests plus five crash/fault tests and
one ignored child driver. A new process unit regression brings the default total
to 47. The original POC suite and M0/M1 regressions remain enabled in their original
feature configurations.

Ignored/skipped entries, none counted as passes:

1. `successful_gpg_signing` remains the original optional POC Cargo ignore. The
   separate **mandatory M2 OpenPGP test passes**, as does SSH signing, with real
   independent signature verification. The POC lab reports **23 passed, 1 skipped**
   because GPG is not on its default PATH; M2 uses its explicit executable path.
2. `non_utf8_filename_remains_git_quoted_without_mutation` remains ignored on this
   macOS host due to the known filesystem EPERM fixture limitation. Byte parser
   tests and literal/newline filesystem tests pass; this exclusion remains open.
3. `crash_worker` is ignored by the parent harness to prevent standalone accidental
   execution. Passing crash tests explicitly invoke it in separate processes and
   assert real signal-9 termination. It is an invoked driver, not a missing crash
   scenario. The matrix covers 13 durable phases, including the ref/journal and
   index-rename/lock-release gaps; other tests use it for foreign-state/retrust and
   uncertain-post-processing checks.

Failure development history is not erased by the final passing result. Tests caught
Git 2.56's duplicate-HEAD transaction rejection, a premature ordinary-exit fallback
in the crash injector, canonical `/var` versus `/private/var` expectations, timeout
versus BrokenPipe error precedence, and missing private-blob visibility in hooks.
The final implementation uses native HEAD/index locks with detached-context branch
CAS, actual SIGKILL delivery, canonical assertions, timeout precedence and an isolated
hook index/object database. Regression coverage retains those findings.

Storage failures are injected at durable phase boundaries, plus corrupt candidate
index data and foreign ref/index/lock/reflog/artifact changes. These tests establish
process-crash and fail-closed behavior. They do not simulate every individual failed
syscall or prove physical power-loss/OS-crash durability. No native Linux/Windows,
network filesystem, non-APFS or hardware-token signing validation was performed.
Advanced indexes and hook managers are rejected, not counted as supported passes.

## Isolated OpenPGP tooling

This host had no GPG on PATH. GnuPG 2.5.24 and its libraries were built under
`/private/tmp/gitfleet-m2-tools/install`, without a global installation. Upstream
archive SHA-256 values were verified before extraction/build; the
[checksum manifest](gnupg-source-checksums.json) records the exact versions/digests.
Sources: [GnuPG downloads](https://gnupg.org/download/) and
[official integrity checks](https://gnupg.org/download/integrity_check.html).
libgcrypt was built with assembly disabled after its default assembler failed on
this toolchain. This is a temporary validation tool, not a shipped GitFleet binary
or reproducible distribution claim. It may be removed independently of the project;
future validation must provide a compatible GPG executable.

Every M2 OpenPGP fixture creates an unprotected **test-only** signing key in a new
short private temporary home, uses no personal keys, verifies the resulting signature,
kills its own isolated agent and removes that home. SSH uses disposable Ed25519
keys and an app-owned allowed-signers file. Private keys/tool binaries are not
included in repository evidence. Configured signer failures never become an
unsigned fallback or a silent skip.
