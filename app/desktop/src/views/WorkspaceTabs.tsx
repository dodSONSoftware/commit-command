import {
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  type Dispatch,
  type SetStateAction,
} from "react";
import { createPortal } from "react-dom";
import type {
  DesktopState,
  StoredWorktree,
} from "../services/response.generated";
import {
  closeTab,
  repoName,
  reorderTab,
  toggleTabPinned,
} from "../stores/model";

export function WorkspaceTabs({
  nav,
  entries,
  onChange,
  overviewActive,
  repositoryActive,
  onSelect,
  onPathAction,
}: {
  nav: DesktopState;
  entries: StoredWorktree[];
  onChange: Dispatch<SetStateAction<DesktopState>>;
  overviewActive: boolean;
  repositoryActive: boolean;
  onSelect: (id: string | null) => void;
  onPathAction?: (entry: StoredWorktree, action: "copy" | "reveal") => void;
}) {
  const [menu, setMenu] = useState<{ id: string; x: number; y: number } | null>(
    null,
  );
  const [drop, setDrop] = useState<{ id: string; after: boolean } | null>(null);
  const menuElement = useRef<HTMLDivElement>(null);
  const origin = useRef<HTMLButtonElement | null>(null);
  const overview = useRef<HTMLButtonElement>(null);
  const dragged = useRef<{ id: string; x: number; active: boolean } | null>(
    null,
  );
  const dropTarget = useRef<{ id: string; after: boolean } | null>(null);
  const suppressClick = useRef(false);
  useLayoutEffect(() => {
    if (!menu || !menuElement.current) return;
    const element = menuElement.current;
    const rect = element.getBoundingClientRect();
    element.style.left = `${Math.max(8, Math.min(menu.x, window.innerWidth - rect.width - 8))}px`;
    element.style.top = `${Math.max(8, Math.min(menu.y, window.innerHeight - rect.height - 8))}px`;
    element.querySelector<HTMLButtonElement>("button:not(:disabled)")?.focus();
  }, [menu]);
  useEffect(() => {
    if (!menu) return;
    function dismiss(event: PointerEvent) {
      if (!menuElement.current?.contains(event.target as Node)) setMenu(null);
    }
    function escape(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setMenu(null);
        origin.current?.focus();
      }
    }
    document.addEventListener("pointerdown", dismiss);
    document.addEventListener("keydown", escape);
    return () => {
      document.removeEventListener("pointerdown", dismiss);
      document.removeEventListener("keydown", escape);
    };
  }, [menu]);
  function close(scope: "current" | "right" | "others" | "all") {
    if (!menu) return;
    onChange((current) => {
      const index = current.tabs.findIndex((t) => t.worktree === menu.id);
      if (index < 0) return current;
      const ids =
        scope === "all"
          ? current.tabs.filter((t) => !t.pinned).map((t) => t.worktree)
          : scope === "others"
            ? current.tabs
                .filter((t) => t.worktree !== menu.id && !t.pinned)
                .map((t) => t.worktree)
            : scope === "right"
              ? current.tabs
                  .slice(index + 1)
                  .filter((t) => !t.pinned)
                  .map((t) => t.worktree)
              : [menu.id];
      return ids.reduce(closeTab, current);
    });
    setMenu(null);
    if (scope === "current" || scope === "all") overview.current?.focus();
    else origin.current?.focus();
  }
  const menuEntry = entries.find((e) => e.id === menu?.id);
  const menuTab = nav.tabs.find((t) => t.worktree === menu?.id);
  function pathAction(action: "copy" | "reveal") {
    if (menuEntry) onPathAction?.(menuEntry, action);
    setMenu(null);
    origin.current?.focus();
  }
  return (
    <>
      <nav className="tabs" aria-label="Open tabs">
        <button
          className="sidebar-toggle"
          aria-expanded={!nav.sidebar_collapsed}
          aria-controls="workspace-sidebar"
          onClick={() =>
            onChange((current) => ({
              ...current,
              sidebar_collapsed: !current.sidebar_collapsed,
            }))
          }
        >
          {nav.sidebar_collapsed ? "Show sidebar" : "Hide sidebar"}
        </button>
        <button
          ref={overview}
          className={`overview-tab ${overviewActive ? "active" : ""}`}
          onClick={() => onSelect(null)}
        >
          ▦ Overview
        </button>
        {nav.tabs.length > 0 && (
          <span className="tab-divider" aria-hidden="true" />
        )}
        {nav.tabs.map((t) => {
          const entry = entries.find((e) => e.id === t.worktree);
          const name = entry ? repoName(entry) : "unavailable repository";
          return (
            <div
              key={t.worktree}
              data-worktree={t.worktree}
              draggable={false}
              className={`tab ${repositoryActive && nav.active_tab === t.worktree ? "active" : ""} ${drop?.id === t.worktree ? (drop.after ? "drop-after" : "drop-before") : ""}`}
              onDragStart={(event) => event.preventDefault()}
              onPointerDown={(event) => {
                if (
                  event.button !== 0 ||
                  (event.target as HTMLElement).closest("[data-close-tab]")
                )
                  return;
                dragged.current = {
                  id: t.worktree,
                  x: event.clientX,
                  active: false,
                };
              }}
              onPointerMove={(event) => {
                const session = dragged.current;
                if (
                  !session ||
                  (!session.active && Math.abs(event.clientX - session.x) < 5)
                )
                  return;
                session.active = true;
                event.currentTarget.setPointerCapture?.(event.pointerId);
                const target = document
                  .elementFromPoint(event.clientX, event.clientY)
                  ?.closest<HTMLElement>("[data-worktree]");
                if (!target || target.dataset.worktree === session.id) {
                  dropTarget.current = null;
                  setDrop(null);
                  return;
                }
                const rect = target.getBoundingClientRect();
                const next = {
                  id: target.dataset.worktree!,
                  after: event.clientX > rect.left + rect.width / 2,
                };
                dropTarget.current = next;
                setDrop(next);
              }}
              onPointerUp={(event) => {
                const session = dragged.current;
                dragged.current = null;
                if (session?.active) {
                  event.preventDefault();
                  suppressClick.current = true;
                  const target = dropTarget.current;
                  if (target)
                    onChange((current) =>
                      reorderTab(current, session.id, target.id, target.after),
                    );
                  setDrop(null);
                  dropTarget.current = null;
                  setTimeout(() => {
                    suppressClick.current = false;
                  }, 0);
                }
                if (event.currentTarget.hasPointerCapture?.(event.pointerId))
                  event.currentTarget.releasePointerCapture(event.pointerId);
              }}
              onPointerCancel={() => {
                dragged.current = null;
                dropTarget.current = null;
                setDrop(null);
              }}
              onClickCapture={(event) => {
                if (suppressClick.current) {
                  event.preventDefault();
                  event.stopPropagation();
                  suppressClick.current = false;
                }
              }}
              onContextMenu={(event) => {
                event.preventDefault();
                origin.current = event.currentTarget.querySelector("button");
                setMenu({ id: t.worktree, x: event.clientX, y: event.clientY });
              }}
              onKeyDown={(event) => {
                if (
                  event.key === "ContextMenu" ||
                  (event.shiftKey && event.key === "F10")
                ) {
                  event.preventDefault();
                  const rect = event.currentTarget.getBoundingClientRect();
                  origin.current = event.currentTarget.querySelector("button");
                  setMenu({ id: t.worktree, x: rect.left, y: rect.bottom });
                } else if (
                  event.altKey &&
                  event.shiftKey &&
                  ["ArrowLeft", "ArrowRight"].includes(event.key)
                ) {
                  const index = nav.tabs.findIndex(
                    (item) => item.worktree === t.worktree,
                  );
                  const target =
                    nav.tabs[index + (event.key === "ArrowLeft" ? -1 : 1)];
                  if (target) {
                    event.preventDefault();
                    onChange((current) =>
                      reorderTab(
                        current,
                        t.worktree,
                        target.worktree,
                        event.key === "ArrowRight",
                      ),
                    );
                  }
                }
              }}
            >
              <button onClick={() => onSelect(t.worktree)}>
                {t.pinned && (
                  <span className="pinned-tab-label" aria-label="Pinned">
                    Pinned ·{" "}
                  </span>
                )}
                {entry ? name : "Unavailable repository"}
              </button>
              <button
                data-close-tab
                aria-label={`Close ${name} tab`}
                onClick={() =>
                  onChange((current) => closeTab(current, t.worktree))
                }
              >
                ×
              </button>
            </div>
          );
        })}
      </nav>
      {menu &&
        createPortal(
          <div
            role="menu"
            aria-label="Repository tab actions"
            className="tab-context-menu"
            ref={menuElement}
            style={{ left: menu.x, top: menu.y }}
            onKeyDown={(event) => {
              if (event.key === "Tab") {
                setMenu(null);
                return;
              }
              const buttons = [
                ...event.currentTarget.querySelectorAll<HTMLButtonElement>(
                  "button:not(:disabled)",
                ),
              ];
              const index = buttons.indexOf(
                document.activeElement as HTMLButtonElement,
              );
              let next: number;
              if (event.key === "ArrowDown")
                next = (index + 1) % buttons.length;
              else if (event.key === "ArrowUp")
                next = (index + buttons.length - 1) % buttons.length;
              else if (event.key === "Home") next = 0;
              else if (event.key === "End") next = buttons.length - 1;
              else return;
              event.preventDefault();
              buttons[next]?.focus();
            }}
          >
            <button role="menuitem" onClick={() => close("current")}>
              Close
            </button>
            <button
              role="menuitem"
              disabled={
                !nav.tabs.some((t) => t.worktree !== menu.id && !t.pinned)
              }
              onClick={() => close("others")}
            >
              Close Others
            </button>
            <button
              role="menuitem"
              disabled={
                !nav.tabs
                  .slice(nav.tabs.findIndex((t) => t.worktree === menu.id) + 1)
                  .some((t) => !t.pinned)
              }
              onClick={() => close("right")}
            >
              Close to the Right
            </button>
            <button
              role="menuitem"
              disabled={!nav.tabs.some((t) => !t.pinned)}
              onClick={() => close("all")}
            >
              Close All
            </button>
            <hr role="separator" />
            <button
              role="menuitem"
              disabled={!menuEntry || !onPathAction}
              onClick={() => pathAction("copy")}
            >
              Copy Path
            </button>
            <button
              role="menuitem"
              disabled={!menuEntry || !onPathAction}
              onClick={() => pathAction("reveal")}
            >
              Reveal in Finder
            </button>
            <hr role="separator" />
            <button
              role="menuitem"
              onClick={() => {
                onChange((current) => toggleTabPinned(current, menu.id));
                setMenu(null);
                origin.current?.focus();
              }}
            >
              {menuTab?.pinned ? "Unpin" : "Pin"}
            </button>
          </div>,
          document.body,
        )}
    </>
  );
}
