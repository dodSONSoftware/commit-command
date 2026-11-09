import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import {
  WorkspaceActivity,
  activityDateRange,
  utcCalendar,
} from "./WorkspaceActivity";
import type { Bridge } from "../services/bridge";
import { fixtureEntry } from "../testdata";
afterEach(cleanup);
it("uses inclusive UTC boundaries across leap days and refuses invalid or excessive dates", () => {
  expect(activityDateRange("2024-02-28", "2024-03-01")).toEqual({
    since: Date.UTC(2024, 1, 28) / 1000,
    until: Date.UTC(2024, 2, 2) / 1000 - 1,
    days: ["2024-02-28", "2024-02-29", "2024-03-01"],
  });
  expect(activityDateRange("1970-01-01", "1970-01-01")).toEqual({
    since: 0,
    until: 86399,
    days: ["1970-01-01"],
  });
  expect(activityDateRange("2024-01-01", "2025-02-03").days).toHaveLength(400);
  for (const [start, end] of [
    ["", "2024-01-01"],
    ["2024-01-01", ""],
    ["2024-02-30", "2024-03-01"],
    ["1969-12-31", "1970-01-01"],
    ["2024-02-01", "2024-01-01"],
    ["2024-01-01", "2025-02-04"],
  ]) {
    expect(() => activityDateRange(start, end)).toThrow();
  }
});

