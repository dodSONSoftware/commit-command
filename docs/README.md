# GitFleet — Specification Package v1.0

**Date:** November 1, 2026 · **Status:** Consolidated product/engineering requirements baseline

GitFleet is a desktop multi-repository Git workspace manager with a replaceable Tauri/React presentation adapter and UI-independent Rust Core. This package consolidates the approved product decisions, technical architecture, security rules, workflows, AI model/profile system, reliability requirements, and a milestone-oriented implementation roadmap.

## Contents

- [Requirement execution crosswalk](REQUIREMENT_EXECUTION_CROSSWALK.md) — Current routing for all 109 original requirements.
- [P0 baseline report](reports/P0_BASELINE_2026-11-02.md) — Current execution baseline and validation.

- [`PROJECT_COMPLETION_PLAN.md`](PROJECT_COMPLETION_PLAN.md) — Proposed implementation sequence from the current M3 checkpoint through M8 release acceptance, with work packages, dependencies and exit gates.
- [`GITFLEET_SPECIFICATION.md`](GITFLEET_SPECIFICATION.md) — **Authoritative product + technical specification** (start here).
- [`ACCEPTANCE_MATRIX.md`](ACCEPTANCE_MATRIX.md) — Testable requirements, POC evidence labels, and milestone gates.
- [`DECISIONS_AND_OPEN_ITEMS.md`](DECISIONS_AND_OPEN_ITEMS.md) — Confirmed decisions, proposed defaults, and unresolved validation.
- [`mockups/01-core-workflows.png`](mockups/01-core-workflows.png) — Six-screen conceptual board: workspace, history, changes, staging, review, bulk commits.
- [`mockups/02-ai-and-settings.png`](mockups/02-ai-and-settings.png) — Six-screen conceptual board: AI assistance, templates, profiles, Actions, backup.
- [`mockups/03-operations-and-reliability.png`](mockups/03-operations-and-reliability.png) — Six-screen conceptual board: push, history, activity, hooks, trust and recovery.

The 18 screen images are **design references**. Generated imagery can depict features, states or labels outside the agreed v1 scope; the written requirements and acceptance matrix are authoritative.

## Recommended Codex review sequence

1. Read the main specification fully, particularly sections 2, 6–9, 15–19.
2. Inspect `app/core/` and `app/desktop/` against the current implementation status and stated capability boundaries.
3. Review `ACCEPTANCE_MATRIX.md` and document POC-covered vs missing production acceptance tests.
4. Review `DECISIONS_AND_OPEN_ITEMS.md` before inventing defaults or adding functionality.
5. Read the current handoff before proposing further work; M3 native gates remain open, and later milestones require authorization.
6. **Do not modify live repositories** or connect to live remotes/AI hosts without explicit authorization. Default testing must use disposable Git repos and mock services.

## Verification baseline

User-supplied macOS Git 2.56.0 / Rust Core v0.1.1 results:

- `cargo build --workspace --all-targets` — passed.
- `cargo test --workspace --all-targets` — 24 acceptance + 6 core behavior tests passed.
- `gitfleet-lab` — 23 scenarios passed, positive GPG scenario skipped (GPG not installed on the Mac).

These do not prove production readiness or complete hook compatibility.

## Implementation progress

M0 and then M1 were authorized after review. See [Implementation status](IMPLEMENTATION_STATUS.md),
[Conversation handoff](IMPLEMENTATION_HANDOFF.md), and the
[M0 service and safety contracts](architecture/M0_SAFETY_BOUNDARIES.md).
The baseline above is historical v0.1.1 evidence; current commands, results and
explicit skips are recorded in implementation status. M2 is implemented for the documented local macOS/APFS envelope. M3 was authorized and implemented; native acceptance is partial and incomplete. The [November 2 checkpoint](M3_NATIVE_ACCEPTANCE_HANDOFF_2026-11-02.md) records later user-observed results and remaining gates after the earlier agent access denial. Ordinary desktop writes remain disabled. Core has no presentation dependency; M4 and later remain unauthorized.


M1 service/CLI usage and safety contracts are in
[Read-only workspace architecture](architecture/M1_READ_ONLY_WORKSPACES.md).
Historical M0 results remain under `validation/m0/`; M1 evidence is separate.


M2 trust, review approvals, controlled writes, hooks/signing and recovery are defined
in [M2 contracts](architecture/M2_CONTROLLED_WRITES.md), with a separate
[evidence index](validation/m2/README.md). Default inspection and the historical
POC remain separate. M1 completion snapshots are preserved under `validation/m1/`.

M3 desktop/IPC contracts are in [M3 architecture](architecture/M3_DESKTOP_AND_IPC.md),
with [separate evidence and open native gates](validation/m3/README.md). Native mock
and renderer tests do not establish native human approval or accessibility acceptance.
