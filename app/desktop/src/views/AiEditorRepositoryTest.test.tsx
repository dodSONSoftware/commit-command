import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import type { Bridge, Payload } from "../services/bridge";
import type { Request } from "../services/request.generated";
import { fixtureEntry } from "../testdata";
import { AiEditorRepositoryTest } from "./AiEditorRepositoryTest";
import type { AiEditorDraft } from "./AiMessages";

beforeEach(() => {
  Object.defineProperty(HTMLDialogElement.prototype, "showModal", {
    configurable: true,
    value: function (this: HTMLDialogElement) {
      this.setAttribute("open", "");
    },
  });
  Object.defineProperty(HTMLDialogElement.prototype, "close", {
    configurable: true,
    value: function (this: HTMLDialogElement) {
      this.removeAttribute("open");
      this.dispatchEvent(new Event("close"));
    },
  });
});
afterEach(() => {
  cleanup();
  Reflect.deleteProperty(HTMLDialogElement.prototype, "showModal");
  Reflect.deleteProperty(HTMLDialogElement.prototype, "close");
});
const editor: AiEditorDraft = {
  connection: {
    id: null,
    name: "Unsaved",
    provider: "ollama",
    endpoint: "http://127.0.0.1:1/",
    timeout_ms: 3000,
  },
  profile: {
    id: null,
    name: "Unsaved profile",
    connection: "",
    model: "fixture-model",
    system_message: "Current editor",
    parameters: {},
    template: "",
    context_bytes: 16384,
  },
  template: {
    id: "00000000-0000-4000-8000-000000000001",
    revision: "00000000-0000-4000-8000-000000000001",
    name: "Plain",
    rules: {
      style: "plain",
      subject_max_chars: 72,
      allowed_types: [],
      allow_scope: false,
      forbid_trailing_period: false,
      instructions: "Observed changes",
    },
    examples: [],
  },
  instructions: [],
};
it("requires explicit path sources, clears stale previews on repository change and retains choices on refresh failure", async () => {
  const user = userEvent.setup();
  let loads = 0;
  const request = vi.fn(async (r: Request): Promise<Payload> => {
    if (r.kind === "inventory") {
      if (++loads > 1) throw new Error("Inventory refresh failed");
      return {
        kind: "inventory",
        value: {
          entries: [
            fixtureEntry("parent", "parent"),
            fixtureEntry("child", "child"),
          ],
          monitor: { running: false, epoch: 1, watched: 0 },
        },
      };
    }
    if (r.kind === "preview_ai_context")
      return {
        kind: "ai_context",
        value: {
          generation: "g",
          snapshot_fingerprint: "s",
          bundle: {
            payload: "captured parent payload",
            fingerprint: "f",
            complete: true,
            warnings: [],
            accounting: [],
          },
        },
      };
    throw new Error(`Unexpected ${r.kind}`);
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
    <AiEditorRepositoryTest
      bridge={bridge}
      workspace="ws"
      worktree="parent"
      editor={editor}
      disabled={false}
    />,
  );
  const trigger = () => screen.getByRole("button", { name: "Test with AI…" });
  expect(trigger()).toBeDisabled();
  await user.click(
    screen.getByRole("button", {
      name: "Load repositories for AI test · no transfer",
    }),
  );
  expect(trigger()).toBeDisabled();
  await user.click(
    screen.getByRole("checkbox", {
      name: "Include AI test file.txt from index",
    }),
  );
  expect(
    screen.getByRole("checkbox", {
      name: "Include AI test file.txt from working_tree",
    }),
  ).toBeDisabled();
  await user.click(trigger());
  await user.click(
    screen.getByRole("button", {
      name: "Capture filtered AI context · no transfer",
    }),
  );
  expect(request).toHaveBeenCalledWith(
    expect.objectContaining({
      kind: "preview_ai_context",
      worktree: "parent",
      spec: expect.objectContaining({
        selected: [
          { path: { encoding: "utf8", value: "file.txt" }, source: "index" },
        ],
      }),
    }),
  );
  expect(
    screen.getByRole("button", { name: "Generate message suggestions" }),
  ).toBeEnabled();
  await user.click(
    screen.getByRole("button", { name: "Close AI suggestions" }),
  );
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Repository for AI test" }),
    "child",
  );
  expect(trigger()).toBeDisabled();
  await user.click(
    screen.getByRole("checkbox", {
      name: "Include AI test file.txt from working_tree",
    }),
  );
  await user.click(trigger());
  expect(
    screen.getByRole("button", { name: "Generate message suggestions" }),
  ).toBeEnabled();
  expect(screen.queryByText("captured parent payload")).not.toBeInTheDocument();
  await user.click(
    screen.getByRole("button", { name: "Close AI suggestions" }),
  );
  await user.click(
    screen.getByRole("button", {
      name: "Load repositories for AI test · no transfer",
    }),
  );
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Inventory refresh failed",
  );
  expect(
    screen.getByRole("checkbox", {
      name: "Include AI test file.txt from working_tree",
    }),
  ).toBeChecked();
  expect(trigger()).toBeEnabled();
  expect(
    request.mock.calls.every(
      ([r]) => r.kind === "inventory" || r.kind === "preview_ai_context",
    ),
  ).toBe(true);
});
function largeFixture(count: number) {
  const entry = fixtureEntry("large", "large");
  if (!entry.status) throw new Error("status fixture required");
  const change = entry.status.changes[0];
  entry.status.changes = Array.from({ length: count }, (_, index) => ({
    ...change,
    path: { encoding: "utf8", value: `file-${String(index).padStart(3, "0")}` },
    worktree: ".",
  }));
  const request = vi.fn(async (r: Request): Promise<Payload> => {
    if (r.kind === "inventory")
      return {
        kind: "inventory",
        value: {
          entries: [entry],
          monitor: { running: false, epoch: 1, watched: 0 },
        },
      };
    if (r.kind === "preview_ai_context")
      return {
        kind: "ai_context",
        value: {
          generation: "g",
          snapshot_fingerprint: "s",
          bundle: {
            payload: "fixture",
            fingerprint: "f",
            complete: true,
            warnings: [],
            accounting: [],
          },
        },
      };
    throw new Error(`Unexpected ${r.kind}`);
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
    <AiEditorRepositoryTest
      bridge={bridge}
      workspace="ws"
      worktree="large"
      editor={editor}
      disabled={false}
    />,
  );
  return { request, user: userEvent.setup() };
}
it("bounds the rendered source list and preserves explicitly selected hidden paths through search", async () => {
  const { user, request } = largeFixture(300);
  await user.click(
    screen.getByRole("button", {
      name: "Load repositories for AI test · no transfer",
    }),
  );
  expect(screen.getAllByRole("checkbox")).toHaveLength(256);
  const search = screen.getByRole("textbox", { name: "Filter AI test paths" });
  fireEvent.change(search, { target: { value: "file-299" } });
  await user.click(
    screen.getByRole("checkbox", {
      name: "Include AI test file-299 from index",
    }),
  );
  fireEvent.change(search, { target: { value: "file-000" } });
  await user.click(
    screen.getByRole("checkbox", {
      name: "Include AI test file-000 from index",
    }),
  );
  await user.click(screen.getByRole("button", { name: "Test with AI…" }));
  await user.click(
    screen.getByRole("button", {
      name: "Capture filtered AI context · no transfer",
    }),
  );
  const captures = request.mock.calls
    .map(([r]) => r)
    .filter((r) => r.kind === "preview_ai_context");
  expect(captures[0].spec.selected).toEqual([
    { path: { encoding: "utf8", value: "file-000" }, source: "index" },
    { path: { encoding: "utf8", value: "file-299" }, source: "index" },
  ]);
});
it("prevents a 129th selected path while allowing an explicit deselection", async () => {
  const { user } = largeFixture(132);
  await user.click(
    screen.getByRole("button", {
      name: "Load repositories for AI test · no transfer",
    }),
  );
  const checkboxes = screen.getAllByRole("checkbox");
  for (const checkbox of checkboxes.slice(0, 128)) fireEvent.click(checkbox);
  expect(checkboxes[128]).toBeDisabled();
  expect(checkboxes[0]).toBeEnabled();
  fireEvent.click(checkboxes[0]);
  expect(checkboxes[128]).toBeEnabled();
});
