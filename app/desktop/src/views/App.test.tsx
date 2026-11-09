import {
  cleanup,
  render,
  screen,
  waitFor,
  fireEvent,
  act,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";
import { App } from "./App";
import type { Bridge, Payload } from "../services/bridge";
import type { Request } from "../services/request.generated";
import type { DesktopState } from "../services/response.generated";
import { initialState, initialBulkComposer } from "../stores/model";
import { fixtureEntry, fixtureWorkspace } from "../testdata";
afterEach(() => {
  cleanup();
  vi.useRealTimers();
});
it("announces sortable headers, persists the chosen direction and preserves selection", async () => {
  const f = fake();
  const user = userEvent.setup();
  render(<App bridge={f.bridge} />);
  const selection = await screen.findByRole("checkbox", {
    name: "Select parent",
  });
  await user.click(selection);
  const header = screen.getByRole("columnheader", { name: "Staged" });
  await user.click(screen.getByRole("button", { name: "Staged" }));
  expect(header).toHaveAttribute("aria-sort", "ascending");
  await user.click(screen.getByRole("button", { name: "Staged ↑" }));
  expect(header).toHaveAttribute("aria-sort", "descending");
  expect(selection).toBeChecked();
  await waitFor(() => expect(f.saved.at(-1)?.sort).toBe("staged_desc"));
  expect(f.saved.at(-1)?.selected).toEqual(["parent"]);
});
it("persists tree collapse and attention filters while keeping hidden children selected", async () => {
  const f = fake();
  const user = userEvent.setup();
  render(<App bridge={f.bridge} />);
  await screen.findByRole("checkbox", { name: "Select parent" });
  await user.click(screen.getByRole("button", { name: "Tree" }));
  await user.click(screen.getByRole("checkbox", { name: "Select child" }));
  await user.click(screen.getByText("parent", { selector: "summary" }));
  await waitFor(() => expect(f.saved.at(-1)?.collapsed).toEqual(["/parent"]));
  expect(f.saved.at(-1)?.selected).toEqual(["child"]);
  expect(
    screen.getByRole("checkbox", { name: "Select child", hidden: true }),
  ).not.toBeVisible();
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Repository attention filter" }),
    "conflicted",
  );
  await waitFor(() => expect(f.saved.at(-1)?.attention).toBe("conflicted"));
  expect(f.saved.at(-1)?.selected).toEqual(["child"]);
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Repository attention filter" }),
    "all",
  );
  expect(
    screen.getByRole("checkbox", { name: "Select child", hidden: true }),
  ).not.toBeVisible();
  await user.click(screen.getByText("parent", { selector: "summary" }));
  expect(screen.getByRole("checkbox", { name: "Select child" })).toBeChecked();
});
function fake(writes = false) {
  const requests: Request[] = [];
  const saved: DesktopState[] = [];
  const bridge: Bridge = {
    epoch: 1,
    capabilities: async () => ({ writes_enabled: writes }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
    request: async (request): Promise<Payload> => {
      requests.push(request);
      switch (request.kind) {
        case "workspaces":
          return {
            kind: "workspaces",
            value: {
              workspaces: [fixtureWorkspace],
              active: "ws",
              recovery: [],
              fetch_import_recovery: [],
              upstream_recovery: [],
            },
          };
        case "load_bulk_composer":
          return {
            kind: "bulk_composer",
            value: { ...initialBulkComposer, configurations: {} },
          };
        case "save_bulk_composer":
          return { kind: "done" };
        case "load_state":
          return { kind: "state", value: { ...initialState, drafts: {} } };
        case "inventory":
        case "scan":
          return {
            kind: "inventory",
            value: {
              entries: [
                fixtureEntry("parent", "parent"),
                fixtureEntry("child", "parent/child"),
              ],
              monitor: { running: true, epoch: 1, watched: 1, degraded: null },
            },
          };
        case "save_state":
          saved.push(request.state);
          return { kind: "done" };
        case "graph":
          return {
            kind: "graph",
            value: {
              rows: [],
              references: [],
              next: null,
              shallow_boundaries: [],
              generation: "g",
            },
          };
        case "history":
          return {
            kind: "history",
            value: {
              commits: [],
              references: [],
              next: null,
              shallow: false,
              shallow_boundaries: [],
              generation: "g",
              observed_at_ms: 1,
            },
          };
        case "fetch_captures":
          return { kind: "fetch_captures", value: { items: [], next: null } };
        case "remote_observations":
          return {
            kind: "remote_observations",
            value: { items: [], next: null },
          };
        case "operation_audit":
          return { kind: "operation_audit", value: { items: [], next: null } };
        case "audit":
          return { kind: "audit", value: { items: [], next: null } };
        case "operations":
          return { kind: "operations", value: { items: [], next: null } };
        default:
          return { kind: "done" };
      }
    },
  };
  return { bridge, requests, saved };
}
it("shares Table/Tree selection without implicit descendants", async () => {
  const { bridge } = fake();
  render(<App bridge={bridge} />);
  const parent = await screen.findByRole("checkbox", { name: "Select parent" });
  await userEvent.click(parent);
  expect(
    screen.getByRole("checkbox", { name: "Select parent/child" }),
  ).not.toBeChecked();
  await userEvent.click(screen.getByRole("button", { name: "Tree" }));
  expect(
    screen.getByRole("button", { name: "Tree", pressed: true }),
  ).toBeInTheDocument();
  expect(
    screen.getByRole("button", { name: "Table", pressed: false }),
  ).toBeInTheDocument();
  expect(screen.getByRole("checkbox", { name: "Select parent" })).toBeChecked();
  expect(
    screen.getByRole("checkbox", { name: "Select child" }),
  ).not.toBeChecked();
  await userEvent.click(screen.getByRole("button", { name: "Table" }));
  expect(screen.getByRole("checkbox", { name: "Select parent" })).toBeChecked();
});
it("preserves a composer draft through overview navigation and saves it", async () => {
  const { bridge, saved } = fake();
  render(<App bridge={bridge} />);
  await screen.findByRole("checkbox", { name: "Select parent" });
  await userEvent.click(
    screen.getAllByRole("button", { name: "parent" }).at(-1)!,
  );
  const composer = screen.getByRole("textbox", { name: "Message" });
  fireEvent.change(composer, { target: { value: "Draft remains mine" } });
  await userEvent.click(screen.getByRole("button", { name: "▦ Overview" }));
  await userEvent.click(
    screen.getAllByRole("button", { name: "parent" }).at(-1)!,
  );
  expect(screen.getByRole("textbox", { name: "Message" })).toHaveValue(
    "Draft remains mine",
  );
  await waitFor(() =>
    expect(saved.some((s) => s.drafts.parent === "Draft remains mine")).toBe(
      true,
    ),
  );
});
it("keeps write controls disabled in the ordinary desktop while inspection works", async () => {
  const { bridge, requests } = fake();
  render(<App bridge={bridge} />);
  await screen.findByRole("checkbox", { name: "Select parent" });
  await userEvent.click(
    screen.getAllByRole("button", { name: "parent" }).at(-1)!,
  );
  expect(screen.getByRole("button", { name: "Review trust" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Review stage" })).toBeDisabled();
  expect(
    screen.getByRole("button", { name: "Inspect trust & eligibility" }),
  ).toBeEnabled();
  expect(
    requests.some((r) => r.kind === "propose" || r.kind === "review_trust"),
  ).toBe(false);
});
it("does not turn repository labels into executable markup", async () => {
  const { bridge } = fake();
  const request = bridge.request;
  bridge.request = async (r) => {
    const data = await request(r);
    if (data.kind === "inventory") {
      data.value.entries[0].display_path = {
        encoding: "utf8",
        value: "/tmp/project/<img src=x onerror=evil()>",
      };
    }
    return data;
  };
  const { container } = render(<App bridge={bridge} />);
  await screen.findByText("<img src=x onerror=evil()>");
  // Repository labels stay text; only the four bundled banner images may render.
  const images = Array.from(container.querySelectorAll("img"));
  expect(images).toHaveLength(4);
  for (const image of images)
    expect(image.getAttribute("src")).toMatch(
      /^\/src\/assets\/brand\/banner-(dark|light)\.svg$/,
    );
  expect(container.querySelector("[onerror]")).toBeNull();
});
it("renders and filters a 100-repository inventory without changing explicit selection", async () => {
  const { bridge } = fake();
  const original = bridge.request;
  bridge.request = async (request) => {
    const result = await original(request);
    if (result.kind === "inventory")
      result.value.entries = Array.from({ length: 100 }, (_, i) =>
        fixtureEntry(`repo-${i}`, `repo-${i}`),
      );
    return result;
  };
  render(<App bridge={bridge} />);
  await screen.findByRole("checkbox", { name: "Select repo-99" });
  await userEvent.click(
    screen.getByRole("checkbox", { name: "Select repo-99" }),
  );
  fireEvent.change(
    screen.getByRole("textbox", { name: "Filter repositories" }),
    { target: { value: "repo-99" } },
  );
  expect(
    screen.getByRole("checkbox", { name: "Select repo-99" }),
  ).toBeChecked();
  expect(screen.queryByRole("checkbox", { name: "Select repo-0" })).toBeNull();
});
it("clears the removed active workspace and operation view, then loads another workspace's drafts", async () => {
  const { bridge, requests } = fake();
  const original = bridge.request;
  let removed = false;
  let active: string | null = "ws";
  bridge.request = async (request) => {
    const data = await original(request);
    if (request.kind === "remove") {
      removed = true;
      active = null;
    }
    if (request.kind === "activate") active = request.workspace;
    if (data.kind === "workspaces") {
      data.value.workspaces = [
        ...(removed ? [] : [fixtureWorkspace]),
        { ...fixtureWorkspace, id: "other", name: "Other workspace" },
      ];
      data.value.active = active;
    }
    if (data.kind === "state" && active === "other")
      data.value.drafts = { parent: "Other workspace draft" };
    return data;
  };
  render(<App bridge={bridge} />);
  await screen.findByRole("checkbox", { name: "Select parent" });
  await userEvent.click(
    screen.getAllByRole("button", { name: "parent" }).at(-1)!,
  );
  fireEvent.change(screen.getByRole("textbox", { name: "Message" }), {
    target: { value: "Removed workspace draft" },
  });
  await userEvent.click(
    screen.getByRole("button", { name: "◷ Operation results" }),
  );
  await screen.findByText("No recorded operations match these filters.");
  await userEvent.click(screen.getByText("Workspace settings"));
  await userEvent.click(
    screen.getByRole("button", { name: "Remove saved definition" }),
  );
  await waitFor(() => {
    expect(
      screen.getByRole("heading", { name: "Welcome" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("checkbox", { name: "Select parent" }),
    ).toBeNull();
    expect(
      screen.queryByRole("button", { name: "Close parent tab" }),
    ).toBeNull();
    expect(
      screen.queryByRole("heading", { name: "Operation results" }),
    ).toBeNull();
  });
  // Changing the theme must not save the old workspace after removal.
  const afterRemoval = requests.length;
  await userEvent.selectOptions(
    screen.getByRole("combobox", { name: "Color theme" }),
    "dark",
  );
  await userEvent.selectOptions(
    screen.getByRole("combobox", { name: "Workspace" }),
    "other",
  );
  await screen.findByRole("checkbox", { name: "Select parent" });
  await userEvent.click(
    screen.getAllByRole("button", { name: "parent" }).at(-1)!,
  );
  expect(screen.getByRole("textbox", { name: "Message" })).toHaveValue(
    "Other workspace draft",
  );
  expect(
    requests
      .slice(afterRemoval)
      .some((r) => "workspace" in r && r.workspace === "ws"),
  ).toBe(false);
});

it("keeps Changes patches and previews out of History and restores the selected historical comparison", async () => {
  const { bridge } = fake();
  const original = bridge.request;
  const encode = (text: string) => Array.from(new TextEncoder().encode(text));
  const oid = { format: "sha1" as const, hex: "1".repeat(40) };
  const commit = {
    oid,
    tree: oid,
    parents: [],
    author: encode("Fixture author"),
    committer: encode("Fixture author"),
    author_time: 1,
    committer_time: 1,
    message: encode("Historical commit"),
  };
  let finishChanges: ((data: Payload) => void) | undefined;
  const pendingChanges = new Promise<Payload>((resolve) => {
    finishChanges = resolve;
  });
  let delayChanges = false;
  bridge.request = async (request) => {
    const data = await original(request);
    if (data.kind === "graph")
      data.value.rows = [
        { commit, lane: 0, tip: true, before: [oid], after: [] },
      ];
    if (request.kind === "commit") return { kind: "commit", value: commit };
    if (request.kind === "preview")
      return {
        kind: "preview",
        value: {
          binary: false,
          bytes: encode("Working preview"),
          complete: true,
          fingerprint: "f",
          kind: "file",
          path: request.path,
        },
      };
    if (request.kind === "diff") {
      if (delayChanges && request.mode.kind !== "historical")
        return pendingChanges;
      return {
        kind: "diff",
        value: {
          complete: true,
          files: [],
          generation: "g",
          mode: request.mode,
          observed_at_ms: 1,
          patch: encode(
            request.mode.kind === "historical"
              ? "Historical patch"
              : "Changes patch",
          ),
        },
      };
    }
    return data;
  };
  render(<App bridge={bridge} />);
  await screen.findByRole("checkbox", { name: "Select parent" });
  await userEvent.click(
    screen.getAllByRole("button", { name: "parent" }).at(-1)!,
  );
  await userEvent.click(screen.getAllByRole("button", { name: "file.txt" })[0]);
  await screen.findByText("Changes patch");
  await userEvent.click(
    screen.getByRole("button", { name: "Read bounded working-file preview" }),
  );
  await screen.findByText("Working preview");
  await userEvent.click(screen.getByRole("button", { name: "History" }));
  expect(screen.queryByText("Changes patch")).toBeNull();
  expect(screen.queryByText("Working preview")).toBeNull();
  expect(
    screen.getByText(
      "Select a commit for its exact message, parents and changes.",
    ),
  ).toBeInTheDocument();
  await userEvent.click(
    await screen.findByRole("button", { name: /Historical commit/ }),
  );
  await screen.findByText("Historical patch");
  await userEvent.click(screen.getByRole("button", { name: "Changes 1" }));
  await screen.findByText("Changes patch");
  expect(screen.queryByText("Historical patch")).toBeNull();
  // A late file diff must not overwrite the restored History comparison.
  delayChanges = true;
  await userEvent.click(screen.getAllByRole("button", { name: "file.txt" })[1]);
  await userEvent.click(screen.getByRole("button", { name: "History" }));
  await screen.findByText("Historical patch");
  finishChanges!({
    kind: "diff",
    value: {
      complete: true,
      files: [],
      generation: "g",
      mode: { kind: "unstaged" },
      observed_at_ms: 1,
      patch: encode("Late Changes patch"),
    },
  });
  await waitFor(() =>
    expect(screen.getByText("Historical patch")).toBeInTheDocument(),
  );
  expect(screen.queryByText("Late Changes patch")).toBeNull();
});

it("refreshes trust automatically and removes selections whose source is no longer changed", async () => {
  vi.useFakeTimers({ toFake: ["setInterval", "clearInterval"] });
  const { bridge } = fake(true);
  const original = bridge.request;
  let clean = false;
  bridge.request = async (request) => {
    if (request.kind === "trust_status")
      return {
        kind: "trust",
        value: [
          {
            worktree: "parent",
            grant: "granted",
            validity: "current",
            eligible: true,
            reason: null,
            policy: "execute",
          },
        ],
      };
    const result = await original(request);
    if (clean && result.kind === "inventory") {
      const status = result.value.entries[0].status!;
      status.changes = [];
      status.counts = { staged: 0, unstaged: 0, untracked: 0, conflicted: 0 };
    }
    return result;
  };
  render(<App bridge={bridge} />);
  await screen.findByRole("checkbox", { name: "Select parent" });
  await userEvent.click(
    screen.getAllByRole("button", { name: "parent" }).at(-1)!,
  );
  expect(
    screen
      .getByRole("button", { name: "Inspect trust & eligibility" })
      .closest(".repository-tabs"),
  ).toBeNull();
  await userEvent.click(
    screen.getByRole("checkbox", { name: "Include file.txt from index" }),
  );
  expect(screen.getByRole("button", { name: "Review stage" })).toBeEnabled();
  expect(
    screen.getByRole("button", { name: "Review commit · 1 files" }),
  ).toBeInTheDocument();
  clean = true;
  await act(async () => {
    await vi.advanceTimersByTimeAsync(3000);
  });
  {
    expect(screen.getByText("granted · current")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Review commit · 0 files" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Review stage" })).toBeDisabled();
  }
  clean = false;
  await act(async () => {
    await vi.advanceTimersByTimeAsync(3000);
  });
  {
    expect(
      screen.getByRole("checkbox", { name: "Include file.txt from index" }),
    ).not.toBeChecked();
    expect(screen.getByRole("button", { name: "Review stage" })).toBeDisabled();
  }
});

it("keeps audit pagination bound to applied filters until edited filters are submitted", async () => {
  const { bridge, requests } = fake();
  const original = bridge.request;
  bridge.request = async (request) => {
    if (request.kind !== "operation_audit") return original(request);
    requests.push(request);
    return {
      kind: "operation_audit",
      value: {
        items:
          request.query.offset === 0 && !request.query.operation_kind
            ? [
                {
                  id: "record",
                  created_at_ms: 1,
                  operation_kind: "stage",
                  result: "complete",
                  record: {
                    kind: "local",
                    value: {
                      outcome: {
                        operation_id: "record",
                        phase: "complete",
                        published: false,
                        index_reconciled: true,
                        new_head: null,
                        post_processing: [],
                        reason: null,
                      },
                      worktree: "parent",
                      kind: "stage",
                      subject: "",
                      created_at_ms: 1,
                      events: [],
                      events_complete: true,
                    },
                  },
                },
              ]
            : [],
        next:
          request.query.offset === 0 && !request.query.operation_kind
            ? 50
            : null,
      },
    };
  };
  render(<App bridge={bridge} />);
  await screen.findByRole("checkbox", { name: "Select parent" });
  await userEvent.click(
    screen.getByRole("button", { name: "◷ Operation results" }),
  );
  await screen.findByRole("button", { name: "Load more results" });
  fireEvent.change(
    screen.getByRole("combobox", { name: "Audit operation type" }),
    { target: { value: "commit" } },
  );
  await userEvent.click(
    screen.getByRole("button", { name: "Load more results" }),
  );
  const queries = requests.filter((r) => r.kind === "operation_audit");
  expect(queries.at(-1)).toMatchObject({
    query: { offset: 50, operation_kind: null },
  });
  await userEvent.click(
    screen.getByRole("button", { name: "Apply audit filters" }),
  );
  expect(
    requests.filter((r) => r.kind === "operation_audit").at(-1),
  ).toMatchObject({
    query: { offset: 0, operation_kind: "commit" },
  });
  await screen.findByText("No recorded operations match these filters.");
});

it("keeps shared bulk messages and content policy across selection navigation and restart", async () => {
  const { bridge } = fake();
  const original = bridge.request;
  let composer = structuredClone(initialBulkComposer);
  let navigation = structuredClone(initialState);
  bridge.request = async (request) => {
    const result = await original(request);
    if (request.kind === "save_bulk_composer")
      composer = structuredClone(request.state);
    if (request.kind === "load_bulk_composer")
      return { kind: "bulk_composer", value: structuredClone(composer) };
    if (request.kind === "save_state")
      navigation = structuredClone(request.state);
    if (request.kind === "load_state")
      return { kind: "state", value: structuredClone(navigation) };
    return result;
  };
  const first = render(<App bridge={bridge} />);
  await userEvent.click(
    await screen.findByRole("checkbox", { name: "Select parent" }),
  );
  await userEvent.click(screen.getByRole("button", { name: "Bulk Commit" }));
  fireEvent.change(screen.getByLabelText("Shared commit message"), {
    target: { value: "Keep shared subject\n\nKeep body" },
  });
  fireEvent.change(screen.getByLabelText("Content · parent"), {
    target: { value: "index_only" },
  });
  fireEvent.click(screen.getByLabelText("Exclude file.txt"));
  await userEvent.click(screen.getByRole("button", { name: "▦ Overview" }));
  await userEvent.click(screen.getByRole("button", { name: "Bulk Commit" }));
  expect(screen.getByLabelText("Shared commit message")).toHaveValue(
    "Keep shared subject\n\nKeep body",
  );
  expect(screen.getByLabelText("Content · parent")).toHaveValue("index_only");
  expect(screen.getByLabelText("Exclude file.txt")).toBeChecked();
  fireEvent.click(
    screen.getByRole("button", { name: "Clear path exclusions · parent" }),
  );
  await waitFor(
    () => {
      expect(composer.message).toBe("Keep shared subject\n\nKeep body");
      expect(composer.configurations.parent.mode).toBe("index_only");
      expect(composer.configurations.parent.excluded).toEqual([]);
    },
    { timeout: 2000 },
  );
  first.unmount();
  render(<App bridge={bridge} />);
  await screen.findByRole("checkbox", { name: "Select parent" });
  await userEvent.click(screen.getByRole("button", { name: "Bulk Commit" }));
  expect(screen.getByLabelText("Shared commit message")).toHaveValue(
    "Keep shared subject\n\nKeep body",
  );
  expect(screen.getByLabelText("Content · parent")).toHaveValue("index_only");
  expect(screen.getByLabelText("Exclude file.txt")).not.toBeChecked();
});

it("keeps Cancel enabled across overlapping graph and commit reads until both settle", async () => {
  const { bridge } = fake();
  const original = bridge.request;
  const finish: Record<string, (data: Payload) => void> = {};
  bridge.cancel = vi.fn(async () => {});
  bridge.request = async (request) => {
    if (request.kind === "load_state")
      return {
        kind: "state",
        value: {
          ...initialState,
          active_tab: "parent",
          tabs: [
            {
              worktree: "parent",
              view: "history",
              path: null,
              source: "working_tree",
              commit: "1".repeat(40),
              parent: null,
              filter: "",
              scroll: 0,
            },
          ],
        },
      };
    if (request.kind === "graph" || request.kind === "commit")
      return new Promise<Payload>((resolve) => {
        finish[request.kind] = resolve;
      });
    return original(request);
  };
  render(<App bridge={bridge} />);
  await waitFor(() =>
    expect(Object.keys(finish).sort()).toEqual(["commit", "graph"]),
  );
  const cancel = screen.getByRole("button", {
    name: "Cancel current operation",
  });
  expect(cancel).toBeEnabled();
  await act(async () => {
    finish.graph(
      await original({
        kind: "graph",
        workspace: "ws",
        worktree: "parent",
        scope: { kind: "local_branches" },
        cursor: null,
      }),
    );
  });
  expect(cancel).toBeEnabled();
  await userEvent.click(cancel);
  expect(bridge.cancel).toHaveBeenCalledOnce();
  expect(cancel).toBeEnabled();
  // Settling the remaining request with an error also releases its pending action.
  await act(async () => {
    finish.commit({ kind: "done" });
  });
  await waitFor(() => expect(cancel).toBeDisabled());
});

it("polls one refresh at a time and pauses repository reads during native writes", async () => {
  vi.useFakeTimers({ toFake: ["setInterval", "clearInterval"] });
  const { bridge, requests } = fake(true);
  const original = bridge.request;
  let running = true;
  let finishInventory: ((data: Payload) => void) | undefined;
  let holdInventory = false;
  bridge.writeStatus = vi.fn(async () => running);
  bridge.request = async (request) => {
    if (request.kind === "inventory" && holdInventory)
      return new Promise<Payload>((resolve) => {
        finishInventory = resolve;
      });
    return original(request);
  };
  render(<App bridge={bridge} />);
  await screen.findByRole("checkbox", { name: "Select parent" });
  const baseline = requests.length;
  await act(async () => {
    await vi.advanceTimersByTimeAsync(12000);
  });
  expect(requests.slice(baseline)).toEqual([]);
  expect(
    screen.getByRole("button", { name: "Cancel current operation" }),
  ).toBeEnabled();
  running = false;
  holdInventory = true;
  await act(async () => {
    await vi.advanceTimersByTimeAsync(3000);
  });
  expect(finishInventory).toBeDefined();
  const polls = vi.mocked(bridge.writeStatus).mock.calls.length;
  await act(async () => {
    await vi.advanceTimersByTimeAsync(12000);
  });
  expect(bridge.writeStatus).toHaveBeenCalledTimes(polls);
  holdInventory = false;
  await act(async () => {
    finishInventory!(await original({ kind: "inventory", workspace: "ws" }));
  });
  await act(async () => {
    await vi.advanceTimersByTimeAsync(3000);
  });
  expect(bridge.writeStatus).toHaveBeenCalledTimes(polls + 1);
  expect(requests.slice(baseline).some((r) => r.kind === "trust_status")).toBe(
    true,
  );
  expect(
    screen.getByRole("button", { name: "Cancel current operation" }),
  ).toBeDisabled();
});

it("refreshes durable bulk results after a fast native completion without replaying work", async () => {
  vi.useFakeTimers({ toFake: ["setInterval", "clearInterval"] });
  const { bridge, requests } = fake(true);
  const original = bridge.request;
  let completed = false;
  bridge.writeRevision = 0;
  bridge.writeStatus = async () => {
    bridge.writeRevision = completed ? 1 : 0;
    return false;
  };
  bridge.request = async (r) => {
    if (r.kind === "batches") {
      requests.push(r);
      return {
        kind: "batches",
        value: {
          items: [
            {
              record: {
                version: 1,
                id: "fast-native-batch",
                workspace: "ws",
                created_at_ms: 0,
                phase: completed ? "complete" : "proposed",
                issue: null,
                items: [
                  {
                    worktree: "parent",
                    path: { encoding: "utf8", value: "/parent" },
                    dependencies: [],
                    operation: "op",
                    issue: null,
                  },
                ],
              },
              outcomes: [
                {
                  operation_id: "op",
                  phase: completed ? "complete" : "proposed",
                  published: completed,
                  index_reconciled: completed,
                  post_processing: [],
                },
              ],
            },
          ],
          next: null,
        },
      };
    }
    return original(r);
  };
  render(<App bridge={bridge} />);
  await screen.findByRole("checkbox", { name: "Select parent" });
  await userEvent.click(screen.getByRole("button", { name: "Bulk Commit" }));
  await screen.findByRole("heading", { name: /^proposed ·/ });
  completed = true;
  await act(async () => {
    await vi.advanceTimersByTimeAsync(3000);
  });
  await screen.findByRole("heading", { name: /^complete ·/ });
  expect(
    screen.getByRole("button", {
      name: "Select unfinished worktrees for fresh review",
    }),
  ).toBeDisabled();
  expect(
    requests.some((r) =>
      ["review_batch", "propose", "review_trust"].includes(r.kind),
    ),
  ).toBe(false);
});

it("captures the latest navigation and bulk draft on close before debounce and unfreezes after a failed save", async () => {
  const { bridge, saved } = fake();
  let snapshot: (() => import("../services/bridge").CloseSnapshot) | undefined;
  let changed: ((closing: boolean, error?: string) => void) | undefined;
  const unsubscribe = vi.fn();
  bridge.onCloseRequested = async (capture, status) => {
    snapshot = capture;
    changed = status;
    return unsubscribe;
  };
  const { container, unmount } = render(<App bridge={bridge} />);
  await screen.findByRole("checkbox", { name: "Select parent" });
  await userEvent.click(
    screen.getAllByRole("button", { name: "parent" }).at(-1)!,
  );
  fireEvent.change(screen.getByRole("textbox", { name: "Message" }), {
    target: { value: "Immediate individual draft" },
  });
  await userEvent.click(screen.getByRole("button", { name: "Bulk Commit" }));
  fireEvent.change(
    screen.getByRole("textbox", { name: "Shared commit message" }),
    { target: { value: "Immediate bulk draft" } },
  );
  expect(
    saved.some((s) => s.drafts.parent === "Immediate individual draft"),
  ).toBe(false);
  const captured = snapshot!();
  expect(captured.workspace).toBe("ws");
  expect(captured.navigation!.drafts.parent).toBe("Immediate individual draft");
  expect(captured.bulk!.message).toBe("Immediate bulk draft");
  act(() => changed!(true));
  expect(container.querySelector(".shell")).toHaveAttribute("inert");
  act(() => changed!(false, "Close cancelled: fixture save failed"));
  expect(container.querySelector(".shell")).not.toHaveAttribute("inert");
  expect(screen.getByRole("alert")).toHaveTextContent("fixture save failed");
  expect(
    screen.getByRole("textbox", { name: "Shared commit message" }),
  ).toHaveValue("Immediate bulk draft");
  unmount();
  expect(unsubscribe).toHaveBeenCalledOnce();
});

it("keeps a historical tab and draft separate from the replacement and active inventory", async () => {
  const { bridge, requests } = fake();
  const request = bridge.request.bind(bridge);
  bridge.request = async (input): Promise<Payload> => {
    if (input.kind === "inventory" || input.kind === "scan")
      return {
        kind: "inventory",
        value: {
          entries: [
            {
              ...fixtureEntry("old", "repo"),
              availability: "missing",
              superseded_by: "replacement",
            },
            fixtureEntry("replacement", "repo"),
          ],
          monitor: { running: true, epoch: 1, watched: 1, degraded: null },
        },
      };
    if (input.kind === "load_state")
      return {
        kind: "state",
        value: {
          ...initialState,
          active_tab: "old",
          tabs: [
            {
              worktree: "old",
              view: "changes",
              filter: "",
              source: "working_tree",
              scroll: 0,
            },
          ],
          drafts: { old: "Keep this draft" },
        },
      };
    return request(input);
  };
  render(<App bridge={bridge} />);
  await screen.findByRole("heading", { name: "Repository unavailable" });
  expect(screen.queryByRole("textbox", { name: "Message" })).toBeNull();
  await userEvent.click(
    screen.getByText("Historical repository identities · 1"),
  );
  expect(
    screen.getByRole("textbox", { name: /Preserved historical draft/ }),
  ).toHaveValue("Keep this draft");
  await userEvent.click(
    screen.getByRole("button", { name: "Open inspected replacement" }),
  );
  expect(await screen.findByRole("textbox", { name: "Message" })).toHaveValue(
    "",
  );
  await userEvent.click(screen.getByRole("button", { name: "▦ Overview" }));
  expect(screen.getAllByRole("checkbox", { name: "Select repo" })).toHaveLength(
    1,
  );
  expect(
    requests.some((r) => r.kind === "review_trust" || r.kind === "propose"),
  ).toBe(false);
});

it("shows incomplete discovery even when all known repositories are available", async () => {
  const { bridge } = fake();
  const request = bridge.request.bind(bridge);
  bridge.request = async (input): Promise<Payload> => {
    const payload = await request(input);
    if (payload.kind === "inventory")
      return {
        ...payload,
        value: {
          ...payload.value,
          scan: {
            complete: false,
            issues: [
              {
                path: { encoding: "utf8", value: "/tmp/project/cache/broken" },
                error: {
                  kind: "inspection_failed",
                  message: "Cannot inspect repository metadata",
                },
              },
            ],
            skipped_caches: [],
          },
        },
      };
    return payload;
  };
  render(<App bridge={bridge} />);
  await screen.findByRole("checkbox", { name: "Select parent" });
  expect(screen.getByText("△ Discovery incomplete")).toBeVisible();
  await userEvent.click(screen.getByText("Discovery details"));
  expect(screen.getByText("/tmp/project/cache/broken")).toBeVisible();
});

it("preserves captured AI context across native review completion without replay or draft replacement", async () => {
  vi.useFakeTimers({ toFake: ["setInterval", "clearInterval"] });
  Object.defineProperty(HTMLDialogElement.prototype, "showModal", {
    configurable: true,
    value: function (this: HTMLDialogElement) {
      this.setAttribute("open", "");
    },
  });
  try {
    const { bridge, requests } = fake();
    const original = bridge.request;
    let completed = false;
    bridge.writeRevision = 0;
    bridge.writeStatus = async () => {
      bridge.writeRevision = completed ? 1 : 0;
      return false;
    };
    bridge.request = async (r) => {
      if (r.kind === "preview_ai_context") {
        requests.push(r);
        return {
          kind: "ai_context",
          value: {
            snapshot_fingerprint: "snapshot",
            generation: "generation",
            bundle: {
              payload: '{"chunks":[]}',
              fingerprint: "context",
              complete: true,
              warnings: [],
              accounting: [],
            },
          },
        };
      }
      if (r.kind === "review_ai_messages") {
        requests.push(r);
        completed = true;
        return { kind: "done" };
      }
      if (r.kind === "ai_jobs") return { kind: "ai_jobs", value: [] };
      return original(r);
    };
    render(<App bridge={bridge} />);
    await screen.findByRole("checkbox", { name: "Select parent" });
    await userEvent.click(
      screen.getAllByRole("button", { name: "parent" }).at(-1)!,
    );
    fireEvent.change(screen.getByRole("textbox", { name: "Message" }), {
      target: { value: "Preserve my manual draft" },
    });
    await userEvent.click(
      screen.getByRole("checkbox", { name: "Include file.txt from index" }),
    );
    await userEvent.click(
      screen.getByRole("button", { name: "AI message suggestions…" }),
    );
    await userEvent.click(
      screen.getByRole("button", {
        name: "Capture filtered AI context · no transfer",
      }),
    );
    await screen.findByRole("region", { name: "Exact filtered AI context" });
    await userEvent.click(
      screen.getByRole("button", { name: "Review exact AI source transfer" }),
    );
    await act(async () => {
      await vi.advanceTimersByTimeAsync(3000);
    });
    expect(
      screen.getByRole("dialog", {
        name: "AI context and message suggestions",
      }),
    ).toHaveAttribute("open");
    expect(
      screen.getByRole("region", { name: "Exact filtered AI context" }),
    ).toHaveTextContent("snapshot");
    expect(screen.getByRole("textbox", { name: "Message" })).toHaveValue(
      "Preserve my manual draft",
    );
    expect(
      requests.filter((r) => r.kind === "review_ai_messages"),
    ).toHaveLength(1);
    expect(
      requests.filter((r) => r.kind === "preview_ai_context"),
    ).toHaveLength(1);
  } finally {
    Reflect.deleteProperty(HTMLDialogElement.prototype, "showModal");
  }
});
