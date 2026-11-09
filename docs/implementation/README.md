# Commit Command completion handoff

Use this package to complete the macOS first release from baseline `17ea585` without rediscovering the remaining implementation and acceptance work.

Read in this order:

1. [Implementation runbook](COMPLETION_RUNBOOK.md) — current foundations, execution order, source ownership, detailed package steps, dependencies, commands and completion conditions.
2. [Decision worksheet](COMPLETION_DECISIONS.md) — nine unresolved policy/support decisions with concrete proposed values, exact required outputs and blocked packages. Proposals remain pending until selected by the appropriate owner.
3. [Test catalog](COMPLETION_TEST_CATALOG.md) — fixture recipes, independent oracles, test actions, expected results, automated/native/installed layers and evidence requirements.
4. [Requirement coverage](COMPLETION_REQUIREMENT_COVERAGE.md) — every original requirement, release invariant and M3 criterion routed to packages/cases.

Begin with P00 reconciliation and P01 fixture/evidence support. P13A backup primitives precede new durable schema changes. Advance independent packages while an operator/facility-dependent criterion is blocked. Do not repeat completed acceptance cases without a changed boundary, uncovered criterion or exact-artifact requirement.

[EXECUTION_STATUS.md](../EXECUTION_STATUS.md) remains the maintained live status/evidence ledger. This package defines remaining execution work; it does not claim that its catalog cases have passed or authorize live-service access, write-default changes, commits or release publication.

The original product specification, decisions and accepted support exclusions remain binding. Linux/Windows, Pull/branch mutation, hunk staging, selective hook toggles, arbitrary Actions and a persistent background service are not implicit new scope.

Validate the handoff structure and identifier coverage from the repository root:

```sh
python3 docs/implementation/validate_handoff.py
```

This checks document file links, code fences, whitespace, case references/ranges and complete requirement/invariant/M3 ID coverage. It does not execute application tests or certify acceptance.

[Execution ledger](execution-ledger.json) records concrete existing source/test/report routes for all requirements and the current execution disposition of each catalog case. Historical evidence candidates do not imply a new pass.
