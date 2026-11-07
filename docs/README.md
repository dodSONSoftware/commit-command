# GitFleet — Specification Package v1.0

**Date:** November 1, 2026 · **Status:** Consolidated product/engineering requirements baseline

GitFleet is a desktop multi-repository Git workspace manager with a replaceable Tauri/React presentation adapter and UI-independent Rust Core. This package consolidates the approved product decisions, technical architecture, security rules, workflows, AI model/profile system, reliability requirements, and a milestone-oriented implementation roadmap.

## Contents

- [What GitFleet is](APPLICATION_OVERVIEW.md) — Application purpose, implemented behavior, default capability gates, architecture, and unfinished product features.

- [M4 graph activity and audit contracts](architecture/M4_GRAPH_ACTIVITY_AUDIT.md) — Implemented read-only services and remaining native acceptance.

- [Completion execution status](EXECUTION_STATUS.md) — Single current implementation, acceptance, open-gate and next-action record.

- [Requirement execution crosswalk](REQUIREMENT_EXECUTION_CROSSWALK.md) — Current routing for all 109 original requirements.
- [P0 baseline report](reports/P0_BASELINE_2026-11-02.md) — Historical P0 execution baseline and validation.

- [`PROJECT_COMPLETION_PLAN.md`](PROJECT_COMPLETION_PLAN.md) — Detailed remaining work packages and exit conditions; use the current record for status and ordering.
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
5. Read the current record and open-gate queue before proposing work; historical handoffs do not define current status.
6. **Do not modify live repositories** or connect to live remotes/AI hosts without explicit authorization. Default testing must use disposable Git repos and mock services.

## Verification baseline

User-supplied macOS Git 2.56.0 / Rust Core v0.1.1 results:

- `cargo build --workspace --all-targets` — passed.
- `cargo test --workspace --all-targets` — 24 acceptance + 6 core behavior tests passed.
- `gitfleet-lab` — 23 scenarios passed, positive GPG scenario skipped (GPG not installed on the Mac).

These do not prove production readiness or complete hook compatibility.

## Current implementation and acceptance

Use [EXECUTION_STATUS.md](EXECUTION_STATUS.md) as the single current record of
implementation, acceptance gaps G01–G11, supported scope and next actions. M3–M5
have implemented capabilities and selected native evidence; their remaining gates
are open. Ordinary writes remain disabled; actual push, AI, Actions and general
backup/release workflows remain unfinished. The crosswalk supplies requirement
ownership, and the completion plan supplies detailed work packages.

`IMPLEMENTATION_STATUS.md` and `IMPLEMENTATION_HANDOFF.md` preserve older checkpoints.
Dated reports, original acceptance tables and validation directories retain their
historical results and exclusions. They must not be interpreted as a newer stop
instruction, an untested feature claim, or current release acceptance.
