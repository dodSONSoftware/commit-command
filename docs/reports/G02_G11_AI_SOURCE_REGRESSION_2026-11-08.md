# G02–G11 Core regression after assisted AI source integration — November 8, 2026

Disposition: complete local regression passed against the Core source committed in
`d9d6954`. This refreshes automated evidence; it does not close native facility,
AI completeness, backup/restore or release gates.

The complete `app/core/scripts/validate.sh` exited zero with its documented four test
threads, offline locked dependencies, four Cargo jobs and isolated GPG executable.
All builds, format/scaffold/prerequisite checks, test configurations and strict Clippy
checks passed. Counts below sum test binaries within each configuration; ignored
subprocess workers/capability cases remain ignored and are not counted as passes.

| Configuration | Passed | Failed | Ignored |
|---|---:|---:|---:|
| Default | 66 | 0 | 1 |
| Experimental writes | 101 | 0 | 2 |
| Controlled writes | 92 | 0 | 1 |
| Application services | 318 | 0 | 4 |
| All features | 367 | 0 | 11 |

The separate disposable lab passed 24 cases, zero skipped. The suite includes
controlled local writes, signing, interruption/recovery, remote authentication,
fetch/import/upstream, expected-old publication, guided queues, version Actions and
AI configuration/provider/context/application boundaries. It does not run a real
server publication, OS session termination, accessibility matrix or release installer.
Only disposable repositories/services were used by this runner.

Environment: macOS 27.2 arm64; Git 2.56.0; Rust/Cargo 1.99.1; isolated GnuPG 2.5.24 /
libgcrypt 1.12.4. Source/artifact manifests in the assisted-message checkpoint match
the committed application/Core files; uncommitted real-provider evidence does not
change the tested Core tree. The current native artifact remains an ordinary,
ad-hoc-signed acceptance bundle with managed repository writes disabled.

Evidence:

- `docs/validation/g09-ai-assisted-messages-2026-11-08/core-matrix.txt`
- `docs/validation/g09-ai-assisted-messages-2026-11-08/core-matrix/environment.json`
- `docs/validation/g09-ai-assisted-messages-2026-11-08/core-matrix/lab-results.json`
- `docs/validation/g09-ai-assisted-messages-2026-11-08/core-matrix/summary.json`

G02 facility restrictions and retained intermittent-native counterevidence remain.
G09 hierarchical analysis/automatic authorization and wider provider acceptance,
G10 backup/restore/retention, and G11 hosted CI/distribution/license/signing/install
acceptance remain open. No push, publication, tag, installer or release was created.
