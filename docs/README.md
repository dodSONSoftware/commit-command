# GitFleet — Specification Package v1.0

**Date:** November 1, 2026 · **Status:** Consolidated product/engineering requirements baseline

GitFleet is a desktop multi-repository Git workspace manager with a replaceable Tauri/React presentation adapter and UI-independent Rust Core. This package consolidates the approved product decisions, technical architecture, security rules, workflows, AI model/profile system, reliability requirements, and a milestone-oriented implementation roadmap.

## Contents

- [`GITFLEET_SPECIFICATION.md`](GITFLEET_SPECIFICATION.md) — **Authoritative product + technical specification** (start here).
- [`ACCEPTANCE_MATRIX.md`](ACCEPTANCE_MATRIX.md) — Testable requirements, POC evidence labels, and milestone gates.
- [`DECISIONS_AND_OPEN_ITEMS.md`](DECISIONS_AND_OPEN_ITEMS.md) — Confirmed decisions, proposed defaults, and unresolved validation.
- [`assets/01-core-workflows.png`](assets/01-core-workflows.png) — Six-screen conceptual board: workspace, history, changes, staging, review, bulk commits.
- [`assets/02-ai-and-settings.png`](assets/02-ai-and-settings.png) — Six-screen conceptual board: AI assistance, templates, profiles, Actions, backup.
- [`assets/03-operations-and-reliability.png`](assets/03-operations-and-reliability.png) — Six-screen conceptual board: push, history, activity, hooks, trust and recovery.

The 18 screen images are **design references**. Generated imagery can depict features, states or labels outside the agreed v1 scope; the written requirements and acceptance matrix are authoritative.

## Recommended Codex review sequence

1. Read the main specification fully, particularly sections 2, 6–9, 15–19.
2. Compare the existing `gitfleet-rust-core-poc-v0.1.1` with the stated commit transaction and capability boundaries.
3. Review `ACCEPTANCE_MATRIX.md` and document POC-covered vs missing production acceptance tests.
4. Review `DECISIONS_AND_OPEN_ITEMS.md` before inventing defaults or adding functionality.
5. Propose an implementation plan for M0/M1/M2 only; treat later milestones as protected product scope, not immediate work.
6. **Do not modify live repositories** or connect to live remotes/AI hosts without explicit authorization. Default testing must use disposable Git repos and mock services.

## Verification baseline

User-supplied macOS Git 2.56.0 / Rust Core v0.1.1 results:

- `cargo build --workspace --all-targets` — passed.
- `cargo test --workspace --all-targets` — 24 acceptance + 6 core behavior tests passed.
- `gitfleet-lab` — 23 scenarios passed, positive GPG scenario skipped (GPG not installed on the Mac).

These do not prove production readiness or complete hook compatibility.

## Future deliverables

A separate Codex implementation handoff and production-ready scaffolding should be created **after** review of this specification and scope signoff. This package is a requirements artifact, not an executable project.
