import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { Bridge } from "../services/bridge";
import type { PushPreview, Observation } from "../services/response.generated";
import { fixtureEntry } from "../testdata";
import { ChildReachabilityForm } from "./ChildReachabilityForm";
afterEach(cleanup);
const oid = { format: "sha1" as const, hex: "a".repeat(40) };
const blob = { format: "sha1" as const, hex: "b".repeat(40) };
const preview: PushPreview = {
  source_ref: "refs/heads/main",
  source_oid: oid,
  remote: "origin",
  destination: "https://example.invalid/parent.git",
  destination_ref: "refs/heads/main",
  observed_oid: oid,
  observation_id: "parent-observation",
  observed_at_ms: 0,
  generated_at_ms: 1,
  generation: "parent-generation",
  config_digest: "config",
  relation: "fast_forward",
  outgoing_count: 1,
  incoming_count: 0,
  outgoing: [],
  outgoing_truncated: false,
  source_gitlinks: [],
  upstream_matches: false,
  outgoing_gitlinks: [
    {
      path: [99],
      oid,
      example_commit: oid,
      mappings: [
        {
          gitmodules_blob: blob,
          declared_url: null,
          resolution_base: null,
          example_commit: oid,
          destination: {
            display: "https://example.invalid/child.git",
            transport: "https",
            credentials_hidden: false,
            restrictions: [],
          },
          issue: null,
        },
      ],
    },
  ],
};
const observation: Observation = {
  plan: {
    id: "child-observation",
    workspace: "ws",
    worktree: "child",
    metadata: fixtureEntry("child", "child").metadata,
    remote: "origin",
    reference: "refs/heads/main",
    destination: "https://example.invalid/child.git",
    transport: "https",
    config_digest: "config",
    generation: "child-generation",
    created_at_ms: 0,
    tools: [],
  },
  phase: "complete",
  oid,
  observed_at_ms: 0,
};
it("requires explicit mapping, repository and approved observation; sends only a local evidence request", async () => {
  const request = vi.fn<Bridge["request"]>().mockImplementation(async (r) => {
    if (r.kind === "inventory")
      return {
        kind: "inventory",
        value: {
          entries: [
            fixtureEntry("parent", "parent"),
            fixtureEntry("child", "child"),
          ],
          monitor: { running: false, epoch: 1, watched: 0, degraded: null },
        },
      };
    if (r.kind === "remote_observations")
      return {
        kind: "remote_observations",
        value: { items: [observation], next: null },
      };
    if (r.kind === "child_reachability")
      return {
        kind: "child_reachability",
        value: {
          parent: preview,
          selection: r.selection,
          mapping: preview.outgoing_gitlinks[0].mappings[0],
          child_remote: "origin",
          child_destination: "https://example.invalid/child.git",
          child_reference: "refs/heads/main",
          observed_child_oid: oid,
          child_observed_at_ms: 0,
          child_generation: "child-generation",
          child_config_digest: "config",
          checked_at_ms: 1,
          reachable: true,
        },
      };
    throw new Error("Unexpected request");
  });
  const bridge: Bridge = {
    epoch: 1,
    request,
    capabilities: async () => ({ writes_enabled: false }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
  };
  render(
    <ChildReachabilityForm
      bridge={bridge}
      workspace="ws"
      worktree="parent"
      preview={preview}
      busy={false}
      act={async (a) => a()}
    />,
  );
  const check = screen.getByRole("button", {
    name: "Check child ancestry locally",
  });
  expect(request).not.toHaveBeenCalled();
  expect(check).toBeDisabled();
  fireEvent.change(screen.getByLabelText("Historical child requirement"), {
    target: { value: "0" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Load child repository choices" }),
  );
  await screen.findByRole("option", { name: "/tmp/project/child" });
  expect(
    screen.queryByRole("option", { name: "/tmp/project/parent" }),
  ).toBeNull();
  expect(check).toBeDisabled();
  fireEvent.change(screen.getByLabelText("Child repository"), {
    target: { value: "child" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Load approved child observations" }),
  );
  await waitFor(() =>
    expect(
      screen.getByLabelText("Child branch observation").children,
    ).toHaveLength(2),
  );
  expect(check).toBeDisabled();
  fireEvent.change(screen.getByLabelText("Child branch observation"), {
    target: { value: "child-observation" },
  });
  fireEvent.click(check);
  expect(await screen.findByRole("status")).toHaveTextContent(
    "Reachable from observed child branch",
  );
  expect(request).toHaveBeenLastCalledWith({
    kind: "child_reachability",
    workspace: "ws",
    worktree: "parent",
    selection: {
      parent_observation: "parent-observation",
      source_oid: oid,
      path: [99],
      child_oid: oid,
      gitmodules_blob: blob,
      child_worktree: "child",
      child_observation: "child-observation",
    },
  });
  expect(
    request.mock.calls.every(([r]) =>
      ["inventory", "remote_observations", "child_reachability"].includes(
        r.kind,
      ),
    ),
  ).toBe(true);
  fireEvent.change(screen.getByLabelText("Child repository"), {
    target: { value: "" },
  });
  expect(screen.queryByRole("status")).toBeNull();
  expect(check).toBeDisabled();
});
