import { afterEach, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import type { Snapshot } from "../services/response.generated";
import { RepositoryRemoteStatus } from "./RepositoryRemoteStatus";

afterEach(cleanup);
const snapshot: Snapshot = {
  head: null,
  branch: "main",
  status: "",
  head_oid: null,
  unborn: false,
  detached: false,
  counts: { staged: 0, unstaged: 0, untracked: 0, conflicted: 0 },
  changes: [],
  tracking: null,
  remotes: ["origin", "backup"],
  operations: [],
  last_commit: null,
  generation: "fixture",
  observed_at_ms: 0,
};
it.each([
  [0, 0, "✓ Up to date", "success"],
  [3, 0, "↑ Ahead 3", "modified"],
  [0, 7, "↓ Behind 7", "warning"],
  [3, 7, "↕ Diverged · ↑ 3 ↓ 7", "warning"],
  [null, null, "? Comparison unavailable", ""],
])(
  "shows local comparison %s/%s without claiming server freshness",
  (ahead, behind, label, tone) => {
    render(
      <RepositoryRemoteStatus
        snapshot={{
          ...snapshot,
          tracking: {
            reference: "refs/remotes/origin/main",
            oid: null,
            ahead,
            behind,
            comparison: "local_tracking_refs_only",
            fetch_freshness: null,
          },
        }}
      />,
    );
    expect(screen.getByText(label)).toHaveClass("badge");
    if (tone) expect(screen.getByText(label)).toHaveClass(tone);
    expect(screen.getByText("Compared with local tracking ref")).toBeVisible();
    expect(screen.getByText(/the server may have changed/)).toBeInTheDocument();
    expect(
      screen.getByText(/Configured remotes: origin, backup/),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Upstream: refs\/remotes\/origin\/main/),
    ).toBeInTheDocument();
  },
);
it.each([
  [[], "○ No remote"],
  [["origin"], "◇ No upstream"],
  [undefined, "? Remote status unknown"],
])("distinguishes configured remotes %s", (remotes, label) => {
  render(<RepositoryRemoteStatus snapshot={{ ...snapshot, remotes }} />);
  expect(screen.getByText(label)).toBeVisible();
});
it("does not label an uninspected repository as having no remote", () => {
  render(<RepositoryRemoteStatus />);
  expect(screen.getByText("? Remote status unknown")).toBeVisible();
});
