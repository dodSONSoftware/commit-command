import {
  cleanup,
  render,
  screen,
  waitFor,
  fireEvent,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it } from "vitest";
import { App } from "./App";
import type { Bridge, Payload } from "../services/bridge";
import type { Request } from "../services/request.generated";
import type { DesktopState } from "../services/response.generated";
import { initialState } from "../stores/model";
import { fixtureEntry, fixtureWorkspace } from "../testdata";
afterEach(cleanup);
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
            },
          };
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
  expect(container.querySelector("img")).toBeNull();
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
  await screen.findByText("No GitFleet operations in this workspace.");
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
    if (data.kind === "history") data.value.commits = [commit];
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
  await waitFor(
    () => {
      expect(screen.getByText("granted · current")).toBeInTheDocument();
      expect(
        screen.getByRole("button", { name: "Review commit · 0 files" }),
      ).toBeInTheDocument();
      expect(
        screen.getByRole("button", { name: "Review stage" }),
      ).toBeDisabled();
    },
    { timeout: 5000 },
  );
  clean = false;
  await waitFor(
    () => {
      expect(
        screen.getByRole("checkbox", { name: "Include file.txt from index" }),
      ).not.toBeChecked();
      expect(
        screen.getByRole("button", { name: "Review stage" }),
      ).toBeDisabled();
    },
    { timeout: 5000 },
  );
}, 10000);
