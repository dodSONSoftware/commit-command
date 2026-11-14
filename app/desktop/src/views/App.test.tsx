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
it("persists compact density and sidebar visibility without losing the saved width", async () => {
  const f = fake();
  const user = userEvent.setup();
  render(<App bridge={f.bridge} />);
  await screen.findByRole("checkbox", { name: "Select parent" });
  fireEvent.keyDown(screen.getByRole("separator", { name: "Sidebar width" }), {
    key: "ArrowRight",
  });
  const sidebar = document.getElementById("workspace-sidebar");
  expect(sidebar).toHaveStyle({ width: "260px" });
  await user.click(screen.getByRole("button", { name: "Hide sidebar" }));
  expect(sidebar).not.toBeVisible();
  expect(screen.getByRole("button", { name: "Show sidebar" })).toHaveAttribute(
    "aria-expanded",
    "false",
  );
  expect(
    screen.queryByRole("separator", { name: "Sidebar width" }),
  ).not.toBeInTheDocument();
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Overview density" }),
    "compact",
  );
  await waitFor(() => expect(f.saved.at(-1)?.density).toBe("compact"));
  expect(f.saved.at(-1)?.sidebar_collapsed).toBe(true);
  expect(f.saved.at(-1)?.sidebar_width).toBe(260);
  await user.click(screen.getByRole("button", { name: "Show sidebar" }));
  expect(sidebar).toBeVisible();
  expect(sidebar).toHaveStyle({ width: "260px" });
  expect(
    screen.getByRole("separator", { name: "Sidebar width" }),
  ).toHaveAttribute("aria-valuenow", "260");
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
        case "app_preferences":
          return {
            kind: "app_preferences",
            value: { textarea_heights: {}, generation_timeout_seconds: 60 },
          };
        case "save_app_preference":
          return {
            kind: "app_preferences",
            value: { textarea_heights: {}, generation_timeout_seconds: 60 },
          };
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
it("disables Git actions when the host explicitly reports them unavailable", async () => {
  const { bridge, requests } = fake();
  render(<App bridge={bridge} />);
  await screen.findByRole("checkbox", { name: "Select parent" });
  await userEvent.click(
    screen.getAllByRole("button", { name: "parent" }).at(-1)!,
  );
  expect(screen.queryByRole("button", { name: "Review trust" })).toBeNull();
  expect(screen.getByRole("button", { name: "Stage" })).toBeDisabled();
  expect(screen.queryByText("Inspect trust & eligibility")).toBeNull();
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
  await userEvent.click(
    screen.getByRole("button", { name: /Switch to (light|dark) mode/ }),
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

it("removes selections whose source is no longer changed", async () => {
  vi.useFakeTimers({ toFake: ["setInterval", "clearInterval"] });
  const { bridge } = fake(true);
  const original = bridge.request;
  let clean = false;
  bridge.request = async (request) => {
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
  await userEvent.click(
    screen.getByRole("checkbox", { name: "Include file.txt from index" }),
  );
  expect(screen.getByRole("button", { name: "Unstage" })).toBeEnabled();
  expect(
    screen.getByRole("button", { name: "Commit · 1 file" }),
  ).toBeInTheDocument();
  clean = true;
  await act(async () => {
    await vi.advanceTimersByTimeAsync(3000);
  });
  {
    expect(
      screen.getByRole("button", { name: "Commit · 0 files" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Unstage" })).toBeDisabled();
  }
  clean = false;
  await act(async () => {
    await vi.advanceTimersByTimeAsync(3000);
  });
  {
    expect(
      screen.getByRole("checkbox", { name: "Include file.txt from index" }),
    ).not.toBeChecked();
    expect(screen.getByRole("button", { name: "Unstage" })).toBeDisabled();
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
  expect(requests.slice(baseline).some((r) => r.kind === "inventory")).toBe(
    true,
  );
  expect(requests.some((r) => r.kind === "trust_status")).toBe(false);
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
  expect(screen.getByRole("alert")).toHaveTextContent("Discovery incomplete");
  expect(screen.getByRole("alert")).toHaveTextContent(
    "Cannot inspect repository metadata",
  );
  await userEvent.click(
    screen.getByRole("button", { name: "Show scan details" }),
  );
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
      if (r.kind === "effective_ai_settings")
        return {
          kind: "effective_ai_settings",
          value: {
            connection: {
              id: "c",
              revision: "r",
              name: "Local",
              provider: "ollama",
              endpoint: "http://127.0.0.1:1/",
              endpoint_approval: "",
              timeout_ms: 3000,
            },
            profile: {
              id: "p",
              revision: "r",
              name: "Profile",
              connection: "c",
              model: "model",
              system_message: "",
              parameters: {},
              template: "t",
              context_bytes: 16384,
            },
            template: {
              id: "t",
              revision: "r",
              name: "Plain",
              examples: [],
              rules: {
                style: "plain",
                subject_max_chars: 72,
                allowed_types: [],
                allow_scope: false,
                forbid_trailing_period: false,
                instructions: "",
              },
            },
            instructions: [],
          },
        };
      if (r.kind === "review_ai_model_load")
        return {
          kind: "executed",
          value: {
            result: null,
            ai_job_id: "load-job",
            ai_request_fingerprint: "load-fingerprint",
          },
        };
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
      if (r.kind === "ai_jobs")
        return {
          kind: "ai_jobs",
          value: [
            {
              id: "load-job",
              phase: "complete",
              error: null,
              review: {
                fingerprint: "load-fingerprint",
                endpoint: "http://localhost/",
                route: "http://localhost/api/generate",
                model: "model",
                purpose: "load_model",
                request_bytes: 1,
                authentication: "none",
                unencrypted: true,
              },
              result: { kind: "model_loaded", value: { model: "model" } },
            },
          ],
        };
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
    await userEvent.click(screen.getByRole("button", { name: "Fill with AI" }));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(3000);
    });
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
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

it("keeps scan diagnostics in troubleshooting and push planning with repository remote controls", async () => {
  const { bridge } = fake();
  render(<App bridge={bridge} />);
  await screen.findByRole("checkbox", { name: "Select parent" });
  const troubleshooting = screen.getByText("Workspace troubleshooting");
  expect(troubleshooting.closest("aside")).not.toBeNull();
  expect(screen.queryByText("Push repositories with submodules")).toBeNull();
  await userEvent.click(
    screen.getAllByRole("button", { name: "parent" }).at(-1)!,
  );
  const remoteControls = await screen.findByText("Remotes and upstream");
  await userEvent.click(remoteControls);
  const planner = screen.getByText("Push repositories with submodules");
  expect(planner).toBeVisible();
  expect(planner.closest(".remote-inspector")).toBe(
    remoteControls.closest("details"),
  );
});

it("opens the repository folder from its path", async () => {
  const { bridge } = fake();
  bridge.repositoryPathAction = vi.fn().mockResolvedValue(undefined);
  render(<App bridge={bridge} />);
  await userEvent.click(await screen.findByRole("button", { name: "parent" }));
  await userEvent.click(
    screen.getByRole("button", { name: "/tmp/project/parent" }),
  );
  expect(bridge.repositoryPathAction).toHaveBeenCalledWith(
    { workspace: fixtureWorkspace.id, worktree: "parent" },
    "open",
  );
});

it("selects every selectable file from none or partial selection, then clears all", async () => {
  const { bridge } = fake();
  const original = bridge.request;
  bridge.request = async (request) => {
    const result = await original(request);
    if (result.kind === "inventory") {
      const changes = result.value.entries[0].status!.changes;
      changes.push(
        {
          ...changes[0],
          path: { encoding: "utf8", value: "new.txt" },
          index: "?",
          worktree: "?",
        },
        {
          ...changes[0],
          path: { encoding: "utf8", value: "conflict.txt" },
          conflict: true,
          index: "U",
          worktree: "U",
        },
      );
    }
    return result;
  };
  render(<App bridge={bridge} />);
  await userEvent.click(await screen.findByRole("button", { name: "parent" }));
  const all = screen.getByRole("checkbox", {
    name: "Select all changed files",
  });
  const files = screen
    .getAllByRole("checkbox", { name: /^Include / })
    .filter((checkbox) => !checkbox.hasAttribute("disabled"));
  expect(all).not.toBeChecked();
  await userEvent.click(all);
  expect(all).toBeChecked();
  files.forEach((file) => expect(file).toBeChecked());
  screen
    .getAllByRole("checkbox", { name: /Include conflict.txt/ })
    .forEach((file) => expect(file).not.toBeChecked());
  expect(screen.getByText("2 selected")).toBeInTheDocument();
  await userEvent.click(files[0]);
  expect(all).toBePartiallyChecked();
  await userEvent.click(all);
  expect(all).toBeChecked();
  files.forEach((file) => expect(file).toBeChecked());
  await userEvent.click(all);
  expect(all).not.toBeChecked();
  expect(all).not.toBePartiallyChecked();
  files.forEach((file) => expect(file).not.toBeChecked());
});

it.each(["Stage", "Unstage", "Commit"] as const)(
  "%s submits only eligible sides from the displayed rename and partial file",
  async (action) => {
    const { bridge, requests } = fake(true);
    const original = bridge.request;
    const oldPath = { encoding: "utf8" as const, value: "old.txt" };
    const newPath = { encoding: "utf8" as const, value: "new [Δ].txt" };
    const file = { encoding: "utf8" as const, value: "file.txt" };
    bridge.request = async (request) => {
      const data = await original(request);
      if (data.kind === "inventory") {
        const changes = data.value.entries[0].status!.changes;
        changes.push({
          ...changes[0],
          index: "R",
          worktree: ".",
          original_path: oldPath,
          path: newPath,
        });
      }
      if (request.kind === "propose")
        return {
          kind: "executed",
          value: { result: { phase: "complete", published: false } },
        };
      return data;
    };
    render(<App bridge={bridge} />);
    await userEvent.click(
      await screen.findByRole("button", { name: "parent" }),
    );
    expect(
      screen.getByRole("button", { name: "old.txt → new [Δ].txt" }),
    ).toBeVisible();
    const checkbox = (name: string) => screen.getByRole("checkbox", { name });
    const button = (name: string) => screen.getByRole("button", { name });
    const rename = checkbox("Include new [Δ].txt from index");
    const working = checkbox("Include file.txt from working_tree");
    await userEvent.click(rename);
    expect(button("Stage")).toBeDisabled();
    await userEvent.click(button("Stage"));
    await userEvent.click(rename);
    await userEvent.click(working);
    expect(button("Unstage")).toBeDisabled();
    await userEvent.click(button("Unstage"));
    expect(requests.filter((r) => r.kind === "propose")).toHaveLength(0);

    if (action === "Commit") {
      await userEvent.click(checkbox("Select all changed files"));
      fireEvent.change(screen.getByRole("textbox", { name: "Message" }), {
        target: { value: "Selected changes" },
      });
      await userEvent.click(button("Commit · 2 files"));
      expect(
        requests.filter((r) => r.kind === "propose").map((r) => r.request),
      ).toEqual([
        expect.objectContaining({
          kind: "commit",
          selected: [
            { path: file, source: "working_tree" },
            { path: oldPath, source: "index" },
            { path: newPath, source: "index" },
          ],
        }),
      ]);
    } else {
      await userEvent.click(rename);
      expect(screen.getByText("2 selected")).toBeVisible();
      await userEvent.click(button(action));
      const staging = action === "Stage";
      expect(
        await screen.findByText(
          staging ? "1 file staged." : "1 file unstaged.",
        ),
      ).toBeVisible();
      expect(
        requests.filter((r) => r.kind === "propose").map((r) => r.request),
      ).toEqual([
        {
          kind: staging ? "stage" : "unstage",
          paths: staging ? [file] : [oldPath, newPath],
          hooks: "execute",
        },
      ]);
    }
  },
);

it("opens the active workspace folder from the sidebar path", async () => {
  const { bridge } = fake();
  bridge.repositoryPathAction = vi.fn().mockResolvedValue(undefined);
  render(<App bridge={bridge} />);
  await userEvent.click(
    await screen.findByRole("button", { name: "/tmp/project" }),
  );
  expect(bridge.repositoryPathAction).toHaveBeenCalledWith(
    { workspace: fixtureWorkspace.id },
    "open",
  );
});

it("commits selected files and a message without a separate trust step", async () => {
  const { bridge, requests } = fake(true);
  render(<App bridge={bridge} />);
  await userEvent.click(await screen.findByRole("button", { name: "parent" }));
  expect(screen.queryByText("Trust & execution")).toBeNull();
  expect(screen.queryByText("Inspect trust & eligibility")).toBeNull();
  await userEvent.click(
    screen.getByRole("checkbox", { name: "Include file.txt from index" }),
  );
  fireEvent.change(screen.getByRole("textbox", { name: "Message" }), {
    target: { value: "fix: selected change" },
  });
  const commit = screen.getByRole("button", { name: "Commit · 1 file" });
  expect(commit).toBeEnabled();
  await userEvent.click(commit);
  expect(requests).toContainEqual(
    expect.objectContaining({
      kind: "propose",
      request: expect.objectContaining({
        kind: "commit",
        message: "fix: selected change",
      }),
    }),
  );
  expect(
    requests.some(
      (r) => r.kind === "review_trust" || r.kind === "trust_status",
    ),
  ).toBe(false);
});

it("rounds fractional sidebar coordinates before saving layout and drafts", async () => {
  const { bridge, saved } = fake();
  const original = bridge.request;
  bridge.request = async (request) => {
    if (
      request.kind === "save_state" &&
      !Number.isInteger(request.state.sidebar_width)
    )
      throw new Error("Malformed or unsupported request");
    return original(request);
  };
  render(<App bridge={bridge} />);
  await userEvent.click(await screen.findByRole("button", { name: "parent" }));
  fireEvent.change(screen.getByRole("textbox", { name: "Message" }), {
    target: { value: "Preserve the draft" },
  });
  const splitter = screen.getByRole("separator", { name: "Sidebar width" });
  Object.defineProperty(splitter, "hasPointerCapture", { value: () => true });
  const event = new Event("pointermove", { bubbles: true });
  Object.defineProperty(event, "clientX", { value: 333.6 });
  Object.defineProperty(event, "pointerId", { value: 1 });
  fireEvent(splitter, event);
  await waitFor(() => expect(saved.at(-1)?.sidebar_width).toBe(334));
  expect(saved.at(-1)?.drafts.parent).toBe("Preserve the draft");
  expect(screen.queryByRole("alert")).toBeNull();
});

it("clears a layout-save error after a successful retry", async () => {
  const { bridge, saved } = fake();
  const original = bridge.request;
  let fail = true;
  bridge.request = async (request) => {
    if (request.kind === "save_state" && fail)
      throw new Error("Storage unavailable");
    return original(request);
  };
  render(<App bridge={bridge} />);
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Layout and drafts were not saved: Storage unavailable",
  );
  fail = false;
  await userEvent.click(screen.getByRole("button", { name: "Hide sidebar" }));
  await waitFor(() => expect(saved.at(-1)?.sidebar_collapsed).toBe(true));
  expect(screen.queryByRole("alert")).toBeNull();
});

it("keeps staging and commit controls outside their scrolling content", async () => {
  const { bridge } = fake(true);
  render(<App bridge={bridge} />);
  await userEvent.click(await screen.findByRole("button", { name: "parent" }));
  const files = screen.getByLabelText("Changed file list");
  const stage = screen.getByRole("button", { name: "Stage" });
  expect(files.contains(stage)).toBe(false);
  expect(stage.closest(".files")).toBe(files.parentElement);
  expect(
    stage.compareDocumentPosition(files) & Node.DOCUMENT_POSITION_FOLLOWING,
  ).not.toBe(0);
  const commit = screen.getByRole("button", { name: "Commit · 0 files" });
  expect(commit.closest(".composer-content")).toBeNull();
  expect(commit.closest(".commit-actions")).not.toBeNull();
});

it.each([
  ["Stage", "complete"],
  ["Stage", "blocked"],
  ["Stage", "published_with_post_processing_failure"],
  ["Unstage", "published_with_post_processing_failure"],
])(
  "shows the %s %s result immediately without opening operation history",
  async (action, phase) => {
    const { bridge } = fake(true);
    const original = bridge.request;
    bridge.request = async (request) =>
      request.kind === "propose"
        ? {
            kind: "executed",
            value: {
              result: {
                phase,
                published: false,
                index_reconciled: phase !== "blocked",
                reason:
                  phase === "published_with_post_processing_failure"
                    ? "post-index-change failed after index/ref publication"
                    : "Fixture Git failure",
              },
            },
          }
        : original(request);
    render(<App bridge={bridge} />);
    await userEvent.click(
      await screen.findByRole("button", { name: "parent" }),
    );
    await userEvent.click(
      screen.getByRole("checkbox", {
        name:
          action === "Stage"
            ? "Include file.txt from working_tree"
            : "Include file.txt from index",
      }),
    );
    fireEvent.change(screen.getByRole("textbox", { name: "Message" }), {
      target: { value: "Keep my draft" },
    });
    await userEvent.click(screen.getByRole("button", { name: action }));
    if (phase === "blocked") {
      expect(await screen.findByRole("alert")).toHaveTextContent(
        "Could not stage files. The staging area was not changed. Fixture Git failure",
      );
    } else if (phase === "published_with_post_processing_failure") {
      expect(await screen.findByRole("alert")).toHaveTextContent(
        `1 file ${action === "Stage" ? "staged" : "unstaged"}. post-index-change failed after index/ref publication`,
      );
    } else {
      expect(await screen.findByText("1 file staged.")).toBeVisible();
      await userEvent.click(
        screen.getByRole("button", { name: "↻ Local refresh" }),
      );
      expect(screen.getByText("1 file staged.")).toBeVisible();
    }
    expect(screen.getByRole("textbox", { name: "Message" })).toHaveValue(
      "Keep my draft",
    );
    expect(
      screen.queryByRole("heading", { name: "Operation results" }),
    ).toBeNull();
  },
);

it("removes obsolete saved selections without selecting their replacements or clearing current selections", async () => {
  const { bridge, saved } = fake();
  const original = bridge.request;
  let replaced = false;
  bridge.request = async (request) => {
    if (request.kind === "load_state")
      return {
        kind: "state",
        value: {
          ...initialState,
          selected: ["old-parent", "child", "removed"],
          drafts: { "old-parent": "Preserve historical draft" },
        },
      };
    if (request.kind === "inventory")
      return {
        kind: "inventory",
        value: {
          entries: [
            {
              ...fixtureEntry("old-parent", "parent"),
              superseded_by: "parent",
            },
            fixtureEntry("parent", "parent"),
            {
              ...fixtureEntry("child", "parent/child"),
              superseded_by: replaced ? "new-child" : null,
            },
            ...(replaced ? [fixtureEntry("new-child", "parent/child")] : []),
          ],
          monitor: { running: true, epoch: 1, watched: 1, degraded: null },
        },
      };
    return original(request);
  };
  render(<App bridge={bridge} />);
  expect(
    await screen.findByRole("checkbox", { name: "Select parent/child" }),
  ).toBeChecked();
  expect(
    screen.getByRole("checkbox", { name: "Select parent" }),
  ).not.toBeChecked();
  expect(screen.getByText("1 explicitly selected")).toBeVisible();
  expect(
    screen.getByText("Needs attention", { selector: ".metrics span" }),
  ).toBeVisible();
  await waitFor(() => expect(saved.at(-1)?.selected).toEqual(["child"]));
  expect(saved.at(-1)?.drafts["old-parent"]).toBe("Preserve historical draft");
  replaced = true;
  await userEvent.click(screen.getByRole("button", { name: "↻ Rescan" }));
  await waitFor(() =>
    expect(screen.getByText("0 explicitly selected")).toBeVisible(),
  );
  expect(
    screen.getByRole("checkbox", { name: "Select parent/child" }),
  ).not.toBeChecked();
  await waitFor(() => expect(saved.at(-1)?.selected).toEqual([]));
});

it.each(["stage", "unstage"] as const)(
  "shows ongoing %s work while its controls are disabled",
  async (kind) => {
    const { bridge } = fake(true);
    const original = bridge.request;
    let finish!: (payload: Payload) => void;
    bridge.request = async (request) =>
      request.kind === "propose"
        ? new Promise<Payload>((resolve) => {
            finish = resolve;
          })
        : original(request);
    render(<App bridge={bridge} />);
    await userEvent.click(
      await screen.findByRole("button", { name: "parent" }),
    );
    await userEvent.click(
      screen.getByRole("checkbox", {
        name: `Include file.txt from ${kind === "stage" ? "working_tree" : "index"}`,
      }),
    );
    await userEvent.click(
      screen.getByRole("button", {
        name: kind === "stage" ? "Stage" : "Unstage",
      }),
    );
    const working = kind === "stage" ? "Staging" : "Unstaging";
    expect(screen.getByRole("button", { name: `${working}…` })).toBeDisabled();
    expect(screen.getByText(`${working} 1 file…`)).toBeVisible();
    expect(
      screen.getByRole("button", {
        name: kind === "stage" ? "Unstage" : "Stage",
      }),
    ).toBeDisabled();
    await act(async () =>
      finish({
        kind: "executed",
        value: { result: { phase: "complete", published: false } },
      }),
    );
    expect(screen.queryByText(`${working} 1 file…`)).toBeNull();
    expect(
      screen.getByRole("button", {
        name: kind === "stage" ? "Stage" : "Unstage",
      }),
    ).toBeEnabled();
  },
);

it.each(["edit", "rename"])(
  "keeps the whole %s checked as its rows move between sides",
  async (kind) => {
    vi.useFakeTimers({ toFake: ["setInterval", "clearInterval"] });
    const { bridge } = fake(true);
    const original = bridge.request;
    const oldPath = { encoding: "utf8" as const, value: "old.txt" };
    let staged = false;
    bridge.request = async (request) => {
      const data = await original(request);
      if (data.kind === "inventory") {
        const changes = data.value.entries[0].status!.changes;
        const destination = changes[0];
        if (kind === "edit") {
          destination.index = staged ? "M" : ".";
          destination.worktree = staged ? "." : "M";
        } else if (staged) {
          Object.assign(destination, {
            index: "R",
            worktree: ".",
            original_path: oldPath,
          });
        } else {
          Object.assign(destination, { index: "?", worktree: "?" });
          changes.push({
            ...destination,
            path: oldPath,
            index: ".",
            worktree: "D",
          });
        }
      }
      if (request.kind === "propose") {
        staged = request.request.kind === "stage";
        return {
          kind: "executed",
          value: { result: { phase: "complete", published: false } },
        };
      }
      return data;
    };
    render(<App bridge={bridge} />);
    await userEvent.click(
      await screen.findByRole("button", { name: "parent" }),
    );
    const all = screen.getByRole("checkbox", {
      name: "Select all changed files",
    });
    await userEvent.click(all);
    await userEvent.click(screen.getByRole("button", { name: "Stage" }));
    await act(() => vi.advanceTimersByTimeAsync(3000));
    expect(
      screen.getByRole("checkbox", { name: "Include file.txt from index" }),
    ).toBeChecked();
    expect(screen.getByText("1 selected")).toBeVisible();
    await userEvent.click(screen.getByRole("button", { name: "Unstage" }));
    await act(() => vi.advanceTimersByTimeAsync(3000));
    expect(
      screen.getByRole("checkbox", {
        name: "Include file.txt from working_tree",
      }),
    ).toBeChecked();
    if (kind === "rename") {
      expect(
        screen.getByRole("checkbox", {
          name: "Include old.txt from working_tree",
        }),
      ).toBeChecked();
      expect(screen.getByText("2 selected")).toBeVisible();
    }
  },
);

it("cancels staging beside its progress indicator and waits for the operation to settle", async () => {
  const { bridge } = fake(true);
  const original = bridge.request;
  let finish!: (payload: Payload) => void;
  let stageOwner: symbol | undefined;
  bridge.request = async (request, owner) =>
    request.kind === "propose"
      ? new Promise<Payload>((resolve) => {
          stageOwner = owner;
          finish = resolve;
        })
      : original(request);
  bridge.cancel = vi.fn(async () => {});
  render(<App bridge={bridge} />);
  await userEvent.click(await screen.findByRole("button", { name: "parent" }));
  await userEvent.click(
    screen.getByRole("checkbox", {
      name: "Include file.txt from working_tree",
    }),
  );
  await userEvent.click(screen.getByRole("button", { name: "Stage" }));
  await userEvent.click(screen.getByRole("button", { name: "Cancel" }));
  expect(typeof stageOwner).toBe("symbol");
  expect(bridge.cancel).toHaveBeenCalledWith(stageOwner);
  expect(screen.getByRole("button", { name: "Cancelling…" })).toBeDisabled();
  await act(async () =>
    finish({
      kind: "executed",
      value: {
        result: { phase: "cancelled_before_publication", published: false },
      },
    }),
  );
  expect(screen.queryByRole("button", { name: "Cancelling…" })).toBeNull();
  expect(screen.getByRole("button", { name: "Stage" })).toBeEnabled();
  expect(
    screen.getByRole("checkbox", {
      name: "Include file.txt from working_tree",
    }),
  ).toBeChecked();
});

it.each(["raw", "structured"])(
  "clears the %s commit draft only after successful completion",
  async (mode) => {
    const { bridge, saved } = fake(true);
    const original = bridge.request;
    let finish!: (payload: Payload) => void;
    bridge.request = async (r) =>
      r.kind === "propose"
        ? new Promise<Payload>((resolve) => {
            finish = resolve;
          })
        : original(r);
    render(<App bridge={bridge} />);
    await userEvent.click(
      await screen.findByRole("button", { name: "parent" }),
    );
    await userEvent.click(
      screen.getByRole("checkbox", { name: "Include file.txt from index" }),
    );
    fireEvent.change(screen.getByRole("textbox", { name: "Message" }), {
      target: { value: "fix: selected change\n\nDetailed explanation" },
    });
    if (mode === "structured")
      await userEvent.click(
        screen.getByRole("button", { name: "Edit subject and body" }),
      );
    await userEvent.click(
      screen.getByRole("button", { name: "Commit · 1 file" }),
    );
    expect(
      screen.getByRole("textbox", {
        name: mode === "raw" ? "Message" : "Message subject",
      }),
    ).toHaveValue(
      mode === "raw"
        ? "fix: selected change\n\nDetailed explanation"
        : "fix: selected change",
    );
    await act(async () =>
      finish({
        kind: "executed",
        value: { result: { phase: "complete", published: true } },
      }),
    );
    expect(
      screen.getByRole("textbox", {
        name: mode === "raw" ? "Message" : "Message subject",
      }),
    ).toHaveValue("");
    if (mode === "structured")
      expect(screen.getByRole("textbox", { name: "Message body" })).toHaveValue(
        "",
      );
    await waitFor(() => expect(saved.at(-1)?.drafts.parent).toBe(""));
  },
);
it.each(["blocked", "cancelled_before_publication"])(
  "preserves the commit draft when the outcome is %s",
  async (phase) => {
    const { bridge } = fake(true);
    const original = bridge.request;
    bridge.request = async (r) =>
      r.kind === "propose"
        ? { kind: "executed", value: { result: { phase, published: false } } }
        : original(r);
    render(<App bridge={bridge} />);
    await userEvent.click(
      await screen.findByRole("button", { name: "parent" }),
    );
    await userEvent.click(
      screen.getByRole("checkbox", { name: "Include file.txt from index" }),
    );
    fireEvent.change(screen.getByRole("textbox", { name: "Message" }), {
      target: { value: "Keep this draft\n\nAnd its body" },
    });
    await userEvent.click(
      screen.getByRole("button", { name: "Commit · 1 file" }),
    );
    expect(screen.getByRole("textbox", { name: "Message" })).toHaveValue(
      "Keep this draft\n\nAnd its body",
    );
  },
);

it.each([false, true, "restore"] as const)(
  "clears only the submitted commit draft, edited=%s",
  async (edited) => {
    const { bridge, saved } = fake(true);
    const original = bridge.request;
    let finish!: (value: Payload) => void;
    bridge.request = async (request) =>
      request.kind === "propose"
        ? new Promise<Payload>((resolve) => {
            finish = resolve;
          })
        : original(request);
    render(<App bridge={bridge} />);
    await userEvent.click(
      await screen.findByRole("button", { name: "parent" }),
    );
    await userEvent.click(
      screen.getByRole("checkbox", { name: "Include file.txt from index" }),
    );
    const message = screen.getByRole("textbox", { name: "Message" });
    fireEvent.change(message, { target: { value: "draft A" } });
    await userEvent.click(
      screen.getByRole("button", { name: "Commit · 1 file" }),
    );
    if (edited) fireEvent.change(message, { target: { value: "draft B" } });
    if (edited === "restore")
      fireEvent.change(message, { target: { value: "draft A" } });
    await act(async () =>
      finish({
        kind: "executed",
        value: { result: { phase: "complete", published: true } },
      }),
    );
    expect(message).toHaveValue(
      edited === "restore" ? "draft A" : edited ? "draft B" : "",
    );
    await waitFor(() =>
      expect(saved.at(-1)?.drafts.parent ?? "").toBe(
        edited === "restore" ? "draft A" : edited ? "draft B" : "",
      ),
    );
  },
);

it("keeps navigation and manual drafts during late AI streaming", async () => {
  const { bridge, saved } = fake(true);
  const original = bridge.request;
  const effective: Payload = {
    kind: "effective_ai_settings",
    value: {
      connection: {
        id: "c",
        revision: "r",
        name: "Local",
        provider: "ollama",
        endpoint: "http://127.0.0.1:1/",
        endpoint_approval: "",
        timeout_ms: 3000,
      },
      profile: {
        id: "p",
        revision: "r",
        name: "Profile",
        connection: "c",
        model: "model",
        system_message: "",
        parameters: {},
        template: "t",
        context_bytes: 16384,
      },
      template: {
        id: "t",
        revision: "r",
        name: "Plain",
        examples: [],
        rules: {
          style: "plain",
          subject_max_chars: 72,
          allowed_types: [],
          allow_scope: false,
          forbid_trailing_period: false,
          instructions: "",
        },
      },
      instructions: [],
    },
  };
  if (effective.kind !== "effective_ai_settings") throw new Error();
  effective.value.connection.provider = "compatible";
  let finishPoll: ((value: Payload) => void) | undefined;
  bridge.request = async (request) => {
    if (request.kind === "effective_ai_settings") return effective;
    if (request.kind === "preview_ai_context")
      return {
        kind: "ai_context",
        value: {
          snapshot_fingerprint: "snapshot",
          generation: "g",
          bundle: {
            payload: "diff",
            fingerprint: "context",
            complete: true,
            warnings: [],
            accounting: [],
          },
        },
      };
    if (request.kind === "review_ai_messages")
      return {
        kind: "executed",
        value: {
          result: null,
          ai_job_id: "job",
          ai_request_fingerprint: "request",
        },
      };
    if (request.kind === "ai_jobs")
      return new Promise<Payload>((resolve) => {
        finishPoll = resolve;
      });
    if (request.kind === "cancel_ai_job") return { kind: "done" };
    return original(request);
  };
  render(<App bridge={bridge} />);
  await userEvent.click(await screen.findByRole("button", { name: "parent" }));
  await userEvent.click(
    screen.getByRole("checkbox", { name: "Include file.txt from index" }),
  );
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Fill with AI" })).toBeEnabled(),
  );
  await userEvent.click(screen.getByRole("button", { name: "Fill with AI" }));
  const emit = async (message: string) => {
    await waitFor(() => expect(finishPoll).toBeDefined());
    const finish = finishPoll!;
    finishPoll = undefined;
    await act(async () =>
      finish({
        kind: "ai_jobs",
        value: [
          {
            id: "job",
            phase: "running",
            error: null,
            error_kind: null,
            review: {
              fingerprint: "request",
              endpoint: "http://localhost/",
              route: "http://localhost/v1/chat/completions",
              model: "model",
              purpose: "assisted_messages",
              request_bytes: 1,
              authentication: "none",
              unencrypted: true,
            },
            result: null,
            progress: {
              message,
              total_duration_ms: null,
              tokens_per_second: null,
              input_tokens: null,
              output_tokens: null,
            },
          },
        ],
      }),
    );
  };
  await emit("partial");
  const message = screen.getByRole("textbox", { name: "Message" });
  expect(message).toHaveValue("partial");
  await userEvent.click(screen.getByRole("button", { name: "Hide sidebar" }));
  await emit("streamed continuation");
  expect(message).toHaveValue("streamed continuation");
  expect(screen.getByRole("button", { name: "Show sidebar" })).toBeVisible();
  fireEvent.change(message, { target: { value: "manual draft" } });
  await emit("late replacement");
  expect(message).toHaveValue("manual draft");
  expect(screen.getByRole("button", { name: "Show sidebar" })).toBeVisible();
  await waitFor(() => expect(saved.at(-1)?.drafts.parent).toBe("manual draft"));
});