it("applies chosen dates to requests and every view, rejects bad ranges and restores defaults", async () => {
  const onStateChange = vi.fn();
  const request = vi
    .fn<Bridge["request"]>()
    .mockImplementation(async (query) => ({
      kind: "activity",
      value: {
        items:
          query.kind === "activity" &&
          query.since === Date.UTC(2024, 1, 29) / 1000
            ? [
                {
                  repository: "r",
                  worktree: "parent",
                  commit: {
                    oid: { format: "sha1", hex: "2".repeat(40) },
                    subject: Array.from(
                      new TextEncoder().encode("Leap-day commit"),
                    ),
                    author: [],
                    committer_time: Date.UTC(2024, 1, 29, 23, 59, 59) / 1000,
                  },
                },
              ]
            : [],
        issues: [],
        complete: true,
        observed_at_ms: Date.now(),
      },
    }));
  const bridge: Bridge = {
    epoch: 1,
    request,
    capabilities: async () => ({ writes_enabled: false }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
  };
  const openCommit = vi.fn();
  render(
    <WorkspaceActivity
      bridge={bridge}
      workspace="ws"
      entries={[fixtureEntry("parent", "parent")]}
      act={async (action) => action()}
      openCommit={openCommit}
      onStateChange={onStateChange}
    />,
  );
  const apply = screen.getByRole("button", { name: "Apply activity filters" });
  await waitFor(() => expect(apply).toBeEnabled());
  const start = screen.getByLabelText("Activity start date");
  const end = screen.getByLabelText("Activity end date");
  fireEvent.change(start, { target: { value: "2024-02-29" } });
  fireEvent.click(apply);
  expect(screen.getByRole("alert")).toHaveTextContent("valid UTC dates");
  expect(request).toHaveBeenCalledTimes(1);
  expect(onStateChange).not.toHaveBeenCalled();
  fireEvent.change(end, { target: { value: "2024-02-28" } });
  fireEvent.click(apply);
  expect(screen.getByRole("alert")).toHaveTextContent(
    "start date must precede",
  );
  expect(request).toHaveBeenCalledTimes(1);
  fireEvent.change(end, { target: { value: "2024-03-01" } });
  fireEvent.change(screen.getByRole("textbox", { name: "Activity author" }), {
    target: { value: "Fixture" },
  });
  fireEvent.change(
    screen.getByRole("textbox", { name: "Activity reference" }),
    { target: { value: "refs/heads/main" } },
  );
  fireEvent.click(apply);
  const commit = await screen.findByRole("button", { name: /Leap-day commit/ });
  expect(request).toHaveBeenLastCalledWith({
    kind: "activity",
    workspace: "ws",
    worktree: null,
    author: "Fixture",
    reference: "refs/heads/main",
    since: Date.UTC(2024, 1, 29) / 1000,
    until: Date.UTC(2024, 2, 2) / 1000 - 1,
  });
  expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  expect(onStateChange).toHaveBeenLastCalledWith(
    expect.objectContaining({
      mode: "timeline",
      author: "Fixture",
      reference: "refs/heads/main",
      since: Date.UTC(2024, 1, 29) / 1000,
      until: Date.UTC(2024, 2, 2) / 1000 - 1,
    }),
  );
  fireEvent.click(commit);
  expect(openCommit).toHaveBeenCalledWith("parent", "2".repeat(40));
  fireEvent.click(
    screen.getByRole("button", { name: "Lanes · selected dates" }),
  );
  expect(onStateChange).toHaveBeenLastCalledWith(
    expect.objectContaining({ mode: "lanes", author: "Fixture" }),
  );
  expect(
    screen.getByRole("button", { name: /Leap-day commit/ }),
  ).toBeInTheDocument();
  fireEvent.click(
    screen.getByRole("button", { name: "Heatmap · selected dates" }),
  );
  expect(screen.getByLabelText("Contribution heatmap").children).toHaveLength(
    2,
  );
  expect(screen.getByLabelText("2024-02-29: 1 commits")).toBeInTheDocument();
  expect(screen.getByLabelText("2024-03-01: 0 commits")).toBeInTheDocument();
  fireEvent.change(start, { target: { value: "" } });
  fireEvent.change(end, { target: { value: "" } });
  fireEvent.click(apply);
  await waitFor(() => expect(apply).toBeEnabled());
  expect(
    screen.getByRole("button", { name: "Heatmap · 12 months" }),
  ).toBeInTheDocument();
  expect(
    screen.getByLabelText("Contribution heatmap").children.length,
  ).toBeGreaterThanOrEqual(365);
  expect(screen.queryByText(/Applied dates:/)).not.toBeInTheDocument();
  expect(onStateChange).toHaveBeenLastCalledWith(
    expect.objectContaining({ mode: "heatmap", since: null, until: null }),
  );
});

it("restores applied filters and mode without rebinding an unavailable repository", async () => {
  const request = vi.fn<Bridge["request"]>().mockResolvedValue({
    kind: "activity",
    value: {
      items: [],
      complete: true,
      issues: [],
      observed_at_ms: Date.now(),
    },
  });
  const onStateChange = vi.fn();
  const bridge: Bridge = {
    epoch: 1,
    request,
    capabilities: async () => ({ writes_enabled: false }),
    writeStatus: async () => false,
    pick: async () => null,
    cancel: async () => {},
  };
  render(
    <WorkspaceActivity
      bridge={bridge}
      workspace="ws"
      entries={[]}
      act={async (action) => action()}
      openCommit={() => {}}
      onStateChange={onStateChange}
      state={{
        mode: "heatmap",
        worktree: "old-id",
        author: "Saved author",
        reference: "refs/heads/main",
        since: 1709164800,
        until: 1709251199,
      }}
    />,
  );
  await waitFor(() =>
    expect(
      screen.getByRole("button", { name: "Apply activity filters" }),
    ).toBeEnabled(),
  );
  expect(request).toHaveBeenCalledWith({
    kind: "activity",
    workspace: "ws",
    worktree: "old-id",
    author: "Saved author",
    reference: "refs/heads/main",
    since: 1709164800,
    until: 1709251199,
  });
  expect(screen.getByLabelText("Activity start date")).toHaveValue(
    "2024-02-29",
  );
  expect(screen.getByLabelText("Activity end date")).toHaveValue("2024-02-29");
  expect(
    screen.getByRole("option", { name: /Unavailable repository/ }),
  ).toHaveValue("old-id");
  expect(
    screen.getByRole("button", { name: "Heatmap · selected dates" }),
  ).toHaveAttribute("aria-pressed", "true");
  expect(onStateChange).not.toHaveBeenCalled();
  fireEvent.change(screen.getByRole("textbox", { name: "Activity author" }), {
    target: { value: "Unsaved author" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Timeline · selected dates" }),
  );
  expect(onStateChange).toHaveBeenLastCalledWith(
    expect.objectContaining({
      mode: "timeline",
      author: "Saved author",
      worktree: "old-id",
    }),
  );
});
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
