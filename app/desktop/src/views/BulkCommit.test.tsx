import { useState } from "react";
import { initialBulkComposer } from "../stores/model";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { BulkCommit } from "./BulkCommit";
import type { Bridge } from "../services/bridge";
import { fixtureEntry } from "../testdata";

afterEach(cleanup);
function setup(enabled = true) {
  const request = vi.fn<Bridge["request"]>().mockImplementation(async (r) => {
    if (r.kind === "batches")
      return { kind: "batches", value: { items: [], next: null } };
    if (r.kind === "cancel_reviewed_operation") return { kind: "done" };
    if (r.kind === "execute_reviewed_operation")
      return { kind: "executed", value: { result: {} } };
    return {
      kind: "review",
      value: {
        id: "review",
        operation: "bulk_commit",
        epoch: 1,
        expires_at_ms: Date.now() + 60000,
        title: "Review",
        summary: "",
        content: "Worktree: alpha\nSelected content: docs.md",
      },
    };
  });
  const bridge: Bridge = {
    epoch: 1,
    request,
    capabilities: async () => ({ writes_enabled: enabled }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
  };
  const setSelected = vi.fn();
  const setDraft = vi.fn();
  const act = async (action: () => Promise<void>) => action();
  function Controlled() {
    const [composer, setComposer] = useState(initialBulkComposer);
    return (
      <BulkCommit
        bridge={bridge}
        workspace="ws"
        entries={[
          fixtureEntry("a", "alpha"),
          fixtureEntry("b", "beta"),
          fixtureEntry("c", "unselected"),
        ]}
        selected={["a", "b"]}
        composer={composer}
        setComposer={setComposer}
        drafts={{ b: "Repository override" }}
        setDraft={setDraft}
        setSelected={setSelected}
        enabled={enabled}
        busy={false}
        act={act}
      />
    );
  }
  render(<Controlled />);
  return { request, setSelected, setDraft };
}
it("prepares only explicit selections with exact overrides, sources and exclusions through review", async () => {
  const { request } = setup();
  fireEvent.change(screen.getByLabelText("Shared commit message"), {
    target: { value: "Shared\n\nBody" },
  });
  fireEvent.change(screen.getByLabelText("Content · alpha"), {
    target: { value: "index_only" },
  });
  fireEvent.click(screen.getAllByLabelText("Exclude file.txt")[0]);
  fireEvent.click(screen.getByRole("button", { name: "Review bulk commit" }));
  await waitFor(() =>
    expect(request).toHaveBeenCalledWith({
      kind: "review_batch",
      workspace: "ws",
      selections: [
        {
          worktree: "a",
          message: "Shared\n\nBody",
          mode: "index_only",
          excluded: [{ encoding: "utf8", value: "file.txt" }],
          hooks: "execute",
          signing: "configured",
        },
        {
          worktree: "b",
          message: "Repository override",
          mode: "review_all",
          excluded: [],
          hooks: "execute",
          signing: "configured",
        },
      ],
    }),
  );
  expect(
    request.mock.calls.every(([r]) =>
      ["review_batch", "batches"].includes(r.kind),
    ),
  ).toBe(true);
  expect(await screen.findByText(/Selected content: docs.md/)).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Cancel review" }));
  await waitFor(() =>
    expect(request).toHaveBeenCalledWith({
      kind: "cancel_reviewed_operation",
      id: "review",
      operation: "bulk_commit",
    }),
  );
  expect(
    request.mock.calls.some(([r]) => r.kind === "execute_reviewed_operation"),
  ).toBe(false);
  fireEvent.click(screen.getByRole("button", { name: "Review bulk commit" }));
  fireEvent.click(
    await screen.findByRole("button", { name: "Commit reviewed changes" }),
  );
  await waitFor(() =>
    expect(request).toHaveBeenCalledWith({
      kind: "execute_reviewed_operation",
      id: "review",
      operation: "bulk_commit",
    }),
  );
});
it("keeps write preparation disabled in read-only builds", () => {
  const { request } = setup(false);
  fireEvent.change(screen.getByLabelText("Shared commit message"), {
    target: { value: "Message" },
  });
  expect(
    screen.getByRole("button", { name: "Review bulk commit" }),
  ).toBeDisabled();
  expect(request).not.toHaveBeenCalled();
});
it.each([
  { phase: "blocked", label: "Not published" },
  { phase: "uncertain_outcome", label: "Publication unconfirmed" },
] as const)(
  "shows durable publication honestly for $phase and retries only unfinished items",
  async ({ phase, label }) => {
    const { request, setSelected } = setup();
    request.mockResolvedValueOnce({
      kind: "batches",
      value: {
        items: [
          {
            record: {
              version: 1,
              id: "batch",
              workspace: "ws",
              created_at_ms: 0,
              phase: "partial_failure",
              issue: null,
              items: [
                {
                  worktree: "a",
                  path: { encoding: "utf8", value: "/alpha" },
                  dependencies: [],
                  operation: "one",
                  issue: null,
                },
                {
                  worktree: "b",
                  path: { encoding: "utf8", value: "/beta" },
                  dependencies: [],
                  operation: "two",
                  issue: null,
                },
              ],
            },
            outcomes: [
              {
                operation_id: "one",
                phase: "complete",
                published: true,
                index_reconciled: true,
                post_processing: [],
              },
              {
                operation_id: "two",
                phase,
                published: false,
                index_reconciled: false,
                post_processing: [],
                reason: "<script>untrusted text</script>",
              },
            ],
          },
        ],
        next: null,
      },
    });
    const { container } = { container: document.body };
    fireEvent.click(
      screen.getByRole("button", { name: "Refresh batch results" }),
    );
    const retry = await screen.findByRole("button", {
      name: "Select unfinished worktrees for fresh review",
    });
    expect(screen.getByText(new RegExp(label))).toBeInTheDocument();
    expect(container.querySelector("script")).toBeNull();
    fireEvent.click(retry);
    expect(setSelected).toHaveBeenCalledWith(["b"]);
    expect(request).toHaveBeenCalledTimes(1);
  },
);

it("adds an exact gitlink exclusion absent from restricted status without preparing a review on Enter", async () => {
  const { request } = setup();
  fireEvent.change(screen.getByLabelText("Shared commit message"), {
    target: { value: "Reviewed parent" },
  });
  fireEvent.change(screen.getByLabelText("Additional exclusion · alpha"), {
    target: { value: "modules/child" },
  });
  fireEvent.keyDown(screen.getByLabelText("Additional exclusion · alpha"), {
    key: "Enter",
  });
  expect(screen.getByLabelText("Exclude modules/child")).toBeChecked();
  expect(request.mock.calls.some(([r]) => r.kind === "review_batch")).toBe(
    false,
  );
  fireEvent.click(screen.getByRole("button", { name: "Review bulk commit" }));
  await waitFor(() => {
    const call = request.mock.calls.find(
      ([r]) => r.kind === "review_batch",
    )?.[0];
    expect(call).toMatchObject({
      kind: "review_batch",
      selections: [
        {
          worktree: "a",
          excluded: [{ encoding: "utf8", value: "modules/child" }],
        },
        { worktree: "b", excluded: [] },
      ],
    });
  });
});

it("keeps manually added exclusions visible, exact and removable", async () => {
  const { request } = setup();
  fireEvent.change(screen.getByLabelText("Shared commit message"), {
    target: { value: "Manual exclusions" },
  });
  const path = " modules/child ";
  fireEvent.change(screen.getByLabelText("Additional exclusion · alpha"), {
    target: { value: path },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Add exclusion · alpha" }),
  );
  const checkbox = screen.getByLabelText("Exclude modules/child");
  expect(checkbox).toBeChecked();
  fireEvent.click(screen.getByRole("button", { name: "Review bulk commit" }));
  await waitFor(() => {
    const call = request.mock.calls.find(
      ([r]) => r.kind === "review_batch",
    )?.[0];
    expect(call).toMatchObject({
      selections: [
        { excluded: [{ encoding: "utf8", value: path }] },
        { excluded: [] },
      ],
    });
  });
  fireEvent.click(checkbox);
  expect(
    screen.queryByLabelText("Exclude modules/child"),
  ).not.toBeInTheDocument();
});
