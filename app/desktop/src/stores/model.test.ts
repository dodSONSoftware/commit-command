import { describe, it, expect } from "vitest";
import {
  initialState,
  toggleSelection,
  openTab,
  closeTab,
  buildTree,
  pathText,
  filtered,
  sortDefinitions,
  toggleSort,
  attentionChoices,
  collapseTree,
} from "./model";
import { fixtureEntry } from "../testdata";
describe("identity and navigation", () => {
  it("does not select descendants or clear selection on tab close", () => {
    const parent = fixtureEntry("parent", "parent");
    const child = fixtureEntry("child", "parent/child");
    const state = toggleSelection(initialState, parent.id);
    expect(state.selected).toEqual(["parent"]);
    const opened = openTab(state, child);
    expect(closeTab(opened, child.id).selected).toEqual(["parent"]);
  });
  it("honors manual view and preserves drafts across changing observations", () => {
    const entry = fixtureEntry("parent", "parent");
    let state = openTab(initialState, entry);
    state = {
      ...state,
      tabs: state.tabs.map((t) => ({ ...t, view: "history" })),
      drafts: { parent: "keep my draft" },
    };
    expect(openTab(state, { ...entry, status: null }).tabs[0].view).toBe(
      "history",
    );
    expect(openTab(state, entry).drafts.parent).toBe("keep my draft");
  });
  it("builds nesting from paths without converting parents into recursive selection", () => {
    const nodes = buildTree(
      [fixtureEntry("a", "a"), fixtureEntry("b", "a/b")],
      "/tmp/project",
    );
    expect(nodes[0].entry?.id).toBe("a");
    expect(nodes[0].children[0].entry?.id).toBe("b");
  });
  it("escapes display controls without changing native execution bytes", () => {
    const value = { encoding: "utf8" as const, value: "line\nname" };
    expect(pathText(value)).toBe("line\\nname");
    expect(value.value).toContain("\n");
  });
});
describe("overview sorting", () => {
  function rows() {
    const a = fixtureEntry("a", "alpha");
    const b = fixtureEntry("b", "beta");
    if (!a.status || !b.status) throw new Error("fixture requires status");
    a.status.branch = "refs/heads/alpha";
    b.status.branch = "refs/heads/beta";
    a.status.counts = { staged: 1, unstaged: 2, untracked: 3, conflicted: 4 };
    b.status.counts = { staged: 2, unstaged: 3, untracked: 4, conflicted: 5 };
    a.status.tracking = {
      reference: "refs/remotes/origin/main",
      comparison: "cached",
      ahead: 1,
      behind: 2,
    };
    b.status.tracking = {
      reference: "refs/remotes/origin/main",
      comparison: "cached",
      ahead: 2,
      behind: 3,
    };
    const commit = {
      author: [],
      committer: [],
      author_time: 1,
      committer_time: 1,
      message: [120],
      oid: { hex: "1".repeat(40), format: "sha1" as const },
      tree: { hex: "2".repeat(40), format: "sha1" as const },
      parents: [],
    };
    a.status.last_commit = commit;
    b.status.last_commit = { ...commit, committer_time: 2 };
    return [a, b];
  }
  it("sorts every observed column in both directions with unknowns last", () => {
    const [a, b] = rows();
    const missing = { ...fixtureEntry("missing", "000-unknown"), status: null };
    const trust = [
      { worktree: "a", eligible: false, grant: "none", validity: "alpha" },
      { worktree: "b", eligible: false, grant: "none", validity: "beta" },
    ];
    for (const definition of sortDefinitions.filter(
      (item) => item.key !== "status",
    )) {
      const expectedAscending =
        definition.key === "path" || definition.key === "name"
          ? ["missing", "a", "b"]
          : ["a", "b", "missing"];
      const expectedDescending = ["b", "a", "missing"];
      expect(
        filtered([b, missing, a], "", "all", definition.ascending, trust).map(
          (e) => e.id,
        ),
        definition.key,
      ).toEqual(expectedAscending);
      expect(
        filtered([a, missing, b], "", "all", definition.descending, trust).map(
          (e) => e.id,
        ),
        definition.key,
      ).toEqual(expectedDescending);
      expect(toggleSort(definition.ascending, definition.key)).toBe(
        definition.descending,
      );
      expect(toggleSort(definition.descending, definition.key)).toBe(
        definition.ascending,
      );
    }
  });
  it("breaks ties by path then ID independently of observation arrival order", () => {
    const a = fixtureEntry("a", "same");
    const b = fixtureEntry("b", "same");
    expect(filtered([b, a], "", "all", "staged_desc").map((e) => e.id)).toEqual(
      ["a", "b"],
    );
    expect(filtered([b, a], "", "all", "changes").map((e) => e.id)).toEqual([
      "a",
      "b",
    ]);
  });
  it("preserves input and explicit selection while filtering, including status sorting", () => {
    const [a, b] = rows();
    const input = [b, a];
    const selected = toggleSelection(initialState, b.id);
    expect(
      filtered(input, "alpha", "issues", "status").map((e) => e.id),
    ).toEqual(["a"]);
    expect(input.map((e) => e.id)).toEqual(["b", "a"]);
    expect(selected.selected).toEqual(["b"]);
    if (a.status && b.status) {
      a.status.counts.conflicted = 0;
      b.status.counts.conflicted = 0;
      a.status.counts.staged = 0;
      a.status.counts.unstaged = 0;
      a.status.counts.untracked = 0;
    }
    expect(filtered(input, "", "all", "status").map((e) => e.id)).toEqual([
      "b",
      "a",
    ]);
    expect(filtered(input, "", "all", "status_desc").map((e) => e.id)).toEqual([
      "a",
      "b",
    ]);
  });
});
it("filters each attention category separately from search without treating unknown tracking as zero evidence", () => {
  const changed = fixtureEntry("changed", "changed");
  const clean = fixtureEntry("clean", "clean");
  const unknown = { ...fixtureEntry("unknown", "unknown"), status: null };
  if (!changed.status || !clean.status)
    throw new Error("fixture status required");
  changed.status.counts = {
    staged: 1,
    unstaged: 2,
    untracked: 3,
    conflicted: 4,
  };
  changed.status.tracking = {
    reference: "refs/remotes/origin/main",
    comparison: "cached",
    ahead: 1,
    behind: 2,
  };
  clean.status.counts = { staged: 0, unstaged: 0, untracked: 0, conflicted: 0 };
  for (const choice of attentionChoices) {
    const expected =
      choice.value === "all"
        ? ["changed", "clean", "unknown"]
        : choice.value === "issues"
          ? ["changed", "unknown"]
          : ["changed"];
    expect(
      filtered([unknown, clean, changed], "", choice.value, "path").map(
        (e) => e.id,
      ),
      choice.value,
    ).toEqual(expected);
    expect(
      filtered(
        [unknown, clean, changed],
        "does-not-match",
        choice.value,
        "path",
      ),
    ).toEqual([]);
  }
});
it("persists collapsed groups without modifying explicit descendant selection", () => {
  const selected = toggleSelection(initialState, "child");
  const collapsed = collapseTree(selected, "/parent", true);
  expect(collapsed.collapsed).toEqual(["/parent"]);
  expect(collapsed.selected).toEqual(["child"]);
  expect(collapseTree(collapsed, "/parent", true)).toBe(collapsed);
  const expanded = collapseTree(collapsed, "/parent", false);
  expect(expanded.collapsed).toEqual([]);
  expect(expanded.selected).toEqual(["child"]);
});
it("keeps invalid UTF-8 and bidi controls visible instead of conflating display identities", () => {
  expect(pathText({ encoding: "unix_bytes", value: [255] })).not.toBe(
    pathText({ encoding: "unix_bytes", value: [254] }),
  );
  expect(pathText({ encoding: "utf8", value: "safe\u202eevil" })).toContain(
    "\\u202e",
  );
});
