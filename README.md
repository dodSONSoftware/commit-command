# GitFleet

GitFleet is a multi-repository workspace manager under development. M1 provides
headless workspaces, inspection, history and diffs; M2 adds explicitly trusted,
reviewed controlled writes for the documented local macOS/APFS envelope.
M3 adds a Tauri/React desktop adapter, scoped IPC and active-workspace monitoring.
Native acceptance is partially exercised but incomplete, so M3 is not complete and
ordinary desktop writes remain disabled. The latest user-observed native results
and remaining gates are in the [November 2 handoff](docs/M3_NATIVE_ACCEPTANCE_HANDOFF_2026-11-02.md).
The single Rust Core crate remains
independent of Tauri/React. Default Core builds remain read-only for managed Git repositories.

- [Specification and documentation](docs/README.md)
- [Implementation status and validation](docs/IMPLEMENTATION_STATUS.md)
- [Conversation handoff](docs/IMPLEMENTATION_HANDOFF.md)
- [Core CLI examples](app/core/README.md)
- [M1 contracts and safety envelope](docs/architecture/M1_READ_ONLY_WORKSPACES.md)
- [M2 controlled-write contracts and limitations](docs/architecture/M2_CONTROLLED_WRITES.md)
- [M3 desktop and IPC contracts](docs/architecture/M3_DESKTOP_AND_IPC.md)
- [Desktop build and pending native acceptance](app/desktop/README.md)

```sh
GITFLEET_TEST_GPG=/absolute/path/to/gpg sh app/core/scripts/validate.sh
```

Validation requires Rust 1.99+, Git 2.56.0+, Python 3 and a C toolchain. All Git
writes use new disposable fixtures. macOS M2 validation requires GPG and SSH signing
tools and an environment permitting isolated local gpg-agent sockets. Workspace state is stored in an explicitly
selected application data directory outside managed repositories. M4 and later
remain unauthorized. No commit, push, publication or deployment was done.
