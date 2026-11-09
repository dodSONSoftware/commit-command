# What Commit Command Is

Updated November 7, 2026. Current acceptance and next work are maintained in
[EXECUTION_STATUS.md](EXECUTION_STATUS.md). This overview separates implemented
capabilities from intended product features; it does not declare production readiness.

## The application in one paragraph

**Commit Command (formerly GitFleet) is a local desktop workspace manager for developers working across multiple Git repositories.** You choose a directory containing your projects, and GitFleet discovers the repositories beneath it, presents their status together, and lets you inspect changes, commit history, and activity without opening each repository separately. Its controlled-write services support explicitly reviewed staging and commits, including batches across selected repositories. Those services are present in the code, but ordinary desktop builds keep trust grants, Git writes, and managed Git remote execution disabled while acceptance work continues. Supported separately reviewed observation, fetch download/import and upstream setup are implemented. Exact individual fast-forward publication and separately reviewed observation recovery are implemented with selected acceptance; guided per-item push queues and guarded deterministic SemVer Actions also have selected acceptance. Optional AI settings, profiles, templates, model discovery, synthetic diagnostics and endpoint-bound native Keychain authentication are implemented. Each provider request requires exact native approval. Bounded repository context, exact native source review and text suggestions for manual drafts are implemented with selected fixture acceptance. Hierarchical model analysis and separately bounded automatic commits remain unfinished; all supported manual workflows work without AI.

## The problem it addresses

A single logical project can contain an application, libraries, services, tools, and documentation in separate repositories. Each repository has its own branch, index, working files, history, and remote configuration. Checking them individually makes it easy to miss changes, confuse staged and unstaged content, or publish repositories in the wrong order.

GitFleet provides one place to answer:

- Which repositories have staged, unstaged, untracked, or conflicted files?
- What branch is each repository on, and what does its local tracking information show?
- What changed, which version of a file is staged, and what would a proposed commit contain?
- What happened recently across the workspace?
- Which GitFleet operations completed, failed, were cancelled, or need recovery?

It works with existing local repositories and the installed Git executable. Repositories retain their independent histories and normal Git structure.

## What a workspace means

A workspace is a saved name, one root directory, discovery settings, and associated application state. GitFleet can save multiple workspaces, with one active at a time. Active-workspace monitoring combines filesystem notifications with reconciliation scans and status refreshes while the application runs.

Discovery recognizes ordinary repositories, independent nested repositories, Git submodules, and linked worktrees. These distinctions matter: a repository inside another directory is not automatically a submodule, and linked worktrees share some Git storage while keeping separate working files and indexes.

Repository selection is explicit. Selecting a parent repository does not select its children or authorize operations on them. Workspace membership also does not grant repository trust.

For example:

```text
projects/                  <- saved workspace root
  desktop-client/          <- repository
  api-service/             <- repository
  shared-library/          <- repository
  tools/
    release-helper/        <- independent nested repository
```

GitFleet brings these repositories into one overview while retaining their individual Git identities. Creating a workspace stores application metadata outside the managed repositories; it does not turn the directory into a monorepo.

## What is implemented today

The following describes source capabilities. Feature availability and acceptance status are separate, as explained below.

| Area | Implemented behavior |
| --- | --- |
| Workspace management | Save, activate, rename, remove, discover, and refresh workspaces; monitor the active workspace. Removing a saved definition removes application metadata, not project files. |
| Workspace overview | Table and tree presentations, repository status, explicit selection, and repository tabs. |
| Changes inspection | Staged and unstaged changes, untracked previews, diffs, and persistent commit drafts. Staged content and working-file content are distinct sources. |
| Repository history | Commit history, parent/merge graph, commit details, and historical diffs. Each graph belongs to its repository. |
| Workspace activity | Timeline, activity lanes, and contribution heatmap derived from Git history, with filters and navigation into repository history. Activity lanes do not imply ancestry between unrelated repositories. |
| Local controlled writes | Feature-gated full-file stage/unstage and selective commits, exact content/message review, trust checks, supported hook policies, signing, durable outcomes, and guarded recovery. |
| Bulk local commits | Explicit selected-worktree batches, shared messages and per-repository choices, exclusions, index-only or working-content selection, dependency checks, and durable results. |
| Operation records | Local operation phases and outcomes, batch results, and remote-observation history. Desktop views expose the unified audit chronology, filters and detailed records. |
| Remote inspection | On-demand inspection of configured remotes, effective destinations, upstream settings, and policy issues, with sensitive URL fields redacted. This inspection does not contact servers. |
| Remote branch observation | Feature-gated, independently reviewed observation of one exact remote branch in isolated transport storage. It records a timestamped object ID or absence without fetching into the managed repository. Selected native acceptance passed; broader supported compatibility remains open. |
| Authentication and fetch | Named nonsecret HTTPS/SSH profiles and explicit assignments; reviewed authenticated observation, isolated verified bundle download, separately reviewed offline import/tracking-ref update, and guarded missing-only upstream setup. Ordinary execution remains gated. |
| Push preparation | Local outgoing-history preview, historical gitlink mappings, explicit child reachability evidence, multi-level dependency planning and separately reviewed pre-push hook checks. Checks and planning grant no publication authority. A separate exact individual push review can publish one existing branch through isolated transport; G07 acceptance remains partial. |
| Desktop state | Saved navigation, tabs, drafts, bulk composer preferences, theme, and panel/window state, with close/restart handling. |

