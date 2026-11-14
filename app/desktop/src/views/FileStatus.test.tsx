import { afterEach, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { FileStatus } from "./FileStatus";
afterEach(cleanup);
it.each([
  ["M", "Modified"],
  ["?", "Untracked"],
  ["A", "Added"],
  ["D", "Deleted"],
  ["R", "Renamed"],
  ["C", "Copied"],
  ["T", "Type changed"],
])("shows Git code %s with its hover description", (code, label) => {
  render(
    <FileStatus
      change={{ index: code, worktree: code, conflict: false }}
      source="working_tree"
    />,
  );
  expect(screen.getByRole("img", { name: label })).toHaveTextContent(code);
  expect(screen.getByRole("img", { name: label })).toHaveAttribute(
    "title",
    label,
  );
});
it("uses the status for the displayed staged or unstaged group", () => {
  const change = { index: "A", worktree: "M", conflict: false };
  const { rerender } = render(<FileStatus change={change} source="index" />);
  expect(screen.getByTitle("Added")).toHaveTextContent("A");
  rerender(<FileStatus change={change} source="working_tree" />);
  expect(screen.getByTitle("Modified")).toHaveTextContent("M");
});
it.each([
  ["U", "U", "Unmerged: both modified"],
  ["A", "A", "Unmerged: both added"],
  ["D", "D", "Unmerged: both deleted"],
])(
  "preserves Git's unmerged status %s%s instead of using the ignored-file marker",
  (index, worktree, label) => {
    render(
      <FileStatus
        change={{ index, worktree, conflict: true }}
        source="working_tree"
      />,
    );
    expect(screen.getByTitle(label)).toHaveTextContent(index + worktree);
    expect(screen.getByTitle(label)).toHaveClass("bad");
  },
);
