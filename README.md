# Commit Command

**Release:** Aluminum Ferret — version 0.2.3.

Commit Command (formerly GitFleet) is a macOS multi-repository Git workspace
manager under development. It provides persistent workspaces, repository discovery
and monitoring, status/diffs/history, commit graphs, workspace activity and operation
audit. A presentation-independent Rust Core powers a Tauri/React desktop adapter.

Controlled local and bulk commits, supported authenticated observation/fetch/import,
upstream setup, push dependency planning, separately reviewed pre-push checks
and exact individual fast-forward publication with uncertain-outcome recovery
are implemented within bounded acceptance envelopes. Guided push queues, guarded
SemVer Actions, optional AI settings/templates and native-approved provider
diagnostics/source message suggestions, read-only unsaved-editor repository tests
and endpoint-bound macOS Keychain authentication are implemented
with selected acceptance. **Ordinary desktop builds enable controlled Git trust, writes and supported remote
execution through the default `desktop-writes` feature. Acceptance remains partial. Hierarchical AI analysis,
bounded automatic commits, general backup/restore
and release readiness remain unfinished. Manual workflows require no AI setup.**

Start with the [current implementation and acceptance record](docs/EXECUTION_STATUS.md)
for completed work, open gates G01–G11, scope limits, validation and next steps.
Historical reports are evidence for their own checkpoints, not current status.

- [Documentation index](docs/README.md)
- [Product specification](docs/GITFLEET_SPECIFICATION.md)
- [Requirement-to-code/test crosswalk](docs/REQUIREMENT_EXECUTION_CROSSWALK.md)
- [Completion work packages](docs/PROJECT_COMPLETION_PLAN.md)
- [Detailed completion handoff and test procedures](docs/implementation/README.md)
- [Core CLI and safety envelope](app/core/README.md)
- [Desktop build and validation](app/desktop/README.md)

```sh
GITFLEET_TEST_GPG=/absolute/path/to/gpg sh app/core/scripts/validate.sh
```

Validation requires Rust 1.99+, Git 2.56.0+, Python 3 and a C toolchain. All Git
writes use new disposable fixtures. macOS M2 validation requires GPG and SSH signing
tools and an environment permitting isolated local gpg-agent sockets. Workspace state is stored in an explicitly
selected application data directory outside managed repositories. The current record
distinguishes implemented capabilities from unaccepted or future work. Development bundles and historical test passes are not release acceptance.