The Core also supplies a headless inspection CLI. A separate feature-gated interactive write CLI and disposable test tools exercise the controlled-write services; the ordinary inspection CLI exposes no Git mutation command. Its `commit` command reads a historical commit.

## What the ordinary desktop build permits

The default desktop application supports local inspection and saves its own workspace and presentation state. **Ordinary builds reject desktop trust grants, write execution, and remote-observation review in Rust, and disable the corresponding controls in the interface.** Read-only here refers to managed Git repositories, not to the application's own settings and database.

The separate Cargo feature `desktop-writes` enables acceptance workflows. It is disabled by default. Enabling it does not itself trust a repository or approve an operation: the normal trust and independent native review requirements still apply. Write validation uses disposable fixtures and isolated signing material.

The default Core crate also has no write features enabled. The desktop adapter explicitly includes Core's application services and controlled-write implementation, then applies its own desktop capability gate. Code being compiled into the desktop backend does not make it available for ordinary execution.

## How a controlled commit works

GitFleet's central write contract is that the committed tree and message must match the snapshot the user approved.

1. The user chooses a repository and explicit content sources. For a partially staged file, its index version and working-file version can differ.
2. Core checks repository identity, trust, configuration, supported Git state, hooks, and signing policy, then prepares an immutable proposal containing the exact intended content and message.
3. The desktop displays Core's proposal in a dedicated review view and requests independent native confirmation. A frontend checkbox or request field cannot grant execution authority.
4. Core revalidates the repository and approved state before executing. Changes to the branch, selected content, index, trust, or relevant configuration can invalidate the review.
5. The transaction records publication, index reconciliation, and post-processing outcomes. Recovery inspects durable records and actual repository state rather than automatically replaying a commit.

The selective-commit engine uses isolated preparation and guarded ref/index updates to preserve unrelated staging and working content within its supported configuration envelope. It is more constrained than an unrestricted Git command runner. Unsupported index formats, Git states, hooks, and platform cases must be rejected or separately validated.

Hooks and signing programs that the user trusts execute with the user's permissions. They are not sandboxed. Review protects GitFleet's approved operation; it does not isolate arbitrary trusted local programs from the machine.

## How bulk commits differ from one transaction

A batch contains only explicitly selected worktrees and their exact proposals. The current implementation accepts up to 20 selected worktrees per batch and executes sequentially. Actual selected submodule dependencies determine child-before-parent ordering; ordinary directory nesting does not.

If a child commit changes a parent's proposed gitlink, the parent requires fresh review. A failure or cancellation stops successors and preserves any commits already published. **A batch is not atomic:** some repositories can succeed before another fails. Retrying unfinished work requires a new review and excludes already published successes.

Here, “published” in a local commit result means that the local branch reference advanced to the new commit. It does not mean the commit was pushed to a server.

## Local status, activity, and operation history mean different things

- **Status** describes current local Git state. Ahead/behind counts reflect locally available tracking information and can be stale relative to a server.
- **Workspace Activity** describes commits found in Git history, including commits made outside GitFleet.
- **Operation History** describes work performed or proposed through GitFleet. External terminal or IDE operations are not fabricated as GitFleet audit events.
- **Remote observation** records what one approved server query saw at a particular time. It does not prove ongoing synchronization or update the managed repository's tracking refs.

