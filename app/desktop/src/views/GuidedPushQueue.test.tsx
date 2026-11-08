import { useState } from "react";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { Bridge } from "../services/bridge";
import type { PushBatch, PushPreview } from "../services/response.generated";
import { GuidedPushQueue } from "./GuidedPushQueue";
afterEach(cleanup);
const oid = { format: "sha1" as const, hex: "a".repeat(40) };
const preview: PushPreview = {
  source_ref: "refs/heads/main",
  source_oid: oid,
  remote: "origin",
  destination: "https://example.invalid/repo.git",
  destination_ref: "refs/heads/main",
  observed_oid: { ...oid, hex: "b".repeat(40) },
  observation_id: "fresh",
  observed_at_ms: 1,
  generated_at_ms: 2,
  generation: "g",
  config_digest: "c",
  relation: "fast_forward",
  outgoing_count: 1,
  incoming_count: 0,
  outgoing: [],
  outgoing_truncated: false,
  source_gitlinks: [],
  outgoing_gitlinks: [],
  upstream_matches: true,
};
function batch(phase: PushBatch["phase"] = "ready"): PushBatch {
  return {
    phase,
    items: [
      { worktree: "child", phase: "pending", publication: null },
      { worktree: "parent", phase: "pending", publication: null },
    ],
    plan: {
      id: "queue",
      workspace: "ws",
      created_at_ms: 1,
      digest: "fixed",
      selections: ["child", "parent"].map((worktree) => ({
        selection: {
          worktree,
          observation: "fresh",
          source_oid: oid,
          children: [],
        },
        hooks: "execute",
      })),
      dependency_plan: {
        generated_at_ms: 1,
        items: ["child", "parent"].map((worktree) => ({ worktree, preview })),
        order: ["child", "parent"],
        dependencies: [],
        cycle: [],
        issues: [],
      },
    },
  };
}
function setup(enabled = true, initial = batch()) {
  let stored = structuredClone(initial);
  const request = vi.fn<Bridge["request"]>().mockImplementation(async (r) => {
    if (r.kind === "push_batch")
      return { kind: "push_batch", value: structuredClone(stored) };
    if (r.kind === "review_push_batch_item") {
      stored = {
        ...stored,
        items: [
          {
            worktree: "child",
            phase: "complete",
            publication: "child-publication",
          },
          { worktree: "parent", phase: "pending", publication: null },
        ],
      };
      return {
        kind: "review",
        value: {
          id: "native-review",
          epoch: 1,
          expires_at_ms: 99,
          title: "Exact item",
          summary: "Only this item",
          content: "{}",
        },
      };
    }
    throw new Error("Unexpected request");
  });
  const bridge: Bridge = {
    epoch: 1,
    request,
    capabilities: async () => ({ writes_enabled: enabled }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
  };
  function Harness() {
    const [state, setState] = useState(initial);
    return (
      <GuidedPushQueue
        bridge={bridge}
        workspace="ws"
        batch={state}
        sources={[
          { worktree: "child", preview },
          { worktree: "parent", preview },
        ]}
        bindings={{}}
        enabled={enabled}
        busy={false}
        revision={0}
        act={async (f) => f()}
        onChange={setState}
        label={(id) => id}
      />
    );
  }
  render(<Harness />);
  return request;
}
it("requests only the next explicit item and never automatically approves the parent", async () => {
  const request = setup();
  await waitFor(() =>
    expect(request).toHaveBeenCalledWith({
      kind: "push_batch",
      workspace: "ws",
      id: "queue",
    }),
  );
  fireEvent.click(
    screen.getByRole("button", { name: "Review next guided push for child" }),
  );
  await screen.findByRole("button", {
    name: "Review next guided push for parent",
  });
  expect(request).toHaveBeenCalledWith({
    kind: "review_push_batch_item",
    workspace: "ws",
    batch: "queue",
    selection: {
      worktree: "child",
      observation: "fresh",
      source_oid: oid,
      children: [],
    },
  });
  expect(
    request.mock.calls.filter(([r]) => r.kind === "review_push_batch_item"),
  ).toHaveLength(1);
  expect(
    request.mock.calls.every(([r]) =>
      ["push_batch", "review_push_batch_item"].includes(r.kind),
    ),
  ).toBe(true);
});
it("keeps native item reviews disabled in the ordinary build", async () => {
  const request = setup(false);
  await waitFor(() => expect(request).toHaveBeenCalled());
  expect(
    screen.getByRole("button", { name: "Review next guided push for child" }),
  ).toBeDisabled();
});
it.each(["stopped", "interrupted"] as const)(
  "does not offer continuation for a %s queue",
  async (phase) => {
    const request = setup(true, batch(phase));
    await waitFor(() => expect(request).toHaveBeenCalled());
    expect(screen.queryByRole("button", { name: /Review next/ })).toBeNull();
    expect(screen.getByText(/This queue cannot continue/)).toHaveTextContent(
      "new plan",
    );
    expect(request.mock.calls.every(([r]) => r.kind === "push_batch")).toBe(
      true,
    );
  },
);
