import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { Bridge } from "../services/bridge";
import type {
  ObjectId,
  Observation,
  PushDependencyPlan,
  PushPreview,
} from "../services/response.generated";
import { fixtureEntry } from "../testdata";
import { PushDependencyPlanner } from "./PushDependencyPlanner";
afterEach(cleanup);
const oid = (hex: string): ObjectId => ({
  format: "sha1",
  hex: hex.repeat(40),
});
function preview(id: string): PushPreview {
  return {
    source_ref: "refs/heads/main",
    source_oid: oid(id === "a" ? "a" : "b"),
    remote: "origin",
    destination: `https://example.invalid/${id}.git`,
    destination_ref: "refs/heads/main",
    observed_oid: oid("f"),
    observation_id: `obs-${id}`,
    observed_at_ms: 1,
    generated_at_ms: 2,
    generation: `g-${id}`,
    config_digest: `c-${id}`,
    relation: "fast_forward",
    outgoing_count: 1,
    incoming_count: 0,
    outgoing: [],
    outgoing_truncated: false,
    source_gitlinks: [],
    upstream_matches: false,
    outgoing_gitlinks:
      id === "a"
        ? [
            {
              path: [99],
              oid: oid("b"),
              example_commit: oid("a"),
              mappings: [
                {
                  gitmodules_blob: oid("c"),
                  example_commit: oid("a"),
                  destination: {
                    display: "https://example.invalid/b.git",
                    transport: "https",
                    credentials_hidden: false,
                    restrictions: [],
                  },
                  issue: null,
                },
              ],
            },
          ]
        : [],
  };
}
function observed(id: string): Observation {
  return {
    plan: {
      id: `obs-${id}`,
      workspace: "ws",
      worktree: id,
      metadata: fixtureEntry(id, id).metadata,
      remote: "origin",
      reference: "refs/heads/main",
      destination: `https://example.invalid/${id}.git`,
      transport: "https",
      config_digest: `c-${id}`,
      generation: `g-${id}`,
      created_at_ms: 1,
      tools: [],
    },
    phase: "complete",
    oid: oid("f"),
    observed_at_ms: 1,
  };
}
function setup(cycle = false, enabled = false) {
  const entries = [fixtureEntry("a", "a"), fixtureEntry("b", "b")];
  const request = vi.fn<Bridge["request"]>().mockImplementation(async (r) => {
    if (r.kind === "remote_observations")
      return {
        kind: "remote_observations",
        value: { items: [observed(r.worktree ?? "a")], next: null },
      };
    if (r.kind === "push_preview")
      return { kind: "push_preview", value: preview(r.worktree) };
    if (r.kind === "inventory")
      return {
        kind: "inventory",
        value: { entries, monitor: { running: false, epoch: 1, watched: 0 } },
      };
    if (r.kind === "child_reachability")
      return {
        kind: "child_reachability",
        value: {
          parent: preview("a"),
          selection: r.selection,
          mapping: preview("a").outgoing_gitlinks[0].mappings[0],
          child_remote: "origin",
          child_destination: "https://example.invalid/b.git",
          child_reference: "refs/heads/main",
          observed_child_oid: oid("f"),
          child_observed_at_ms: 1,
          child_generation: "g-b",
          child_config_digest: "c-b",
          checked_at_ms: 2,
          reachable: false,
        },
      };
    if (r.kind === "review_push_check")
      return {
        kind: "review",
        value: {
          id: "separate-check",
          epoch: 1,
          expires_at_ms: 99,
          title: "Pre-push check",
          summary: "No publication",
          content: "{}",
        },
      };
    if (r.kind === "push_dependency_plan") {
      const selected = r.selections.some((s) => s.worktree === "b");
      const bound = r.selections.some(
        (s) => s.worktree === "a" && s.children.length === 1,
      );
      const conditional = selected && bound && !cycle;
      const plan: PushDependencyPlan = {
        generated_at_ms: 3,
        items: r.selections.map((s) => ({
          worktree: s.worktree,
          preview: preview(s.worktree),
        })),
        dependencies: [
          {
            parent_worktree: "a",
            path: [99],
            child_oid: oid("b"),
            gitmodules_blob: oid("c"),
            child_worktree: bound ? "b" : null,
            child_selected: selected,
            child_observation: bound ? "obs-b" : null,
            observed_child_oid: oid("f"),
            child_observed_at_ms: 1,
            state: conditional ? "selected_child_required" : "blocked",
            issue: conditional
              ? null
              : "Unselected or unproven child blocks the parent",
          },
        ],
        order: conditional ? ["b", "a"] : [],
        cycle: cycle ? ["a", "b", "a"] : [],
        issues: cycle ? ["Cyclic selected child dependencies"] : [],
      };
      return { kind: "push_dependency_plan", value: plan };
    }
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
    <PushDependencyPlanner
      bridge={bridge}
      enabled={enabled}
      workspace="ws"
      entries={entries}
      busy={false}
      act={async (a) => a()}
    />,
  );
  return request;
}
async function choose(id: string) {
  fireEvent.click(
    screen.getByRole("checkbox", { name: `Plan push for /tmp/project/${id}` }),
  );
  fireEvent.click(
    screen.getByRole("button", {
      name: `Load branch observations for /tmp/project/${id}`,
    }),
  );
  await waitFor(() =>
    expect(
      screen.getByLabelText(`Planned branch for /tmp/project/${id}`).children,
    ).toHaveLength(2),
  );
  fireEvent.change(
    screen.getByLabelText(`Planned branch for /tmp/project/${id}`),
    { target: { value: `obs-${id}` } },
  );
}
async function inspect() {
  fireEvent.click(
    screen.getByRole("button", { name: "Inspect selected push sources" }),
  );
  await screen.findByRole("heading", {
    name: "Child selections in this plan · 0",
  });
}
async function saveChild() {
  fireEvent.change(screen.getByLabelText("Historical child requirement"), {
    target: { value: "0" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Load child repository choices" }),
  );
  await screen.findByRole("option", { name: "/tmp/project/b" });
  fireEvent.change(screen.getByLabelText("Child repository"), {
    target: { value: "b" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Load approved child observations" }),
  );
  await waitFor(() =>
    expect(
      screen.getByLabelText("Child branch observation").children,
    ).toHaveLength(2),
  );
  fireEvent.change(screen.getByLabelText("Child branch observation"), {
    target: { value: "obs-b" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Check child ancestry locally" }),
  );
  fireEvent.click(
    await screen.findByRole("button", {
      name: "Use checked child selection for planning",
    }),
  );
  await screen.findByRole("heading", {
    name: "Child selections in this plan · 1",
  });
}
it("does not auto-select or auto-enroll an unavailable child and sends no network review", async () => {
  const request = setup();
  expect(request).not.toHaveBeenCalled();
  expect(
    screen.getByRole("button", { name: "Inspect selected push sources" }),
  ).toBeDisabled();
  fireEvent.click(screen.getByText("Plan push dependencies locally"));
  await choose("a");
  await inspect();
  await saveChild();
  expect(
    screen.getByRole("checkbox", { name: "Plan push for /tmp/project/b" }),
  ).not.toBeChecked();
  fireEvent.click(
    screen.getByRole("button", { name: "Build local dependency plan" }),
  );
  await screen.findByRole("heading", { name: "Dependency planning blocked" });
  const call = [...request.mock.calls]
    .reverse()
    .find(([r]) => r.kind === "push_dependency_plan")?.[0];
  if (call?.kind !== "push_dependency_plan")
    throw new Error("No planning request");
  expect(call.selections).toHaveLength(1);
  expect(call.selections[0].children[0].child_worktree).toBe("b");
  expect(call.selections[0].worktree).toBe("a");
  expect(
    request.mock.calls.every(([r]) =>
      [
        "remote_observations",
        "push_preview",
        "inventory",
        "child_reachability",
        "push_dependency_plan",
      ].includes(r.kind),
    ),
  ).toBe(true);
  expect(screen.queryByRole("button", { name: /^Push now/ })).toBeNull();
});
it("shows conditional child-before-parent order after explicit selection and clears stale results", async () => {
  setup();
  fireEvent.click(screen.getByText("Plan push dependencies locally"));
  await choose("a");
  await choose("b");
  fireEvent.click(
    screen.getByRole("button", { name: "Inspect selected push sources" }),
  );
  await waitFor(() =>
    expect(
      screen.getAllByRole("heading", {
        name: "Child selections in this plan · 0",
      }),
    ).toHaveLength(2),
  );
  await saveChild();
  fireEvent.click(
    screen.getByRole("button", { name: "Build local dependency plan" }),
  );
  await screen.findByRole("heading", { name: "Conditional review order" });
  expect(screen.getByRole("list").textContent).toBe(
    "/tmp/project/b/tmp/project/a",
  );
  expect(screen.getByText(/Selected child must publish first/)).toBeVisible();
  expect(
    screen.getByText(/No Git writes or network operations were executed/),
  ).toHaveTextContent("grants no push authority");
  fireEvent.click(
    screen.getByRole("checkbox", { name: "Plan push for /tmp/project/b" }),
  );
  expect(
    screen.queryByRole("heading", { name: "Conditional review order" }),
  ).toBeNull();
  expect(
    screen.getByRole("button", { name: "Build local dependency plan" }),
  ).toBeDisabled();
});
it("shows the closed cycle witness without a partial order", async () => {
  setup(true);
  fireEvent.click(screen.getByText("Plan push dependencies locally"));
  await choose("a");
  await inspect();
  fireEvent.click(
    screen.getByRole("button", { name: "Build local dependency plan" }),
  );
  await screen.findByText(
    "Selected dependency cycle: /tmp/project/a → /tmp/project/b → /tmp/project/a",
  );
  expect(screen.getByRole("list").children).toHaveLength(0);
  expect(
    screen.getByRole("heading", { name: "Dependency planning blocked" }),
  ).toBeVisible();
});

it("keeps separately approved hook checks disabled in the ordinary build", async () => {
  setup();
  fireEvent.click(screen.getByText("Plan push dependencies locally"));
  await choose("a");
  await inspect();
  expect(
    screen.getByRole("button", {
      name: "Review pre-push check for /tmp/project/a",
    }),
  ).toBeDisabled();
});
it("submits exact saved child selections and explicit hook policy for independent review", async () => {
  const request = setup(false, true);
  fireEvent.click(screen.getByText("Plan push dependencies locally"));
  await choose("a");
  await inspect();
  await saveChild();
  fireEvent.change(
    screen.getByLabelText("Pre-push hook policy for /tmp/project/a"),
    { target: { value: "bypass_all" } },
  );
  fireEvent.click(
    screen.getByRole("button", {
      name: "Review pre-push check for /tmp/project/a",
    }),
  );
  await waitFor(() =>
    expect(request).toHaveBeenCalledWith({
      kind: "review_push_check",
      workspace: "ws",
      selection: {
        worktree: "a",
        observation: "obs-a",
        source_oid: oid("a"),
        children: [
          expect.objectContaining({
            child_worktree: "b",
            child_observation: "obs-b",
          }),
        ],
      },
      hooks: "bypass_all",
    }),
  );
  expect(
    screen.getByText(/This separate approval can run the reviewed hook/),
  ).toHaveTextContent("publishes nothing");
});
