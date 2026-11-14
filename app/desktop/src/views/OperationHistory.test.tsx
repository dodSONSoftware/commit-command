import {
  cleanup,
  fireEvent,
  render,
  screen,
  act as reactAct,
} from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { Bridge, Payload } from "../services/bridge";
import type {
  Observation,
  OperationAuditEntry,
} from "../services/response.generated";
import { fixtureEntry } from "../testdata";
import { OperationHistory } from "./OperationHistory";
afterEach(cleanup);
const run = async (action: () => Promise<void>) => action();
function observation(
  id: string,
  phase: Observation["phase"] = "complete",
): Observation {
  return {
    plan: {
      id,
      workspace: "ws",
      worktree: "repo",
      metadata: fixtureEntry("repo", "repo").metadata,
      remote: "origin",
      reference: "refs/heads/main",
      destination: "https://example.invalid/<img src=x>",
      transport: "https",
      config_digest: "digest",
      generation: "generation",
      created_at_ms: 0,
      tools: [],
      local_target: null,
      local_binding: null,
    },
    phase,
    oid: null,
    observed_at_ms: phase === "complete" ? 0 : null,
    issue:
      phase === "blocked"
        ? { kind: "inspection_failed", message: "Server rejected <script>" }
        : null,
  };
}
function record(
  id: string,
  phase: "complete" | "blocked" | "interrupted" = "complete",
): OperationAuditEntry {
  return {
    id,
    created_at_ms: 0,
    operation_kind: "remote_observation",
    result: phase,
    record: { kind: "remote_observation", value: observation(id, phase) },
  };
}
function bridge(request: Bridge["request"]): Bridge {
  return {
    epoch: 1,
    request,
    capabilities: async () => ({ writes_enabled: false }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
  };
}
it("reads workspace-wide history, pages without replay, and refreshes to current results", async () => {
  const request = vi
    .fn<Bridge["request"]>()
    .mockResolvedValueOnce({
      kind: "operation_audit",
      value: { items: [record("first")], next: 50 },
    })
    .mockResolvedValueOnce({
      kind: "operation_audit",
      value: { items: [record("second", "blocked")], next: null },
    })
    .mockResolvedValueOnce({
      kind: "operation_audit",
      value: { items: [], next: null },
    });
  const { container } = render(
    <OperationHistory
      bridge={bridge(request)}
      workspace="ws"
      entries={[]}
      busy={false}
      act={run}
      revision={0}
    />,
  );
  await screen.findAllByText("first");
  expect(screen.getByText(/Reference absent/)).toHaveTextContent(
    "1970-01-01T00:00:00.000Z",
  );
  fireEvent.click(screen.getByRole("button", { name: "Load more results" }));
  await screen.findAllByText("second");
  expect(screen.getAllByText("first")[0]).toBeInTheDocument();
  expect(screen.getByRole("alert")).toHaveTextContent(
    "Server rejected <script>",
  );
  expect(container.querySelector("img,script")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Refresh" }));
  await screen.findByText("No recorded operations match these filters.");
  expect(screen.queryByText("first")).toBeNull();
  expect(request.mock.calls.map(([r]) => r)).toEqual(
    [0, 50, 0].map((offset) => ({
      kind: "operation_audit",
      workspace: "ws",
      query: {
        offset,
        worktree: null,
        operation_kind: null,
        result: null,
        since_ms: null,
        until_ms: null,
      },
    })),
  );
});
it("ignores superseded responses when native completion refreshes history", async () => {
  let oldResult!: (data: Payload) => void;
  const request = vi
    .fn<Bridge["request"]>()
    .mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          oldResult = resolve;
        }),
    )
    .mockResolvedValueOnce({
      kind: "operation_audit",
      value: { items: [record("current", "interrupted")], next: null },
    });
  const host = bridge(request);
  const { rerender } = render(
    <OperationHistory
      bridge={host}
      workspace="ws"
      entries={[]}
      busy={false}
      act={run}
      revision={0}
    />,
  );
  rerender(
    <OperationHistory
      bridge={host}
      workspace="ws"
      entries={[]}
      busy={false}
      act={run}
      revision={1}
    />,
  );
  await screen.findAllByText("current");
  await reactAct(async () => {
    oldResult({
      kind: "operation_audit",
      value: { items: [record("obsolete")], next: null },
    });
  });
  expect(screen.queryByText("obsolete")).toBeNull();
  expect(screen.getAllByText("current")[0]).toBeInTheDocument();
  expect(
    screen.getAllByRole("heading", { name: /interrupted/ })[0],
  ).toBeInTheDocument();
});

