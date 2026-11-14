import { expect, it } from "vitest";
import { fixtureEntry } from "../testdata";
import { actionSelection, changeRows } from "./changeSelection";

const modified = fixtureEntry("id", "repo").status!.changes[0];
const original = { encoding: "unix_bytes" as const, value: [111, 255] };
const destination = { encoding: "unix_bytes" as const, value: [110, 254] };
const change = { ...modified, path: destination, original_path: original };

it("expands a rename only on the side reporting it, preserving native path bytes", () => {
  const stagedRename = changeRows([{ ...change, index: "R", worktree: "M" }]);
  const working = { path: destination, source: "working_tree" };
  const deletion = { path: original, source: "index" };
  expect(actionSelection(stagedRename, "stage")).toEqual({
    count: 1,
    selected: [working],
  });
  expect(actionSelection(stagedRename, "unstage")).toEqual({
    count: 1,
    selected: [deletion, { path: destination, source: "index" }],
  });
  expect(actionSelection([...stagedRename].reverse(), "commit")).toEqual({
    count: 1,
    selected: [working, deletion],
  });

  const workingRename = changeRows([{ ...change, index: ".", worktree: "R" }]);
  expect(actionSelection(workingRename, "stage").selected).toEqual([
    { path: original, source: "working_tree" },
    working,
  ]);
});

it("takes content from a copy's source only when that path is separately checked", () => {
  const [copy] = changeRows([{ ...change, index: "C", worktree: "." }]);
  expect(actionSelection([copy], "unstage").selected).toEqual([
    { path: destination, source: "index" },
  ]);
  const [source] = changeRows([
    { ...modified, path: original, index: ".", worktree: "M" },
  ]);
  expect(actionSelection([copy, source], "commit")).toEqual({
    count: 2,
    selected: [
      { path: destination, source: "index" },
      { path: original, source: "working_tree" },
    ],
  });
});

it("uses a recreated rename source when its working row is explicitly checked", () => {
  const rows = changeRows([
    { ...change, index: "R", worktree: "." },
    { ...modified, path: original, index: "?", worktree: "?" },
  ]);
  expect(actionSelection(rows, "commit")).toEqual({
    count: 2,
    selected: [
      { path: original, source: "working_tree" },
      { path: destination, source: "index" },
    ],
  });
});
