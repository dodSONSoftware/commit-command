import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { GraphLines, HistoryGraph } from "./HistoryGraph";
import type { Bridge } from "../services/bridge";
import type { Commit } from "../services/response.generated";
afterEach(cleanup);
const oid = { format: "sha1" as const, hex: "1".repeat(40) };
const parent = { format: "sha1" as const, hex: "2".repeat(40) };
const commit: Commit = {
  oid,
  tree: oid,
  parents: [parent],
  author: [],
  committer: [],
  author_time: 1,
  committer_time: 1,
  message: Array.from(new TextEncoder().encode("Graph commit")),
};
it("draws only real parent edges and no synthetic incoming edge for a new tip", () => {
  const { container } = render(
    <GraphLines
      row={{ commit, lane: 0, tip: true, before: [oid], after: [parent] }}
    />,
  );
  expect(container.querySelectorAll("line")).toHaveLength(1);
  expect(container.querySelectorAll("circle")).toHaveLength(1);
});
it("renders real commit selection and keeps large loaded histories virtualized", async () => {
  const onSelect = vi.fn();
  const request = vi.fn<Bridge["request"]>().mockResolvedValue({
    kind: "graph",
    value: {
      rows: Array.from({ length: 100 }, (_, i) => ({
        commit: {
          ...commit,
          oid: { ...oid, hex: i.toString(16).padStart(40, "0") },
        },
        lane: 0,
        tip: i === 0,
        before: [oid],
        after: [parent],
      })),
      references: [],
      next: null,
      shallow_boundaries: [],
      generation: "g",
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
  render(
    <HistoryGraph
      bridge={bridge}
      act={(action) => action()}
      workspace="ws"
      worktree="wt"
      selected={null}
      reference=""
      onReference={() => {}}
      onSelect={onSelect}
    />,
  );
  const rows = await screen.findAllByRole("button", { name: /Graph commit/ });
  expect(rows.length).toBeLessThan(100);
  fireEvent.click(rows[0]);
  expect(onSelect).toHaveBeenCalledWith("0".repeat(40));
  fireEvent.scroll(screen.getByLabelText("Commit graph"), {
    target: { scrollTop: 34 * 80 },
  });
  const lastRows = screen.getAllByRole("button", { name: /Graph commit/ });
  fireEvent.click(lastRows.at(-1)!);
  expect(onSelect).toHaveBeenLastCalledWith(
    (99).toString(16).padStart(40, "0"),
  );
});