it("shows partial publication truthfully and requests only guarded recovery or filtered reads", async () => {
  const items: OperationAuditEntry[] = [
    {
      id: "local",
      created_at_ms: 0,
      operation_kind: "commit",
      result: "needs_attention",
      record: {
        kind: "local",
        value: {
          kind: "commit",
          worktree: "repo",
          subject: "Published fixture",
          created_at_ms: 0,
          events: [],
          events_complete: true,
          outcome: {
            operation_id: "local",
            phase: "published_needs_reconciliation",
            published: true,
            index_reconciled: false,
            new_head: "a".repeat(40),
            post_processing: [],
            reason: "Inspect reconciliation",
          },
        },
      },
    },
    {
      id: "batch",
      created_at_ms: 0,
      operation_kind: "batch_commit",
      result: "needs_attention",
      record: {
        kind: "batch",
        value: {
          record: {
            version: 1,
            id: "batch",
            workspace: "ws",
            created_at_ms: 0,
            phase: "partial_failure",
            issue: null,
            items: [],
          },
          outcomes: [
            {
              operation_id: "completed-child",
              phase: "complete",
              published: true,
              index_reconciled: true,
              new_head: "b".repeat(40),
              post_processing: [],
              reason: null,
            },
          ],
        },
      },
    },
  ];
  const request = vi.fn<Bridge["request"]>().mockImplementation(async (r) =>
    r.kind === "operation_audit"
      ? {
          kind: "operation_audit",
          value: { items: r.query.result ? [] : items, next: null },
        }
      : { kind: "done" },
  );
  render(
    <OperationHistory
      bridge={bridge(request)}
      workspace="ws"
      entries={[fixtureEntry("repo", "repo")]}
      busy={false}
      act={run}
      revision={0}
    />,
  );
  await screen.findByText("Published fixture");
  expect(screen.getByText(/completed-child/)).toHaveTextContent("Published");
  expect(screen.getByText(/Batch phase/)).toHaveTextContent("partial failure");
  fireEvent.click(
    screen.getByRole("button", { name: "Inspect guarded recovery" }),
  );
  expect(request).toHaveBeenCalledWith({
    kind: "review_recovery",
    workspace: "ws",
    operation: "local",
  });
  fireEvent.change(screen.getByLabelText("Audit operation type"), {
    target: { value: "remote_observation" },
  });
  fireEvent.change(screen.getByLabelText("Audit result"), {
    target: { value: "interrupted" },
  });
  fireEvent.change(screen.getByLabelText("Audit repository"), {
    target: { value: "repo" },
  });
  fireEvent.change(screen.getByLabelText("Audit since"), {
    target: { value: "2026-01-01" },
  });
  fireEvent.change(screen.getByLabelText("Audit until"), {
    target: { value: "2026-01-02" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Apply audit filters" }));
  await screen.findByText("No recorded operations match these filters.");
  expect(request).toHaveBeenLastCalledWith({
    kind: "operation_audit",
    workspace: "ws",
    query: {
      offset: 0,
      operation_kind: "remote_observation",
      result: "interrupted",
      worktree: "repo",
      since_ms: Date.parse("2026-01-01T00:00:00Z"),
      until_ms: Date.parse("2026-01-02T23:59:59.999Z"),
    },
  });
  expect(
    request.mock.calls.every(([r]) =>
      ["operation_audit", "review_recovery"].includes(r.kind),
    ),
  ).toBe(true);
});
it("renders interrupted pre-push checks as unpublished history without retry authority", async () => {
  const item = checkRecord();
  const request = vi.fn<Bridge["request"]>().mockResolvedValue({
    kind: "operation_audit",
    value: { items: [item], next: null },
  });
  render(
    <OperationHistory
      bridge={bridge(request)}
      workspace="ws"
      entries={[fixtureEntry("repo", "repo")]}
      busy={false}
      revision={0}
      act={run}
    />,
  );
  await screen.findByRole("heading", { name: "Pre-push check · interrupted" });
  expect(
    screen.getByText(/No publication was started by this check/),
  ).toHaveTextContent("never replayed");
  expect(
    screen.queryByRole("button", { name: /retry|push now|resume/i }),
  ).toBeNull();
  expect(request.mock.calls.every(([r]) => r.kind === "operation_audit")).toBe(
    true,
  );
});

function checkRecord(): OperationAuditEntry {
  const transport = observation("observed").plan;
  const source = { format: "sha1" as const, hex: "a".repeat(40) };
  const previous = { format: "sha1" as const, hex: "b".repeat(40) };
  const item: OperationAuditEntry = {
    id: "check",
    created_at_ms: 1,
    operation_kind: "push_check",
    result: "interrupted",
    record: {
      kind: "push_check",
      value: {
        phase: "interrupted",
        checked_at_ms: null,
        source_oid: source,
        hook_passed: false,
        published: false,
        issue: "Hook interrupted; inspect effects before fresh review",
        plan: {
          id: "check",
          workspace: "ws",
          worktree: "repo",
          created_at_ms: 1,
          selection: {
            worktree: "repo",
            observation: "observed",
            source_oid: source,
            children: [],
          },
          preview: {
            source_ref: "refs/heads/main",
            source_oid: source,
            remote: "origin",
            destination: transport.destination,
            destination_ref: "refs/heads/main",
            observed_oid: previous,
            observation_id: "observed",
            observed_at_ms: 1,
            generated_at_ms: 1,
            generation: "g",
            config_digest: "c",
            relation: "fast_forward",
            outgoing_count: 1,
            incoming_count: 0,
            outgoing: [],
            outgoing_truncated: false,
            source_gitlinks: [],
            outgoing_gitlinks: [],
            upstream_matches: false,
          },
          dependencies: [],
          transport,
          hooks: "execute",
          hook: null,
          hook_args: ["origin", transport.destination],
          hook_stdin: `refs/heads/main ${source.hex} refs/heads/main ${previous.hex}\n`,
          digest: "bound",
        },
      },
    },
  };
  return item;
}

it("records uncertain publication only through separately selected fresh observation review", async () => {
  const check = checkRecord();
  if (check.record.kind !== "push_check") throw new Error("Fixture mismatch");
  const item: OperationAuditEntry = {
    id: "publication",
    created_at_ms: 1,
    operation_kind: "push_publication",
    result: "needs_attention",
    record: {
      kind: "push_publication",
      value: {
        plan: {
          id: "publication",
          created_at_ms: 1,
          digest: "bound",
          check: check.record.value.plan,
        },
        phase: "unconfirmed",
        published: false,
        may_have_published: true,
        finished_at_ms: null,
        observed_remote_oid: null,
        remote_state: null,
        reconciled_observation: null,
        object_count: null,
        object_bytes: null,
        issue: null,
      },
    },
  };
  const request = vi.fn<Bridge["request"]>().mockImplementation(async (r) => {
    if (r.kind === "remote_observations")
      return {
        kind: "remote_observations",
        value: { items: [observation("fresh")], next: null },
      };
    if (r.kind === "review_push_publication_recovery")
      return {
        kind: "review",
        value: {
          id: "recovery",
          epoch: 1,
          expires_at_ms: 99,
          title: "Recovery",
          summary: "Record only",
          content: "{}",
        },
      };
    return { kind: "operation_audit", value: { items: [item], next: null } };
  });
  render(
    <OperationHistory
      bridge={bridge(request)}
      workspace="ws"
      entries={[]}
      busy={false}
      revision={0}
      act={run}
    />,
  );
  await screen.findByText(/Publication may have occurred/);
  expect(
    screen.getByRole("button", { name: "Review publication recovery" }),
  ).toBeDisabled();
  fireEvent.click(
    screen.getByRole("button", { name: "Load destination observations" }),
  );
  await screen.findByRole("option", { name: /origin.*main/ });
  fireEvent.change(screen.getByLabelText("Recovery observation"), {
    target: { value: "fresh" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Review publication recovery" }),
  );
  await reactAct(async () => {});
  expect(request).toHaveBeenCalledWith({
    kind: "review_push_publication_recovery",
    workspace: "ws",
    operation: "publication",
    observation: "fresh",
  });
  expect(
    request.mock.calls.every(([r]) =>
      [
        "operation_audit",
        "remote_observations",
        "review_push_publication_recovery",
      ].includes(r.kind),
    ),
  ).toBe(true);
});

it("shows a failed stage result without expanding technical details or offering needless recovery", async () => {
  const request = vi.fn<Bridge["request"]>().mockResolvedValue({
    kind: "operation_audit",
    value: {
      items: [
        {
          id: "blocked-stage",
          created_at_ms: 0,
          operation_kind: "stage",
          result: "blocked",
          record: {
            kind: "local",
            value: {
              kind: "stage",
              worktree: "repo",
              subject: "",
              created_at_ms: 0,
              events: [],
              events_complete: true,
              outcome: {
                operation_id: "blocked-stage",
                phase: "blocked",
                published: false,
                index_reconciled: false,
                new_head: null,
                post_processing: [],
                reason: "Fixture failure",
              },
            },
          },
        },
      ],
      next: null,
    },
  });
  render(
    <OperationHistory
      bridge={bridge(request)}
      workspace="ws"
      entries={[]}
      busy={false}
      act={run}
      revision={0}
    />,
  );
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Files were not staged. Fixture failure",
  );
  expect(
    screen.getByText("Technical details").closest("details"),
  ).not.toHaveAttribute("open");
  expect(
    screen.queryByRole("button", { name: "Inspect guarded recovery" }),
  ).toBeNull();
});
