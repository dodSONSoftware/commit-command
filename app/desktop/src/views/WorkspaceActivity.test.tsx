import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { WorkspaceActivity, utcCalendar } from "./WorkspaceActivity";
import type { Bridge } from "../services/bridge";
import { fixtureEntry } from "../testdata";
afterEach(cleanup);
it("builds a twelve-calendar-month UTC heatmap across leap days and year boundaries", () => {
  const days = utcCalendar(new Date("2024-02-29T23:00:00Z"));
  expect(days[0]).toBe("2023-02-28");
  expect(days.at(-1)).toBe("2024-02-29");
  expect(new Set(days).size).toBe(days.length);
  for (let i = 1; i < days.length; i++)
    expect(Date.parse(days[i]) - Date.parse(days[i - 1])).toBe(86400000);
});
it("shows partial coverage, inert commit text, filters and navigation to a real commit", async () => {
  const openCommit = vi.fn();
  const request = vi.fn<Bridge["request"]>().mockResolvedValue({
    kind: "activity",
    value: {
      items: [
        {
          repository: "r",
          worktree: "parent",
          commit: {
            oid: { format: "sha1", hex: "1".repeat(40) },
            subject: Array.from(new TextEncoder().encode("<img src=x>")),
            author: [],
            committer_time: Math.floor(Date.now() / 1000),
          },
        },
      ],
      issues: [],
      complete: false,
      observed_at_ms: Date.now(),
    },
  });
  const bridge: Bridge = {
    epoch: 1,
    request,
    capabilities: async () => ({ writes_enabled: false }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
  };
  const act = async (action: () => Promise<void>) => action();
  const { container } = render(
    <WorkspaceActivity
      bridge={bridge}
      workspace="ws"
      entries={[fixtureEntry("parent", "parent")]}
      act={act}
      openCommit={openCommit}
    />,
  );
  const commit = await screen.findByRole("button", { name: /<img src=x>/ });
  expect(container.querySelector("img")).toBeNull();
  expect(screen.getByRole("status")).toHaveTextContent("Partial activity");
  fireEvent.click(commit);
  expect(openCommit).toHaveBeenCalledWith("parent", "1".repeat(40));
  fireEvent.change(screen.getByRole("textbox", { name: "Activity author" }), {
    target: { value: "Fixture" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Apply activity filters" }),
  );
  await waitFor(() =>
    expect(request).toHaveBeenLastCalledWith(
      expect.objectContaining({
        kind: "activity",
        author: "Fixture",
        workspace: "ws",
      }),
    ),
  );
  fireEvent.click(screen.getByRole("button", { name: "Heatmap · 12 months" }));
  expect(
    screen.getByLabelText("Contribution heatmap").children.length,
  ).toBeGreaterThanOrEqual(365);
  expect(screen.getAllByRole("img").length).toBeGreaterThanOrEqual(365);
  fireEvent.click(screen.getByText("Daily activity counts"));
  expect(
    screen.getByRole("table", { name: "Daily observed commits · UTC" }),
  ).toBeInTheDocument();
  expect(screen.getAllByRole("row").length).toBeGreaterThanOrEqual(366);
});