Refreshing local status is therefore different from contacting a remote, fetching objects, or pushing commits.

## How the application is built

| Component | Responsibility |
| --- | --- |
| Rust Core — `app/core/` | Workspace/discovery services, Git inspection, history and graph data, application requests, monitoring, trust, controlled transactions, recovery, and operation records. It has no Tauri or React dependency. |
| Native desktop adapter — `app/desktop/src-tauri/` | Tauri 2 host, directory picker, scoped IPC, caller checks, background jobs/cancellation, review windows, native confirmation, and desktop lifecycle coordination. |
| Desktop interface — `app/desktop/src/` | React/TypeScript workspace and repository views, composers, graphs, activity, and result presentation. Generated TypeScript contracts describe Core requests and responses. |
| Git executable | Executes bounded Git commands selected by the Rust services. The interface has no generic shell or arbitrary Git-command endpoint. |
| Application storage | TOML workspace/configuration data plus SQLite structured state, operation records, and private transaction artifacts outside managed checkouts. |

On macOS, the default desktop data directory is `~/Library/Application Support/dev.gitfleet.desktop`. A trusted process-start `--data-dir` option can select another location outside managed repositories. Application data has a single-owner lock and versioned storage/migration handling.

The initial desktop target is macOS. Portable Core design and cross-platform dependencies do not establish acceptance of other desktop platforms.

## What belongs to the intended product but remains unfinished

The specification describes a broader product than the current executable. Outstanding capabilities include:

- Broader individual push acceptance and bulk push coordination; exact individual publication, conservative uncertain outcomes and separately reviewed observation recovery now have selected automated/native evidence.
- Remaining native acceptance for implemented fetch/authentication/upstream and dependency planning/check workflows.
- Hierarchical model analysis and broader actual-provider/native acceptance for the implemented optional source context, message suggestions, profiles/templates/settings and synthetic diagnostics.
- User-initiated, bounded, explicitly authorized AI automatic commits.
- Broader native Action recovery/usability acceptance; guarded SemVer replacement, retained-original inspection and separate Stage/Commit reviews are implemented.
- Additional accessibility/scale acceptance and clearly documented supported hook compatibility. Hunk staging and individual hook toggles remain post-v1.
- Backup/import/export and retention interfaces, release packaging, signing/notarization, and distribution acceptance.

GitFleet is a developer tool running on the local machine. It does not currently provide a cloud service, hosting platform, general coding agent, background auto-commit daemon, or full Git conflict-resolution suite. AI output remains separate from authorization to change files or execute Git operations.

## Current maturity and evidence

The project is under active development. Package manifests identify Core and desktop as version `0.1.1`; this is not a production-readiness claim. The execution tracker records automated coverage and selected native acceptance across desktop lifecycle, graph/activity, local batches, and remote inspection. Broader native lifecycle, accessibility, scale, remote, and release gates remain open, and ordinary desktop writes remain disabled.

Some repository introductions and historical milestone documents describe older checkpoints. Use the current execution tracker and specific reports for acceptance evidence, and the implementation for capability details. The current record contains the November 7 focused validation results and environment blocks. Historical native reports attest only their recorded source/artifacts; implementation is not release acceptance.

## Further reading

- [Product and technical specification](GITFLEET_SPECIFICATION.md) — intended requirements and workflows.
- [Current execution status](EXECUTION_STATUS.md) — implementation checkpoints, acceptance evidence, and remaining work.
- [Requirement execution crosswalk](REQUIREMENT_EXECUTION_CROSSWALK.md) — requirement-by-requirement routing.
- [Desktop build and validation instructions](../app/desktop/README.md) — development commands and capability gates; its historical acceptance text should be read alongside the execution tracker.
- [Desktop and IPC architecture](architecture/M3_DESKTOP_AND_IPC.md) — presentation and authority boundaries.
- [Controlled-write contracts](architecture/M2_CONTROLLED_WRITES.md) — transaction behavior and supported envelope; introductory milestone status is historical.
- [Local bulk-commit contracts](architecture/M5A_LOCAL_BULK_COMMITS.md) — batch selection, dependencies, and recovery.
- [Remote boundary](architecture/M5_REMOTE_BOUNDARY.md) — local inspection, reviewed observation, and unfinished fetch/push services.
