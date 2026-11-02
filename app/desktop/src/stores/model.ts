import type {
  DesktopState,
  StoredWorktree,
  NativePath,
  RepositoryTab,
} from "../services/response.generated";
export const initialState: DesktopState = {
  version: 1,
  tabs: [],
  active_tab: null,
  selected: [],
  drafts: {},
  filter: "",
  layout: "table",
  sort: "path",
  theme: "system",
  sidebar_width: 240,
  inspector_width: 400,
};
function escapedControl(c: string): string {
  const n = c.charCodeAt(0);
  return n > 255
    ? `\\u${n.toString(16).padStart(4, "0")}`
    : `\\x${n.toString(16).padStart(2, "0")}`;
}
export function bytes(input: number[]): string {
  let text: string;
  try {
    text = new TextDecoder("utf-8", { fatal: true }).decode(
      Uint8Array.from(input),
    );
  } catch {
    return (
      "[non-UTF-8 bytes] " +
      input
        .map((n) =>
          n >= 32 && n < 127
            ? String.fromCharCode(n)
            : `\\x${n.toString(16).padStart(2, "0")}`,
        )
        .join("")
    );
  }
  return text.replace(
    /[\u0000-\u0008\u000b-\u001f\u007f\u061c\u200e\u200f\u202a-\u202e\u2066-\u2069]/g,
    escapedControl,
  );
}
export function pathText(path: NativePath): string {
  if (path.encoding === "utf8")
    return path.value.replace(
      /[\u0000-\u001f\u007f\u061c\u200e\u200f\u202a-\u202e\u2066-\u2069]/g,
      (c) =>
        c.charCodeAt(0) <= 31
          ? JSON.stringify(c).slice(1, -1)
          : escapedControl(c),
    );
  if (path.encoding === "unix_bytes") return bytes(path.value);
  return String.fromCharCode(...path.value).replace(
    /[\u0000-\u001f\u007f\u061c\u200e\u200f\u202a-\u202e\u2066-\u2069]/g,
    escapedControl,
  );
}
export function relative(entry: StoredWorktree, root: string): string {
  const path = pathText(entry.display_path);
  return path.startsWith(root + "/")
    ? path.slice(root.length + 1)
    : path === root
      ? "."
      : path;
}
export function repoName(entry: StoredWorktree): string {
  return (
    pathText(entry.metadata.root).split("/").filter(Boolean).at(-1) ?? entry.id
  );
}
export function toggleSelection(state: DesktopState, id: string): DesktopState {
  return {
    ...state,
    selected: state.selected.includes(id)
      ? state.selected.filter((v) => v !== id)
      : [...state.selected, id],
  };
}
export function openTab(
  state: DesktopState,
  entry: StoredWorktree,
): DesktopState {
  if (state.tabs.some((t) => t.worktree === entry.id))
    return { ...state, active_tab: entry.id };
  const c = entry.status?.counts;
  const dirty = c && c.staged + c.unstaged + c.untracked + c.conflicted > 0;
  const tab: RepositoryTab = {
    worktree: entry.id,
    view: dirty ? "changes" : "history",
    path: null,
    commit: null,
    parent: null,
    source: "working_tree",
    scroll: 0,
    filter: "",
  };
  return { ...state, active_tab: entry.id, tabs: [...state.tabs, tab] };
}
export function closeTab(state: DesktopState, id: string): DesktopState {
  return {
    ...state,
    tabs: state.tabs.filter((t) => t.worktree !== id),
    active_tab: state.active_tab === id ? null : state.active_tab,
  };
}
export function updateTab(
  state: DesktopState,
  id: string,
  update: Partial<RepositoryTab>,
): DesktopState {
  return {
    ...state,
    tabs: state.tabs.map((t) => (t.worktree === id ? { ...t, ...update } : t)),
  };
}
export function filtered(
  entries: StoredWorktree[],
  query: string,
  attention: string,
  sort: string,
): StoredWorktree[] {
  return entries
    .filter((e) =>
      `${repoName(e)} ${pathText(e.display_path)} ${e.status?.branch ?? ""}`
        .toLowerCase()
        .includes(query.toLowerCase()),
    )
    .filter(
      (e) =>
        attention === "all" ||
        (attention === "modified" &&
          !!e.status &&
          e.status.counts.staged +
            e.status.counts.unstaged +
            e.status.counts.untracked >
            0) ||
        (attention === "issues" &&
          (e.availability !== "available" ||
            !!e.issue ||
            !!e.status?.counts.conflicted)),
    )
    .sort((a, b) =>
      sort === "changes"
        ? (b.status?.counts.unstaged ?? 0) +
          (b.status?.counts.staged ?? 0) -
          ((a.status?.counts.unstaged ?? 0) + (a.status?.counts.staged ?? 0))
        : pathText(a.display_path).localeCompare(pathText(b.display_path)),
    );
}
export interface TreeNode {
  key: string;
  label: string;
  entry?: StoredWorktree;
  children: TreeNode[];
}
export function buildTree(entries: StoredWorktree[], root: string): TreeNode[] {
  const nodes: TreeNode[] = [];
  for (const entry of entries) {
    const parts = relative(entry, root).split("/");
    let level = nodes;
    let prefix = "";
    parts.forEach((part, i) => {
      prefix += `/${part}`;
      let node = level.find((n) => n.key === prefix);
      if (!node) {
        node = { key: prefix, label: part, children: [] };
        level.push(node);
      }
      if (i === parts.length - 1) node.entry = entry;
      level = node.children;
    });
  }
  return nodes;
}
