import {
  cleanup,
  fireEvent,
  render,
  screen,
  act as reactAct,
} from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { Bridge, Payload } from "../services/bridge";
import type { Observation } from "../services/response.generated";
import { fixtureEntry } from "../testdata";
import { WorkspaceObservationHistory } from "./ObservationHistory";
afterEach(cleanup);
const run = async (action: () => Promise<void>) => action();
function record(
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
      kind: "remote_observations",
      value: { items: [record("first")], next: 50 },
    })
    .mockResolvedValueOnce({
      kind: "remote_observations",
      value: { items: [record("second", "blocked")], next: null },
    })
    .mockResolvedValueOnce({
      kind: "remote_observations",
      value: { items: [], next: null },
    });
  const { container } = render(
    <WorkspaceObservationHistory
      bridge={bridge(request)}
      workspace="ws"
      busy={false}
      act={run}
      revision={0}
    />,
  );
  await screen.findByText("first");
  expect(screen.getByText(/Reference absent/)).toHaveTextContent(
    "1970-01-01T00:00:00.000Z",
  );
  fireEvent.click(
    screen.getByRole("button", { name: "Load older workspace observations" }),
  );
  await screen.findByText("second");
  expect(screen.getByText("first")).toBeInTheDocument();
  expect(screen.getByRole("alert")).toHaveTextContent(
    "Server rejected <script>",
  );
  expect(container.querySelector("img,script")).toBeNull();
  fireEvent.click(
    screen.getByRole("button", { name: "Refresh remote observation history" }),
  );
  await screen.findByText("No recorded remote observations in this workspace.");
  expect(screen.queryByText("first")).toBeNull();
  expect(request.mock.calls.map(([r]) => r)).toEqual(
    [0, 50, 0].map((offset) => ({
      kind: "remote_observations",
      workspace: "ws",
      worktree: null,
      offset,
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
      kind: "remote_observations",
      value: { items: [record("current", "interrupted")], next: null },
    });
  const host = bridge(request);
  const { rerender } = render(
    <WorkspaceObservationHistory
      bridge={host}
      workspace="ws"
      busy={false}
      act={run}
      revision={0}
    />,
  );
  rerender(
    <WorkspaceObservationHistory
      bridge={host}
      workspace="ws"
      busy={false}
      act={run}
      revision={1}
    />,
  );
  await screen.findByText("current");
  await reactAct(async () => {
    oldResult({
      kind: "remote_observations",
      value: { items: [record("obsolete")], next: null },
    });
  });
  expect(screen.queryByText("obsolete")).toBeNull();
  expect(screen.getByText("current")).toBeInTheDocument();
  expect(
    screen.getByRole("heading", { name: /interrupted/ }),
  ).toBeInTheDocument();
});
