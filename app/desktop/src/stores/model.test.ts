import { describe, it, expect } from "vitest";
import {
  initialState,
  toggleSelection,
  openTab,
  closeTab,
  buildTree,
  pathText,
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
it("keeps invalid UTF-8 and bidi controls visible instead of conflating display identities", () => {
  expect(pathText({ encoding: "unix_bytes", value: [255] })).not.toBe(
    pathText({ encoding: "unix_bytes", value: [254] }),
  );
  expect(pathText({ encoding: "utf8", value: "safe\u202eevil" })).toContain(
    "\\u202e",
  );
});
