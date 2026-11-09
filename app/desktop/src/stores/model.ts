import type {
  DesktopState,
  StoredWorktree,
  NativePath,
  RepositoryTab,
  BulkComposerState,
  OverviewSort,
  OverviewFilter,
  TrustState,
} from "../services/response.generated";
export const initialBulkComposer: BulkComposerState = {
  version: 1,
  message: "",
  configurations: {},
};
export const initialState: DesktopState = {
  version: 1,
  tabs: [],
  active_tab: null,
  selected: [],
  drafts: {},
  filter: "",
  layout: "table",
  sort: "path",
  attention: "all",
  collapsed: [],
  density: "comfortable",
  sidebar_collapsed: false,
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
export const sortDefinitions = [
  { key: "path", label: "Path", ascending: "path", descending: "path_desc" },
  {
    key: "name",
    label: "Repository",
    ascending: "name",
    descending: "name_desc",
  },
  {
    key: "branch",
    label: "Branch / HEAD",
    ascending: "branch",
    descending: "branch_desc",
  },
  {
    key: "staged",
    label: "Staged",
    ascending: "staged",
    descending: "staged_desc",
  },
  {
    key: "unstaged",
    label: "Unstaged",
    ascending: "unstaged",
    descending: "unstaged_desc",
  },
  {
    key: "untracked",
    label: "Untracked",
    ascending: "untracked",
    descending: "untracked_desc",
  },
  {
    key: "conflicts",
    label: "Conflicts",
    ascending: "conflicts",
    descending: "conflicts_desc",
  },
  {
    key: "ahead",
    label: "Ahead (cached)",
    ascending: "ahead",
    descending: "ahead_desc",
  },
  {
    key: "behind",
    label: "Behind (cached)",
    ascending: "behind",
    descending: "behind_desc",
  },
  {
    key: "last_commit",
    label: "Last commit time",
    ascending: "last_commit",
    descending: "last_commit_desc",
  },
  {
    key: "trust",
    label: "Trust",
    ascending: "trust",
    descending: "trust_desc",
  },
  {
    key: "status",
    label: "Status",
    ascending: "status",
    descending: "status_desc",
  },
] as const satisfies readonly {
  key: string;
  label: string;
  ascending: OverviewSort;
  descending: OverviewSort;
}[];
export type SortColumn = (typeof sortDefinitions)[number]["key"];
export const sortChoices: {
  value: OverviewSort;
  label: string;
  column: SortColumn | "changes";
  descending: boolean;
}[] = [
  ...sortDefinitions.flatMap((definition) => [
    {
      value: definition.ascending,
      label: `${definition.label} ↑`,
      column: definition.key,
      descending: false,
    },
    {
      value: definition.descending,
      label: `${definition.label} ↓`,
      column: definition.key,
      descending: true,
    },
  ]),
  {
    value: "changes",
    label: "Changes (staged + unstaged) ↓",
    column: "changes",
    descending: true,
  },
];
export function sortChoice(sort: OverviewSort) {
  return sortChoices.find((choice) => choice.value === sort) ?? sortChoices[0];
}
export function toggleSort(
  sort: OverviewSort,
  column: SortColumn,
): OverviewSort {
  const definition = sortDefinitions.find((item) => item.key === column);
  if (!definition) return sort;
  return sort === definition.ascending
    ? definition.descending
    : definition.ascending;
}
function compareText(a: string, b: string): number {
  return a < b ? -1 : a > b ? 1 : 0;
}
function sortValue(
  entry: StoredWorktree,
  column: SortColumn | "changes",
  trust: ReadonlyMap<string, string>,
): string | number | null {
  const snapshot = entry.status;
  switch (column) {
    case "path":
      return pathText(entry.display_path);
    case "name":
      return repoName(entry);
    case "branch":
      return snapshot
        ? (snapshot.branch ??
            (snapshot.detached ? "Detached HEAD" : "Unborn HEAD"))
        : null;
    case "staged":
      return snapshot?.counts.staged ?? null;
    case "unstaged":
      return snapshot?.counts.unstaged ?? null;
    case "untracked":
      return snapshot?.counts.untracked ?? null;
    case "conflicts":
      return snapshot?.counts.conflicted ?? null;
    case "ahead":
      return snapshot?.tracking?.ahead ?? null;
    case "behind":
      return snapshot?.tracking?.behind ?? null;
    case "last_commit":
      return snapshot?.last_commit?.committer_time ?? null;
    case "trust":
      return trust.get(entry.id) ?? null;
    case "changes":
      return snapshot
        ? snapshot.counts.staged + snapshot.counts.unstaged
        : null;
    case "status":
      return overviewStatus(entry);
  }
}
export function overviewStatus(entry: StoredWorktree): string | null {
  if (entry.availability !== "available") return entry.availability;
  if (entry.issue) return "Issue";
  const snapshot = entry.status;
  if (!snapshot) return null;
  if (snapshot.counts.conflicted) return "Conflicted";
  return snapshot.counts.staged +
    snapshot.counts.unstaged +
    snapshot.counts.untracked
    ? "Changes"
    : "Clean";
}
export const attentionChoices = [
  { value: "all", label: "All repositories" },
  { value: "modified", label: "With changes" },
  { value: "staged", label: "With staged changes" },
  { value: "unstaged", label: "With unstaged changes" },
  { value: "untracked", label: "With untracked files" },
  { value: "conflicted", label: "With conflicts" },
  { value: "ahead", label: "Ahead (cached upstream)" },
  { value: "behind", label: "Behind (cached upstream)" },
  { value: "issues", label: "Needs attention" },
] as const satisfies readonly { value: OverviewFilter; label: string }[];
export function matchesAttention(
  entry: StoredWorktree,
  attention: OverviewFilter,
): boolean {
  const status = entry.status;
  switch (attention) {
    case "all":
      return true;
    case "modified":
      return (
        !!status &&
        status.counts.staged +
          status.counts.unstaged +
          status.counts.untracked +
          status.counts.conflicted >
          0
      );
    case "staged":
      return (status?.counts.staged ?? 0) > 0;
    case "unstaged":
      return (status?.counts.unstaged ?? 0) > 0;
    case "untracked":
      return (status?.counts.untracked ?? 0) > 0;
    case "conflicted":
      return (status?.counts.conflicted ?? 0) > 0;
    case "ahead":
      return (status?.tracking?.ahead ?? 0) > 0;
    case "behind":
      return (status?.tracking?.behind ?? 0) > 0;
    case "issues":
      return (
        entry.availability !== "available" ||
        !!entry.issue ||
        !status ||
        status.counts.conflicted > 0
      );
  }
}
export function collapseTree(
  state: DesktopState,
  key: string,
  collapsed: boolean,
): DesktopState {
  const current = state.collapsed ?? [];
  if (current.includes(key) === collapsed) return state;
  return {
    ...state,
    collapsed: collapsed
      ? [...current, key]
      : current.filter((value) => value !== key),
  };
}
export function filtered(
  entries: StoredWorktree[],
  query: string,
  attention: OverviewFilter,
  sort: OverviewSort,
  trustStates: readonly TrustState[] = [],
): StoredWorktree[] {
  const choice = sortChoice(sort);
  const trust = new Map(
    trustStates.map((state) => [state.worktree, state.validity]),
  );
  return entries
    .filter((e) =>
      `${repoName(e)} ${pathText(e.display_path)} ${e.status?.branch ?? ""}`
        .toLowerCase()
        .includes(query.toLowerCase()),
    )
    .filter((entry) => matchesAttention(entry, attention))
    .sort((a, b) => {
      const left = sortValue(a, choice.column, trust);
      const right = sortValue(b, choice.column, trust);
      // Unknown observations remain last in either direction; ties never depend on arrival order.
      if (left === null && right !== null) return 1;
      if (right === null && left !== null) return -1;
      const comparison =
        left === null || right === null
          ? 0
          : typeof left === "number" && typeof right === "number"
            ? left - right
            : compareText(String(left), String(right));
      return (
        comparison * (choice.descending ? -1 : 1) ||
        compareText(pathText(a.display_path), pathText(b.display_path)) ||
        compareText(a.id, b.id)
      );
    });
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
