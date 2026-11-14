import { useState } from "react";
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { WorkspaceTabs } from "./WorkspaceTabs";
import { initialState, openTab } from "../stores/model";
import type { DesktopState } from "../services/response.generated";
import { fixtureEntry } from "../testdata";
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});
function fixture() {
  const entries = [
    fixtureEntry("a", "a"),
    fixtureEntry("b", "b"),
    fixtureEntry("c", "c"),
  ];
  const initial = entries.reduce<DesktopState>(openTab, {
    ...initialState,
    drafts: { a: "Keep A", b: "Keep B", c: "Keep C" },
  });
  let latest: DesktopState = initial;
  const onPathAction = vi.fn();
  function Harness() {
    const [nav, setNav] = useState(initial);
    latest = nav;
    return (
      <WorkspaceTabs
        nav={nav}
        entries={entries}
        onChange={setNav}
        overviewActive={!nav.active_tab}
        repositoryActive
        onSelect={(id) => setNav((current) => ({ ...current, active_tab: id }))}
        onPathAction={onPathAction}
      />
    );
  }
  render(<Harness />);
  const menu = (id: string) =>
    fireEvent.contextMenu(
      screen.getByRole("button", { name: id }).closest(".tab")!,
      { clientX: 20, clientY: 20 },
    );
  return {
    menu,
    onPathAction,
    entries,
    state: () => latest,
    user: userEvent.setup(),
  };
}
it("provides the requested context actions and closes others without discarding drafts", async () => {
  const { menu, user, state } = fixture();
  menu("b");
  expect(
    screen.getAllByRole("menuitem").map((item) => item.textContent),
  ).toEqual([
    "Close",
    "Close Others",
    "Close to the Right",
    "Close All",
    "Copy Path",
    "Reveal in Finder",
    "Pin",
  ]);
  await user.click(screen.getByRole("menuitem", { name: "Close Others" }));
  expect(state().tabs.map((t) => t.worktree)).toEqual(["b"]);
  expect(state().drafts).toEqual({ a: "Keep A", b: "Keep B", c: "Keep C" });
  expect(
    screen.getByRole("button", { name: "▦ Overview" }),
  ).toBeInTheDocument();
});
it("closes the current tab and tabs to its right using their displayed order", async () => {
  const { menu, user, state } = fixture();
  menu("b");
  await user.click(
    screen.getByRole("menuitem", { name: "Close to the Right" }),
  );
  expect(state().tabs.map((t) => t.worktree)).toEqual(["a", "b"]);
  menu("b");
  expect(
    screen.getByRole("menuitem", { name: "Close to the Right" }),
  ).toBeDisabled();
  await user.click(screen.getByRole("menuitem", { name: "Close" }));
  expect(state().tabs.map((t) => t.worktree)).toEqual(["a"]);
  expect(state().drafts.c).toBe("Keep C");
});
it("pins tabs to the left and protects them from bulk closing", async () => {
  const { menu, user, state } = fixture();
  menu("c");
  await user.click(screen.getByRole("menuitem", { name: "Pin" }));
  expect(state().tabs.map((t) => t.worktree)).toEqual(["c", "a", "b"]);
  expect(state().tabs[0].pinned).toBe(true);
  menu("b");
  await user.click(screen.getByRole("menuitem", { name: "Close Others" }));
  expect(state().tabs.map((t) => t.worktree)).toEqual(["c", "b"]);
  fireEvent.contextMenu(
    screen.getByRole("button", { name: /Pinned.*c/ }).closest(".tab")!,
  );
  await user.click(screen.getByRole("menuitem", { name: "Unpin" }));
  expect(state().tabs.every((t) => !t.pinned)).toBe(true);
});
it("dispatches path actions for the right-clicked repository, not the active tab", async () => {
  const { menu, user, onPathAction, entries } = fixture();
  menu("a");
  await user.click(screen.getByRole("menuitem", { name: "Copy Path" }));
  expect(onPathAction).toHaveBeenLastCalledWith(entries[0], "copy");
  menu("b");
  await user.click(screen.getByRole("menuitem", { name: "Reveal in Finder" }));
  expect(onPathAction).toHaveBeenLastCalledWith(entries[1], "reveal");
});
it("reorders repository tabs by drag and keyboard while keeping Overview fixed", async () => {
  const { state } = fixture();
  const source = screen.getByRole("button", { name: "c" }).closest(".tab")!;
  const target = screen.getByRole("button", { name: "a" }).closest(".tab")!;
  vi.stubGlobal("PointerEvent", MouseEvent);
  const previous = document.elementFromPoint;
  document.elementFromPoint = () => target;
  vi.spyOn(target, "getBoundingClientRect").mockReturnValue({
    left: 100,
    width: 100,
  } as DOMRect);
  fireEvent.pointerDown(source, { button: 0, clientX: 300, clientY: 20 });
  fireEvent.pointerMove(source, { clientX: 110, clientY: 20 });
  fireEvent.pointerUp(source, { clientX: 110, clientY: 20 });
  document.elementFromPoint = previous;
  expect(state().tabs.map((t) => t.worktree)).toEqual(["c", "a", "b"]);
  fireEvent.keyDown(screen.getByRole("button", { name: "c" }), {
    key: "ArrowRight",
    altKey: true,
    shiftKey: true,
  });
  expect(state().tabs.map((t) => t.worktree)).toEqual(["a", "c", "b"]);
  expect(state().active_tab).toBe("c");
  expect(state().drafts.a).toBe("Keep A");
  expect(
    screen.getByRole("button", { name: "▦ Overview" }).closest(".tab"),
  ).toBeNull();
});
it("opens the menu by keyboard and dismisses it with Escape", async () => {
  const { user } = fixture();
  const tab = screen.getByRole("button", { name: "a" });
  tab.focus();
  await user.keyboard("{Shift>}{F10}{/Shift}");
  expect(screen.getByRole("menuitem", { name: "Close" })).toHaveFocus();
  await user.keyboard("{Escape}");
  expect(screen.queryByRole("menu")).toBeNull();
  expect(tab).toHaveFocus();
});
it("keeps the menu available when WebKit blurs a button before a pointer click", async () => {
  const { menu, user, state } = fixture();
  menu("b");
  fireEvent.blur(screen.getByRole("menuitem", { name: "Close" }), {
    relatedTarget: null,
  });
  await user.click(screen.getByRole("menuitem", { name: "Pin" }));
  expect(state().tabs[0]).toMatchObject({ worktree: "b", pinned: true });
});

it("closes all unpinned tabs while preserving pinned tabs, Overview and drafts", async () => {
  const { menu, user, state } = fixture();
  menu("b");
  await user.click(screen.getByRole("menuitem", { name: "Pin" }));
  menu("a");
  await user.click(screen.getByRole("menuitem", { name: "Close All" }));
  expect(state().tabs.map((t) => t.worktree)).toEqual(["b"]);
  expect(state().drafts).toEqual({ a: "Keep A", b: "Keep B", c: "Keep C" });
  expect(screen.getByRole("button", { name: "▦ Overview" })).toHaveFocus();
  fireEvent.contextMenu(
    screen.getByRole("button", { name: /Pinned.*b/ }).closest(".tab")!,
  );
  expect(screen.getByRole("menuitem", { name: "Close All" })).toBeDisabled();
});
it("returns to Overview when Close All removes every repository tab", async () => {
  const { menu, user, state } = fixture();
  menu("a");
  await user.click(screen.getByRole("menuitem", { name: "Close All" }));
  expect(state().tabs).toEqual([]);
  expect(state().active_tab).toBeNull();
  expect(state().drafts.c).toBe("Keep C");
});
